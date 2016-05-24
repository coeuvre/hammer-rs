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
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            x: 0,
            y: 0,
            w: 800,
            h: 600,
        }
    }

    pub fn pos(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn size(mut self, w: i32, h: i32) -> Self {
        self.w = w;
        self.h = h;
        self
    }

    pub fn build(self) -> Result<Window, Error> {
        WINDOW_THREAD_INIT.call_once(|| {
            thread::spawn(move || {
                unsafe { window_thread_main(); }
            });

            unsafe { while WINDOW_THREAD_ID == 0 as DWORD {} }
        });

        let (tx, rx) = channel();
        let create_window_param = CreateWindowParam {
            builder: &self,
            tx: tx,
        };

        unsafe {
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
        unsafe { ShowWindow((*self.state).hwnd, SW_SHOW); }
    }

    pub fn wait_for_close(&self) {
        loop {
            match self.event_rx.recv().unwrap() {
                Event::Close => {
                    info!("Event::Close");
                    break;
                }
            }
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

pub struct WindowState {
    event_tx: Sender<Event>,

    hwnd: HWND,

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

            x: 0,
            y: 0,
            w: 0,
            h: 0,
        }
    }
}

#[allow(unused_variables)]
unsafe extern "system"
fn window_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let state = &mut *(GetWindowLongPtrW(hwnd, 0) as (*mut WindowState));

    match msg {
        WM_CREATE => {
            let cs = &*(lparam as (*mut CREATESTRUCTW));
            let state = &mut *(cs.lpCreateParams as (*mut WindowState));
            SetWindowLongPtrW(hwnd, 0, state as (*mut WindowState) as LONG_PTR);

            state.hwnd = hwnd;

            let mut rect = mem::uninitialized();
            GetWindowRect(hwnd, &mut rect);

            state.x = rect.left;
            state.y = rect.top;
            state.w = rect.right - rect.left;
            state.h = rect.bottom - rect.top;

            SetWindowLongPtrW(hwnd, GWL_STYLE, WS_POPUP as LONG_PTR);
            //SetWindowLongPtrW(hwnd, GWL_EXSTYLE, WS_EX_LAYERED as LONG_PTR);
            /*
            window.context = RenderContext::new(GetDC(hwnd)).unwrap();
            window.buffer = RenderBuffer::new(&window.context, window.w, window.h).unwrap();

            window.context.make_current().unwrap();

            window.render().unwrap();
            */
            ShowWindow(hwnd, SW_SHOW);
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

static WM_CREATE_WINDOW: UINT = WM_USER;
static WINDOW_THREAD_INIT: Once = ONCE_INIT;
static mut WINDOW_THREAD_ID: DWORD = 0 as DWORD;

struct CreateWindowParam<'a> {
    builder: &'a WindowBuilder,
    tx: Sender<Result<Window, Error>>,
}

unsafe fn window_thread_main() {
    let hinstance = GetModuleHandleW(0 as LPCWSTR);

    let class_name = wstr!("HAMMEREDITORWINDOWCLASS");

    let mut wc: WNDCLASSEXW = mem::zeroed();
    wc.cbSize = mem::size_of_val(&wc) as UINT;
    wc.style = 0;
    wc.lpfnWndProc = Some(window_proc);
    wc.hCursor = LoadCursorW(0 as HINSTANCE, winapi::winuser::IDC_ARROW);
    wc.hInstance = hinstance;
    wc.lpszClassName = class_name.as_ptr();
    wc.cbWndExtra = mem::size_of::<Window>() as winapi::c_int;

    if RegisterClassExW(&wc) == 0 {
        panic!("Failed to register window class.");
    }

    WINDOW_THREAD_ID = GetCurrentThreadId();

    let mut msg = mem::uninitialized();
    while GetMessageW(&mut msg, 0 as HWND, 0, 0) != 0 {
        if msg.message == WM_CREATE_WINDOW {
            let create_window_param = &*(msg.lParam as *const CreateWindowParam);
            let result = create_window(hinstance, &class_name, create_window_param.builder);
            create_window_param.tx.send(result).unwrap();
        } else {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe fn create_window(hinstance: HINSTANCE, class_name: &Vec<u16>, builder: &WindowBuilder) -> Result<Window, Error> {
    let (event_tx, event_rx) = channel();

    let state = Box::into_raw(Box::new(WindowState::new(event_tx)));

    let title = wstr!("Hammer Editor");
    CreateWindowExW(
        0,
        class_name.as_ptr(),
        title.as_ptr(),
        0,
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
