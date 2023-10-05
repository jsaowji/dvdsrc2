use std::path::{Path, PathBuf};

fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

fn main() {
    println!("cargo:rustc-link-lib=dvdread");
    println!("cargo:rustc-link-lib=mpeg2");
    println!("cargo:rustc-link-lib=a52");

    let target = std::env::var("TARGET").unwrap();

    if target == "x86_64-pc-windows-gnu" {
        println!(
            "cargo:rustc-link-search={}",
            workspace_dir().join("windows_libs").to_string_lossy()
        );

        // cc::Build::new()
        //  //   .compiler("zig cc")
        //     .file("jsonstuff.cpp")
        //     .include(format!("{}/libdvdread-6.1.3/src",workspace_dir().join("windows_libs").to_string_lossy()))
        //     .compile("jsonstuff");
    }
    //    println!("cargo:rustc-link-lib=jsonstuff");
}
