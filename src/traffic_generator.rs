use crate::rpc::Rpc;
use crate::sim_element::SimElement;
use std::fmt;
use std::convert::TryInto;

pub struct TrafficGenerator {
    rate : u32, // Rpcs per tick
    id   : u32,
}

impl TrafficGenerator {
    pub fn new(rate : u32, id : u32) -> Self {
        return TrafficGenerator { rate : rate, id : id };
    }
}

impl SimElement for TrafficGenerator {
    fn recv(&mut self, _rpc : Rpc, _tick : u64) {
        unimplemented!("TrafficGenerator can not receive.");
    }

    fn tick(&mut self, tick : u64) -> Vec<Rpc> {
        let mut ret = vec![];
        for _ in 0..self.rate { ret.push(Rpc::new_rpc(tick.try_into().unwrap())); }
        return ret;
    }
}

impl fmt::Display for TrafficGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(f, "{:width$}", &format!("TrafficGenerator {{ rate : {}, id : {} }}",
                   &self.rate, &self.id), width = width)
        } else {
            write!(f, "TrafficGenerator {{ rate : {}, id : {} }}",
                   &self.rate, &self.id)
        }
    }
}
