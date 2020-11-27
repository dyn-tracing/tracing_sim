use crate::rpc::Rpc;
use crate::sim_element::SimElement;
use std::fmt;
use std::convert::TryInto;

pub struct TrafficGenerator {}

impl SimElement for TrafficGenerator {
    fn recv(&mut self, _rpc : Rpc, _tick : u64) {
        unimplemented!("TrafficGenerator can not receive.");
    }

    fn tick(&mut self, tick : u64) -> Vec<Rpc> {
        let mut ret = vec![];
        for _ in 0..10 { ret.push(Rpc::new_rpc(tick.try_into().unwrap())); }
        return ret;
    }
}

impl fmt::Display for TrafficGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(f, "{:width$}", &format!("TrafficGenerator"), width = width)
        } else {
            write!(f, "TrafficGenerator")
        }
    }
}
