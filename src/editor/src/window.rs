use std::cell::RefCell;
use std::ffi::{CString, CStr};
use std::mem;
use std::sync::*;
use std::sync::mpsc::*;
use std::thread;

use winapi;
use winapi::basetsd::*;
use winapi::minwindef::*;
use winapi::windef::*;
use winapi::wingdi::*;
use winapi::winnt::*;
use winapi::winuser::*;
use gdi32::*;
use kernel32::*;
use opengl32::*;
use user32::*;

use gl;

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

pub enum Event {
    Close,
}

pub struct WindowBuilder {
    title: String,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            title: "Hammer".to_string(),
            x: 0,
            y: 0,
            w: 800,
            h: 600,
        }
    }

    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_string();
        self
    }

    pub fn pos(&mut self, x: i32, y: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn size(&mut self, w: i32, h: i32) -> &mut Self {
        self.w = w;
        self.h = h;
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
    /*
    context: RenderContext,
    buffer: RenderBuffer,
    */

    event_rx: Receiver<Event>,
    state: *const WindowState,
}

unsafe impl Send for Window {}

impl Window {
    pub fn show(&mut self) {
        unsafe {
            PostThreadMessageW(WINDOW_THREAD_ID, WM_SHOW_WINDOW, 0, (*self.state).hwnd as LPARAM);
        }
    }

    pub fn close(self) {
        drop(self);
    }

    pub fn poll_events(&self) -> PollEventIter {
        PollEventIter {
            window: self,
        }
    }

    pub fn wait_events(&self) -> WaitEventIter {
        WaitEventIter {
            window: self,
        }
    }

    /*
    pub unsafe fn render(&self) -> Result<(), Error> {
        try!(self.context.make_current());

        gl::BindFramebuffer(gl::FRAMEBUFFER, self.buffer.fbo);

        gl::Enable(gl::FRAMEBUFFER_SRGB);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
        gl::BlendEquation(gl::FUNC_ADD);

        gl::Enable(gl::SCISSOR_TEST);
        gl::Scissor(0, 0, self.w, self.h);


        gl::ClearColor(0.3, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        gl::ReadPixels(0, 0, self.buffer.w, self.buffer.h, gl::RGBA, gl::UNSIGNED_BYTE, self.buffer.pixels as *mut std::os::raw::c_void);

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        self.copy_buffer_to_window();

        Ok(())
    }

    unsafe fn copy_buffer_to_window(&self) {
        const AC_SRC_OVER: u8 = 0x00;
        const AC_SRC_ALPHA: u8 = 0x01;
        const ULW_ALPHA: u32 = 0x00000002;

        let mut pos = POINT {
            x: self.x,
            y: self.y,
        };
        let mut size = SIZE {
            cx: self.w,
            cy: self.h,
        };
        let mut src_pos = POINT {
            x: 0,
            y: 0,
        };
        let mut blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA,
        };

        UpdateLayeredWindow(self.hwnd, GetDC(self.hwnd),
                            &mut pos, &mut size,
                            self.buffer.hdc,
                            &mut src_pos, 0, &mut blend, ULW_ALPHA);
    }
    */
}

pub struct PollEventIter<'a> {
    window: &'a Window,
}

impl<'a> Iterator for PollEventIter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.window.event_rx.try_recv().ok()
    }
}

pub struct WaitEventIter<'a> {
    window: &'a Window,
}

impl<'a> Iterator for WaitEventIter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.window.event_rx.recv().ok()
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let (tx, rx) = channel();

        unsafe {
            let destroy_window_param = DestroyWindowParam {
                hwnd: (*self.state).hwnd,
                tx: tx,
            };

            PostThreadMessageW(WINDOW_THREAD_ID, WM_DESTROY_WINDOW, 0,
                               &destroy_window_param as *const DestroyWindowParam as LPARAM);

            rx.recv().unwrap().unwrap();
        }
    }
}

pub struct WindowState {
    event_tx: Sender<Event>,

