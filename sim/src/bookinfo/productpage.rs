//! An abstraction of a node.  The node can have a plugin, which is meant to reprsent a WebAssembly filter
//! A node is a sim_element.

use crate::node::node_fmt_with_name;
use crate::node::Node;
use crate::sim_element::SimElement;
use core::any::Any;
use queues::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rpc_lib::rpc::Rpc;
use std::cmp::min;
use std::fmt;

pub struct ProductPage {
    core_node: Node,
}

impl fmt::Display for ProductPage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        node_fmt_with_name(&self.core_node, f, "ProductPage")
    }
}

impl SimElement for ProductPage {
    fn tick(&mut self, tick: u64) -> Vec<(Rpc, String)> {
        let mut ret = vec![];
        let review_nodes = vec!["reviews-v1", "reviews-v2", "reviews-v3"];
        let mut rng: StdRng = SeedableRng::seed_from_u64(self.core_node.seed);

        for _ in 0..min(
            self.core_node.queue.size(),
            self.core_node.egress_rate as usize,
        ) {
            let mut rpc: Rpc;
            if self.core_node.queue.size() > 0 {
                let deq = self.core_node.dequeue(tick);
                rpc = deq.unwrap();
            } else {
                // No RPC in the queue and we only forward. So nothing to do
                continue;
            }
            // Forward requests/responses from the gateway to one of reviews
            // Also send a request to details-v1
            // If we get a reply from one of reviews or details,
            // send it back to the gateway
            if rpc.headers.contains_key("src") {
                let source: &str = &rpc.headers["src"];
                if review_nodes.contains(&source) || source == "details-v1" {
                    rpc.headers
                        .insert("src".to_string(), self.core_node.id.to_string());
                    ret.push((rpc, "gateway".to_string()));
                } else if source == "gateway" {
                    let idx = rng.gen_range(0, review_nodes.len());
                    let dest = review_nodes[idx];
                    rpc.headers
                        .insert("src".to_string(), self.core_node.id.to_string());
                    ret.push((rpc.clone(), dest.to_string()));
                    // also send a request to details-v1
                    ret.push((rpc, "details-v1".to_string()));
                } 
                else if source == &self.core_node.id {
                    let dest: &str = &rpc.headers["dest"];
                    ret.push((rpc.clone(), dest.to_string()));
                }
            } else {
                panic!("ProductPage node is missing source header for forwarding! Invalid RPC.");
            }
        }
        ret
    }
    fn recv(&mut self, rpc: Rpc, tick: u64, sender: &str) {
        self.core_node.recv(rpc, tick, sender)
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

impl ProductPage {
    pub fn new(
        id: &str,
        capacity: u32,
        egress_rate: u32,
        plugin: Option<&str>,
        seed: u64,
    ) -> ProductPage {
        assert!(capacity >= 1);
        let core_node = Node::new(id, capacity, egress_rate, 0, plugin, seed);
        ProductPage { core_node }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_node_creation() {
        let _node = ProductPage::new("0", 2, 2, None, 0);
    }

    #[test]
    fn test_node_capacity_and_egress_rate() {
        let mut node = ProductPage::new("0", 2, 1, None, 0);
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
        let node = ProductPage::new("0", 2, 1, Some(library_str), 0);
        assert!(!node.core_node.plugin.is_none());
    }
}
