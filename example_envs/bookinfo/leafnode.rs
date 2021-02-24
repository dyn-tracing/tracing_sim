//! An abstraction of a node.  The node can have a plugin, which is meant to reprsent a WebAssembly filter
//! A node is a sim_element.

use core::any::Any;
use queues::*;
use rpc_lib::rpc::Rpc;
use sim::node::node_fmt_with_name;
use sim::node::Node;
use sim::sim_element::SimElement;
use std::cmp::min;
use std::fmt;

pub struct LeafNode {
    core_node: Node,
}

impl fmt::Display for LeafNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        node_fmt_with_name(&self.core_node, f, "LeafNode")
    }
}

impl SimElement for LeafNode {
    fn tick(&mut self, tick: u64) -> Vec<(Rpc, String)> {
        let mut ret = vec![];
        for _ in 0..min(
            self.core_node.queue.size(),
            self.core_node.egress_rate as usize,
        ) {
            if self.core_node.queue.size() > 0 {
                let mut deq = self.core_node.dequeue(tick).unwrap();
                // 1. change src/dst headers appropriately, as well as request/response stuff
                let new_dest = self.route_rpc(&deq);

                let dest = deq.headers.get_mut("dest").unwrap();
                *dest = new_dest.clone();
                let src = deq.headers.get_mut("src").unwrap();
                *src = self.core_node.id.clone();
                let direction = deq.headers.get_mut("direction").unwrap();
                if direction == "request" {
                    *direction = "response".to_string();
                } else {
                    *direction = "request".to_string();
                }

                // 2. If there's a plugin, run it through the plugin and then put into ret
                if self.core_node.plugin.is_some() {
                    self.core_node
                        .plugin
                        .as_mut()
                        .unwrap()
                        .recv(deq, tick, &self.core_node.id);
                    let filtered_rpcs = self.core_node.plugin.as_mut().unwrap().tick(tick);
                    for filtered_rpc in filtered_rpcs {
                        ret.push((
                            filtered_rpc.0.clone(),
                            filtered_rpc.0.headers["dest"].clone(),
                        ));
                    }
                } else {
                    ret.push((deq, new_dest.clone()));
                }
            }
        }
        ret
    }
    fn recv(&mut self, rpc: Rpc, tick: u64, _sender: &str) {
        if (self.core_node.queue.size() as u32) < self.core_node.capacity {
            // drop packets you cannot accept
            if self.core_node.plugin.is_none() {
                self.core_node.enqueue(rpc, tick);
            } else {
                // inbound filter check
                self.core_node
                    .plugin
                    .as_mut()
                    .unwrap()
                    .recv(rpc, tick, &self.core_node.id);
                let ret = self.core_node.plugin.as_mut().unwrap().tick(tick);
                for inbound_rpc in ret {
                    self.core_node.enqueue(inbound_rpc.0, tick);
                }
            }
        }
    }
    fn add_connection(&mut self, neighbor: String) {
        self.core_node.add_connection(neighbor)
    }

    fn whoami(&self) -> &str {
        return &self.core_node.whoami();
    }
    fn neighbors(&self) -> &Vec<String> {
        return &self.core_node.neighbors();
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LeafNode {
    pub fn new(id: &str, capacity: u32, egress_rate: u32, plugin: Option<&str>) -> LeafNode {
        assert!(capacity >= 1);
        let core_node = Node::new(id, capacity, egress_rate, 0, plugin, 0);
        LeafNode { core_node }
    }

    pub fn route_rpc(&self, rpc: &Rpc) -> String {
        // "respond to every node that has sent a request"
        if rpc.headers.contains_key("src") {
            return rpc.headers["src"].to_string();
        }
        panic!("Leaf node is missing source header for response! Invalid request.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_node_creation() {
        let _node = LeafNode::new("0", 2, 2, None);
    }

    #[test]
    fn test_node_capacity_and_egress_rate() {
        let mut node = LeafNode::new("0", 2, 1, None);
        node.add_connection("foo".to_string()); // without at least one neighbor, it will just drop rpcs
        assert!(node.core_node.capacity == 2);
        assert!(node.core_node.egress_rate == 1);
        node.core_node.recv(Rpc::new_rpc("0"), 0, "0");
        node.core_node.recv(Rpc::new_rpc("0"), 0, "0");
        assert!(node.core_node.queue.size() == 2);
        node.core_node.recv(Rpc::new_rpc("0"), 0, "0");
        assert!(node.core_node.queue.size() == 2);
        node.core_node.tick(0);
        assert!(node.core_node.queue.size() == 1);
    }

    #[test]
    fn test_plugin_initialization() {
        let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_dir.push("../target/debug/libfilter_example");
        let library_str = cargo_dir.to_str().unwrap();
        let node = LeafNode::new("0", 2, 1, Some(library_str));
        assert!(!node.core_node.plugin.is_none());
    }
}
