extern crate hammer;

use std::default::Default;

pub struct Demo;

impl hammer::Game for Demo {
    fn update() {
    }
}

fn main() {
    let mut config = hammer::Config::default();
    config.window.title = "Hammer Demo".to_string();
    config.window.width = 980;
    config.window.height = 540;
    config.debug.is_exit_on_esc = true;
    hammer::run::<Demo>(config);
}
