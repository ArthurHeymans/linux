use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    println!(
        "cargo:rustc-link-arg-bin=mailbox=-T{}",
        manifest_dir.join("link.x").display()
    );
    println!("cargo:rustc-link-arg-bin=mailbox=--nmagic");
    for binary in ["download-boot", "download-boot-low"] {
        println!(
            "cargo:rustc-link-arg-bin={binary}=-T{}",
            manifest_dir.join("link-download.x").display()
        );
        println!("cargo:rustc-link-arg-bin={binary}=--nmagic");
    }
    let target = env::var("TARGET").unwrap();
    let main_linker = if target.starts_with("thumbv5te-") {
        "link-main-low.x"
    } else {
        "link-main.x"
    };
    for binary in ["main-mailbox", "hif-startup"] {
        println!(
            "cargo:rustc-link-arg-bin={binary}=-T{}",
            manifest_dir.join(main_linker).display()
        );
        println!("cargo:rustc-link-arg-bin={binary}=--nmagic");
    }
    println!("cargo:rerun-if-changed=link.x");
    println!("cargo:rerun-if-changed=link-download.x");
    println!("cargo:rerun-if-changed=link-main.x");
    println!("cargo:rerun-if-changed=link-main-low.x");
}
