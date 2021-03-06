//! An abstraction of a node.  The node can have a plugin, which is meant to reprsent a WebAssembly filter
//! A node is a sim_element.

use crate::sim_element::SimElement;
use core::any::Any;
use rpc_lib::rpc::Rpc;
use std::fmt;

#[derive(Default)]
pub struct Storage {
    data: String,           // all the data that has been stored
    id: String,             // id
    neighbors: Vec<String>, // who is the node connected to
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

impl Storage {
    pub fn store(&mut self, x: Rpc, _now: u64) {
        // we don't want to store everything, just the stuff that was sent to us
        if x.headers.contains_key("dest") && x.headers["dest"].contains(&self.id) {
            self.data.push_str(&x.data);
            self.data.push_str("\n");
        }
    }
    pub fn query(&self) -> &str {
        &self.data
    }
    pub fn new(id: &str) -> Storage {
        Storage {
            data: String::new(),
            id: id.to_string(),
            neighbors: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let _storage = Storage::new("0");
    }

    #[test]
    fn test_query_storage() {
        let mut storage = Storage::new("storage");
        let mut rpc = Rpc::new("0");
        rpc.headers
            .insert("dest".to_string(), "storage".to_string());
        storage.recv(rpc, 0);
        let ret = storage.query();
        assert!(ret == "0\n".to_string());
    }
}
