fn main() {
    if let Ok(dir) = std::env::var("MSYSTEM_PREFIX") {
        println!("cargo:rustc-link-search=native={}/lib", dir);
        println!("cargo:rustc-link-lib=static=a52");
    } else {
        println!("cargo:rustc-link-lib=a52");
    }
}
