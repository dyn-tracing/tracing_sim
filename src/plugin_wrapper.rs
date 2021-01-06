use crate::rpc::Rpc;
//use crate::codelet::CodeletType;
use crate::sim_element::SimElement;
use crate::filter::{CodeletType, Filter};
use std::fmt;

pub struct PluginWrapper {
    // https://docs.rs/libloading/0.6.5/libloading/os/index.html
    // TODO: Currently uses a platform-specific binding, which isn't very safe.
    filter: Filter,
    loaded_function : libloading::os::unix::Symbol<CodeletType>,
    id : u32,
    stored_rpc : Option<Rpc>,
    neighbor : Option<u32>,
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
    fn tick(&mut self, _tick : u64) -> Vec<(Rpc, Option<u32>)> {
        if self.stored_rpc.is_some() {
            let ret = self.execute(self.stored_rpc.as_ref().unwrap());
            self.stored_rpc = None;
            if ret.is_none() { vec![] } else { vec!((ret.unwrap(), self.neighbor)) }
        } else {
            vec![]
        }
    }
    fn recv(&mut self, rpc : Rpc, _tick : u64) {
        assert!(self.stored_rpc.is_none(), "Overwriting previous RPC");
        self.stored_rpc = Some(rpc);
    }
    fn add_connection(&mut self, neighbor : u32) {
        self.neighbor = Some(neighbor);
    }
}

impl PluginWrapper {
    pub fn new(plugin_path : &str, id : u32) -> PluginWrapper {
        let dyn_lib = libloading::Library::new(plugin_path).expect("load library");
        
        let filter_init = unsafe {
            let tmp_loaded_function : libloading::Symbol<fn() -> Filter> =
                dyn_lib.get("new".as_bytes()).expect("load symbol");
            tmp_loaded_function.into_raw()
        };
        
        let loaded_function = unsafe {
            let tmp_loaded_function : libloading::Symbol<CodeletType> =
                dyn_lib.get("execute".as_bytes()).expect("load symbol");
            tmp_loaded_function.into_raw()
        };

        let new_filter = filter_init();
        PluginWrapper { filter: new_filter, loaded_function : loaded_function, id : id, stored_rpc : None, neighbor : None }
    }

    pub fn execute(&self, input : &Rpc) -> Option<Rpc> {
        (self.loaded_function)(&self.filter, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static LIBRARY : &str = "libsample_filter.dylib";
    #[test]
    fn test_plugin_creation() {
        let plugin = PluginWrapper::new(LIBRARY, 0);
        assert!(plugin.execute(&Rpc::new_rpc(55)).unwrap().data == 60);
    }

    #[test]
    fn test_chained_plugins() {
        let plugin1 = PluginWrapper::new(LIBRARY, 0);
        let plugin2 = PluginWrapper::new(LIBRARY, 1);
        let plugin3 = PluginWrapper::new(LIBRARY, 2);
        let plugin4 = PluginWrapper::new(LIBRARY, 3);
        assert!(25 == plugin4.execute(
                      &plugin3.execute(
                      &plugin2.execute(
                      &plugin1.execute(
                      &Rpc::new_rpc(5))
                      .unwrap()).unwrap()).unwrap()).unwrap().data);
    }
}
