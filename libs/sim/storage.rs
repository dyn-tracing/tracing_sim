//! An abstraction of a node.  The node can have a plugin, which is meant to reprsent a WebAssembly filter
//! A node is a sim_element.

use crate::plugin_wrapper::PluginWrapper;
use crate::sim_element::SimElement;
use core::any::Any;
use rpc_lib::rpc::Rpc;
use std::fmt;

#[derive(Default)]
pub struct Storage {
    data: String,                  // all the data that has been stored
    id: String,                    // id
    neighbors: Vec<String>,        // who is the node connected to
    plugin: Option<PluginWrapper>, // aggregation filter
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
        let mut rpc_to_store = rpc.clone();
        rpc_to_store
            .headers
            .insert("direction".to_string(), "request".to_string());
        rpc_to_store
            .headers
            .insert("location".to_string(), "ingress".to_string());
        self.store(rpc_to_store, tick);
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

impl Storage {
    pub fn store(&mut self, x: Rpc, now: u64) {
        // we don't want to store everything, just the stuff that was sent to us
        if x.headers.contains_key("dest") && x.headers["dest"].contains(&self.id) {
            let mut new_rpcs;
            if let Some(mut plugin) = self.plugin.as_mut() {
                plugin.recv(x, now);
                new_rpcs = plugin.tick(now);
            } else {
                new_rpcs = vec![x];
            }
            for rpc in new_rpcs {
                print!("rpc data is {:?}", rpc.data);
                self.data.push_str(&rpc.data);
                self.data.push_str("\n");
            }
        }
    }
    pub fn query(&self) -> String {
        return self.data.clone();
    }

    pub fn new(id: &str, aggregation_file: Option<&str>) -> Storage {
        if aggregation_file.is_none() {
            Storage {
                data: String::new(),
                id: id.to_string(),
                neighbors: Vec::new(),
                plugin: None,
            }
        } else {
            let mut aggr_plugin = PluginWrapper::new(id, aggregation_file.unwrap());
            aggr_plugin.add_connection(id.to_string());
            Storage {
                data: String::new(),
                id: id.to_string(),
                neighbors: Vec::new(),
                plugin: Some(aggr_plugin),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
        rpc.headers
            .insert("direction".to_string(), "request".to_string());
        rpc.headers
            .insert("location".to_string(), "ingress".to_string());
        storage.recv(rpc, 0);
        assert!(
            storage.query() == "4\navg: 4\n".to_string(),
            "storage has {:?}",
            storage.query()
        );

        let mut rpc_2 = Rpc::new("2");
        rpc_2
            .headers
            .insert("dest".to_string(), "storage".to_string());
        rpc_2
            .headers
            .insert("direction".to_string(), "request".to_string());
        rpc_2
            .headers
            .insert("location".to_string(), "ingress".to_string());
        storage.recv(rpc_2, 1);
        assert!(
            storage.query() == "4\navg: 4\n2\navg: 3\n".to_string(),
            "storage has {:?}",
            storage.query()
        );
    }
}
