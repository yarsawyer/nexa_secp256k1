use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Navigate to the C library directory
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_dir = format!("{}/depends/secp256k1", crate_dir);

    // Run autogen.sh
    Command::new("./autogen.sh")
        .current_dir(&lib_dir)
        .status()
        .expect("Failed to run autogen.sh");

    // Create and navigate to the 'build' directory
    let build_dir = format!("{}/build", lib_dir);
    std::fs::create_dir_all(&build_dir).expect("Failed to create 'build' directory");

    // Run configure
    Command::new("../configure")
        .current_dir(&build_dir)
        .status()
        .expect("Failed to run configure");

    // Run make
    Command::new("make")
        .current_dir(&build_dir)
        .status()
        .expect("Failed to run make");

    // Link the compiled library
    println!("cargo:rustc-link-search=native={}/.libs", build_dir);
    println!("cargo:rustc-link-lib=static=secp256k1");
    println!("cargo:rustc-link-lib=gmp");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header(format!("{}/include/combined.h", lib_dir))
        .clang_arg(format!("-I{}/include/", lib_dir))
        .generate()
        .expect("Failed to generate bindings.");

    // Write the bindings to a file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}