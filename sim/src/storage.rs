//! An abstraction of a node.  The node can have a plugin, which is meant to reprsent a WebAssembly filter
//! A node is a sim_element.

use crate::sim_element::SimElement;
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
    fn tick(&mut self, _tick: u64) -> Vec<(Rpc, String)> {
        return vec![];
    }

    fn recv(&mut self, rpc: Rpc, tick: u64, _sender: &str) {
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
    fn type_specific_info(&self) -> Option<&str> {
        Some(&self.data)
    }
}

impl Storage {
    pub fn store(&mut self, x: Rpc, _now: u64) {
        self.data.push_str(&x.data);
        self.data.push_str("\n");
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
        let mut storage = Storage::new("0");
        storage.recv(Rpc::new_rpc("0"), 0, "node");
        let ret = storage.type_specific_info().unwrap();
        assert!(ret == "0\n".to_string());
    }
}
