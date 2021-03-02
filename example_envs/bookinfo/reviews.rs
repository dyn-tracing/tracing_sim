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
        while let Some(mut rpc) = self.core_node.dequeue_ingress(tick) {
            let mut queued_rpcs: Vec<Rpc> = vec![];
            if !rpc.headers.contains_key("src") {
                panic!("Reviews received an RPC without a source");
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
    fn process_rpc(&mut self, rpc: &mut Rpc, new_rpcs: &mut Vec<Rpc>) {
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
        // without at least one neighbor, it will just drop rpcs
        node.add_connection("foo".to_string());
        let mut queue_size: usize;
        assert!(node.core_node.capacity == 2);
        assert!(node.core_node.egress_rate == 1);
        node.recv(Rpc::new_with_src("0", "ratings-v1"), 0);
        node.recv(Rpc::new_with_src("0", "productpage-v1"), 0);
        queue_size = node.core_node.ingress_queue.size();
        assert!(
            node.core_node.ingress_queue.size() == 2,
            "Queue size was `{}`",
            queue_size
        );
        node.recv(Rpc::new_with_src("0", "productpage-v1"), 0);
        queue_size = node.core_node.ingress_queue.size();
        assert!(
            node.core_node.ingress_queue.size() == 2,
            "Queue size was `{}`",
            queue_size
        );
        node.tick(0);
        queue_size = node.core_node.egress_queue.size();
        assert!(queue_size == 1, "Queue size was `{}`", queue_size);
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
