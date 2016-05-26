#![allow(unused_imports)]
#![feature(const_fn)]

#[macro_use]
extern crate lazy_static;

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

use window::*;

#[macro_use]
mod window;

pub type Error = Box<std::error::Error + Send + Sync>;

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
*/

fn main() {
    env_logger::init().unwrap();

    let mut win1 = WindowBuilder::new().title("Window 1").build().unwrap();
    win1.show();

    let handle = thread::spawn(move || {
        let mut win2 = WindowBuilder::new().title("Window 2").pos(0, 600).build().unwrap();
        win2.show();

        // Game like loop
        'event_loop: loop {
            for event in win2.poll_events() {
                match event {
                    Event::Close => break 'event_loop,
                }
            }
            thread::yield_now();
        }

        win2.close();
    });

    // GUI like loop
    for event in win1.wait_events() {
        match event {
            Event::Close => break,
        }
    }
    win1.close();

    handle.join().unwrap();
}
