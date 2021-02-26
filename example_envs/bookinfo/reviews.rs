use core::any::Any;
use queues::*;
use rpc_lib::rpc::Rpc;
use sim::node::node_fmt_with_name;
use sim::node::Node;
use sim::node::NodeTraits;
use sim::sim_element::SimElement;
use std::cmp::min;
use std::fmt;

pub struct Reviews {
    core_node: Node,
}

impl fmt::Display for Reviews {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        node_fmt_with_name(&self.core_node, f, "Reviews")
    }
}

impl SimElement for Reviews {
    fn tick(&mut self, tick: u64) -> Vec<Rpc> {
        let mut outgoing_rpcs: Vec<Rpc> = vec![];
        for _ in 0..min(
            self.core_node.queue.size(),
            self.core_node.egress_rate as usize,
        ) {
            let mut rpc: Rpc;
            if self.core_node.queue.size() > 0 {
                let deq = self.core_node.dequeue(tick);
                rpc = deq.unwrap();
            } else {
                // no rpc in the queue, we only forward so nothing to do
                continue;
            }
            if !rpc.headers.contains_key("src") {
                panic!("Reviews node is missing source header for forwarding! Invalid RPC.");
            }
            // Process the RPC
            let mut new_rpcs: Vec<Rpc> = vec![];
            self.process_rpc(&mut rpc, &mut new_rpcs);

            // Pass the rpc we have through the plugin
            self.core_node
                .pass_through_plugin(new_rpcs, &mut outgoing_rpcs, tick, "egress");
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

impl NodeTraits for Reviews {
    fn process_rpc(&self, rpc: &mut Rpc, new_rpcs: &mut Vec<Rpc>) {
        let source = &rpc.headers["src"];
        if source == "ratings-v1" {
            rpc.headers
                .insert("dest".to_string(), "productpage-v1".to_string());
        } else if source == "productpage-v1" {
            rpc.headers
                .insert("dest".to_string(), "ratings-v1".to_string());
        } else {
            panic!("Unexpected RPC source {:?}", source);
        }
        rpc.headers
            .insert("src".to_string(), self.core_node.id.to_string());
        new_rpcs.push(rpc.clone());
    }
}

impl Reviews {
    pub fn new(id: &str, capacity: u32, egress_rate: u32, plugin: Option<&str>) -> Reviews {
        assert!(capacity >= 1);
        let core_node = Node::new(id, capacity, egress_rate, 0, plugin, 0);
        Reviews { core_node }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    #[test]
    fn test_node_creation() {
        let _node = Reviews::new("0", 2, 2, None);
    }

    #[test]
    fn test_node_capacity_and_egress_rate() {
        let mut node = Reviews::new("0", 2, 1, None);
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
        let node = Reviews::new("0", 2, 1, Some(library_str));
        assert!(!node.core_node.plugin.is_none());
    }
}
