//! An abstraction of an edge.  The edge can be unidirectional or bidirectional, depending on who its
//! neighbors are (an edge can only send RPCs to its neighbors).  An edge is a sim_element.
extern crate test;

use crate::sim_element::SimElement;
use core::any::Any;
use queues::*;
use rpc_lib::rpc::Rpc;
use std::fmt;

#[derive(Clone)]
struct TimestampedRpc {
    pub start_time: u64,
    pub rpc: Rpc,
}

pub struct Edge {
    queue: Queue<TimestampedRpc>,
    delay: u64,
    id: String,
    left: String,
    right: String,
    neighbors: Vec<String>,
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

impl fmt::Debug for Edge {
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
    fn tick(&mut self, tick: u64) -> Vec<Rpc> {
        let ret = self.dequeue(tick);
        let mut to_return = Vec::new();
        for element in ret {
            to_return.push(element);
        }
        return to_return;
    }
    fn recv(&mut self, rpc: Rpc, tick: u64) {
        self.enqueue(rpc, tick);
    }
    fn add_connection(&mut self, neighbor: String) {
        assert!(self.neighbors.len() < 2);
        self.neighbors.push(neighbor);
    }
    fn whoami(&self) -> &str {
        &self.id
    }
    fn neighbors(&self) -> &Vec<String> {
        &self.neighbors
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Edge {
    pub fn enqueue(&mut self, x: Rpc, now: u64) {
        self.queue
            .add(TimestampedRpc {
                start_time: now,
                rpc: x,
            })
            .unwrap();
    }
    pub fn dequeue(&mut self, now: u64) -> Vec<Rpc> {
        if self.queue.size() == 0 {
            return vec![];
        } else if self.queue.peek().unwrap().start_time + self.delay <= now {
            let mut ret = vec![];
            // TODO: Clean this up
            while self.queue.size() > 0 && self.queue.peek().unwrap().start_time + self.delay <= now
            {
                // Check that the inequality is an equality, i.e., we didn't skip any ticks.
                assert!(self.queue.peek().unwrap().start_time + self.delay == now);

                // Remove RPC from the head of the queue.
                let queue_element_to_remove = self.queue.remove().unwrap();
                ret.push(queue_element_to_remove.rpc);
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
    pub fn new(left: String, right: String, delay: u64) -> Self {
        let id = left.to_string() + "_" + &right;
        Edge {
            id: id.to_string(),
            delay,
            queue: queue![],
            left: left.clone(),
            right: right.clone(),
            neighbors: vec![left, right],
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
            left: "left".to_string(),
            right: "right".to_string(),
            neighbors: Vec::new(),
        };
    }

    #[bench]
    fn benchmark_enqueue(b: &mut Bencher) {
        let mut edge = Edge::new("left".to_string(), "right".to_string(), 0);
        b.iter(|| {
            for i in 1..100 {
                edge.enqueue(Rpc::new("0"), i)
            }
        });
    }

    #[bench]
    fn benchmark_dequeue(b: &mut Bencher) {
        let mut edge = Edge::new("left".to_string(), "right".to_string(), 0);
        b.iter(|| {
            for i in 1..100 {
                edge.enqueue(Rpc::new("0"), i);
            }
            edge.dequeue(0);
        });
    }
}
