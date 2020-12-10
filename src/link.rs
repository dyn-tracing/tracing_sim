use crate::rpc::Rpc;
use crate::sim_element::SimElement;
use queues::*;
use std::fmt;
use std::cmp::min;

pub struct Link {
    queue    : Queue<Rpc>,
    id       : u32,
    capacity : u32,
    neighbor : Option<u32>,
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
    fn tick(&mut self, tick : u64) -> Vec<(Rpc, Option<u32>)> {
        let mut ret = vec![];
        for _ in 0..min(self.capacity as usize, self.queue.size()) {
            let deq = self.dequeue(tick);
            assert!(deq.is_some());
            ret.push((deq.unwrap(), self.neighbor));
        }
        ret
    }
    fn recv(&mut self, rpc : Rpc, tick : u64) {
        self.enqueue(rpc, tick);
    }
    fn add_connection(&mut self, neighbor : u32) {
        self.neighbor = Some(neighbor);
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
    pub fn new(capacity : u32, id : u32) -> Self {
        assert!(capacity >= 1);
        Link { queue : queue![], id : id, capacity : capacity, neighbor : None }
    }
}
