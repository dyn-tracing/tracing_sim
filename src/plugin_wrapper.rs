use crate::rpc::Rpc;
use crate::codelet::CodeletType;
use crate::sim_element::SimElement;
use std::fmt;

pub struct PluginWrapper {
    // https://docs.rs/libloading/0.6.5/libloading/os/index.html
    // TODO: Currently uses a platform-specific binding, which isn't very safe.
    loaded_function : libloading::os::unix::Symbol<CodeletType>,
    id : u32,
    stored_rpc : Option<Rpc>,
}

impl fmt::Display for PluginWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(f, "{:width$}", &format!("PluginWrapper {{ id : {} }}", &self.id), width = width)
        } else {
            write!(f, "PluginWrapper {{ id : {} }}", &self.id)
        }
    }
}

impl SimElement for PluginWrapper {
    fn tick(&mut self, _tick : u64) -> Vec<Rpc> {
        if self.stored_rpc.is_some() {
            let ret = self.execute(&self.stored_rpc.unwrap());
            self.stored_rpc = None;
            if ret.is_none() { vec![] } else { vec!(ret.unwrap()) }
        } else {
            vec![]
        }
    }
    fn recv(&mut self, rpc : Rpc, _tick : u64) {
        assert!(self.stored_rpc.is_none(), "Overwriting previous RPC");
        self.stored_rpc = Some(rpc);
    }
}

impl PluginWrapper {
    pub fn new(plugin_path : &str, function_name : &str, id : u32) -> PluginWrapper {
        let dyn_lib = libloading::Library::new(plugin_path).expect("load library");
        let loaded_function = unsafe {
            let tmp_loaded_function : libloading::Symbol<CodeletType> =
                dyn_lib.get(function_name.as_bytes()).expect("load symbol");
            tmp_loaded_function.into_raw()
        };
        PluginWrapper { loaded_function : loaded_function, id : id, stored_rpc : None }
    }

    pub fn execute(&self, input : &Rpc) -> Option<Rpc> {
        (self.loaded_function)(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static LIBRARY : &str = "target/debug/libsample_plugin.dylib";
    static FUNCTION: &str = "codelet";
    #[test]
    fn test_plugin_creation() {
        let plugin = PluginWrapper::new(LIBRARY, FUNCTION, 0);
        assert!(plugin.execute(&Rpc::new_rpc(55)).unwrap().data == 60);
    }

    #[test]
    fn test_chained_plugins() {
        let plugin1 = PluginWrapper::new(LIBRARY, FUNCTION, 0);
        let plugin2 = PluginWrapper::new(LIBRARY, FUNCTION, 1);
        let plugin3 = PluginWrapper::new(LIBRARY, FUNCTION, 2);
        let plugin4 = PluginWrapper::new(LIBRARY, FUNCTION, 3);
        assert!(25 == plugin4.execute(
                      &plugin3.execute(
                      &plugin2.execute(
                      &plugin1.execute(
                      &Rpc::new_rpc(5))
                      .unwrap()).unwrap()).unwrap()).unwrap().data);
    }
}
