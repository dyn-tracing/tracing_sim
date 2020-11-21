extern crate test;

use queues::*;

struct Channel {
    channel_queue : Queue<u32>,
}

impl Channel {
    pub fn enqueue(&mut self, x : u32) {
        self.channel_queue.add(x).unwrap();
    }
    pub fn dequeue(&mut self) -> u32 {
        assert!(self.channel_size() > 0);
        self.channel_queue.remove().unwrap()
    }
    pub fn channel_size(& self) -> usize {
        return self.channel_queue.size();
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use test::Bencher;

    #[test]
    fn test_channel() {
        let _channel = Channel { channel_queue : queue![] };
    }

    #[bench]
    fn benchmark_enqueue(b : &mut Bencher) {
        let mut channel = Channel{ channel_queue : queue![] };
        b.iter(|| for _ in 1..100 { channel.enqueue(0) } );
    }

    #[bench]
    fn benchmark_dequeue(b : &mut Bencher) {
        let mut channel = Channel{ channel_queue : queue![] };
        for _ in 1..100 { channel.enqueue(0); }
        b.iter(|| for _ in 1..100 { channel.dequeue(); } );
    }
}
