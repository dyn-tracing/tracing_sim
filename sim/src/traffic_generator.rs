use crate::sim_element::SimElement;
use rpc_lib::rpc::Rpc;
use std::convert::TryInto;
use std::fmt;

pub struct TrafficGenerator {
    rate: u32, // Rpcs per tick
    id: u32,
    neighbor: Option<u32>,
}

impl TrafficGenerator {
    pub fn new(rate: u32, id: u32) -> Self {
        return TrafficGenerator {
            rate: rate,
            id: id,
            neighbor: None,
        };
    }
}

impl SimElement for TrafficGenerator {
    fn recv(&mut self, _rpc: Rpc, _tick: u64, _sender: u32) {
        unimplemented!("TrafficGenerator can not receive.");
    }

    fn tick(&mut self, tick: u64) -> Vec<(Rpc, Option<u32>)> {
        let mut ret = vec![];
        for _ in 0..self.rate {
            ret.push((Rpc::new_rpc(tick.try_into().unwrap()), self.neighbor));
        }
        return ret;
    }

    fn add_connection(&mut self, neighbor: u32) {
        self.neighbor = Some(neighbor);
    }

    fn whoami(&self) -> (&str, u32, Vec<u32>) {
        let mut neighbors = Vec::new();
        if !self.neighbor.is_none() {
            neighbors.push(self.neighbor.unwrap());
        }
        return ("TrafficGenerator", self.id, neighbors);
    }
}

impl fmt::Display for TrafficGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(
                f,
                "{:width$}",
                &format!(
                    "TrafficGenerator {{ rate : {}, id : {} }}",
                    &self.rate, &self.id
                ),
                width = width
            )
        } else {
            write!(
                f,
                "TrafficGenerator {{ rate : {}, id : {} }}",
                &self.rate, &self.id
            )
        }
    }
}
