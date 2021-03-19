//! A plugin wrapper is a sim_element that takes an outside library that does some computation on RPCs.
//! It is meant to represent a WebAssembly filter, and is a sim_element.  A plugin wrapper should only be
//! created as a field of a node object.

use std::env;
use std::path::PathBuf;

pub fn load_lib(plugin_str: &str) -> libloading::Library {
    // Convert the library string into a Path object
    let mut plugin_path = PathBuf::from(plugin_str);
    // We have to load the library differently, depending on whether we are
    // working with MacOS or Linux. Windows is not supported.
    match env::consts::OS {
        "macos" => {
            plugin_path.set_extension("dylib");
        }
        "linux" => {
            plugin_path.set_extension("so");
        }
        _ => panic!("Unexpected operating system."),
    }
    // Load library with  RTLD_NODELETE | RTLD_NOW to avoid freeing the lib
    // https://github.com/nagisa/rust_libloading/issues/41#issuecomment-448303856
    // also works on MacOS
    let os_lib = libloading::os::unix::Library::open(
        plugin_path.to_str(),
        libc::RTLD_NODELETE | libc::RTLD_NOW,
    )
    .unwrap();
    let dyn_lib = libloading::Library::from(os_lib);
    dyn_lib
}
