use std::{env, path::PathBuf, process::Command};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
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

    // Rerun if sources change.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/ruby/glue.c");
    println!(
        "cargo:rerun-if-changed={}",
        mruby_dir.join("build/host/lib/libmruby.a").display()
    );
}
