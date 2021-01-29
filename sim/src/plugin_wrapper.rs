//! A plugin wrapper is a sim_element that takes an outside library that does some computation on RPCs.
//! It is meant to represent a WebAssembly filter, and is a sim_element.  A plugin wrapper should only be
//! created as a field of a node object.

use crate::filter_types::{CodeletType, Filter, NewWithEnvoyProperties};
use crate::sim_element::SimElement;
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
    stored_rpc: Option<Rpc>,
    neighbor: Option<String>,
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
    fn tick(&mut self, _tick: u64) -> Vec<(Rpc, Option<String>)> {
        if self.stored_rpc.is_some() {
            let ret = self.execute(self.stored_rpc.as_ref().unwrap());
            self.stored_rpc = None;
            if ret.is_none() {
                vec![]
            } else {
                vec![(ret.unwrap(), self.neighbor.clone())]
            }
        } else {
            vec![]
        }
    }
    fn recv(&mut self, rpc: Rpc, _tick: u64, _sender: String) {
        assert!(self.stored_rpc.is_none(), "Overwriting previous RPC");
        self.stored_rpc = Some(rpc);
    }
    fn add_connection(&mut self, neighbor: String) {
        self.neighbor = Some(neighbor);
    }
    fn whoami(&self) -> (bool, String, Vec<String>) {
        let mut neighbors = Vec::new();
        if !self.neighbor.is_none() {
            neighbors.push(self.neighbor.clone().unwrap());
        }
        return (false, self.id.clone(), neighbors.clone());
    }
}

fn load_lib(plugin_str: String) -> libloading::Library {
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
    pub fn new(id: String, plugin_str: String) -> PluginWrapper {
        let dyn_lib = load_lib(plugin_str);
        // Dynamically load one function to initialize hash table in filter.
        let init: libloading::Symbol<NewWithEnvoyProperties>;
        let mut envoy_properties = HashMap::new();
        envoy_properties.insert(String::from("node.metadata.WORKLOAD_NAME"), id.to_string());
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
            id,
            stored_rpc: None,
            neighbor: None,
        }
    }

    pub fn execute(&self, input: &Rpc) -> Option<Rpc> {
        (self.loaded_function)(self.filter, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_plugin_creation() {
        let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_dir.push("../target/debug/libfilter_example");
        let library_str = cargo_dir.to_str().unwrap().to_string();
        let plugin = PluginWrapper::new("0".to_string(), library_str);
        let rpc = &Rpc::new_rpc(55);
        let rpc_data = plugin.execute(rpc).unwrap().data;
        assert!(rpc_data == 55);
    }

    #[test]
    fn test_chained_plugins() {
        let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_dir.push("../target/debug/libfilter_example");
        let library_str = cargo_dir.to_str().unwrap().to_string();
        let plugin1 = PluginWrapper::new("0".to_string(), library_str.clone());
        let plugin2 = PluginWrapper::new("1".to_string(), library_str.clone());
        let plugin3 = PluginWrapper::new("2".to_string(), library_str.clone());
        let plugin4 = PluginWrapper::new("3".to_string(), library_str.clone());
        assert!(
            5 == plugin4
                .execute(
                    &plugin3
                        .execute(
                            &plugin2
                                .execute(&plugin1.execute(&Rpc::new_rpc(5)).unwrap())
                                .unwrap()
                        )
                        .unwrap()
                )
                .unwrap()
                .data
        );
    }
}
