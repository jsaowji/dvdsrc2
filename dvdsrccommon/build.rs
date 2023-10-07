//use std::path::{Path, PathBuf};

//fn workspace_dir() -> PathBuf {
//    let output = std::process::Command::new(env!("CARGO"))
//        .arg("locate-project")
//        .arg("--workspace")
//        .arg("--message-format=plain")
//        .output()
//        .unwrap()
//        .stdout;
//    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
//    cargo_path.parent().unwrap().to_path_buf()
//}

//const LIBRARY_DIR_VARIABLE: &str = "WINDOWSLIB_DIR";

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

    //let target = std::env::var("TARGET").unwrap();

    //    if target == "x86_64-pc-windows-gnu" {
    //        println!("cargo:rustc-link-lib=static=dvdread");
    //        println!("cargo:rustc-link-lib=static=mpeg2");
    //
    //     //does not have pkg config on msys2
    //        println!("cargo:rustc-link-lib=static=a52");
    //
    //        // cc::Build::new()
    //        //  //   .compiler("zig cc")
    //        //     .file("jsonstuff.cpp")
    //        //     .include(format!("{}/libdvdread-6.1.3/src",workspace_dir().join("windows_libs").to_string_lossy()))
    //        //     .compile("jsonstuff");
    //    } else {
    //     //   println!("cargo:rustc-link-lib=dvdread");
    //     //   println!("cargo:rustc-link-lib=mpeg2");
    //     //   println!("cargo:rustc-link-lib=a52");
    //    }
    //    println!("cargo:rustc-link-lib=jsonstuff");
}
