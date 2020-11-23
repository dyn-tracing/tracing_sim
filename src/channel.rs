extern crate test;

use crate::rpc::Rpc;
use queues::*;

#[derive(Clone)]
struct TimestampedRpc {
    pub start_time : u64,
    pub rpc       : Rpc,
}

struct Channel {
    channel_queue : Queue<TimestampedRpc>,
    channel_delay : u64,
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

            return Some(rpc);
        } else {
            return None;
        }
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
        b.iter(|| for i in 1..100 { channel.enqueue(Rpc { id : 0}, i) });
    }

    #[bench]
    fn benchmark_dequeue(b : &mut Bencher) {
        let mut channel = Channel{ channel_queue : queue![], channel_delay : 0 };
        b.iter(|| { for i in 1..100 { channel.enqueue(Rpc { id : 0}, i); } for i in 1..100 { channel.dequeue(i); } } );
    }
}
