use std::env::var;

fn main() {
    println!(
        "cargo:rustc-env=convert-js-macros-cache={}",
        var("OUT_DIR").unwrap(),
    );
}
