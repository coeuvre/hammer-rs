extern crate winapi;
extern crate gdi32;
extern crate kernel32;
extern crate opengl32;
extern crate user32;

use std::cell::RefCell;
use std::ffi::CString;
use std::mem;
use std::sync::*;
use std::sync::mpsc::*;
use std::thread;

use input::keyboard::Key;

use self::winapi::basetsd::*;
use self::winapi::minwindef::*;
use self::winapi::windef::*;
use self::winapi::wingdi::*;
use self::winapi::winnt::*;
use self::winapi::winuser::*;
use self::gdi32::*;
use self::kernel32::*;
use self::opengl32::*;
use self::user32::*;

use Error;

macro_rules! wstr {
    ($str:expr) => ({
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new($str)
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect::<Vec<_>>()
    });
}

#[derive(Debug)]
pub enum Event {
    Resize { x: i32, y: i32, w: i32, h: i32, },
    KeyDown(Key),
    KeyUp(Key),
    Close,
}

pub struct WindowBuilder {
    title: String,
    x: Option<i32>,
    y: Option<i32>,
    w: Option<i32>,
    h: Option<i32>,
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            title: "Untitled".to_string(),
            x: None,
            y: None,
            w: None,
            h: None,
        }
    }

    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_string();
        self
    }

    pub fn pos(&mut self, x: i32, y: i32) -> &mut Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    pub fn size(&mut self, w: i32, h: i32) -> &mut Self {
        self.w = Some(w);
        self.h = Some(h);
        self
    }

    pub fn build(&self) -> Result<Window, Error> {
        WINDOW_THREAD_INIT.call_once(|| {
            let (tx, rx) = channel();

            thread::spawn(move || {
                unsafe { window_thread_main(tx); }
            });

            // Wait for the window thread to finish initialization
            rx.recv().unwrap();
        });

        let (tx, rx) = channel();
        let create_window_param = CreateWindowParam {
            builder: self,
            tx: tx,
        };

        unsafe {
            assert!(WINDOW_THREAD_ID != 0);
            PostThreadMessageW(WINDOW_THREAD_ID, WM_CREATE_WINDOW, 0,
                               &create_window_param as *const CreateWindowParam as LPARAM);
        }

        rx.recv().unwrap()
    }
}

pub struct Window {
    event_rx: Receiver<Event>,
    state: *const WindowState,

    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Window {
    pub fn show(&mut self) {
        unsafe {
            PostThreadMessageW(WINDOW_THREAD_ID, WM_SHOW_WINDOW, 0, self.hwnd() as LPARAM);
        }
    }

    pub fn poll_events(&mut self) -> PollEventIter {
        PollEventIter {
            window: self,
        }
    }

    pub fn wait_events(&mut self) -> WaitEventIter {
        WaitEventIter {
            window: self,
        }
    }

    pub fn close(self) {
        drop(self);
    }

    pub fn size(&self) -> (i32, i32) {
        (self.w, self.h)
    }

    pub fn create_gl_context(&self) -> GlContext {
        unsafe {
            let hdc = self.hdc();
            GlContext {
                hglrc: create_gl_context(hdc),
                hdc: hdc,
            }
        }
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            &Event::Resize { x, y, w, h } => {
                self.x = x;
                self.y = y;
                self.w = w;
                self.h = h;
            }
            _ => {}
        }
    }

    unsafe fn hwnd(&self) -> HWND {
        (*self.state).hwnd
    }

    unsafe fn hdc(&self) -> HDC {
        (*self.state).hdc
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let (tx, rx) = channel();

        unsafe {
            let destroy_window_param = DestroyWindowParam {
                window: self,
                tx: tx,
            };

            PostThreadMessageW(WINDOW_THREAD_ID, WM_DESTROY_WINDOW, 0,
                               &destroy_window_param as *const DestroyWindowParam as LPARAM);

            rx.recv().unwrap().unwrap();
        }
    }
}

