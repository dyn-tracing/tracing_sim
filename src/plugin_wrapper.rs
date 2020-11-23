use crate::rpc::Rpc;
use crate::codelet::CodeletType;

#[derive(Debug)]
pub struct PluginWrapper {
    // https://docs.rs/libloading/0.6.5/libloading/os/index.html
    // TODO: Currently uses a platform-specific binding, which isn't very safe.
    loaded_function : libloading::os::unix::Symbol<CodeletType>,
}

impl PluginWrapper {
    pub fn new(plugin_path : &str, function_name : &str) -> PluginWrapper {
        let dyn_lib = libloading::Library::new(plugin_path).expect("load library");
        let loaded_function = unsafe {
            let tmp_loaded_function : libloading::Symbol<CodeletType> =
                dyn_lib.get(function_name.as_bytes()).expect("load symbol");
            tmp_loaded_function.into_raw()
        };
        PluginWrapper { loaded_function : loaded_function }
    }

    pub fn execute(self, input : Rpc) -> Rpc {
        (self.loaded_function)(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static LIBRARY : &str = "target/debug/libplugin_sample.dylib";
    static FUNCTION: &str = "codelet";
    #[test]
    fn test_plugin_creation() {
        let plugin = PluginWrapper::new(LIBRARY, FUNCTION);
        assert!(plugin.execute(Rpc{id:55}) == Rpc{id:60});
    }

    #[test]
    fn test_chained_plugins() {
        let plugin1 = PluginWrapper::new(LIBRARY, FUNCTION);
        let plugin2 = PluginWrapper::new(LIBRARY, FUNCTION);
        let plugin3 = PluginWrapper::new(LIBRARY, FUNCTION);
        let plugin4 = PluginWrapper::new(LIBRARY, FUNCTION);
        assert!(Rpc{id:25} == plugin4.execute(plugin3.execute(plugin2.execute(plugin1.execute(Rpc{id:5})))));
    }
}