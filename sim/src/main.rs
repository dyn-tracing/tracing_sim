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
use std::borrow::BorrowMut;
use std::path::PathBuf;

static COMPILED: &str = "target/debug/libfilter_lib";

fn main() {
    // Set up library access
    let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    cargo_dir.push("../target/debug/libfilter_lib");
    let library_str = cargo_dir.to_str().unwrap();

    // Create simulator object.
    let mut simulator: Simulator = Simulator::new();

    let tgen = simulator.add_element(TrafficGenerator::new(1, 0));
    let cid0 = simulator.add_element(Channel::new(2, 0));

    let lid = simulator.add_element(Link::new(2, 1, Some(library_str), 0));
    let cid1 = simulator.add_element(Channel::new(2, 1));

    // Connect them
    simulator.add_connection(tgen, cid0);
    simulator.add_connection(cid0, lid);
    simulator.add_connection(lid, cid1);

    // Execute the simulator
    for tick in 0..20 {
        simulator.tick(tick);
    }
}
