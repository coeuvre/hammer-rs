extern crate typemap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod prelude;

pub mod asset;
pub mod scene;
pub mod window;
pub mod renderer;
pub mod math;
pub mod util;
pub mod input;

pub type Error = Box<std::error::Error + Send + Sync>;

use std::collections::VecDeque;
use math::Trans;
use scene::*;

pub fn run<T: HasScene>(mut scene: T) {
    let mut window = window::WindowBuilder::new().size(800, 600).build().unwrap();
    window.show();

    let mut renderer = renderer::Renderer::new(&window).unwrap();

    scene.start();

    let view_w = 640.0;
    let view_h = 480.0;
    renderer.ortho(0.0, view_w, 0.0, view_h);

    'game_loop: loop {
        for event in window.poll_events() {
            match event {
                window::Event::Close => break 'game_loop,
                _ => {}
            }
        }

        scene.update(&input::Input{});

        renderer.clear(0.0, 0.0, 0.0, 1.0);

        let mut queue = VecDeque::new();
        queue.push_back((Trans::identity(), scene.scene().root()));

        while queue.len() > 0 {
            let (trans, node) = queue.pop_front().unwrap();
            renderer.set_trans(trans);
            node.render(&mut renderer);
            let trans = *renderer.trans();
            queue.extend(node.node().children().map(|node| (trans, &**node)));
        }

        renderer.present();
    }
}
