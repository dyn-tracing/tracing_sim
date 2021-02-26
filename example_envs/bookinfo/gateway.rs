//! An abstraction of the envoy gateway
//! A gateway is a sim_element.

use core::any::Any;
use rpc_lib::rpc::Rpc;
use sim::node::node_fmt_with_name;
use sim::node::Node;
use sim::sim_element::SimElement;
use std::cmp::min;
use std::fmt;

pub struct Gateway {
    core_node: Node,
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        node_fmt_with_name(&self.core_node, f, "Gateway")
    }
}

impl SimElement for Gateway {
    fn tick(&mut self, tick: u64) -> Vec<Rpc> {
        let mut ret = vec![];
        for _ in 0..min(
            self.core_node.generation_rate as usize,
            self.core_node.egress_rate as usize,
        ) {
            let mut rpc: Rpc;
            rpc = Rpc::new_rpc(&tick.to_string());
            rpc.headers
                .insert("direction".to_string(), "request".to_string());
            rpc.headers
                .insert("src".to_string(), self.core_node.id.to_string());
            rpc.headers
                .insert("dest".to_string(), "productpage-v1".to_string());
            ret.push(rpc);
        }
        ret
    }
    fn recv(&mut self, _rpc: Rpc, _tick: u64) {
        // we discard anything we receive
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
        Gateway { core_node }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use queues::*;

    #[test]
    fn test_node_creation() {
        let _node = Gateway::new("0", 2, 2, 1, 1);
    }

    #[test]
    fn test_node_capacity_and_egress_rate() {
        let mut node = Gateway::new("0", 2, 1, 0, 1);
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
}
