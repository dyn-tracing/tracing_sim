//! A sim_element is something that takes in RPCs and give them to other sim_elements.
//! Right now the only sim_elements are nodes, edges, and plugin_wrappers.

use core::any::Any;
use rpc_lib::rpc::Rpc;

pub trait SimElement {
    fn add_connection(&mut self, neighbor: String);

    fn tick(&mut self, tick: u64) -> Vec<Rpc>;

    fn recv(&mut self, rpc: Rpc, tick: u64);

    fn whoami(&self) -> &str;

    fn neighbors(&self) -> &Vec<String>;

    fn as_any(&self) -> &dyn Any;
}

pub trait Node {}
