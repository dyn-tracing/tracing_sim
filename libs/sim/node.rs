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

pub trait NodeTraits {
    fn process_rpc(&self, rpc: &mut Rpc, new_rpcs: &mut Vec<Rpc>);
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
    fn tick(&mut self, tick: u64) -> Vec<Rpc> {
        let mut outgoing_rpcs: Vec<Rpc> = vec![];
        for _ in 0..min(
            self.queue.size() + (self.generation_rate as usize),
            self.egress_rate as usize,
        ) {
            // Dequeue an RPC, or generate one
            let mut rpc: Rpc;
            if self.queue.size() > 0 {
                let deq = self.dequeue(tick);
                rpc = deq.unwrap();
            } else {
                rpc = Rpc::new_rpc(&tick.to_string());
                rpc.headers
                    .insert("direction".to_string(), "request".to_string());
            }

            // Select the destination
            let mut new_rpcs: Vec<Rpc> = vec![];
            self.process_rpc(&mut rpc, &mut new_rpcs);

            // Pass the RPCs we have through the plugin
            self.pass_through_plugin(new_rpcs, &mut outgoing_rpcs, tick, "egress");
        }
        outgoing_rpcs
    }

    // once the RPC is received, the plugin executes, the rpc gets a new destination,
    // the RPC once again goes through the plugin, this time as an outbound rpc, and then it is
    // placed in the outbound queue
    fn recv(&mut self, mut rpc: Rpc, tick: u64) {
        // drop packets you cannot accept
        if (self.queue.size() as u32) < self.capacity {
            if self.plugin.is_none() {
                self.enqueue(rpc, tick);
            } else {
                // inbound filter check
                rpc.headers
                    .insert("location".to_string(), "ingress".to_string());
                self.plugin.as_mut().unwrap().recv(rpc, tick);
                let ret = self.plugin.as_mut().unwrap().tick(tick);
                for inbound_rpc in ret {
                    self.enqueue(inbound_rpc, tick);
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

impl NodeTraits for Node {
    fn process_rpc(&self, rpc: &mut Rpc, new_rpcs: &mut Vec<Rpc>) {
        // Set yourself as the source
        rpc.headers.insert("src".to_string(), self.id.to_string());

        // Select a new destination at random
        if self.neighbors.len() > 0 {
            let mut rng: StdRng = SeedableRng::seed_from_u64(self.seed);
            let idx = rng.gen_range(0, self.neighbors.len());
            rpc.headers
                .insert("dest".to_string(), self.neighbors[idx].clone());
        } else {
            panic!("Node has no neighbors and no one to send to");
        }
        new_rpcs.push(rpc.clone());
    }
}

impl Node {
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

    pub fn pass_through_plugin(
        &mut self,
        input_rcps: Vec<Rpc>,
        processed_rpcs: &mut Vec<Rpc>,
        tick: u64,
        direction: &str,
    ) {
        for mut rpc in input_rcps {
            // If the plugin exists, run the RPC through
            // Otherwise just push it into the egress queue
            if let Some(plugin) = self.plugin.as_mut() {
                rpc.headers
                    .insert("location".to_string(), direction.to_string());
                plugin.recv(rpc, tick);
                let filtered_rpcs = plugin.tick(tick);
                for filtered_rpc in filtered_rpcs {
                    processed_rpcs.push(filtered_rpc.clone());
                }
            } else {
                processed_rpcs.push(rpc);
            }
        }
    }

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
        node.recv(Rpc::new_rpc("0"), 0);
        node.recv(Rpc::new_rpc("0"), 0);
        assert!(node.queue.size() == 2);
        node.recv(Rpc::new_rpc("0"), 0);
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
