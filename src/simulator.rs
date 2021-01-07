use crate::sim_element::SimElement;
use crate::rpc::Rpc;
use std::fmt::Display;
use std::convert::TryInto;

// Need to combine SimElement for simulation
// and Debug for printing.
// Uses this suggestion: https://stackoverflow.com/a/28898575
pub trait PrintableElement : SimElement + Display {}
impl<T: SimElement + Display> PrintableElement for T {}

#[derive(Default)]
pub struct Simulator {
    elements     : Vec<Box<dyn PrintableElement>>,
    rpc_buffer   : Vec<Vec<(Rpc, Option<u32>)>>,
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
        self.elements[src].add_connection(dst.try_into().unwrap());
    }

    pub fn tick(&mut self, tick : u64) {
        // tick all elements to generate Rpcs
        for i in 0..self.elements.len() {
            self.rpc_buffer[i] = self.elements[i].tick(tick);
            println!("After tick {:5}, {:45} outputs {:?}",
                     tick,
                     self.elements[i],
                     self.rpc_buffer[i]);
        }

        // Send these elements to the next hops
        for src in 0..self.elements.len() {
            for (rpc, dst) in &self.rpc_buffer[src] {
                if (*dst).is_some() {
                    // Before we send this rpc on, we should update its path to include the most recently traversed element
                    // TODO: is cloning the best way to do this?
                    let mut new_rpc = rpc.clone();
                    new_rpc.add_to_path(&src.to_string());
                    self.elements[(*dst).unwrap() as usize].recv(new_rpc, tick);
                }
            }
        }
        println!("");
    }
}
