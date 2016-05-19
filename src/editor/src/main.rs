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

use winapi::basetsd::*;
use winapi::c_int;
use winapi::minwindef::*;
use winapi::windef::*;
use winapi::wingdi::*;
use winapi::winnt::*;
use winapi::winuser::*;
use gdi32::*;
use kernel32::*;
use opengl32::*;
use user32::*;

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

pub struct Window {
    hwnd: HWND,

    context: RenderContext,

    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Window {
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
}

#[allow(unused_variables)]
unsafe extern "system"
fn window_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let window = &mut *(GetWindowLongPtrW(hwnd, 0) as (*mut Window));

    match msg {
        WM_CREATE => {
            let cs = &*(lparam as (*mut CREATESTRUCTW));
            let window = &mut *(cs.lpCreateParams as (*mut Window));
            SetWindowLongPtrW(hwnd, 0, window as (*mut Window) as LONG_PTR);

            window.hwnd = hwnd;

            let mut rect = mem::uninitialized();
            GetWindowRect(hwnd, &mut rect);

            window.x = rect.left;
            window.y = rect.top;
            window.w = rect.right - rect.left;
            window.h = rect.bottom - rect.top;

            SetWindowLongPtrW(hwnd, GWL_STYLE, WS_POPUP as LONG_PTR);
            //SetWindowLongPtrW(hwnd, GWL_EXSTYLE, winuser::WS_EX_LAYERED as LONG_PTR);

            window.context = RenderContext::new(GetDC(hwnd)).unwrap();
        }

        WM_PAINT => {
            window.context.make_current().unwrap();
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            window.context.swap_buffers();
        }

        WM_CLOSE => {
            PostQuitMessage(0);
        }

        WM_KEYDOWN => {
            let key = wparam as c_int;
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

fn main() {
    env_logger::init().unwrap();

    unsafe {
        let hinstance = GetModuleHandleW(0 as LPCWSTR);
        let window_class_name = wstr!("HAMMEREDITORWINDOWCLASS");

        let mut wc: WNDCLASSEXW = mem::zeroed();
        wc.cbSize = mem::size_of_val(&wc) as UINT;
        wc.style = 0;
        wc.lpfnWndProc = Some(window_proc);
        wc.hCursor = LoadCursorW(0 as HINSTANCE, winapi::winuser::IDC_ARROW);
        wc.hInstance = hinstance;
        wc.lpszClassName = window_class_name.as_ptr();
        wc.cbWndExtra = mem::size_of::<Window>() as c_int;

        if RegisterClassExW(&wc) == 0 {
            error!("Failed to register window class.");
        }

        let window: Window = mem::zeroed();

        let title = wstr!("Hammer Editor");
        let hwnd = CreateWindowExW(
            0,
            window_class_name.as_ptr(),
            title.as_ptr(),
            0,
            CW_USEDEFAULT, CW_USEDEFAULT,
            CW_USEDEFAULT, CW_USEDEFAULT,
            0 as HWND,
            0 as HMENU,
            hinstance,
            mem::transmute(&window)
        );

        ShowWindow(hwnd, SW_SHOW);

        let mut msg = mem::uninitialized();
        while GetMessageW(&mut msg, 0 as HWND, 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
