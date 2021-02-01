//! An abstraction of an edge.  The edge can be unidirectional or bidirectional, depending on who its
//! neighbors are (an edge can only send RPCs to its neighbors).  An edge is a sim_element.
extern crate test;

use crate::sim_element::SimElement;
use queues::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rpc_lib::rpc::Rpc;
use std::fmt;

#[derive(Clone)]
struct TimestampedRpc {
    pub start_time: u64,
    pub rpc: Rpc,
    pub sender: String,
}

pub struct Edge {
    queue: Queue<TimestampedRpc>,
    delay: u64,
    id: String,
    neighbors: Vec<String>,
    seed: Option<u64>,
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(
                f,
                "{:width$}",
                &format!(
                    "Edge {{ delay : {}, queue : {}, id : {} }}",
                    &self.delay,
                    &self.queue.size(),
                    self.id
                ),
                width = width
            )
        } else {
            write!(
                f,
                "Edge {{ delay : {}, id : {}, queue : {} }}",
                &self.delay,
                self.id,
                &self.queue.size()
            )
        }
    }
}

impl SimElement for Edge {
    fn tick(&mut self, tick: u64) -> Vec<(Rpc, Option<String>)> {
        let ret = self.dequeue(tick);
        let mut to_return = Vec::new();
        for element in ret {
            to_return.push((element.0, element.1));
        }
        return to_return;
    }
    fn recv(&mut self, rpc: Rpc, tick: u64, sender: &str) {
        self.enqueue(rpc, tick, sender);
    }
    fn add_connection(&mut self, neighbor: String) {
        assert!(self.neighbors.len() < 2);
        self.neighbors.push(neighbor);
    }
    fn whoami(&self) -> (bool, &str, &Vec<String>) {
        return (false, &self.id, &self.neighbors);
    }
}

impl Edge {
    pub fn enqueue(&mut self, x: Rpc, now: u64, sender: &str) {
        self.queue
            .add(TimestampedRpc {
                start_time: now,
                rpc: x,
                sender: sender.to_string(),
            })
            .unwrap();
    }
    pub fn dequeue(&mut self, now: u64) -> Vec<(Rpc, Option<String>, String)> {
        if self.queue.size() == 0 {
            return vec![];
        } else if self.queue.peek().unwrap().start_time + self.delay <= now {
            let mut ret = vec![];
            while self.queue.size() > 0 && self.queue.peek().unwrap().start_time + self.delay <= now
            {
                // Check that the inequality is an equality, i.e., we didn't skip any ticks.
                assert!(self.queue.peek().unwrap().start_time + self.delay == now);

                // Remove RPC from the head of the queue.
                let queue_element_to_remove = self.queue.remove().unwrap();
                let neigh_len = self.neighbors.len();
                if neigh_len > 0 {
                    let idx;
                    if self.seed.is_none() {
                        idx = rand::thread_rng().gen_range(0, neigh_len);
                    } else {
                        let mut rng: StdRng = SeedableRng::seed_from_u64(self.seed.unwrap());
                        idx = rng.gen_range(0, neigh_len);
                    }
                    let which_neighbor = self.neighbors[idx].clone();
                    ret.push((
                        queue_element_to_remove.rpc,
                        Some(which_neighbor),
                        queue_element_to_remove.sender,
                    ));
                } else {
                    ret.push((
                        queue_element_to_remove.rpc,
                        None,
                        queue_element_to_remove.sender,
                    ));
                }
            }
            // Either the queue has emptied or no other RPCs are ready.
            assert!(
                (self.queue.size() == 0)
                    || (self.queue.peek().unwrap().start_time + self.delay > now)
            );
            return ret;
        } else {
            return vec![];
        }
    }
    pub fn new(id: &str, delay: u64, seed: Option<u64>) -> Self {
        Edge {
            id: id.to_string(),
            delay,
            queue: queue![],
            neighbors: Vec::new(),
            seed,
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use test::Bencher;

    #[test]
    fn test_edge() {
        let _edge = Edge {
            id: "0".to_string(),
            queue: queue![],
            delay: 0,
            neighbors: Vec::new(),
        };
    }

    #[bench]
    fn benchmark_enqueue(b: &mut Bencher) {
        let mut edge = Edge::new("0", 0);
        b.iter(|| {
            for i in 1..100 {
                edge.enqueue(Rpc::new_rpc(0), i, "0")
            }
        });
    }

    #[bench]
    fn benchmark_dequeue(b: &mut Bencher) {
        let mut edge = Edge::new("0", 0);
        b.iter(|| {
            for i in 1..100 {
                edge.enqueue(Rpc::new_rpc(0), i, "0");
            }
            edge.dequeue(0);
        });
    }
}
