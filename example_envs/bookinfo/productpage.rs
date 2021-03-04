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
    details_reply: Option<Rpc>,
    reviews_reply: Option<Rpc>,
}

impl PendingRpc {
    fn default() -> PendingRpc {
        PendingRpc {
            details_reply: None,
            reviews_reply: None,
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
        while let Some(mut rpc) = self.core_node.dequeue_ingress(tick) {
            let mut queued_rpcs: Vec<Rpc> = vec![];
            // Forward requests/responses from productpage or reviews
            if !rpc.headers.contains_key("src") {
                log::error!("Productpage received an RPC without a source");
                std::process::exit(1);
            }
            // Process the RPC
            let mut new_rpcs: Vec<Rpc> = vec![];
            self.process_rpc(&mut rpc, &mut new_rpcs);

            // Pass the RPCs we have through the plugin
            for rpc in new_rpcs {
                self.core_node
                    .pass_through_plugin(rpc, &mut queued_rpcs, tick, "egress");
            }
            for queued_rpcs in &queued_rpcs {
                self.core_node.enqueue_egress(queued_rpcs.clone())
            }
        }

        let max_output = min(
            self.core_node.egress_queue.size(),
            self.core_node.egress_rate as usize,
        );
        let mut outbound_rpcs: Vec<Rpc> = vec![];
        for _ in 0..max_output {
            outbound_rpcs.push(self.core_node.dequeue_egress().unwrap())
        }
        outbound_rpcs
    }

    fn recv(&mut self, rpc: Rpc, tick: u64) {
        // drop packets you cannot accept
        let mut cur_queue = self.core_node.ingress_queue.size() as u32;
        cur_queue += self.core_node.egress_queue.size() as u32;
        if cur_queue >= self.core_node.capacity {
            return;
        }
        let uid = rpc.uid;
        let mut inbound_rpcs: Vec<Rpc> = vec![];
        self.core_node
            .pass_through_plugin(rpc, &mut inbound_rpcs, tick, "ingress");

        // Check the inbound rpcs
        for inbound_rpc in inbound_rpcs {
            let rpc_source = &inbound_rpc.headers["src"];
            // Custom receive if we have a response from reviews or details
            if rpc_source == "details-v1" || rpc_source.starts_with("reviews") {
                // We only enqueue if handle+reply returns an RPC
                if let Some(merged_rpc) = self.handle_reply(uid, inbound_rpc) {
                    self.core_node.enqueue_ingress(merged_rpc, tick);
                }
            } else {
                self.core_node.enqueue_ingress(inbound_rpc, tick);
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
    fn process_rpc(&mut self, rpc: &mut Rpc, new_rpcs: &mut Vec<Rpc>) {
        let review_nodes = vec!["reviews-v1", "reviews-v2", "reviews-v3"];
        let mut rng: StdRng = SeedableRng::seed_from_u64(self.core_node.seed);
        let source: &str = &rpc.headers["src"];
        // internal_rpc represents the merged header of details and reviews responses
        if source == "internal_rpc" {
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
        ProductPage {
            core_node,
            pending_rpcs: HashMap::new(),
        }
    }

    fn handle_reply(&mut self, uid: u64, inbound_rpc: Rpc) -> Option<Rpc> {
        // If the trace is not tracked yet insert the id
        if !self.pending_rpcs.contains_key(&uid) {
            self.pending_rpcs.insert(uid, PendingRpc::default());
        }
        let pending_rpc = self.pending_rpcs.get_mut(&uid).unwrap();
        // Check the source and "activate" the member of the struct
        if inbound_rpc.headers["src"] == "details-v1" {
            pending_rpc.details_reply = Some(inbound_rpc);
        } else if inbound_rpc.headers["src"].starts_with("reviews") {
            pending_rpc.reviews_reply = Some(inbound_rpc);
        }
        // Only if both struct members are active we return an RPC
        if pending_rpc.details_reply.is_some() && pending_rpc.reviews_reply.is_some() {
            // Create a dummy for now, the filter is supposed to do this
            let mut merged_rpc = Rpc::new("response");
            merged_rpc
                .headers
                .insert("direction".to_string(), "response".to_string());
            merged_rpc
                .headers
                .insert("src".to_string(), "internal_rpc".to_string());
            // We are done with this trace. Clear the map enty
            self.pending_rpcs.remove(&uid);
            return Some(merged_rpc);
        }
        return None;
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
        // without at least one neighbor, it will just drop rpcs
        node.add_connection("foo".to_string());
        let mut queue_size: usize;
        assert!(node.core_node.capacity == 2);
        assert!(node.core_node.egress_rate == 1);
        node.recv(Rpc::new_with_src("0", "gateway"), 0);
        node.recv(Rpc::new_with_src("0", "gateway"), 0);
        queue_size = node.core_node.ingress_queue.size();
        assert!(
            node.core_node.ingress_queue.size() == 2,
            "Queue size was `{}`",
            queue_size
        );
        node.recv(Rpc::new_with_src("0", "gateway"), 0);
        queue_size = node.core_node.ingress_queue.size();
        assert!(
            node.core_node.ingress_queue.size() == 2,
            "Queue size was `{}`",
            queue_size
        );
        node.tick(0);
        queue_size = node.core_node.egress_queue.size();
        assert!(queue_size == 3, "Queue size was `{}`", queue_size);
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
