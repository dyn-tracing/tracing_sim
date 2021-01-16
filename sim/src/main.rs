#![feature(test)]
mod channel;
mod filter_types;
mod link;
mod plugin_wrapper;
mod sim_element;
mod simulator;
mod traffic_generator;

use channel::Channel;
use link::Link;
use plugin_wrapper::PluginWrapper;
use simulator::Simulator;
use traffic_generator::TrafficGenerator;

static COMPILED: &str = "target/debug/libfilter_lib";

fn main() {
    // Create simulator object.
    let mut simulator: Simulator = Simulator::new();

    // Add simulator elements to it
    let tgen = simulator.add_element(TrafficGenerator::new(1, 0));
    let cid0 = simulator.add_element(Channel::new(2, 5, 0));
    let lid = simulator.add_element(Link::new(5, 0));
    let cid1 = simulator.add_element(Channel::new(2, 5, 1));
    let pid0 = simulator.add_element(PluginWrapper::new(COMPILED, 0));

    // Connect them
    simulator.add_connection(tgen, cid0);
    simulator.add_connection(cid0, lid);
    simulator.add_connection(lid, cid1);
    simulator.add_connection(cid1, pid0);

    // Execute the simulator
    for tick in 0..20 {
        simulator.tick(tick);
    }
}
