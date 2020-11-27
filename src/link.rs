use crate::rpc::Rpc;
use crate::sim_element::SimElement;
use queues::*;
use std::fmt;
use std::cmp::min;

pub struct Link {
    queue    : Queue<Rpc>,
    id       : u32,
    capacity : u32,
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(f, "{:width$}",
                   &format!("Link {{ capacity : {}, id : {}, queue : {} }}",
                   &self.capacity, &self.id, &self.queue.size()),
                   width = width)
        } else {
            write!(f, "Link {{ capacity : {}, id : {}, queue : {} }}",
                   &self.capacity, &self.id, &self.queue.size())
        }
    }
}

impl SimElement for Link {
    fn tick(&mut self, tick : u64) -> Vec<Rpc> {
        let mut ret = vec![];
        for _ in 0..min(self.capacity as usize, self.queue.size()) {
            let deq = self.dequeue(tick);
            assert!(deq.is_some());
            ret.push(deq.unwrap());
        }
        ret
    }
    fn recv(&mut self, rpc : Rpc, tick : u64) {
        self.enqueue(rpc, tick);
    }
}

impl Link {
    pub fn enqueue(&mut self, x : Rpc, _now : u64) {
        self.queue.add(x).unwrap();
    }
    pub fn dequeue(&mut self, _now : u64) -> Option<Rpc> {
        if self.queue.size() == 0 {
            return None;
        } else  {
            return Some(self.queue.remove().unwrap())
        }
    }
    pub fn new(id : u32, capacity : u32) -> Self {
        Link { queue : queue![], id : id, capacity : capacity}
    }
}
