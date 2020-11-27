#![feature(test)]

mod channel;
mod plugin_wrapper;
mod rpc;
mod codelet;
mod sim_element;
mod simulator;
mod traffic_generator;
mod link;

use link::Link;
use channel::Channel;
use plugin_wrapper::PluginWrapper;
use simulator::Simulator;
use traffic_generator::TrafficGenerator;

static LIBRARY : &str = "target/debug/libplugin_sample.dylib";
static FUNCTION: &str = "codelet";

fn main() {
    // Create simulator object.
    let mut simulator : Simulator = Simulator::new();

    // Add simulator elements to it
    let tgen = simulator.add_element(TrafficGenerator{});
    let pid0 = simulator.add_element(PluginWrapper::new(LIBRARY, FUNCTION, 0));
    let cid0 = simulator.add_element(Channel::new(2, 0));
    let lid  = simulator.add_element(Link::new(0, 5));
    let pid1 = simulator.add_element(PluginWrapper::new(LIBRARY, FUNCTION, 1));
    let cid1 = simulator.add_element(Channel::new(2, 1));

    // Connect them
    simulator.add_connection(tgen, pid0);
    simulator.add_connection(pid0, cid0);
    simulator.add_connection(cid0, lid);
    simulator.add_connection(lid, pid1);
    simulator.add_connection(pid1, cid1);

    // Execute the simulator
    for tick in 0..20 { simulator.tick(tick) ; }
}
