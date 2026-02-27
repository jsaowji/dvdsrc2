fn main() {
    if let Ok(_dir) = std::env::var("MSYSTEM_PREFIX") {
        println!("cargo:rustc-link-lib=static=mpeg2");
    } else {
        println!("cargo:rustc-link-lib=mpeg2");
    }
}
