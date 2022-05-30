fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!(
        "cargo:rustc-env=CONVERT_JS_MACROS_CACHE={}",
        std::env::var("OUT_DIR").unwrap(),
    );
}
