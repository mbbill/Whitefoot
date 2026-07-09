use std::env;
use std::path::PathBuf;

fn main() {
    // Link against the separately-built cdylib (true dynamic boundary).
    let mut dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dir.push("target");
    dir.push(env::var("PROFILE").unwrap());
    println!("cargo:rustc-link-search=native={}", dir.display());
    println!("cargo:rustc-link-lib=dylib=puredylib");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
}
