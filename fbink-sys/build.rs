// adapted from https://rust-lang.github.io/rust-bindgen/non-system-libraries.html
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let fbink_root = PathBuf::from("FBInk")
        // `rustc-link-search` requires an absolute path.
        .canonicalize()
        .expect("cannot canonicalize path");

    let src_files = [
        "fbink.c",
        "qimagescale/qimagescale.c",
        "cutef8/dfa.c",
        "cutef8/utf8.c",
        "i2c-tools/lib/smbus.c",
        "libunibreak/src/linebreak.c",
        "libunibreak/src/linebreakdata.c",
        "libunibreak/src/unibreakdef.c",
        "libunibreak/src/linebreakdef.c",
    ];
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib_path = out_dir.join("libfbink.a");
    // Tell cargo to look for libraries in the specified directory
    println!("cargo:rustc-link-search={}", out_dir.to_str().unwrap());
    // Tell cargo to tell rustc to link our `libfbink.a`
    println!("cargo:rustc-link-lib=static=fbink");

    // Compile the C source files into object files using clang.
    let mut obj_paths = Vec::new();
    for src_file in src_files.iter() {
        let mut args = Vec::new();
        let src_path = fbink_root.join(src_file);
        let obj_path = out_dir
            .join(src_path.file_name().unwrap())
            .with_extension("o");
        if *src_file != "fbink.c" {
            args.push("-fvisibility=hidden");
        };
        compile_object(&obj_path, &src_path, &args);
        obj_paths.push(obj_path);
    }

    // Run `ar` to generate the `libfbink.a` file from the object files.
    let output = std::process::Command::new("ar")
        .arg("rcs")
        .arg(lib_path)
        .args(obj_paths)
        .output()
        .expect("could not spawn `ar`");
    if !output.status.success() {
        eprintln!("{}", String::from_utf8(output.stderr).unwrap());
        println!("{}", String::from_utf8(output.stdout).unwrap());
        panic!("could not emit library file");
    }
    let output = std::process::Command::new("printenv").output().unwrap();
    println!("{}", String::from_utf8(output.stdout).unwrap());

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for.
        .header("FBInk/fbink.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Only generate bindings for things declared in fbink.h
        .allowlist_file("FBInk/fbink.h")
        // Make the comments from fbink.h appear as doc comments for our bindings
        .clang_arg("-fparse-all-comments")
        .derive_default(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn compile_object(obj_path: &Path, src_path: &Path, args: &[&str]) {
    let mut toolchain_args = Vec::new();
    // Only set these args if cross-compiling so the docs can still be built on the host.
    // They both need to be set for cross-compilation to produce a working binary.
    if let Ok(sysroot_path) = env::var("CROSS_SYSROOT_PATH") {
        toolchain_args.push(format!("--sysroot={sysroot_path}"));
    }
    if let Ok(include_path) = env::var("CROSS_INCLUDE_PATH") {
        toolchain_args.push(format!("-I{include_path}"));
    };
    let output = std::process::Command::new("clang")
        .arg("-c")
        .arg("-static")
        .arg("-IFBInk/i2c-tools/include")
        .arg("-target")
        .arg(env::var("TARGET").unwrap())
        .args(toolchain_args)
        .args(args)
        .arg("-o")
        .arg(obj_path)
        .arg(src_path)
        .output()
        .expect("could not spawn `clang`");
    if !output.status.success() {
        eprintln!("{}", String::from_utf8(output.stderr).unwrap());
        println!("{}", String::from_utf8(output.stdout).unwrap());
        panic!("could not compile object file");
    }
}
