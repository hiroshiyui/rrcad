use std::{env, path::PathBuf, process::Command};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // -------------------------------------------------------------------------
    // mRuby
    // -------------------------------------------------------------------------
    let mruby_dir = manifest.join("vendor/mruby");
    let lib_path = mruby_dir.join("build/host/lib/libmruby.a");

    // Build mRuby with its own build system (requires `rake`).
    // Skipped if the library already exists to keep incremental builds fast.
    if !lib_path.exists() {
        let status = Command::new("rake")
            .current_dir(&mruby_dir)
            .status()
            .expect(
                "failed to run `rake` — is Ruby (with rake) installed? \
                 Run `gem install rake` if missing.",
            );
        assert!(status.success(), "mRuby build failed");
    }

    // Link libmruby.a.
    println!(
        "cargo:rustc-link-search=native={}",
        mruby_dir.join("build/host/lib").display()
    );
    println!("cargo:rustc-link-lib=static=mruby");
    println!("cargo:rustc-link-lib=m"); // mRuby uses libm on Linux

    // Compile our C glue shims.
    cc::Build::new()
        .file("src/ruby/glue.c")
        .include(mruby_dir.join("include"))
        .compile("rrcad_ruby_glue");

    // -------------------------------------------------------------------------
    // OCCT C++ bridge (cxx)
    // -------------------------------------------------------------------------
    cxx_build::bridge("src/occt/mod.rs")
        .file("src/occt/bridge.cpp")
        .include("/usr/include/opencascade")
        .include(manifest.join("src/occt")) // so bridge.cpp can #include "bridge.h"
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-w") // suppress OCCT header warnings
        .compile("rrcad_occt_bridge");

    // Link OCCT shared libraries.
    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    for lib in &[
        "TKernel",
        "TKMath",
        "TKG2d",
        "TKG3d",
        "TKBRep",
        "TKGeomBase",
        "TKGeomAlgo",
        "TKTopAlgo",
        "TKPrim",
        "TKBool",
        "TKBO",
        "TKFillet",
        "TKOffset",
        "TKShHealing",
        "TKMesh",
        "TKCDF",
        "TKCAF",
        "TKLCAF",
        "TKXCAF",
        "TKXSBase",
        "TKDESTEP",
        "TKDESTL",
        "TKDEGLTF",
        "TKDEOBJ", // RWObj_CafWriter — Wavefront OBJ export
        "TKRWMesh",
    ] {
        println!("cargo:rustc-link-lib={lib}");
    }

    // -------------------------------------------------------------------------
    // Rerun triggers
    // -------------------------------------------------------------------------
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/ruby/glue.c");
    println!("cargo:rerun-if-changed=src/occt/mod.rs");
    println!("cargo:rerun-if-changed=src/occt/bridge.h");
    println!("cargo:rerun-if-changed=src/occt/bridge.cpp");
    println!(
        "cargo:rerun-if-changed={}",
        mruby_dir.join("build/host/lib/libmruby.a").display()
    );
}