pub struct PollEventIter<'a> {
    window: &'a mut Window,
}

impl<'a> Iterator for PollEventIter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.window.event_rx.try_recv().ok();
        if let Some(ref event) = event {
            self.window.handle_event(event);
        }
        event
    }
}

pub struct WaitEventIter<'a> {
    window: &'a mut Window,
}

impl<'a> Iterator for WaitEventIter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.window.event_rx.recv().ok();
        if let Some(ref event) = event {
            self.window.handle_event(event);
        }
        event
    }
}

pub struct WindowState {
    event_tx: Sender<Event>,

    hwnd: HWND,
    hdc: HDC,
}

impl WindowState {
    fn new(event_tx: Sender<Event>) -> WindowState {
        WindowState {
            event_tx: event_tx,

            hwnd: 0 as HWND,
            hdc: 0 as HDC,
        }
    }
}

thread_local!(static THREAD_CURRENT_CONTEXT: RefCell<HGLRC> = RefCell::new(0 as HGLRC));

pub struct GlContext {
    hglrc: HGLRC,
    hdc: HDC,
}

impl GlContext {
    // One thread can only have one renderer be _current_.
    pub fn make_current(&mut self) {
        unsafe {
            if let Some(old_hglrc) = THREAD_CURRENT_CONTEXT.with(|thread_current_context| {
                let mut thread_current_context = thread_current_context.borrow_mut();
                if *thread_current_context == self.hglrc {
                    None
                } else {
                    let old = *thread_current_context;
                    wglMakeCurrent(0 as HDC, 0 as HGLRC);
                    wglMakeCurrent(self.hdc, self.hglrc);
                    *thread_current_context = self.hglrc;
                    Some(old)
                }
            }) {
                context_changed(old_hglrc, self.hglrc);
            }
        }
    }

    pub fn load_function(&self, symbol: &str) -> *const winapi::c_void {
        unsafe {
            let cstr = CString::new(symbol).unwrap();
            let mut ptr = wglGetProcAddress(cstr.as_ptr());
            if ptr == 0 as PROC {
                ptr = GetProcAddress(OPENGL_LIB, cstr.as_ptr());
            }
            ptr
        }
    }

    pub fn swap_buffers(&mut self) {
        unsafe { SwapBuffers(self.hdc); }
    }
}

fn context_changed(old: HGLRC, new: HGLRC) {
    info!("Thread {} has changed current wgl context from {:?} to {:?}",
          thread::current().name().unwrap_or(& unsafe { GetCurrentThreadId() }.to_string()),
          old, new);
}

impl Drop for GlContext {
    fn drop(&mut self) {
        THREAD_CURRENT_CONTEXT.with(|thread_current_context| {
            let mut thread_current_context = thread_current_context.borrow_mut();
            if *thread_current_context == self.hglrc {
                unsafe {
                    wglMakeCurrent(0 as HDC, 0 as HGLRC);
                }

                *thread_current_context = 0 as HGLRC;

                context_changed(self.hglrc, 0 as HGLRC);
            }
        });
    }
}

const WM_CREATE_WINDOW: UINT = WM_USER;
const WM_SHOW_WINDOW: UINT = WM_USER + 1;
const WM_DESTROY_WINDOW: UINT = WM_USER + 2;

static WINDOW_THREAD_INIT: Once = ONCE_INIT;
static mut WINDOW_THREAD_ID: DWORD = 0;

struct CreateWindowParam<'a> {
    builder: &'a WindowBuilder,
    tx: Sender<Result<Window, Error>>,
}

struct DestroyWindowParam<'a> {
    window: &'a mut Window,
    tx: Sender<Result<(), Error>>,
}

