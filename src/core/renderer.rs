use libc::c_void;
use std::ptr;

use sdl2::render::{
    SDL_Renderer, SDL_Texture, SDL_UpdateTexture, SDL_RenderCopyEx, SDL_RenderPresent,
    SDL_FLIP_VERTICAL,
};

pub struct Renderer {
    pub renderer: *mut SDL_Renderer,
    pub texture: *mut SDL_Texture,
}

impl Renderer {
    pub unsafe fn present(&mut self, texture: &Texture2) {
        SDL_UpdateTexture(self.texture, ptr::null(), texture.pixels as *mut c_void, texture.width * 4);
        SDL_RenderCopyEx(self.renderer, self.texture, ptr::null(), ptr::null(), 0.0, ptr::null(), SDL_FLIP_VERTICAL);
        SDL_RenderPresent(self.renderer);
    }
}

pub struct Texture2 {
    // The pixels is in sRGB color space with pre-multiplied alpha.
    // The coordinates are bottom-up which means the first row pointer
    // points to the bottom-most row of raw bitmap.
    //
    // Bit pattern: 0xRRGGBBAA
    pub pixels: *mut u32,

    pub width: i32,
    pub height: i32,
    pub pitch: isize,
}
