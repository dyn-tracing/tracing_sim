//! An abstraction of a node.  The node can have a plugin, which is meant to reprsent a WebAssembly filter
//! A node is a sim_element.

use core::any::Any;
use queues::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rpc_lib::rpc::Rpc;
use sim::node::node_fmt_with_name;
use sim::node::Node;
use sim::sim_element::SimElement;
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
    fn tick(&mut self, tick: u64) -> Vec<Rpc> {
        let mut outgoing_rpcs: Vec<Rpc> = vec![];
        for _ in 0..min(
            self.core_node.queue.size(),
            self.core_node.egress_rate as usize,
        ) {
            if self.core_node.queue.size() == 0 {
                // no rpc in the queue, we only forward so nothing to do
                continue;
            }
            let mut rpc = self.core_node.dequeue(tick).unwrap();
            // forward requests/responses from productpage or reviews
            if !rpc.headers.contains_key("src") {
                panic!("Product page got rpc without a source");
            }
            let mut new_rpcs: Vec<Rpc> = vec![];
            self.choose_destination(&mut rpc, &mut new_rpcs);
            new_rpcs.push(rpc.clone());
            for mut rpc in new_rpcs {
                // If the plugin exists, run the RPC through
                // Otherwise just push it into the egress queue
                if let Some(plugin) = self.core_node.plugin.as_mut() {
                    rpc.headers
                        .insert("location".to_string(), "egress".to_string());
                    plugin.recv(rpc, tick, &self.core_node.id);
                    let filtered_rpcs = plugin.tick(tick);
                    for filtered_rpc in filtered_rpcs {
                        outgoing_rpcs.push(filtered_rpc.clone());
                    }
                } else {
                    outgoing_rpcs.push(rpc);
                }
            }
        }
        outgoing_rpcs
    }

    fn recv(&mut self, rpc: Rpc, tick: u64, sender: &str) {
        self.core_node.recv(rpc, tick, sender);
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

    pub fn choose_destination(&self, rpc: &mut Rpc, new_rpcs: &mut Vec<Rpc>) {
        let review_nodes = vec!["reviews-v1", "reviews-v2", "reviews-v3"];
        let mut rng: StdRng = SeedableRng::seed_from_u64(self.core_node.seed);
        let source: &str = &rpc.headers["src"];
        if review_nodes.contains(&source) || source == "details-v1" {
            rpc.headers
                .insert("dest".to_string(), "gateway".to_string());
        } else if source == "gateway" {
            let idx = rng.gen_range(0, review_nodes.len());
            let dest = review_nodes[idx];
            rpc.headers.insert("dest".to_string(), dest.to_string());
            let mut details_rpc = rpc.clone();
            details_rpc
                .headers
                .insert("dest".to_string(), "details-v1".to_string());
            details_rpc
                .headers
                .insert("src".to_string(), self.core_node.id.to_string());
            new_rpcs.push(details_rpc);
        } else if source == &self.core_node.id {
            // if we are creating a new RPC, eg, we are sending to storage
            // do nothing here
        } else {
            panic!("ProductPage node does not have a valid source!");
        }
        rpc.headers
            .insert("src".to_string(), self.core_node.id.to_string());
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
        let node = ProductPage::new("0", 2, 1, Some(library_str), 0);
        assert!(!node.core_node.plugin.is_none());
    }
}
