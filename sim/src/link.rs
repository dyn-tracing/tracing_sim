use crate::plugin_wrapper::PluginWrapper;
use crate::sim_element::SimElement;
use queues::*;
use rand::seq::SliceRandom;
use rpc_lib::rpc::Rpc;
use std::cmp::min;
use std::fmt;

pub struct Link {
    queue: Queue<Rpc>,
    id: u32,
    capacity: u32,
    egress_rate: u32,
    plugin: Option<PluginWrapper>,
    neighbor: Vec<u32>,
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            if self.plugin.is_none() {
                write!(f, "{:width$}",
                       &format!("Link {{ capacity : {}, egress_rate : {}, plugin : None, id : {}, queue : {} }}",
                       &self.capacity, &self.egress_rate, &self.id, &self.queue.size()),
                       width = width)
            } else {
                write!(f, "{:width$}",
                       &format!("Link {{ capacity : {}, egress_rate : {}, plugin : {}, id : {}, queue : {} }}",
                       &self.capacity, &self.egress_rate, self.plugin.as_ref().unwrap(), &self.id, &self.queue.size()),
                       width = width)
            }
        } else {
            if self.plugin.is_none() {
                write!(f, "Link {{ capacity : {}, egress_rate : {}, plugin : None, id : {}, queue : {} }}",
                       &self.capacity, &self.egress_rate, &self.id, &self.queue.size())
            } else {
                write!(
                    f,
                    "Link {{ capacity : {}, egress_rate : {}, plugin : {}, id : {}, queue : {} }}",
                    &self.capacity,
                    &self.egress_rate,
                    self.plugin.as_ref().unwrap(),
                    &self.id,
                    &self.queue.size()
                )
            }
        }
    }
}

impl SimElement for Link {
    fn tick(&mut self, tick: u64) -> Vec<(Rpc, Option<u32>)> {
        let mut ret = vec![];
        for _ in 0..min(self.queue.size(), self.egress_rate as usize) {
            let deq = self.dequeue(tick);
            assert!(deq.is_some());
            if self.neighbor.len() > 0 {
                let which_neighbor: u32 = *self.neighbor.choose(&mut rand::thread_rng()).unwrap();
                ret.push((deq.unwrap(), Some(which_neighbor)));
            } else {
                ret.push((deq.unwrap(), None));
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
    fn whoami(&self) -> (&str, u32, Vec<u32>) {
        return ("Link", self.id, self.neighbor.clone());
    }
}

impl Link {
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
    pub fn new(capacity: u32, egress_rate: u32, plugin: Option<&str>, id: u32) -> Link {
        assert!(capacity >= 1);
        if plugin.is_none() {
            Link {
                queue: queue![],
                id,
                capacity,
                egress_rate,
                plugin: None,
                neighbor: Vec::new(),
            }
        } else {
            let created_plugin = PluginWrapper::new(plugin.unwrap(), id);
            Link {
                queue: queue![],
                id,
                capacity,
                egress_rate,
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
    fn test_link_creation() {
        let _link = Link::new(2, 2, None, 0);
    }

    #[test]
    fn test_link_capacity_and_egress_rate() {
        let mut link = Link::new(2, 1, None, 0);
        assert!(link.capacity == 2);
        assert!(link.egress_rate == 1);
        link.recv(Rpc::new_rpc(0), 0, 0);
        link.recv(Rpc::new_rpc(0), 0, 0);
        assert!(link.queue.size() == 2);
        link.recv(Rpc::new_rpc(0), 0, 0);
        assert!(link.queue.size() == 2);
        link.tick(0);
        assert!(link.queue.size() == 1);
    }

    #[test]
    fn test_plugin_initialization() {
        let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_dir.push("../target/debug/libfilter_lib");
        let library_str = cargo_dir.to_str().unwrap();
        let link = Link::new(2, 1, Some(library_str), 0);
        assert!(!link.plugin.is_none());
    }
}
