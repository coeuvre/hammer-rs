extern crate winapi;
extern crate gdi32;
extern crate kernel32;
extern crate opengl32;
extern crate user32;

use std;
use std::cell::RefCell;
use std::ffi::{CString, CStr};
use std::mem;
use std::sync::*;
use std::sync::mpsc::*;
use std::thread;

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

use gl;
use gl::types::*;

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

use std::rc::Rc;

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

            /*
            window.context = RenderContext::new(GetDC(hwnd)).unwrap();
            window.buffer = RenderBuffer::new(&window.context, window.w, window.h).unwrap();

            window.context.make_current().unwrap();

            window.render().unwrap();
            */
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

        x: builder.x,
        y: builder.y,
        w: builder.w,
        h: builder.h,
    };

    Ok(window)
}

static OPENGL_LIB_INIT: Once = ONCE_INIT;
static OPENGL_FUNCTION_INIT: Once = ONCE_INIT;
static mut OPENGL_LIB: HMODULE = 0 as HMODULE;

thread_local!(static THREAD_CURRENT_CONTEXT: RefCell<HGLRC> = RefCell::new(0 as HGLRC));

pub struct Renderer {
    hglrc: HGLRC,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            hglrc: 0 as HGLRC,
        }
    }

    // One thread can only have one renderer be actived.
    pub fn active(&mut self, window: &Window) -> Result<RenderContext, Error> {
        unsafe {
            let state = &*window.state;
            if self.hglrc == 0 as HGLRC {
                self.hglrc = create_render_context(state.hdc);
            }

            let hglrc = self.hglrc;
            let result: Result<(), Error> = THREAD_CURRENT_CONTEXT.with(|thread_current_context| {
                let mut thread_current_context = thread_current_context.borrow_mut();
                if *thread_current_context == 0 as HGLRC {
                    *thread_current_context = hglrc;
                    Ok(())
                } else {
                    Err(From::from("Failed to active the renderer, there are another renderer been actived."))
                }
            });

            if result.is_err() {
                return Err(result.unwrap_err());
            }

            let hdc = state.hdc;
            wglMakeCurrent(hdc, self.hglrc);

            // TODO: Make sure that initialize OpenGL function once is enough.
            OPENGL_FUNCTION_INIT.call_once(|| {
                gl::load_with(|symbol| {
                    let cstr = CString::new(symbol).unwrap();
                    let mut ptr = wglGetProcAddress(cstr.as_ptr());
                    if ptr == 0 as PROC {
                        ptr = GetProcAddress(OPENGL_LIB, cstr.as_ptr());
                    }
                    ptr
                });
            });

            let (w, h) = window.size();

            gl::Viewport(0, 0, w, h);

            //let screen_w = GetSystemMetrics(SM_CXSCREEN);
            //let screen_h = GetSystemMetrics(SM_CYSCREEN);
            //let buffer = RenderBuffer::new(screen_w, screen_h).unwrap();

            //gl::BindFramebuffer(gl::FRAMEBUFFER, buffer.fbo);

            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::BlendEquation(gl::FUNC_ADD);

            Ok(RenderContext {
                renderer: self,
                hdc: window.hdc(),
                //buffer: buffer,
            })
        }
    }
}

pub struct RenderContext<'a> {
    renderer: &'a mut Renderer,
    hdc: HDC,
    //buffer: RenderBuffer,
}

impl<'a> RenderContext<'a> {
    pub fn resize(&mut self, w: i32, h: i32) {
        unsafe {
            //self.buffer.resize(w, h);
            gl::Viewport(0, 0, w, h);
        }
    }