unsafe fn window_thread_main(tx: Sender<()>) {
    let hinstance = GetModuleHandleW(0 as LPCWSTR);

    let class_name = wstr!("HAMMERWINDOWCLASS");

    let mut wc: WNDCLASSEXW = mem::zeroed();
    wc.cbSize = mem::size_of_val(&wc) as UINT;
    wc.style = 0;
    wc.lpfnWndProc = Some(window_proc);
    wc.hCursor = LoadCursorW(0 as HINSTANCE, winapi::winuser::IDC_ARROW);
    wc.hInstance = hinstance;
    wc.lpszClassName = class_name.as_ptr();
    wc.cbWndExtra = mem::size_of::<Window>() as winapi::c_int;

    RegisterClassExW(&wc);

    WINDOW_THREAD_ID = GetCurrentThreadId();

    tx.send(()).unwrap();

    let mut msg = mem::uninitialized();
    while GetMessageW(&mut msg, 0 as HWND, 0, 0) != 0 {
        match msg.message {
            WM_CREATE_WINDOW => {
                let create_window_param = &*(msg.lParam as *const CreateWindowParam);
                let result = create_window(hinstance, &class_name, create_window_param.builder);
                create_window_param.tx.send(result).unwrap();
            }

            WM_SHOW_WINDOW => {
                let hwnd = msg.lParam as HWND;
                ShowWindow(hwnd, SW_SHOW);
            }

            WM_DESTROY_WINDOW => {
                let destroy_window_param = &*(msg.lParam as *const DestroyWindowParam);
                let state = &*destroy_window_param.window.state;
                ReleaseDC(state.hwnd, state.hdc);
                DestroyWindow(state.hwnd);
                destroy_window_param.tx.send(Ok(())).unwrap();
            }

            _ => {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

#[allow(non_snake_case)]
#[cfg(target_arch = "x86_64")]
unsafe fn SetWindowLongPtr(hwnd: HWND, index: winapi::c_int, data: LONG_PTR) {
    SetWindowLongPtrW(hwnd, index, data);
}

#[allow(non_snake_case)]
#[cfg(target_arch = "x86")]
unsafe fn SetWindowLongPtr(hwnd: HWND, index: winapi::c_int, data: LONG_PTR) {
    SetWindowLongW(hwnd, index, data);
}

#[allow(non_snake_case)]
#[cfg(target_arch = "x86_64")]
unsafe fn GetWindowLongPtr(hwnd: HWND, index: winapi::c_int) -> LONG_PTR {
    GetWindowLongPtrW(hwnd, index)
}

#[allow(non_snake_case)]
#[cfg(target_arch = "x86")]
unsafe fn GetWindowLongPtr(hwnd: HWND, index: winapi::c_int) -> LONG_PTR {
    GetWindowLongW(hwnd, index)
}

#[allow(unused_variables)]
unsafe extern "system"
fn window_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let state = &mut *(GetWindowLongPtr(hwnd, 0) as (*mut WindowState));

    match msg {
        WM_CREATE => {
            let cs = &*(lparam as (*mut CREATESTRUCTW));
            let state = &mut *(cs.lpCreateParams as (*mut WindowState));
            SetWindowLongPtr(hwnd, 0, state as (*mut WindowState) as LONG_PTR);

            state.hwnd = hwnd;

            let mut rect = mem::uninitialized();
            GetWindowRect(hwnd, &mut rect);

            //SetWindowLongPtr(hwnd, GWL_STYLE, WS_POPUP as LONG_PTR);
            //SetWindowLongPtr(hwnd, GWL_EXSTYLE, WS_EX_LAYERED as LONG_PTR);

            state.hdc = GetDC(hwnd);
        }

        WM_SIZE => {
            let mut rect = mem::uninitialized();
            GetWindowRect(hwnd, &mut rect);
            let x = rect.left;
            let y = rect.top;
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            state.event_tx.send(Event::Resize { x: x, y: y, w: w, h: h }).unwrap();
        }

        WM_CLOSE => {
            state.event_tx.send(Event::Close).unwrap();
        }

        WM_KEYDOWN => {
            let key = wparam as winapi::c_int;
            state.event_tx.send(Event::KeyDown(winkey_to_key(key))).unwrap();
        }

        WM_KEYUP => {
            let key = wparam as winapi::c_int;
            state.event_tx.send(Event::KeyUp(winkey_to_key(key))).unwrap();
        }

        _ => {
            return DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }

    return 0;
}

unsafe fn create_window(hinstance: HINSTANCE, class_name: &Vec<u16>, builder: &WindowBuilder) -> Result<Window, Error> {
    let (event_tx, event_rx) = channel();

    let state = Box::into_raw(Box::new(WindowState::new(event_tx)));

    let style = WS_OVERLAPPEDWINDOW ^ WS_THICKFRAME ^ WS_MAXIMIZEBOX;
    let ex_style = 0;

    let title = wstr!(&builder.title);
    let x = builder.x.unwrap_or(CW_USEDEFAULT);
    let y = builder.y.unwrap_or(CW_USEDEFAULT);

    // Calcuate window size base on client area size
    let (w, h) = if builder.w.is_some() && builder.h.is_some() {
        let w = builder.w.unwrap();
        let h = builder.h.unwrap();
        let mut rect = RECT {
            left: 0,
            right: w,
            top: 0,
            bottom: h,
        };
        AdjustWindowRectEx(&mut rect, style, 0, ex_style);
        (rect.right - rect.left, rect.bottom - rect.top)
    } else {
        (CW_USEDEFAULT, CW_USEDEFAULT)
    };

    CreateWindowExW(
        ex_style,
        class_name.as_ptr(),
        title.as_ptr(),
        style,
        x, y, w, h,
        0 as HWND,
        0 as HMENU,
        hinstance,
        state as LPVOID,
    );

    let window = Window {
        event_rx: event_rx,
        state: state,

        x: x,
        y: y,
        w: w,
        h: h,
    };

    Ok(window)
}

static OPENGL_LIB_INIT: Once = ONCE_INIT;
static mut OPENGL_LIB: HMODULE = 0 as HMODULE;

unsafe fn create_gl_context(hdc: HDC) -> HGLRC {
    OPENGL_LIB_INIT.call_once(|| {
        let name = wstr!("opengl32.dll");
        OPENGL_LIB = LoadLibraryW(name.as_ptr());
        assert!(OPENGL_LIB != 0 as HMODULE, "Failed to load opengl32.dll");
    });

    let mut pfd: PIXELFORMATDESCRIPTOR = mem::zeroed();
    pfd.nSize = mem::size_of_val(&pfd) as WORD;
    pfd.nVersion = 1;
    pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
    pfd.iPixelType = PFD_TYPE_RGBA;
    pfd.cColorBits = 32;
    pfd.cDepthBits = 24;
    pfd.cStencilBits = 8;
    pfd.iLayerType = PFD_MAIN_PLANE;
    let pfi = ChoosePixelFormat(hdc, &mut pfd);
    SetPixelFormat(hdc, pfi, &mut pfd);

    wglCreateContext(hdc)
}

fn winkey_to_key(key: winapi::c_int) -> Key {
    match key {
        VK_UP => Key::Up,
        VK_DOWN => Key::Down,
        VK_LEFT => Key::Left,
        VK_RIGHT => Key::Right,
        VK_RETURN => Key::Return,
        VK_ESCAPE => Key::Escape,
        VK_SPACE => Key::Space,
        0x30 => Key::Num0,
        0x31 => Key::Num1,
        0x32 => Key::Num2,
        0x33 => Key::Num3,
        0x34 => Key::Num4,
        0x35 => Key::Num5,
        0x36 => Key::Num6,
        0x37 => Key::Num7,
        0x38 => Key::Num8,
        0x39 => Key::Num9,
        0x45 => Key::E,
        0x51 => Key::Q,
        VK_OEM_PERIOD => Key::Period,
        VK_BACK => Key::Backspace,
        _ => Key::Unknown,
    }
}
