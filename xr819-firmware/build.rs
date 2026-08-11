use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rerun-if-env-changed=XR819_TX_BISECT_STAGE");
    println!("cargo:rerun-if-env-changed=XR819_TX_BISECT_SUBTYPE");
    let bisect_stage = env::var("XR819_TX_BISECT_STAGE")
        .unwrap_or_else(|_| "0".to_owned())
        .parse::<u8>()
        .expect("XR819_TX_BISECT_STAGE must be an integer from 0 through 8");
    assert!(
        bisect_stage <= 8,
        "XR819_TX_BISECT_STAGE must be at most 8"
    );
    println!("cargo:rustc-env=XR819_TX_BISECT_STAGE={bisect_stage}");
    let bisect_subtype = env::var("XR819_TX_BISECT_SUBTYPE")
        .unwrap_or_else(|_| "255".to_owned())
        .parse::<u16>()
        .expect(
            "XR819_TX_BISECT_SUBTYPE must be 0 through 15, 254 for non-authentication, or 255 for all frames",
        );
    assert!(
        bisect_subtype <= 15 || matches!(bisect_subtype, 254 | 255),
        "XR819_TX_BISECT_SUBTYPE must be 0 through 15, 254 for non-authentication, or 255 for all frames"
    );
    println!("cargo:rustc-env=XR819_TX_BISECT_SUBTYPE={bisect_subtype}");
    println!(
        "cargo:rustc-link-arg-bin=mailbox=-T{}",
        manifest_dir.join("link.x").display()
    );
    println!("cargo:rustc-link-arg-bin=mailbox=--nmagic");
    for binary in [
        "download-boot",
        "download-boot-low",
        "download-boot-sectioned",
    ] {
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
    for binary in ["main-mailbox", "hif-startup", "tx_trace_extract"] {
        println!(
            "cargo:rustc-link-arg-bin={binary}=-T{}",
            manifest_dir.join(main_linker).display()
        );
        println!("cargo:rustc-link-arg-bin={binary}=--nmagic");
    }
    println!(
        "cargo:rustc-link-arg-bin=hif-extension-probe=-T{}",
        manifest_dir.join("link-extension.x").display()
    );
    println!("cargo:rustc-link-arg-bin=hif-extension-probe=--nmagic");
    println!("cargo:rerun-if-changed=link.x");
    println!("cargo:rerun-if-changed=link-download.x");
    println!("cargo:rerun-if-changed=link-main.x");
    println!("cargo:rerun-if-changed=link-main-low.x");
    println!("cargo:rerun-if-changed=link-extension.x");
}
