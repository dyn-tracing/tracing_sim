use std::process::Command;

// Example custom build script.
fn main() {
    // TODO: use handlebars to automatically generate filters, that way you abstract away the code that is used
    //       for all filters

    // Note: needs to be run with cargo -vv to generate output from these commands.
    // eprintln!("{:?}",
    //          Command::new("rustc").args(&["plugins/graph_utils.rs", "--crate-type=dylib"]).output().unwrap());
    eprintln!("{:?}",
              Command::new("rustc").args(&["plugins/compiled_filter.rs", "--crate-type=dylib"]).output().unwrap());

    // Don't go any further if these commands failed.
    // assert!(Command::new("rustc").args(&["plugins/graph_utils.rs",  "--crate-type=dylib"]).status().unwrap().success());
    assert!(Command::new("rustc").args(&["plugins/compiled_filter.rs",  "--crate-type=dylib"]).status().unwrap().success());

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=./plugins");
    // println!("cargo:rerun-if-changed=./plugins/graph_utils.rs");
    println!("cargo:rerun-if-changed=./plugins/compiled_filter.rs");
}
