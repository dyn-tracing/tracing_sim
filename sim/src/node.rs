//! An abstraction of a node.  The node can have a plugin, which is meant to reprsent a WebAssembly filter
//! A node is a sim_element.

use crate::plugin_wrapper::PluginWrapper;
use crate::sim_element::SimElement;
use queues::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rpc_lib::rpc::Rpc;
use std::cmp::min;
use std::fmt;

pub struct Node {
    queue: Queue<Rpc>,             // queue of rpcs
    id: String,                    // id of the node
    capacity: u32,                 // capacity of the node;  how much it can hold at once
    egress_rate: u32,              // rate at which the node can send out rpcs
    generation_rate: u32, // rate at which the node can generate rpcs, which are generated regardless of input to the node
    plugin: Option<PluginWrapper>, // filter to the node
    neighbors: Vec<String>, // who is the node connected to
    seed: Option<u64>,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            if self.plugin.is_none() {
                write!(f, "{:width$}",
                       &format!("Node {{ id : {}, capacity : {}, egress_rate : {}, generation_rate : {}, queue: {}, plugin : None}}",
                       &self.id, &self.capacity, &self.egress_rate, &self.generation_rate, &self.queue.size()),
                       width = width)
            } else {
                write!(f, "{:width$}",
                       &format!("Node {{ id : {}, capacity : {}, egress_rate : {}, generation_rate : {}, queue : {}, \n\tplugin : {} }}",
                       &self.id, &self.capacity, &self.egress_rate, &self.generation_rate, &self.queue.size(), self.plugin.as_ref().unwrap()),
                       width = width)
            }
        } else {
            if self.plugin.is_none() {
                write!(f, "Node {{ capacity : {}, egress_rate : {}, generation_rate : {}, plugin : None, id : {}, queue : {} }}",
                       &self.capacity, &self.egress_rate, &self.generation_rate, &self.id, &self.queue.size())
            } else {
                write!(
                    f,
                    "Node {{ capacity : {}, egress_rate : {}, generation_rate : {}, plugin : {}, id : {}, queue : {} }}",
                    &self.capacity,
                    &self.egress_rate,
                    &self.generation_rate,
                    self.plugin.as_ref().unwrap(),
                    &self.id,
                    &self.queue.size()
                )
            }
        }
    }
}

impl SimElement for Node {
    fn tick(&mut self, tick: u64) -> Vec<(Rpc, Option<String>)> {
        let mut ret = vec![];
        for _ in 0..min(
            self.queue.size() + (self.generation_rate as usize),
            self.egress_rate as usize,
        ) {
            // send the rpc to a random neighbor, if there are any
            let mut which_neighbor = None;
            let neigh_len = self.neighbors.len();
            if neigh_len > 0 {
                let idx;
                if self.seed.is_none() {
                    idx = rand::thread_rng().gen_range(0, neigh_len);
                } else {
                    let mut rng: StdRng = SeedableRng::seed_from_u64(self.seed.unwrap());
                    idx = rng.gen_range(0, neigh_len);
                }
                which_neighbor = Some(self.neighbors[idx].clone());
            }
            if self.queue.size() > 0 {
                let deq = self.dequeue(tick);
                assert!(deq.is_some());
                ret.push((deq.unwrap(), which_neighbor));
            } else {
                ret.push((Rpc::new_rpc(tick as u32), which_neighbor));
            }
        }
        ret
    }
    fn recv(&mut self, rpc: Rpc, tick: u64, _sender: &str) {
        if (self.queue.size() as u32) < self.capacity {
            // drop packets you cannot accept
            if self.plugin.is_none() {
                self.enqueue(rpc, tick);
            } else {
                self.plugin.as_mut().unwrap().recv(rpc, tick, &self.id);
                let ret = self.plugin.as_mut().unwrap().tick(tick);
                for filtered_rpc in ret {
                    self.enqueue(filtered_rpc.0, tick);
                }
            }
        }
    }
    fn add_connection(&mut self, neighbor: String) {
        self.neighbors.push(neighbor);
    }
    fn whoami(&self) -> (bool, &str, &Vec<String>) {
        return (true, &self.id, &self.neighbors);
    }
}

impl Node {
    pub fn enqueue(&mut self, x: Rpc, _now: u64) {
        self.queue.add(x).unwrap();
    }
    pub fn dequeue(&mut self, _now: u64) -> Option<Rpc> {
        if self.queue.size() == 0 {
            return None;
        } else {
            return Some(self.queue.remove().unwrap());
        }
    }
    pub fn new(
        id: &str,
        capacity: u32,
        egress_rate: u32,
        generation_rate: u32,
        plugin: Option<&str>,
        seed: Option<u64>,
    ) -> Node {
        assert!(capacity >= 1);
        let mut created_plugin = None;
        if !plugin.is_none() {
            let mut plugin_id = id.to_string();
            plugin_id.push_str("_plugin");
            created_plugin = Some(PluginWrapper::new(&plugin_id, plugin.unwrap()));
        }
        Node {
            queue: queue![],
            id: id.to_string(),
            capacity,
            egress_rate,
            generation_rate,
            plugin: created_plugin,
            neighbors: Vec::new(),
            seed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_node_creation() {
        let _node = Node::new("0", 2, 2, 1, None, None);
    }

    #[test]
    fn test_node_capacity_and_egress_rate() {
        let mut node = Node::new("0", 2, 1, 0, None, None);
        assert!(node.capacity == 2);
        assert!(node.egress_rate == 1);
        node.recv(Rpc::new_rpc(0), 0, "0");
        node.recv(Rpc::new_rpc(0), 0, "0");
        assert!(node.queue.size() == 2);
        node.recv(Rpc::new_rpc(0), 0, "0");
        assert!(node.queue.size() == 2);
        node.tick(0);
        assert!(node.queue.size() == 1);
    }

    #[test]
    fn test_plugin_initialization() {
        let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_dir.push("../target/debug/libfilter_example");
        let library_str = cargo_dir.to_str().unwrap();
        let node = Node::new("0", 2, 1, 0, Some(library_str), None);
        assert!(!node.plugin.is_none());
    }
}
