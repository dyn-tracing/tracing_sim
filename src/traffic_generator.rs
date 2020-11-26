use crate::rpc::Rpc;
use crate::sim_element::SimElement;
use std::fmt;

pub struct TrafficGenerator {}

impl SimElement for TrafficGenerator {
    fn recv(&mut self, _rpc : Rpc, _tick : u64) {
        unimplemented!("TrafficGenerator can not receive.");
    }

    fn tick(&mut self, _tick : u64) -> Option<Rpc> {
        return Some(Rpc::new_rpc(0));
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
