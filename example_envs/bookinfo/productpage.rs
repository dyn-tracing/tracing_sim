use core::any::Any;
use queues::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rpc_lib::rpc::Rpc;
use sim::node::node_fmt_with_name;
use sim::node::Node;
use sim::node::NodeTraits;
use sim::sim_element::SimElement;
use std::cmp::min;
use std::fmt;
use std::collections::HashMap;

pub struct PendingRpc {
    details_rpc: Option<Rpc>,
    review_rpc: Option<Rpc>,
}

impl PendingRpc {
    fn default() -> PendingRpc { PendingRpc { details_rpc: None, review_rpc: None } }
}

pub struct ProductPage {
    core_node: Node,
    pending_rpcs: HashMap<u64, PendingRpc>
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
                // No RPC in the queue. We only forward, so nothing to do
                continue;
            }
            let mut rpc = self.core_node.dequeue(tick).unwrap();
            // Forward requests/responses from productpage or reviews
            if !rpc.headers.contains_key("src") {
                panic!("Product page got rpc without a source");
            }
            // Process the RPC
            let mut new_rpcs: Vec<Rpc> = vec![];
            self.process_rpc(&mut rpc, &mut new_rpcs);

            // Pass the RPCs we have through the plugin
            for rpc in new_rpcs {
                self.core_node
                    .pass_through_plugin(rpc, &mut outgoing_rpcs, tick, "egress");
            }
        }
        outgoing_rpcs
    }

    fn recv(&mut self, rpc: Rpc, tick: u64) {
        // drop packets you cannot accept
        if (self.core_node.queue.size() as u32) < self.core_node.capacity {
            let mut inbound_rpcs: Vec<Rpc> = vec![];
            self.core_node.pass_through_plugin(rpc, &mut inbound_rpcs, tick, "ingress");
            for inbound_rpc in inbound_rpcs {
                if self.pending_rpcs.contains_key(&inbound_rpc.uid) {
                    if inbound_rpc.headers["src"].contains("review") {
                        self.pending_rpcs.get_mut(&inbound_rpc.uid).unwrap().review_rpc = Some(inbound_rpc.clone());
                    }
                    else if inbound_rpc.headers["src"].contains("details") {
                        self.pending_rpcs.get_mut(&inbound_rpc.uid).unwrap().details_rpc = Some(inbound_rpc.clone());
                    }
                }
                else { 
                    self.pending_rpcs.insert(inbound_rpc.uid, PendingRpc::default());
                    self.core_node.enqueue(inbound_rpc, tick);
                }
            }
            let mut to_remove = Vec::new();
            for trace_id in self.pending_rpcs.keys() {
                let pending = &self.pending_rpcs[trace_id];
                if pending.review_rpc.is_some() && pending.details_rpc.is_some() {
                    let new_rpc = ProductPage::merge_rpcs(pending.review_rpc.as_ref().unwrap().clone(), pending.details_rpc.as_ref().unwrap().clone());
                    self.core_node.enqueue(new_rpc, tick);
                    to_remove.push(trace_id.clone());
                }
            }
            for trace_id_to_remove in to_remove {
                self.pending_rpcs.remove(&trace_id_to_remove);
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

impl NodeTraits for ProductPage {
    fn process_rpc(&self, rpc: &mut Rpc, new_rpcs: &mut Vec<Rpc>) {
        let review_nodes = vec!["reviews-v1", "reviews-v2", "reviews-v3"];
        let mut rng: StdRng = SeedableRng::seed_from_u64(self.core_node.seed);
        let source: &str = &rpc.headers["src"];
        if review_nodes.contains(&source) || source.contains("details-v1") {
            // contains is used rather than an exact match in order to accomodate merging rpcs
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
        } else {
            panic!("ProductPage node does not have a valid source!");
        }
        rpc.headers
            .insert("src".to_string(), self.core_node.id.to_string());
        new_rpcs.push(rpc.clone());
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
        ProductPage { core_node, pending_rpcs: HashMap::new() }
    }

    pub fn merge_rpcs(rpc1: Rpc, rpc2: Rpc) -> Rpc {
        assert!(rpc1.uid==rpc2.uid);
        let mut new_headers = rpc1.headers.clone();
        for key in rpc2.headers.keys() {
            if new_headers.contains_key(key) {
                let mut new_value = rpc1.headers[key].clone();
                new_value.push_str(&rpc2.headers[key]); // concatenate
                new_headers.insert(key.to_string(), new_value);
            }
            else { new_headers.insert(key.to_string(), rpc2.headers[key].clone()); }
        }
        let mut new_data = rpc1.data;
        new_data.push_str(&rpc2.data);

        let mut new_path = rpc1.path;
        new_path.push_str(&rpc2.path);

        Rpc {
            data: new_data,
            uid: rpc1.uid,
            path: new_path,
            headers: new_headers,
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
