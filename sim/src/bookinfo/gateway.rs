//! An abstraction of the envoy gateway
//! A gateway is a sim_element.

use crate::sim_element::SimElement;
use core::any::Any;
use queues::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rpc_lib::rpc::Rpc;
use std::cmp::min;
use std::fmt;

pub struct Gateway {
    queue: Queue<Rpc>,      // queue of rpcs
    id: String,             // id of the node
    capacity: u32,          // capacity of the node;  how much it can hold at once
    egress_rate: u32,       // rate at which the node can send out rpcs
    generation_rate: u32, // rate at which the node can generate rpcs, which are generated regardless of input to the node
    neighbors: Vec<String>, // who is the node connected to
    seed: u64,
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(f, "{:width$}",
                   &format!("Gateway {{ id : {}, capacity : {}, egress_rate : {}, generation_rate : {}, queue: {}}}",
                   &self.id, &self.capacity, &self.egress_rate, &self.generation_rate, &self.queue.size()),
                   width = width)
        } else {
            write!(f, "Gateway {{ id : {}, egress_rate : {}, generation_rate : {}, capacity : {}, queue : {} }}",
                   &self.id, &self.egress_rate, &self.generation_rate, &self.capacity, &self.queue.size())
        }
    }
}

impl SimElement for Gateway {
    fn tick(&mut self, tick: u64) -> Vec<(Rpc, String)> {
        let mut ret = vec![];
        for _ in 0..min(
            self.queue.size() + (self.generation_rate as usize),
            self.egress_rate as usize,
        ) {
            let mut rpc: Rpc;
            if self.queue.size() > 0 {
                let deq = self.dequeue(tick);
                rpc = deq.unwrap();
            } else {
                rpc = Rpc::new_rpc(&tick.to_string());
                rpc.headers
                    .insert("direction".to_string(), "request".to_string());
            }
            let neigh_len = self.neighbors.len();
            if rpc.headers.contains_key("dest") {
                let mut have_dest = false;
                let dest = &rpc.headers["dest"].clone();
                for n in &self.neighbors {
                    if n == dest {
                        have_dest = true;
                        ret.push((rpc, dest.clone()));
                        break;
                    }
                }
                if !have_dest {
                    print!("WARNING:  RPC given with invalid destination {0}\n", dest);
                }
            } else if neigh_len > 0 && rpc.headers["direction"] == "request".to_string() {
                let mut rng: StdRng = SeedableRng::seed_from_u64(self.seed);
                let idx = rng.gen_range(0, self.neighbors.len());
                let which_neighbor = self.neighbors[idx].clone();
                // the purpose of this for loop is to find the edge with the correct reviews name
                ret.push((rpc, which_neighbor.to_string()));
            }
            // we would normally send client the responses, so the trace stops here;  don't do anything with responses
        }
        ret
    }
    fn recv(&mut self, rpc: Rpc, tick: u64, _sender: &str) {
        if (self.queue.size() as u32) < self.capacity {
            // drop packets you cannot accept
            self.enqueue(rpc, tick);
        }
    }
    fn add_connection(&mut self, neighbor: String) {
        self.neighbors.push(neighbor);
    }

    fn whoami(&self) -> &str {
        return &self.id;
    }
    fn neighbors(&self) -> &Vec<String> {
        return &self.neighbors;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Gateway {
    pub fn enqueue(&mut self, x: Rpc, _now: u64) {
        let _res = self.queue.add(x);
    }
    pub fn dequeue(&mut self, _now: u64) -> Option<Rpc> {
        if self.queue.size() == 0 {
            return None;
        } else {
            return Some(self.queue.remove().unwrap());
        }
    }
    pub fn new(
        id: &str,
        capacity: u32,
        egress_rate: u32,
        generation_rate: u32,
        seed: u64,
    ) -> Gateway {
        assert!(capacity >= 1);
        Gateway {
            queue: queue![],
            id: id.to_string(),
            capacity,
            egress_rate,
            generation_rate,
            neighbors: Vec::new(),
            seed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_node_creation() {
        let _node = Gateway::new("0", 2, 2, 1, 1);
    }

    #[test]
    fn test_node_capacity_and_egress_rate() {
        let mut node = Gateway::new("0", 2, 1, 0, 1);
        assert!(node.capacity == 2);
        assert!(node.egress_rate == 1);
        node.recv(Rpc::new_rpc("0"), 0, "0");
        node.recv(Rpc::new_rpc("0"), 0, "0");
        assert!(node.queue.size() == 2);
        node.recv(Rpc::new_rpc("0"), 0, "0");
        assert!(node.queue.size() == 2);
        node.tick(0);
        assert!(node.queue.size() == 1);
    }
}
