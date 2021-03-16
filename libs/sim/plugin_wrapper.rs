//! A plugin wrapper is a sim_element that takes an outside library that does some computation on RPCs.
//! It is meant to represent a WebAssembly filter, and is a sim_element.  A plugin wrapper should only be
//! created as a field of a node object.

use crate::filter_types::{CodeletType, Filter, NewWithEnvoyProperties};
use crate::sim_element::SimElement;
use core::any::Any;
use rpc_lib::rpc::Rpc;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::path::PathBuf;

pub struct PluginWrapper {
    // https://docs.rs/libloading/0.6.5/libloading/os/index.html
    // TODO: Currently uses a platform-specific binding, which isn't very safe.
    filter: *mut Filter,
    loaded_function: libloading::os::unix::Symbol<CodeletType>,
    id: String,
    stored_rpc: Vec<Rpc>,
    neighbor: Vec<String>,
}

impl fmt::Display for PluginWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(
                f,
                "{:width$}",
                &format!("PluginWrapper {{ id : {} }}", &self.id),
                width = width
            )
        } else {
            write!(f, "PluginWrapper {{ id : {} }}", &self.id)
        }
    }
}

impl SimElement for PluginWrapper {
    fn tick(&mut self, _tick: u64) -> Vec<Rpc> {
        let mut to_return = vec![];
        while !self.stored_rpc.is_empty() {
            let input_rpc = self.stored_rpc.pop();
            let ret = self.execute(input_rpc.as_ref().unwrap());
            if ret.len() > 0 {
                for rpc in ret {
                    if self.neighbor.len() > 0 {
                        to_return.push(rpc);
                    }
                }
            }
        }
        return to_return;
    }
    fn recv(&mut self, rpc: Rpc, _tick: u64) {
        self.stored_rpc.push(rpc);
    }
    fn add_connection(&mut self, neighbor: String) {
        // override the connection if there is already an element in it
        if self.neighbor.len() > 0 {
            let val = &mut self.neighbor[0];
            *val = neighbor;
        } else {
            self.neighbor.push(neighbor);
        }
    }
    fn whoami(&self) -> &str {
        &self.id
    }
    fn neighbors(&self) -> &Vec<String> {
        &self.neighbor
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn load_lib(plugin_str: &str) -> libloading::Library {
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

impl PluginWrapper {
    pub fn new(id: &str, plugin_str: &str) -> PluginWrapper {
        let dyn_lib = load_lib(plugin_str);
        // Dynamically load one function to initialize hash table in filter.
        let init: libloading::Symbol<NewWithEnvoyProperties>;
        let mut envoy_properties = HashMap::new();
        let mut id_without_plugin = id.to_string();
        if id_without_plugin.contains("_plugin") {
            id_without_plugin.truncate(id_without_plugin.len() - "_plugin".to_string().len());
        }
        envoy_properties.insert(
            String::from("node.metadata.WORKLOAD_NAME"),
            id_without_plugin,
        );
        envoy_properties.insert(String::from("response.total_size"), "1".to_string());
        envoy_properties.insert(String::from("response.code"), "200".to_string());
        let new_filter = unsafe {
            init = dyn_lib.get(b"new_with_envoy_properties\0").unwrap();
            // Put in envoy properties in the new filter
            init.into_raw()(envoy_properties)
        };

        // Dynamically load another function to execute filter functionality.
        let loaded_function = unsafe {
            let tmp_loaded_function: libloading::Symbol<CodeletType> =
                dyn_lib.get(b"execute").expect("load symbol");
            tmp_loaded_function.into_raw()
        };

        PluginWrapper {
            filter: new_filter,
            loaded_function,
            id: id.to_string(),
            stored_rpc: Vec::new(),
            neighbor: vec![],
        }
    }

    pub fn execute(&self, input: &Rpc) -> Vec<Rpc> {
        (self.loaded_function)(self.filter, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_plugin_creation() {
        let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_dir.push("../../target/debug/libfilter_example");
        let library_str = cargo_dir.to_str().unwrap();
        let plugin = PluginWrapper::new("0", library_str);
        let rpc = &mut Rpc::new("55");
        rpc.headers.insert("direction".to_string(), "request".to_string());
        rpc.headers.insert("location".to_string(), "ingress".to_string());
        let rpc_data = &plugin.execute(rpc)[0].data;
        assert!(rpc_data == &"55".to_string());
    }

    #[test]
    fn test_chained_plugins() {
        let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_dir.push("../../target/debug/libfilter_example");
        let library_str = cargo_dir.to_str().unwrap();
        let plugin1 = PluginWrapper::new("0", library_str);
        let plugin2 = PluginWrapper::new("1", library_str);
        let plugin3 = PluginWrapper::new("2", library_str);
        let plugin4 = PluginWrapper::new("3", library_str);
        let rpc = &mut Rpc::new("5");
        rpc.headers.insert("direction".to_string(), "request".to_string());
        rpc.headers.insert("location".to_string(), "ingress".to_string());
        let ret1: &Rpc = &plugin1.execute(rpc)[0];
        let ret2: &Rpc = &plugin2.execute(&ret1)[0];
        let ret3: &Rpc = &plugin3.execute(&ret2)[0];
        let ret4: &Rpc = &plugin4.execute(&ret3)[0];
        assert!("5".to_string() == ret4.data);
        //assert!("5".to_string() == plugin4.execute(&plugin3.execute(&plugin2.execute(&plugin1.execute(&Rpc::new_rpc("5"))[0])[0])[0].data));
    }
}
