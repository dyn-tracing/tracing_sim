//! An abstraction of a node.  The node can have a plugin, which is meant to reprsent a WebAssembly filter
//! A node is a sim_element.

use crate::plugin_wrapper::PluginWrapper;
use crate::sim_element::SimElement;
use core::any::Any;
use queues::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rpc_lib::rpc::Rpc;
use std::cmp::min;
use std::fmt;

pub struct Node {
    pub queue: Queue<Rpc>,             // queue of rpcs
    pub id: String,                    // id of the node
    pub capacity: u32,                 // capacity of the node;  how much it can hold at once
    pub egress_rate: u32,              // rate at which the node can send out rpcs
    pub generation_rate: u32, // rate at which the node can generate rpcs, which are generated regardless of input to the node
    pub plugin: Option<PluginWrapper>, // filter to the node
    pub neighbors: Vec<String>, // who is the node connected to
    pub seed: u64,
}

pub fn node_fmt_with_name(node: &Node, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
    if let Some(width) = f.width() {
        if node.plugin.is_none() {
            write!(f, "{:width$}",
                       &format!("{} {{ id : {}, capacity : {}, egress_rate : {}, generation_rate : {}, queue: {}, plugin : None}}", name,
                       &node.id, &node.capacity, &node.egress_rate, &node.generation_rate, &node.queue.size()),
                       width = width)
        } else {
            write!(f, "{:width$}",
                       &format!("{} {{ id : {}, capacity : {}, egress_rate : {}, generation_rate : {}, queue : {}, \n\tplugin : {} }}", name,
                       &node.id, &node.capacity, &node.egress_rate, &node.generation_rate, &node.queue.size(), node.plugin.as_ref().unwrap()),
                       width = width)
        }
    } else {
        if node.plugin.is_none() {
            write!(f, "{} {{ id : {}, egress_rate : {}, generation_rate : {}, plugin : None, capacity : {}, queue : {} }}",
                        name,&node.id, &node.egress_rate, &node.generation_rate, &node.capacity, &node.queue.size())
        } else {
            write!(
                    f,
                    "{} {{ id : {}, egress_rate : {}, generation_rate : {}, plugin : {}, capacity : {}, queue : {} }}",
                     name,&node.id,
                    &node.egress_rate,
                    &node.generation_rate,
                    node.plugin.as_ref().unwrap(),
                    &node.capacity,
                    &node.queue.size()
                )
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        node_fmt_with_name(self, f, "Node")
    }
}

impl SimElement for Node {
    fn tick(&mut self, tick: u64) -> Vec<(Rpc, String)> {
        let mut ret = vec![];
        for _ in 0..min(
            self.queue.size() + (self.generation_rate as usize),
            self.egress_rate as usize,
        ) {
            if self.queue.size() > 0 {
                let mut deq = self.dequeue(tick).unwrap();

                // 1. change src/dst headers appropriately
                let new_dest = self.route_rpc(&deq);
                if !deq.headers.contains_key("dest") {
                    deq.headers.insert("dest".to_string(), new_dest.clone());
                } else {
                    let dest = deq.headers.get_mut("dest").unwrap();
                    *dest = new_dest.clone();
                }
                if !deq.headers.contains_key("src") {
                    deq.headers.insert("src".to_string(), self.id.clone());
                } else {
                    let src = deq.headers.get_mut("src").unwrap();
                    *src = self.id.clone();
                }

                // 2. If there's a plugin, run it through the plugin and then put into ret
                if self.plugin.is_some() {
                    deq.headers.insert("location".to_string(), "egress".to_string());
                    self.plugin.as_mut().unwrap().recv(deq, tick, &self.id);
                    let filtered_rpcs = self.plugin.as_mut().unwrap().tick(tick);
                    for filtered_rpc in filtered_rpcs {
                        ret.push((
                            filtered_rpc.0.clone(),
                            filtered_rpc.0.headers["dest"].clone(),
                        ));
                    }
                } else {
                    ret.push((deq, new_dest.clone()));
                }
            } else {
                let mut new_rpc = Rpc::new_rpc(&tick.to_string());
                new_rpc
                    .headers
                    .insert("direction".to_string(), "request".to_string());
                new_rpc
                    .headers
                    .insert("dest".to_string(), self.route_rpc(&new_rpc));
                if self.plugin.is_some() {
                    self.plugin.as_mut().unwrap().recv(new_rpc, tick, &self.id);
                    let filtered_rpcs = self.plugin.as_mut().unwrap().tick(tick);
                    for filtered_rpc in filtered_rpcs {
                        ret.push((
                            filtered_rpc.0.clone(),
                            filtered_rpc.0.headers["dest"].clone(),
                        ));
                    }
                } else {
                    let new_dest = self.route_rpc(&new_rpc);
                    ret.push((new_rpc, new_dest));
                }
            }
        }

        ret
    }

    // once the RPC is received, the plugin executes, the rpc gets a new destination,
    // the RPC once again goes through the plugin, this time as an outbound rpc, and then it is
    // placed in the outbound queue
    fn recv(&mut self, mut rpc: Rpc, tick: u64, _sender: &str) {
        // drop packets you cannot accept
        if (self.queue.size() as u32) < self.capacity {
            if self.plugin.is_none() {
                self.enqueue(rpc, tick);
            } else {
                // inbound filter check
                rpc.headers.insert("location".to_string(), "ingress".to_string());
                self.plugin.as_mut().unwrap().recv(rpc, tick, &self.id);
                let ret = self.plugin.as_mut().unwrap().tick(tick);
                for inbound_rpc in ret {
                    self.enqueue(inbound_rpc.0, tick);
                }
            }
        }
    }

    fn add_connection(&mut self, neighbor: String) {
        self.neighbors.push(neighbor);
    }

    fn whoami(&self) -> &str {
        return &self.id;
    }
    fn neighbors(&self) -> &Vec<String> {
        return &self.neighbors;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node {
    pub fn enqueue(&mut self, x: Rpc, _now: u64) {
        let _res = self.queue.add(x);
    }
    pub fn dequeue(&mut self, _now: u64) -> Option<Rpc> {
        if self.queue.size() == 0 {
            return None;
        } else {
            return Some(self.queue.remove().unwrap());
        }
    }

    // given an RPC, returns the neighbor it should be sent to
    pub fn route_rpc(&mut self, rpc: &Rpc) -> String {
        if rpc.headers.contains_key("dest") {
            let dest = &rpc.headers["dest"].clone();
            for n in &self.neighbors {
                if n == dest {
                    return dest.to_string();
                }
            }
        } else if self.neighbors.len() > 0 {
            let mut rng: StdRng = SeedableRng::seed_from_u64(self.seed);
            let idx = rng.gen_range(0, self.neighbors.len());
            let which_neighbor = self.neighbors[idx].clone();
            return which_neighbor;
        }
        panic!("Node has no neighbors and no one to send to");
    }

    pub fn new(
        id: &str,
        capacity: u32,
        egress_rate: u32,
        generation_rate: u32,
        plugin: Option<&str>,
        seed: u64,
    ) -> Node {
        assert!(capacity >= 1);
        let mut created_plugin = None;
        if !plugin.is_none() {
            let mut plugin_id = id.to_string();
            plugin_id.push_str("_plugin");
            let mut unwrapped_plugin = PluginWrapper::new(&plugin_id, plugin.unwrap());
            unwrapped_plugin.add_connection(id.to_string());
            created_plugin = Some(unwrapped_plugin);
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
        let _node = Node::new("0", 2, 2, 1, None, 1);
    }

    #[test]
    fn test_node_capacity_and_egress_rate() {
        let mut node = Node::new("0", 2, 1, 0, None, 1);
        node.add_connection("foo".to_string()); // without at least one neighbor, it will just drop rpcs
        assert!(node.capacity == 2);
        assert!(node.egress_rate == 1);
        node.recv(Rpc::new_rpc("0"), 0, "0");
        node.recv(Rpc::new_rpc("0"), 0, "0");
        assert!(node.queue.size() == 2);
        node.recv(Rpc::new_rpc("0"), 0, "0");
        assert!(node.queue.size() == 2);
        node.tick(0);
        assert!(node.queue.size() == 1);
    }

    #[test]
    fn test_plugin_initialization() {
        let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_dir.push("../../target/debug/libfilter_example");
        let library_str = cargo_dir.to_str().unwrap();
        let node = Node::new("0", 2, 1, 0, Some(library_str), 1);
        assert!(!node.plugin.is_none());
    }
}
