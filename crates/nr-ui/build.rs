use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(nightrun_japanese_font)");

    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let source = root.join("build/nightrun-fonts");
    let files = ["noto-jp-16.psf", "noto-jp-24.psf", "noto-jp-32.psf"];
    for file in files {
        println!("cargo:rerun-if-changed={}", source.join(file).display());
    }
    println!("cargo:rerun-if-env-changed=NIGHTRUN_DISABLE_JAPANESE_FONT");

    if env::var_os("NIGHTRUN_DISABLE_JAPANESE_FONT").is_some()
        || !source.join(".complete").is_file()
        || !files.iter().all(|file| source.join(file).is_file())
    {
        return;
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    for file in files {
        fs::copy(source.join(file), out.join(file)).expect("copy generated Japanese font");
    }
    println!("cargo:rustc-cfg=nightrun_japanese_font");
}
