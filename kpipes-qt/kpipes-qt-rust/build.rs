use std::env;
use std::fs::create_dir;
use std::path::Path;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let ffi_dir = Path::new(&crate_dir)
        .join("..")
        .join("..")
        .join("target")
        .join(&target)
        .join(&profile)
        .join("ffi");

    println!("FFI Dir: {:?}", &ffi_dir);

    if !ffi_dir.exists() {
        create_dir(&ffi_dir).unwrap();
    }

    let bindings_file = ffi_dir.join("kpipes_qt_rust.h");

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&bindings_file);
}
