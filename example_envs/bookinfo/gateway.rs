//! An abstraction of the envoy gateway
//! A gateway is a sim_element.

use core::any::Any;
use core::cmp::min;
use queues::*;
use rpc_lib::rpc::Rpc;
use sim::node::node_fmt_with_name;
use sim::node::Node;
use sim::node::NodeTraits;
use sim::sim_element::SimElement;
use std::fmt;

pub struct Gateway {
    core_node: Node,
    collected_responses: Vec<Rpc>,
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        node_fmt_with_name(&self.core_node, f, "Gateway")
    }
}

impl SimElement for Gateway {
    fn tick(&mut self, tick: u64) -> Vec<Rpc> {
        while let Some(mut rpc) = self.core_node.dequeue_ingress(tick) {
            let mut queued_rpcs: Vec<Rpc> = vec![];

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
        if let Some(direction) = rpc.headers.get("direction") {
            if direction == "reponse" {
                self.collected_responses.push(rpc);
            }
        } else {
            // RPC without direction information, just pass it on
            self.core_node.recv(rpc, tick);
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

impl NodeTraits for Gateway {
    fn process_rpc(&mut self, rpc: &mut Rpc, new_rpcs: &mut Vec<Rpc>) {
        rpc.headers
            .insert("direction".to_string(), "request".to_string());
        rpc.headers
            .insert("src".to_string(), self.core_node.id.to_string());
        rpc.headers
            .insert("dest".to_string(), "productpage-v1".to_string());
        new_rpcs.push(rpc.clone());
    }
}

impl Gateway {
    pub fn new(
        id: &str,
        capacity: u32,
        egress_rate: u32,
        generation_rate: u32,
        seed: u64,
    ) -> Gateway {
        assert!(capacity >= 1);
        let core_node = Node::new(id, capacity, egress_rate, generation_rate, None, seed);
        Gateway {
            core_node,
            collected_responses: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let _node = Gateway::new("0", 2, 2, 1, 1);
    }

    #[test]
    fn test_node_capacity_and_egress_rate() {
        let mut node = Gateway::new("0", 2, 1, 0, 1);
        // without at least one neighbor, it will just drop rpcs
        node.add_connection("foo".to_string());
        let mut queue_size: usize;
        assert!(node.core_node.capacity == 2);
        assert!(node.core_node.egress_rate == 1);
        node.core_node.enqueue_ingress(Rpc::new("0"), 0);
        node.core_node.enqueue_ingress(Rpc::new("0"), 0);
        queue_size = node.core_node.ingress_queue.size();
        assert!(
            node.core_node.ingress_queue.size() == 2,
            "Queue size was `{}`",
            queue_size
        );
        node.recv(Rpc::new("0"), 0);
        queue_size = node.core_node.ingress_queue.size();
        assert!(
            node.core_node.ingress_queue.size() == 2,
            "Queue size was `{}`",
            queue_size
        );
        let outbound_rpcs = node.tick(0);
        queue_size = node.core_node.egress_queue.size();
        assert!(outbound_rpcs.len() == 1, "Queue size was `{}`", queue_size);
    }
}
