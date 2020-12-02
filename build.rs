use std::process::Command;

// Example custom build script.
fn main() {
    // Note: needs to be run with cargo -vv to generate output from these commands.
    eprintln!("{:?}",
              Command::new("rustc").args(&["plugins/sample.rs", "--crate-type=dylib"]).output().unwrap());
    eprintln!("{:?}",
              Command::new("rustc").args(&["plugins/sink.rs", "--crate-type=dylib"]).output().unwrap());

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=./plugins");
    println!("cargo:rerun-if-changed=./plugins/sample.rs");
    println!("cargo:rerun-if-changed=./plugins/sink.rs");
}
