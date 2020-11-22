#[derive(Debug)]
pub struct Plugin {
    // https://docs.rs/libloading/0.6.5/libloading/os/index.html
    // TODO: Currently uses a platform-specific binding, which isn't very safe.
    loaded_function : libloading::os::unix::Symbol<extern fn(u32) -> u32>,
}

impl Plugin {
    pub fn new(plugin_path : &str, function_name : &str) -> Plugin {
        let dyn_lib = libloading::Library::new(plugin_path).expect("load library");
        let loaded_function = unsafe {
            let tmp_loaded_function : libloading::Symbol<extern fn(u32) -> u32> =
                dyn_lib.get(function_name.as_bytes()).unwrap();
            tmp_loaded_function.into_raw()
        };
        Plugin { loaded_function : loaded_function }
    }

    pub fn execute(self, input : u32) -> u32 {
        (self.loaded_function)(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = Plugin::new("test_files/test.dylib", "codelet");
        assert!(plugin.execute(55) == 60);
    }

    #[test]
    fn test_chained_plugins() {
        let plugin1 = Plugin::new("test_files/test.dylib", "codelet");
        let plugin2 = Plugin::new("test_files/test.dylib", "codelet");
        let plugin3 = Plugin::new("test_files/test.dylib", "codelet");
        let plugin4 = Plugin::new("test_files/test.dylib", "codelet");
        assert!(25 == plugin4.execute(plugin3.execute(plugin2.execute(plugin1.execute(5)))));
    }
}
