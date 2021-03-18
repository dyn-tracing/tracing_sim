//! An abstraction of a node.  The node can have a plugin, which is meant to reprsent a WebAssembly filter
//! A node is a sim_element.

use crate::sim_element::SimElement;
use core::any::Any;
use rpc_lib::rpc::Rpc;
use std::env;
use std::fmt;
use std::path::PathBuf;

pub type AggregationInputFunc = fn(*mut AggregationStruct, String);
pub type AggregationRetType = fn(*mut AggregationStruct) -> String;

extern "Rust" {
    pub type AggregationStruct;
}

pub type AggregationInitFunc = fn() -> *mut AggregationStruct;

#[derive(Default)]
pub struct Storage {
    data: String,           // all the data that has been stored
    id: String,             // id
    neighbors: Vec<String>, // who is the node connected to
    aggr_struct: Option<*mut AggregationStruct>,
    aggr_output_func: Option<libloading::os::unix::Symbol<AggregationRetType>>,
    aggr_input_func: Option<libloading::os::unix::Symbol<AggregationInputFunc>>,
}

impl fmt::Display for Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(
                f,
                "{:width$}",
                &format!("Storage {{ queue : {} }}", self.id),
                width = width
            )
        } else {
            write!(f, "Storage {{ id : {} }}", self.id,)
        }
    }
}

impl SimElement for Storage {
    // storage never sends messages out, only receives them, so we return an empty vector
    fn tick(&mut self, _tick: u64) -> Vec<Rpc> {
        return vec![];
    }

    fn recv(&mut self, rpc: Rpc, tick: u64) {
        self.store(rpc, tick);
    }
    fn add_connection(&mut self, neighbor: String) {
        self.neighbors.push(neighbor);
    }
    fn whoami(&self) -> &str {
        &self.id
    }
    fn neighbors(&self) -> &Vec<String> {
        &self.neighbors
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

impl Storage {
    pub fn store(&mut self, x: Rpc, _now: u64) {
        // we don't want to store everything, just the stuff that was sent to us
        if x.headers.contains_key("dest") && x.headers["dest"].contains(&self.id) {
            if self.aggr_input_func.is_some() {
                (self.aggr_input_func.clone().unwrap())(self.aggr_struct.unwrap(), x.data);
            } else {
                self.data.push_str(&x.data);
                self.data.push_str("\n");
            }
        }
    }
    pub fn query(&self) -> String {
        if self.aggr_struct.is_none() {
            self.data.clone()
        } else {
            (self.aggr_output_func.clone().unwrap())(self.aggr_struct.unwrap()).clone()
        }
    }
    pub fn new(id: &str, aggregation_file: Option<&str>) -> Storage {
        if aggregation_file.is_none() {
            Storage {
                data: String::new(),
                id: id.to_string(),
                neighbors: Vec::new(),
                aggr_struct: None,
                aggr_input_func: None,
                aggr_output_func: None,
            }
        } else {
            let aggr_lib = load_lib(&aggregation_file.unwrap());
            let init: libloading::Symbol<AggregationInitFunc>;

            let aggregation_struct = unsafe {
                init = aggr_lib.get(b"init").unwrap();
                init.into_raw()()
            };

            let input_function = unsafe {
                let tmp_loaded_function: libloading::Symbol<AggregationInputFunc> =
                    aggr_lib.get(b"input").expect("load symbol");
                tmp_loaded_function.into_raw()
            };

            let output_function = unsafe {
                let tmp_loaded_function: libloading::Symbol<AggregationRetType> =
                    aggr_lib.get(b"return_value").expect("load symbol");
                tmp_loaded_function.into_raw()
            };

            Storage {
                data: String::new(),
                id: id.to_string(),
                neighbors: Vec::new(),
                aggr_struct: Some(aggregation_struct),
                aggr_input_func: Some(input_function),
                aggr_output_func: Some(output_function),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let _storage = Storage::new("0", None);
    }

    #[test]
    fn test_query_storage() {
        let mut storage = Storage::new("storage", None);
        let mut rpc = Rpc::new("0");
        rpc.headers
            .insert("dest".to_string(), "storage".to_string());
        storage.recv(rpc, 0);
        let ret = storage.query();
        assert!(ret == "0\n".to_string());
    }

    #[test]
    fn test_storage_with_aggr() {
        // aggregation example is a simple avg function
        let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_dir.push("../../target/debug/libaggregation_example");
        let aggr_str = cargo_dir.to_str().unwrap();
        let mut storage = Storage::new("storage", Some(aggr_str));
        let mut rpc = Rpc::new("4");
        rpc.headers
            .insert("dest".to_string(), "storage".to_string());
        storage.recv(rpc, 0);
        assert!(storage.query() == "4".to_string());

        let mut rpc_2 = Rpc::new("2");
        rpc_2
            .headers
            .insert("dest".to_string(), "storage".to_string());
        storage.recv(rpc_2, 1);
        assert!(storage.query() == "3".to_string());
    }
}
