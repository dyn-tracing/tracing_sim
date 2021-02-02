//! A sim_element is something that takes in RPCs and give them to other sim_elements.
//! Right now the only sim_elements are nodes, edges, and plugin_wrappers.

use rpc_lib::rpc::Rpc;

pub trait SimElement {
    fn add_connection(&mut self, neighbor: String);

    fn tick(&mut self, tick: u64) -> Vec<(Rpc, String)>;

    fn recv(&mut self, rpc: Rpc, tick: u64, sender: &str);

    // This returns the following information about a simulator element
    // 1. what its ID is
    // 2. who its neighbors are
    fn whoami(&self) -> (&str, &Vec<String>);
}

pub trait Node {}
