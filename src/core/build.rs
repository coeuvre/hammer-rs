extern crate gcc;

fn main() {
    gcc::compile_library("libstb_image.a", &["src/util/stb_image.c"]);
    println!("cargo:rustc-link-lib=static=stb_image");
}
