#[macro_use]
extern crate log;
extern crate env_logger;

extern crate winapi;
extern crate kernel32;
extern crate user32;

use std::mem;

use winapi::*;
use kernel32::*;
use user32::*;

macro_rules! wstr {
    ($str:expr) => ({
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new($str).encode_wide()
                        .chain(Some(0).into_iter())
                        .collect::<Vec<_>>()
    });
}

pub struct Window {
    hwnd: HWND,

    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Window {
}

#[allow(unused_variables)]
unsafe extern "system"
fn window_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let window = &mut *(GetWindowLongPtrW(hwnd, 0) as (*mut Window));

    match msg {
        WM_CREATE => {
            let cs = &*(lparam as (*const CREATESTRUCTW));
            let window = &mut *(cs.lpCreateParams as (*mut Window));
            SetWindowLongPtrW(hwnd, 0, window as (*mut Window) as LONG_PTR);

            window.hwnd = hwnd;

            let mut rect = mem::uninitialized();
            GetWindowRect(hwnd, &mut rect);

            window.x = rect.left;
            window.y = rect.top;
            window.w = rect.right - rect.left;
            window.h = rect.bottom - rect.top;

            SetWindowLongPtrW(hwnd, GWL_STYLE, winuser::WS_POPUP as LONG_PTR);
            //SetWindowLongPtrW(hwnd, GWL_EXSTYLE, winuser::WS_EX_LAYERED as LONG_PTR);
        }

        WM_CLOSE => {
            PostQuitMessage(0);
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
