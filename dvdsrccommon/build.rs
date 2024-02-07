fn main() {
    // Make sure the build script is re-run if our env variable is changed.
    // println!("cargo:rerun-if-env-changed={}", LIBRARY_DIR_VARIABLE);
    if let Ok(dir) = std::env::var("MSYSTEM_PREFIX") {
        println!("cargo:rustc-link-search=native={}/lib", dir);
        println!("cargo:rustc-link-lib=static=dvdcss");
        println!("cargo:rustc-link-lib=static=dvdread");
        println!("cargo:rustc-link-lib=static=mpeg2");
        println!("cargo:rustc-link-lib=static=a52");
    } else {
        println!("cargo:rustc-link-lib=dvdread");
        println!("cargo:rustc-link-lib=mpeg2");
        println!("cargo:rustc-link-lib=a52");
    }
}
