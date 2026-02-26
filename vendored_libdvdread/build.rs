use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use fs_extra::dir::CopyOptions;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    if true {
        println!("cargo::rerun-if-changed=dvdread/");

        let out_dir = std::env::var_os("OUT_DIR").expect("missing OUT_DIR");
        let out_dir = Path::new(&out_dir);

        std::fs::remove_dir_all(out_dir).unwrap();
        std::fs::create_dir(out_dir).unwrap();

        let out_src = out_dir.join("src");
        std::fs::create_dir(&out_src).unwrap();
        let out_build = out_dir.join("build");
        std::fs::create_dir(&out_build).unwrap();
        let out_install = out_dir.join("install");
        std::fs::create_dir(&out_install).unwrap();

        assert!(Path::new("dvdread/README.md").exists());

        fs_extra::dir::copy("dvdread/", &out_src, &CopyOptions::new().content_only(true)).unwrap();

        let o0 = Command::new("patch")
            .stderr(std::io::stderr())
            .stdout(std::io::stderr())
            .stdin(Stdio::piped())
            .current_dir(&out_src)
            .args(["-p1"])
            .spawn()
            .expect("fail");
        let mut stdd = o0.stdin.as_ref().unwrap();

        stdd.write(
            std::fs::read_to_string("libdvdread_patch.patch")
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        stdd.flush().unwrap();
        let o0 = o0.wait_with_output().unwrap();

        assert!(o0.status.success());
        let o1 = Command::new("meson")
            .stderr(std::io::stderr())
            .stdout(std::io::stderr())
            .current_dir(&out_src)
            .args(["setup"])
            .arg(&out_build)
            .arg("--prefix")
            .arg(std::path::absolute(&out_install).unwrap())
            //.args(&["-Dlibdvdcss=disabled"])
            .args(&["-Dbuildtype=release","-Dlibdir=lib","-Dlibdvdcss=disabled"])
            .output()
            .expect("fail");
        assert!(o1.status.success());

        let o2 = Command::new("meson")
            .stderr(std::io::stderr())
            .stdout(std::io::stderr())
            .current_dir(&out_src)
            .args(["compile", "-C"])
            .arg(&out_build)
            .output()
            .expect("fail");
        assert!(o2.status.success());
        let o3 = Command::new("meson")
            .stderr(std::io::stderr())
            .stdout(std::io::stderr())
            .current_dir(&out_src)
            .args(["install", "-C"])
            .arg(&out_build)
            .output()
            .expect("fail");
        assert!(o3.status.success());
        let dst = out_install.join("lib");
        println!("cargo:rustc-link-search=native={}", dst.display());
    }

    if let Ok(dir) = std::env::var("MSYSTEM_PREFIX") {
        println!("cargo:rustc-link-search=native={}/lib", dir);
        println!("cargo:rustc-link-lib=static=dvdread");
        println!("cargo:rustc-link-lib=static=dvdcss");
    } else {
        println!("cargo:rustc-link-lib=static=dvdread");
        //println!("cargo:rustc-link-lib=dvdcss");
    }

    //if let Ok(dir) = std::env::var("MSYSTEM_PREFIX") {
    //   //    println!("cargo:rustc-link-search=native={}/lib", dir);
    //       println!("cargo:rustc-link-lib=static=dvdread");
    //   } else {
    //       println!("cargo:rustc-link-lib=dvdread");
    //   }
}
