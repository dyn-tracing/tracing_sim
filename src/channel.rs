extern crate test;

use rpc_lib::rpc::Rpc;
use crate::sim_element::SimElement;
use queues::*;
use std::fmt;

#[derive(Clone)]
struct TimestampedRpc {
    pub start_time : u64,
    pub rpc        : Rpc,
}

pub struct Channel {
    queue : Queue<TimestampedRpc>,
    delay : u64,
    id    : u32,
    neighbor : Option<u32>,
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(f, "{:width$}", &format!("Channel {{ delay : {}, id : {}, queue : {} }}",
                   &self.delay, &self.id, &self.queue.size()),
                   width = width)
        } else {
            write!(f, "Channel {{ delay : {}, id : {}, queue : {} }}",
                   &self.delay, self.id, &self.queue.size())
        }
    }
}

impl SimElement for Channel {
    fn tick(&mut self, tick : u64) -> Vec<(Rpc, Option<u32>)> {
        self.dequeue(tick)
    }
    fn recv(&mut self, rpc : Rpc, tick : u64) {
        self.enqueue(rpc, tick);
    }
    fn add_connection(&mut self, neighbor : u32) {
        self.neighbor = Some(neighbor);
    }
}

impl Channel {
    pub fn enqueue(&mut self, x : Rpc, now : u64) {
        self.queue.add(TimestampedRpc{start_time : now, rpc : x}).unwrap();
    }
    pub fn dequeue(&mut self, now : u64) -> Vec<(Rpc, Option<u32>)> {
        if self.queue.size() == 0 {
            return vec![];
        } else if self.queue.peek().unwrap().start_time + self.delay <= now {
            let mut ret = vec![];
            while self.queue.peek().unwrap().start_time + self.delay <= now {
                // Check that the inequality is an equality, i.e., we didn't skip any ticks.
                assert!(self.queue.peek().unwrap().start_time + self.delay == now);

                // Remove RPC from the head of the queue.
                ret.push((self.queue.remove().unwrap().rpc, self.neighbor));
            }
            // Either the queue has emptied or no other RPCs are ready.
            assert!((self.queue.size() == 0) ||
                    (self.queue.peek().unwrap().start_time + self.delay > now));
            return ret;
        } else {
            return vec![];
        }
    }
    pub fn new(delay : u64, id : u32) -> Self {
        Channel { delay : delay, queue : queue![], id : id, neighbor : None }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use test::Bencher;

    #[test]
    fn test_channel() {
        let _channel = Channel { queue : queue![], delay : 0, id : 0, neighbor : None};
    }

    #[bench]
    fn benchmark_enqueue(b : &mut Bencher) {
        let mut channel = Channel{ queue : queue![], delay : 0, id : 0, neighbor : None };
        b.iter(|| for i in 1..100 { channel.enqueue(Rpc::new_rpc(0), i) });
    }

    #[bench]
    fn benchmark_dequeue(b : &mut Bencher) {
        let mut channel = Channel{ queue : queue![], delay : 0, id : 0, neighbor : None };
        b.iter(|| { for i in 1..100 { channel.enqueue(Rpc::new_rpc(0), i); } channel.dequeue(0); } );
    }
}
