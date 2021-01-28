use crate::plugin_wrapper::PluginWrapper;
use crate::sim_element::SimElement;
use queues::*;
use rand::seq::SliceRandom;
use rpc_lib::rpc::Rpc;
use std::cmp::min;
use std::fmt;

pub struct Node {
    queue: Queue<Rpc>,             // queue of rpcs
    id: u32,                       // id of the node
    capacity: u32,                 // capacity of the node;  how much it can hold at once
    egress_rate: u32,              // rate at which the node can send out rpcs
    generation_rate: u32, // rate at which the node can generate rpcs, which are generated regardless of input to the node
    plugin: Option<PluginWrapper>, // filter to the node
    neighbor: Vec<u32>,   // who is the node connected to
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            if self.plugin.is_none() {
                write!(f, "{:width$}",
                       &format!("Node {{ capacity : {}, egress_rate : {}, generation_rate : {}, plugin : None, id : {}, queue : {} }}",
                       &self.capacity, &self.egress_rate, &self.generation_rate, &self.id, &self.queue.size()),
                       width = width)
            } else {
                write!(f, "{:width$}",
                       &format!("Node {{ capacity : {}, egress_rate : {}, generation_rate : {}, plugin : {}, id : {}, queue : {} }}",
                       &self.capacity, &self.egress_rate, &self.generation_rate, self.plugin.as_ref().unwrap(), &self.id, &self.queue.size()),
                       width = width)
            }
        } else {
            if self.plugin.is_none() {
                write!(f, "Node {{ capacity : {}, egress_rate : {}, generation_rate : {}, plugin : None, id : {}, queue : {} }}",
                       &self.capacity, &self.egress_rate, &self.generation_rate, &self.id, &self.queue.size())
            } else {
                write!(
                    f,
                    "Node {{ capacity : {}, egress_rate : {}, generation_rate : {}, plugin : {}, id : {}, queue : {} }}",
                    &self.capacity,
                    &self.egress_rate,
                    &self.generation_rate,
                    self.plugin.as_ref().unwrap(),
                    &self.id,
                    &self.queue.size()
                )
            }
        }
    }
}

impl SimElement for Node {
    fn tick(&mut self, tick: u64) -> Vec<(Rpc, Option<u32>)> {
        let mut ret = vec![];
        let mut rng = rand::thread_rng();
        for _ in 0..min(
            self.queue.size() + (self.generation_rate as usize),
            self.egress_rate as usize,
        ) {
            let mut which_neighbor = None;
            if self.neighbor.len() > 0 {
                which_neighbor = Some(*self.neighbor.choose(&mut rng).unwrap());
            }
            if self.queue.size() > 0 {
                let deq = self.dequeue(tick);
                assert!(deq.is_some());
                ret.push((deq.unwrap(), which_neighbor));
            } else {
                ret.push((Rpc::new_rpc(tick as u32), which_neighbor));
            }
        }
        ret
    }
    fn recv(&mut self, rpc: Rpc, tick: u64, _sender: u32) {
        if (self.queue.size() as u32) < self.capacity {
            // drop packets you cannot accept
            if self.plugin.is_none() {
                self.enqueue(rpc, tick);
            } else {
                self.plugin.as_mut().unwrap().recv(rpc, tick, self.id);
                let ret = self.plugin.as_mut().unwrap().tick(tick);
                for filtered_rpc in ret {
                    self.enqueue(filtered_rpc.0, tick);
                }
            }
        }
    }
    fn add_connection(&mut self, neighbor: u32) {
        self.neighbor.push(neighbor);
    }
    fn whoami(&self) -> (bool, u32, Vec<u32>) {
        return (true, self.id, self.neighbor.clone());
    }
}

impl Node {
    pub fn enqueue(&mut self, x: Rpc, _now: u64) {
        self.queue.add(x).unwrap();
    }
    pub fn dequeue(&mut self, _now: u64) -> Option<Rpc> {
        if self.queue.size() == 0 {
            return None;
        } else {
            return Some(self.queue.remove().unwrap());
        }
    }
    pub fn new(
        capacity: u32,
        egress_rate: u32,
        generation_rate: u32,
        plugin: Option<&str>,
        id: u32,
    ) -> Node {
        assert!(capacity >= 1);
        if plugin.is_none() {
            Node {
                queue: queue![],
                id,
                capacity,
                egress_rate,
                generation_rate,
                plugin: None,
                neighbor: Vec::new(),
            }
        } else {
            let created_plugin = PluginWrapper::new(plugin.unwrap(), id);
            Node {
                queue: queue![],
                id,
                capacity,
                egress_rate,
                generation_rate,
                plugin: Some(created_plugin),
                neighbor: Vec::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_node_creation() {
        let _node = Node::new(2, 2, 1, None, 0);
    }

    #[test]
    fn test_node_capacity_and_egress_rate() {
        let mut node = Node::new(2, 1, 0, None, 0);
        assert!(node.capacity == 2);
        assert!(node.egress_rate == 1);
        node.recv(Rpc::new_rpc(0), 0, 0);
        node.recv(Rpc::new_rpc(0), 0, 0);
        assert!(node.queue.size() == 2);
        node.recv(Rpc::new_rpc(0), 0, 0);
        assert!(node.queue.size() == 2);
        node.tick(0);
        assert!(node.queue.size() == 1);
    }

    #[test]
    fn test_plugin_initialization() {
        let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_dir.push("../target/debug/libfilter_example");
        let library_str = cargo_dir.to_str().unwrap();
        let node = Node::new(2, 1, 0, Some(library_str), 0);
        assert!(!node.plugin.is_none());
    }
}