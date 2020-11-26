extern crate test;

use crate::rpc::Rpc;
use crate::sim_element::SimElement;
use queues::*;
use std::fmt;

#[derive(Clone)]
struct TimestampedRpc {
    pub start_time : u64,
    pub rpc        : Rpc,
}

pub struct Channel {
    channel_queue : Queue<TimestampedRpc>,
    channel_delay : u64,
}

impl fmt::Debug for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Channel")
            .field("channel_delay", &self.channel_delay) // TODO: Add channel ID
            .finish()
    }
}

impl SimElement for Channel {
    fn tick(&mut self, tick : u64) -> Option<Rpc> {
        return self.dequeue(tick);
    }
    fn recv(&mut self, rpc : Rpc, tick : u64) {
        self.enqueue(rpc, tick);
    }
}

impl Channel {
    pub fn enqueue(&mut self, x : Rpc, now : u64) {
        self.channel_queue.add(TimestampedRpc{start_time : now, rpc : x}).unwrap();
    }
    pub fn dequeue(&mut self, now : u64) -> Option<Rpc> {
        if self.channel_queue.size() == 0 {
            return None;
        } else if self.channel_queue.peek().unwrap().start_time + self.channel_delay <= now {
            // Check that the inequality is an equality, i.e., we didn't skip any ticks.
            assert!(self.channel_queue.peek().unwrap().start_time + self.channel_delay == now);

            // Remove RPC from the head of the queue.
            let rpc = self.channel_queue.remove().unwrap().rpc;

            // Either the queue has emptied or no other RPCs are ready.
            assert!((self.channel_queue.size() == 0) ||
                    (self.channel_queue.peek().unwrap().start_time + self.channel_delay > now));
            // println!("Dequeue {:?} out of channel at {}", rpc, now);
            return Some(rpc);
        } else {
            return None;
        }
    }
    pub fn new(channel_delay : u64) -> Self {
        Channel { channel_delay : channel_delay, channel_queue : queue![] }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use test::Bencher;

    #[test]
    fn test_channel() {
        let _channel = Channel { channel_queue : queue![], channel_delay : 0 };
    }

    #[bench]
    fn benchmark_enqueue(b : &mut Bencher) {
        let mut channel = Channel{ channel_queue : queue![], channel_delay : 0 };
        b.iter(|| for i in 1..100 { channel.enqueue(Rpc::new_rpc(0), i) });
    }

    #[bench]
    fn benchmark_dequeue(b : &mut Bencher) {
        let mut channel = Channel{ channel_queue : queue![], channel_delay : 0 };
        b.iter(|| { for i in 1..100 { channel.enqueue(Rpc::new_rpc(0), i); } for i in 1..100 { channel.dequeue(i); } } );
    }
}
