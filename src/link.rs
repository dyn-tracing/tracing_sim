use crate::rpc::Rpc;
use crate::sim_element::SimElement;
use queues::*;
use std::fmt;

pub struct Link {
    queue    : Queue<Rpc>,
    id       : u32,
    capacity : u32, // TODO: Unused for now
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(f, "{:width$}",
                   &format!("Link {{ capacity : {}, id : {} }}", &self.capacity, &self.id),
                   width = width)
        } else {
            write!(f, "Link {{ capacity : {}, id : {} }}", &self.capacity, &self.id)
        }
    }
}

impl SimElement for Link {
    fn tick(&mut self, tick : u64) -> Option<Rpc> {
        return self.dequeue(tick);
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
    pub fn new(id : u32) -> Self {
        Link { queue : queue![], id : id, capacity : 0}
    }
}
