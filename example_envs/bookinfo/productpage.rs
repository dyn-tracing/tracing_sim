use core::any::Any;
use queues::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rpc_lib::rpc::Rpc;
use sim::node::node_fmt_with_name;
use sim::node::Node;
use sim::node::NodeTraits;
use sim::sim_element::SimElement;
use std::cmp::min;
use std::collections::HashMap;
use std::fmt;

pub struct PendingRpc {
    details_rpc: Option<Rpc>,
    review_rpc: Option<Rpc>,
}

impl PendingRpc {
    fn default() -> PendingRpc {
        PendingRpc {
            details_rpc: None,
            review_rpc: None,
        }
    }
}

pub struct ProductPage {
    core_node: Node,
    pending_rpcs: HashMap<u64, PendingRpc>,
}

impl fmt::Display for ProductPage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        node_fmt_with_name(&self.core_node, f, "ProductPage")
    }
}

impl SimElement for ProductPage {
    fn tick(&mut self, tick: u64) -> Vec<Rpc> {
        let mut outgoing_rpcs: Vec<Rpc> = vec![];
        let max_output = min(
            self.core_node.queue.size(),
            self.core_node.egress_rate as usize,
        );
        let mut sent_rpcs = 0;
        while sent_rpcs < max_output {
            if self.core_node.queue.size() == 0 {
                // No RPC in the queue. We only forward, so nothing to do
                return outgoing_rpcs;
            }
            let mut rpc = self.core_node.dequeue(tick).unwrap();
            // Forward requests/responses from productpage or reviews
            if !rpc.headers.contains_key("src") {
                panic!("Product page got rpc without a source");
            }
            // Process the RPC
            let mut new_rpcs: Vec<Rpc> = vec![];
            self.process_rpc(&mut rpc, &mut new_rpcs);

            // make sure to count new rpcs among all outgoing rpcs
            sent_rpcs += new_rpcs.len();

            // Pass the RPCs we have through the plugin
            for rpc in new_rpcs {
                self.core_node
                    .pass_through_plugin(rpc, &mut outgoing_rpcs, tick, "egress");
            }
        }
        outgoing_rpcs
    }

    fn recv(&mut self, rpc: Rpc, tick: u64) {
        self.core_node.recv(rpc, tick);
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

impl NodeTraits for ProductPage {
    fn process_rpc(&mut self, rpc: &mut Rpc, new_rpcs: &mut Vec<Rpc>) {
        if !self.pending_rpcs.contains_key(&rpc.uid) {
            self.pending_rpcs.insert(rpc.uid, PendingRpc::default());
            // this RPC must have come from the gateway;  if it came from
            // reviews or details, we should already have its uid in our pending_rpcs
            assert!(rpc.headers["src"] == "gateway");

            // make and add details rpc
            let mut details_rpc = rpc.clone();
            details_rpc
                .headers
                .insert("dest".to_string(), "details-v1".to_string());
            details_rpc
                .headers
                .insert("src".to_string(), self.core_node.id.to_string());
            new_rpcs.push(details_rpc);

            // make and add reviews rpc
            let review_nodes = vec!["reviews-v1", "reviews-v2", "reviews-v3"];
            let mut rng: StdRng = SeedableRng::seed_from_u64(self.core_node.seed);
            let idx = rng.gen_range(0, review_nodes.len());
            let dest = review_nodes[idx];
            rpc.headers.insert("dest".to_string(), dest.to_string());
            rpc.headers
                .insert("src".to_string(), self.core_node.id.to_string());
            new_rpcs.push(rpc.clone());
        } else {
            let pending_struct = self.pending_rpcs.get_mut(&rpc.uid).unwrap();
            if rpc.headers["src"].contains("reviews") {
                pending_struct.review_rpc = Some(rpc.clone());
            }
            if rpc.headers["src"].contains("details") {
                pending_struct.details_rpc = Some(rpc.clone());
            }
            if pending_struct.review_rpc.is_some() && pending_struct.details_rpc.is_some() {
                // send back to gateway
                rpc.headers
                    .insert("src".to_string(), self.core_node.id.to_string());
                rpc.headers
                    .insert("dest".to_string(), "gateway".to_string());
                new_rpcs.push(rpc.clone());
            }
        }
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
        ProductPage {
            core_node,
            pending_rpcs: HashMap::new(),
        }
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
        node.core_node.recv(Rpc::new_rpc("0"), 0);
        node.core_node.recv(Rpc::new_rpc("0"), 0);
        assert!(node.core_node.queue.size() == 2);
        node.core_node.recv(Rpc::new_rpc("0"), 0);
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
