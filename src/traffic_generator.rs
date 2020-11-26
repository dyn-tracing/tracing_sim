use crate::rpc::Rpc;
use crate::sim_element::SimElement;

#[derive(Debug)]
pub struct TrafficGenerator {}

impl SimElement for TrafficGenerator {
    fn recv(&mut self, _rpc : Rpc, _tick : u64) {
        unimplemented!("TrafficGenerator can not receive.");
    }

    fn tick(&mut self, _tick : u64) -> Option<Rpc> {
        return Some(Rpc::new_rpc(0));
    }
}
