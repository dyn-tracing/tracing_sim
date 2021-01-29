//! A sim_element is something that takes in RPCs and give them to other sim_elements.
//! Right now the only sim_elements are nodes, edges, and plugin_wrappers.

use rpc_lib::rpc::Rpc;

pub trait SimElement {
    fn add_connection(&mut self, neighbor: String);

    fn tick(&mut self, tick: u64) -> Vec<(Rpc, Option<String>)>;

    fn recv(&mut self, rpc: Rpc, tick: u64, sender: String);

    // This returns the following information about a simulator element
    // 1. whether it should be included in the path
    // 2. what its ID is
    // 3. who its neighbors are
    fn whoami(&self) -> (bool, &'static str, Vec<String>);
}

pub trait Node {}
