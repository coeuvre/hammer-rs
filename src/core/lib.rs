extern crate libc;
extern crate sdl2_sys as sdl2;

use std::ptr;
use std::mem;

use renderer::*;

pub mod renderer;

macro_rules! cstr {
    ($str:expr) => ({
        use std::ffi::CString;
        CString::new($str).unwrap().as_ptr()
    });
}

pub trait Game {
    fn update();
}

#[derive(Default)]
pub struct Config {
    pub window: WindowConfig,
    pub memory: MemoryConfig,
    pub debug: DebugConfig,
}

#[derive(Default)]
pub struct WindowConfig {
    pub title: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Default)]
pub struct MemoryConfig {
    pub size: MemorySizeConfig,
}

#[derive(Default)]
pub struct MemorySizeConfig {
    pub perm: usize,
    pub tran: usize,
}

#[derive(Default)]
pub struct DebugConfig {
    pub is_exit_on_esc: bool,
}

pub fn run<G: Game>(config: Config) {
    unsafe {
        run_unsafe::<G>(config);
    }
}

unsafe fn run_unsafe<G: Game>(config: Config) {
    use sdl2::sdl::{SDL_Init, SDL_INIT_VIDEO};
    use sdl2::video::{SDL_CreateWindow, SDL_WINDOWPOS_CENTERED};
    use sdl2::video::SDL_WindowFlags::{SDL_WINDOW_OPENGL};
    use sdl2::render::{
        SDL_CreateRenderer,
        SDL_CreateTexture,
        SDL_RENDERER_ACCELERATED,
        SDL_RENDERER_PRESENTVSYNC,
        SDL_TEXTUREACCESS_STREAMING
    };
    use sdl2::pixels::{SDL_PIXELFORMAT_RGBA8888};
    use sdl2::event::{SDL_PollEvent, SDL_QUIT};
    use sdl2::keyboard::{SDL_GetKeyboardState};
    use sdl2::scancode::*;

    if SDL_Init(SDL_INIT_VIDEO) != 0 {
        //HM_LOG_ERROR("Failed to initialize SDL: %s\n", SDL_GetError());
        return;
    }

    let window = SDL_CreateWindow(cstr!(config.window.title),
                                  SDL_WINDOWPOS_CENTERED,
                                  SDL_WINDOWPOS_CENTERED,
                                  config.window.width,
                                  config.window.height,
                                  SDL_WINDOW_OPENGL as u32);
    if window == ptr::null_mut() {
        //HM_LOG_ERROR("Failed to create window: %s\n", SDL_GetError());
        return;
    }

    let sdl_renderer = SDL_CreateRenderer(window, -1,
                                          SDL_RENDERER_ACCELERATED |
                                          SDL_RENDERER_PRESENTVSYNC);

    let sdl_texture = SDL_CreateTexture(sdl_renderer,
                                        SDL_PIXELFORMAT_RGBA8888,
                                        SDL_TEXTUREACCESS_STREAMING as i32,
                                        config.window.width, config.window.height);

    let mut renderer = Renderer {
        renderer: sdl_renderer,
        texture: sdl_texture,
    };

    let mut buffer = vec![0 as u32; (config.window.width * config.window.height) as usize];

    let framebuffer = Texture2 {
        pixels: buffer.as_mut_ptr(),
        width: config.window.width,
        height: config.window.height,
        pitch: config.window.width as isize * 4,
    };

    let mut quit = false;
    while !quit {
        let mut e = mem::uninitialized();
        while SDL_PollEvent(&mut e) != 0 {
            match *e.type_() {
                SDL_QUIT => {
                    quit = true;
                }

                _ => {}
            }
        }

        let mut key_count = mem::uninitialized();
        let keys = SDL_GetKeyboardState(&mut key_count);
        /*
        for key_index in 0..key_count {
            let key = *keys.offset(key_index as isize);
        }
        */


        if config.debug.is_exit_on_esc && *keys.offset(SDL_SCANCODE_ESCAPE as isize) != 0 {
            quit = true;
        }

        G::update();

        renderer.present(&framebuffer);
    }
}