    pub fn present(&mut self) {
        unsafe {
            SwapBuffers(self.hdc);
        }
        /*
        unsafe {
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
            gl::ReadPixels(0, 0, self.buffer.w, self.buffer.h, gl::BGRA, gl::UNSIGNED_BYTE, self.buffer.pixels as *mut std::os::raw::c_void);

            const AC_SRC_OVER: u8 = 0x00;
            const AC_SRC_ALPHA: u8 = 0x01;
            const ULW_ALPHA: u32 = 0x00000002;

            let mut pos = POINT {
                x: self.window.inner.borrow().x,
                y: self.window.inner.borrow().y,
            };
            let mut size = SIZE {
                cx: self.window.inner.borrow().w,
                cy: self.window.inner.borrow().h,
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

            let state = &*self.window.inner.borrow().state;
            UpdateLayeredWindow(state.hwnd, state.hdc,
                                &mut pos, &mut size,
                                self.buffer.hdc,
                                &mut src_pos, 0, &mut blend, ULW_ALPHA);
        }
        */
    }
}

impl<'a> Drop for RenderContext<'a> {
    fn drop(&mut self) {
        unsafe {
            wglMakeCurrent(0 as HDC, 0 as HGLRC);
        }

        THREAD_CURRENT_CONTEXT.with(|thread_current_context| {
            *thread_current_context.borrow_mut() = 0 as HGLRC;
        });
    }
}

unsafe fn create_render_context(hdc: HDC) -> HGLRC {
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

/*
pub struct RenderBuffer {
    w: i32,
    h: i32,

    fbo: GLuint,
    texture: GLuint,

    hdc: HDC,
    bi: BITMAPINFO,
    bitmap: HBITMAP,
    prev_bitmap: HBITMAP,
    pixels: *mut BYTE,
}

impl RenderBuffer {
    pub unsafe fn new(w: i32, h: i32) -> Result<RenderBuffer, Error> {
        let hdc = CreateCompatibleDC(0 as HDC);

        let mut bi: BITMAPINFO = mem::uninitialized();
        bi.bmiHeader.biSize = mem::size_of_val(&bi.bmiHeader) as DWORD;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biWidth = w;
        bi.bmiHeader.biHeight = -h;
        bi.bmiHeader.biCompression = BI_RGB;
        bi.bmiHeader.biPlanes = 1;

        let mut pixels = mem::uninitialized();
        let bitmap = CreateDIBSection(hdc, &bi, DIB_RGB_COLORS, &mut pixels, 0 as HANDLE, 0);
        let prev_bitmap = SelectObject(hdc, bitmap as HGDIOBJ) as HBITMAP;

        let mut fbo = mem::uninitialized();
        gl::GenFramebuffers(1, &mut fbo);

        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

        let mut texture = mem::uninitialized();
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB_ALPHA as i32, w, h, 0, gl::RGBA, gl::UNSIGNED_BYTE, 0 as *const std::os::raw::c_void);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture, 0);

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        Ok(RenderBuffer {
            w: w,
            h: h,
            fbo: fbo,
            texture: texture,
            hdc: hdc,
            bi: bi,
            bitmap: bitmap,
            prev_bitmap: prev_bitmap,
            pixels: pixels as *mut BYTE,
        })
    }

    pub unsafe fn resize(&mut self, w: i32, h: i32) {
        info!("Resize, request: {}x{}, had: {}x{}", w, h, self.w, self.h);
        if self.w < w || self.h < h {
            self.w = std::cmp::max(self.w, w);
            self.h = std::cmp::max(self.h, h);

            self.bi.bmiHeader.biWidth = self.w;
            self.bi.bmiHeader.biHeight = -self.h;
            SelectObject(self.hdc, self.prev_bitmap as HGDIOBJ);
            DeleteObject(self.bitmap as HGDIOBJ);
            let mut pixels = mem::uninitialized();
            self.bitmap = CreateDIBSection(self.hdc, &self.bi, DIB_RGB_COLORS, &mut pixels, 0 as HANDLE, 0);
            self.pixels = pixels as *mut BYTE;
            self.prev_bitmap = SelectObject(self.hdc, self.bitmap as HGDIOBJ) as HBITMAP;

            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB_ALPHA as i32, self.w, self.h, 0, gl::RGBA, gl::UNSIGNED_BYTE, 0 as *const std::os::raw::c_void);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}
*/