    hwnd: HWND,
    hdc: HDC,

    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl WindowState {
    fn new(event_tx: Sender<Event>) -> WindowState {
        WindowState {
            event_tx: event_tx,

            hwnd: 0 as HWND,
            hdc: 0 as HDC,

            x: 0,
            y: 0,
            w: 0,
            h: 0,
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

            state.x = rect.left;
            state.y = rect.top;
            state.w = rect.right - rect.left;
            state.h = rect.bottom - rect.top;

            //SetWindowLongPtr(hwnd, GWL_STYLE, WS_POPUP as LONG_PTR);
            //SetWindowLongPtr(hwnd, GWL_EXSTYLE, WS_EX_LAYERED as LONG_PTR);

            OPENGL_CONTEXT_INIT.call_once(|| {
                create_render_context(hwnd);
            });

            let hdc = GetDC(hwnd);
            let pfi = ChoosePixelFormat(hdc, &mut RENDER_CONTEXT.pfd);
            SetPixelFormat(hdc, pfi, &mut RENDER_CONTEXT.pfd);
            state.hdc = hdc;
            /*
            window.context = RenderContext::new(GetDC(hwnd)).unwrap();
            window.buffer = RenderBuffer::new(&window.context, window.w, window.h).unwrap();

            window.context.make_current().unwrap();

            window.render().unwrap();
            */
        }

        WM_PAINT => {
            let mut ps = mem::uninitialized();
            BeginPaint(hwnd, &mut ps);

            RENDER_CONTEXT.make_current(state.hdc);
            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            SwapBuffers(state.hdc);
            RENDER_CONTEXT.make_not_current();

            EndPaint(hwnd, &ps);
        }

        WM_CLOSE => {
            state.event_tx.send(Event::Close).unwrap();
        }

        WM_KEYDOWN => {
            let key = wparam as winapi::c_int;
            if key == VK_ESCAPE {
                state.event_tx.send(Event::Close).unwrap();
            }
        }

        _ => {
            return DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }

    return 0;
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

struct DestroyWindowParam {
    hwnd: HWND,
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
                let hwnd = destroy_window_param.hwnd;
                let state = &mut *(GetWindowLongPtr(hwnd, 0) as (*mut WindowState));
                ReleaseDC(hwnd, state.hdc);
                DestroyWindow(hwnd);
                destroy_window_param.tx.send(Ok(())).unwrap();
            }

            _ => {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

unsafe fn create_window(hinstance: HINSTANCE, class_name: &Vec<u16>, builder: &WindowBuilder) -> Result<Window, Error> {
    let (event_tx, event_rx) = channel();

    let state = Box::into_raw(Box::new(WindowState::new(event_tx)));

    let title = wstr!(&builder.title);
    CreateWindowExW(
        0,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_OVERLAPPEDWINDOW,
        builder.x, builder.y,
        builder.w, builder.h,
        0 as HWND,
        0 as HMENU,
        hinstance,
        state as LPVOID,
    );

    let window = Window {
        event_rx: event_rx,
        state: state,
    };

    Ok(window)
}

struct RenderContextWrapper {
    lib: HMODULE,
    hglrc: HGLRC,
    pfd: PIXELFORMATDESCRIPTOR,
}

impl RenderContextWrapper {
    pub unsafe fn make_current(&self, hdc: HDC) {
        wglMakeCurrent(hdc, self.hglrc);
        gl::load_with(|symbol| {
            let cstr = CString::new(symbol).unwrap();
            let mut ptr = wglGetProcAddress(cstr.as_ptr());
            if ptr == 0 as PROC {
                ptr = GetProcAddress(self.lib, cstr.as_ptr());
            }
            ptr
        });
    }

    pub unsafe fn make_not_current(&self) {
        wglMakeCurrent(0 as HDC, 0 as HGLRC);
    }
}

static OPENGL_CONTEXT_INIT: Once = ONCE_INIT;
static mut RENDER_CONTEXT: RenderContextWrapper = RenderContextWrapper {
    lib: 0 as HMODULE,
    hglrc: 0 as HGLRC,
    pfd: PIXELFORMATDESCRIPTOR {
        nSize: 0,
        nVersion: 1,
        dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
        iPixelType: PFD_TYPE_RGBA,
        cColorBits: 32,
        cRedBits: 0,
        cRedShift: 0,
        cGreenBits: 0,
        cGreenShift: 0,
        cBlueBits: 0,
        cBlueShift: 0,
        cAlphaBits: 0,
        cAlphaShift: 0,
        cAccumBits: 0,
        cAccumRedBits: 0,
        cAccumGreenBits: 0,
        cAccumBlueBits: 0,
        cAccumAlphaBits: 0,
        cDepthBits: 24,
        cStencilBits: 8,
        cAuxBuffers: 0,
        iLayerType: PFD_MAIN_PLANE,
        bReserved: 0,
        dwLayerMask: 0,
        dwVisibleMask: 0,
        dwDamageMask: 0,
    },
};

unsafe fn create_render_context(hwnd: HWND) {
    let name = wstr!("opengl32.dll");
    RENDER_CONTEXT.lib = LoadLibraryW(name.as_ptr());
    assert!(RENDER_CONTEXT.lib != 0 as HMODULE, "Failed to load opengl32.dll");

    let hdc = GetDC(hwnd);

    RENDER_CONTEXT.pfd.nSize = mem::size_of_val(&RENDER_CONTEXT.pfd) as WORD;

    let pfi = ChoosePixelFormat(hdc, &mut RENDER_CONTEXT.pfd);

    SetPixelFormat(hdc, pfi, &mut RENDER_CONTEXT.pfd);

    RENDER_CONTEXT.hglrc = wglCreateContext(hdc);
    assert!(RENDER_CONTEXT.hglrc != 0 as HGLRC, "Failed to create rendering context");

    ReleaseDC(hwnd, hdc);
}
