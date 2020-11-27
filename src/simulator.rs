use crate::sim_element::SimElement;
use crate::rpc::Rpc;
use std::fmt::Display;

// Need to combine SimElement for simulation
// and Debug for printing.
// Uses this suggestion: https://stackoverflow.com/a/28898575
pub trait PrintableElement : SimElement + Display {}
impl<T: SimElement + Display> PrintableElement for T {}

#[derive(Default)]
pub struct Simulator {
    elements     : Vec<Box<dyn PrintableElement>>,
    connections  : Vec<(usize, usize)>,
    rpc_buffer   : Vec<Vec<Rpc>>,
}

impl Simulator {
    pub fn new() -> Self {
        Simulator{..Default::default()}
    }

    pub fn add_element<T : 'static + PrintableElement>(&mut self, element : T) -> usize {
        self.elements.push(Box::new(element));
        self.rpc_buffer.push(vec![]);
        return self.elements.len() - 1;
    }

    pub fn add_connection(&mut self, src : usize, dst : usize) {
        self.connections.push((src, dst));
    }

    pub fn tick(&mut self, tick : u64) {
        // tick all elements to generate Rpcs
        for i in 0..self.elements.len() {
            self.rpc_buffer[i] = self.elements[i].tick(tick);
            println!("@ tick {:5}, {:30} outputs {:3?}",
                     tick,
                     self.elements[i],
                     self.rpc_buffer[i]);
        }

        // Send these elements to the next hops
        for (src, dst) in &self.connections {
            for rpc in &self.rpc_buffer[*src] {
                self.elements[*dst].recv(*rpc, tick);
            }
        }
        println!("");
    }
}
