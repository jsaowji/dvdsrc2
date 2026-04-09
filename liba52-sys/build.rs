use pkg_config::Config;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    Config::new().probe("liba52").unwrap();
}
