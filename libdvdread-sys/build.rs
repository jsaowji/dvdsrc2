use pkg_config::Config;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    Config::new()
        //.atleast_version("7.1.0")
        .probe("dvdread")
        .unwrap();
}
