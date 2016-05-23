extern crate gl;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate winapi;
extern crate gdi32;
extern crate kernel32;
extern crate opengl32;
extern crate user32;

use std::mem;
use std::ffi::{CString, CStr};

use std::sync::mpsc::*;
use std::thread;

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

use gl::types::*;

//mod ui;

pub type Error = Box<std::error::Error + Send + Sync>;

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
        let (tx, rx) = channel();
        thread::spawn(move || {
            unsafe { window_thread_main(self, tx); }
        });
        rx.recv().unwrap()
    }
}

pub struct Window {
    /*
    context: RenderContext,
    buffer: RenderBuffer,
    */

    quit_rx: Receiver<()>,
    state: *const WindowState,
}

pub struct WindowState {
    quit_tx: Option<Sender<()>>,

    hwnd: HWND,

    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

unsafe impl Send for Window {}

impl Window {
    pub fn show(&mut self) {
        unsafe { ShowWindow((*self.state).hwnd, SW_SHOW); }
    }

    pub fn wait(&self) {
        self.quit_rx.recv().unwrap();
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

pub struct RenderContext {
    lib: HMODULE,
    hdc: HDC,
    hglrc: HGLRC,
    _pfd: PIXELFORMATDESCRIPTOR,
}

impl RenderContext {
    pub unsafe fn new(hdc: HDC) -> Result<RenderContext, Error> {
        let name = wstr!("opengl32.dll");
        let lib = LoadLibraryW(name.as_ptr());
        if lib != 0 as HMODULE {
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
            if pfi != 0 {
                if SetPixelFormat(hdc, pfi, &mut pfd) != 0 {
                    let hglrc = wglCreateContext(hdc);

                    if hglrc != 0 as HGLRC {
                        Ok(RenderContext {
                            lib: lib,
                            hdc: hdc,
                            hglrc: hglrc,
                            _pfd: pfd,
                        })
                    } else {
                        Err(From::from("Failed to create OpenGL context"))
                    }
                } else {
                    Err(From::from("Failed to set pixel format"))
                }
            } else {
                Err(From::from("Failed to choose pixel format"))
            }
        } else {
            Err(From::from("Failed to load opengl32.dll"))
        }
    }

    pub unsafe fn make_current(&self) -> Result<(), Error> {
        if wglGetCurrentContext() != self.hglrc {
            if wglMakeCurrent(self.hdc, self.hglrc) != 0 {
                gl::load_with(|symbol| {
                    let cstr = CString::new(symbol).unwrap();
                    let mut ptr = wglGetProcAddress(cstr.as_ptr());
                    if ptr == 0 as PROC {
                        ptr = GetProcAddress(self.lib, cstr.as_ptr());
                    }
                    ptr
                });
                let version = CStr::from_ptr(mem::transmute(gl::GetString(gl::VERSION)));
                println!("OpenGL Version: {:?}", version);
                Ok(())
            } else {
                Err(From::from("Failed to make context be current"))
            }
        } else {
            Ok(())
        }
    }

    pub unsafe fn swap_buffers(&self) {
        SwapBuffers(self.hdc);
    }

    pub unsafe fn resize(&mut self, w: i32, h: i32) {
        gl::Viewport(0, 0, w, h);
    }
}

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
    pub unsafe fn new(context: &RenderContext, w: i32, h: i32) -> Result<RenderBuffer, Error> {
        try!(context.make_current());

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
        if self.w < w || self.h < h {
            self.w = w;
            self.h = h;

            self.bi.bmiHeader.biWidth = w;
            self.bi.bmiHeader.biHeight = -h;
            SelectObject(self.hdc, self.prev_bitmap as HGDIOBJ);
            DeleteObject(self.bitmap as HGDIOBJ);
            self.bitmap = CreateDIBSection(self.hdc, &self.bi, DIB_RGB_COLORS, &mut (self.pixels as *mut winapi::c_void), 0 as HANDLE, 0);
            self.prev_bitmap = SelectObject(self.hdc, self.bitmap as HGDIOBJ) as HBITMAP;

            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB_ALPHA as i32, w, h, 0, gl::RGBA, gl::UNSIGNED_BYTE, 0 as *const std::os::raw::c_void);
            gl::BindTexture(gl::TEXTURE_2D, 0);
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
            PostQuitMessage(0);
        }

        WM_KEYDOWN => {
            let key = wparam as winapi::c_int;
            if key == VK_ESCAPE {
                PostQuitMessage(0);
            }
        }

        _ => {
            return DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }

    return 0;
}

static WINDOW_CLASS_REGISTER: std::sync::Once = std::sync::ONCE_INIT;

unsafe fn window_thread_main(builder: WindowBuilder, tx: Sender<Result<Window, Error>>) {
    let hinstance = GetModuleHandleW(0 as LPCWSTR);

    let class_name = wstr!("HAMMEREDITORWINDOWCLASS");

    WINDOW_CLASS_REGISTER.call_once(|| {
        info!("Registing window class");

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
    });

    let (quit_tx, quit_rx) = channel();

    let state: *mut WindowState = Box::into_raw(Box::new(mem::zeroed()));
    (*state).quit_tx = Some(quit_tx);

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
        quit_rx: quit_rx,
        state: state,
    };

    tx.send(Ok(window)).unwrap();

    let mut msg = mem::uninitialized();
    while GetMessageW(&mut msg, 0 as HWND, 0, 0) != 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    (*state).quit_tx.as_mut().unwrap().send(()).unwrap();
}

fn main() {
    env_logger::init().unwrap();

    let mut win1 = WindowBuilder::new().build().unwrap();
    win1.show();

    let mut win2 = WindowBuilder::new().pos(0, 600).build().unwrap();
    win2.show();

    win1.wait();
    win2.wait();
}
