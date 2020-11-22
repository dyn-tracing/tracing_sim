#[derive(Debug)]
struct Plugin {
    // https://docs.rs/libloading/0.6.5/libloading/os/index.html
    // TODO: Currently uses a platform-specific binding, which isn't very safe.
    pub loaded_function : libloading::os::unix::Symbol<extern fn(u32) -> ()>,
}

impl Plugin {
    fn new(plugin_path : &str, function_name : &str) -> Plugin {
        let dyn_lib = libloading::Library::new(plugin_path).expect("load library");
        let loaded_function = unsafe {
            let tmp_loaded_function : libloading::Symbol<extern fn(u32) -> ()> =
                dyn_lib.get(function_name.as_bytes()).unwrap();
            tmp_loaded_function.into_raw()
        };
        Plugin { loaded_function : loaded_function }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let x = Plugin::new("test_files/test.dylib", "codelet");
        (x.loaded_function)(55);
    }
}
