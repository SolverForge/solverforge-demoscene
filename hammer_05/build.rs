// Compiles the vendored glyphfx engine (vendor/glyphfx, pinned v0.3.4) plus
// the demo's C shim into one static library.
use std::fs;
use std::path::{Path, PathBuf};

fn c_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read vendor dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            c_sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "c") {
            out.push(path);
        }
    }
}

fn main() {
    let vendor = Path::new("vendor/glyphfx");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));

    let version = fs::read_to_string(vendor.join("VERSION")).expect("glyphfx VERSION");
    fs::write(
        out_dir.join("glyphfx_version.h"),
        format!(
            "#ifndef GLYPHFX_VERSION_H\n#define GLYPHFX_VERSION_H\n#define GLYPHFX_VERSION \"{}\"\n#endif\n",
            version.trim()
        ),
    )
    .expect("write version header");

    let mut sources = Vec::new();
    c_sources(&vendor.join("src"), &mut sources);
    sources.sort();

    cc::Build::new()
        .std("c17")
        .opt_level(3)
        .define("_POSIX_C_SOURCE", "200809L")
        .define("_DEFAULT_SOURCE", None)
        .define("NDEBUG", None)
        .include(vendor.join("src"))
        .include(&out_dir)
        .warnings(false)
        // The effect registry references entries weakly; whole-archive keeps
        // every effect object in the link.
        .link_lib_modifier("+whole-archive")
        .files(&sources)
        .file("csrc/gfx_shim.c")
        .compile("glyphfx");

    println!("cargo:rustc-link-lib=m");
    println!("cargo:rerun-if-changed=csrc/gfx_shim.c");
    println!("cargo:rerun-if-changed=vendor/glyphfx");
}
