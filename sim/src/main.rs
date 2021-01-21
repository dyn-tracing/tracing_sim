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
use simulator::Simulator;
use traffic_generator::TrafficGenerator;
use std::path::PathBuf;

static COMPILED: &str = "../target/debug/libfilter_lib";

fn main() {
    // Set up library access
    let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    cargo_dir.push(COMPILED);
    let library_str = cargo_dir.to_str().unwrap();

    // Create simulator object.
    let mut simulator: Simulator = Simulator::new();

    let tgen = simulator.add_node(TrafficGenerator::new(1, 0));
    let node1 = simulator.add_node(Link::new(2, 1, Some(library_str), 1));
    let node2 = simulator.add_node(Link::new(2, 1, Some(library_str), 2));
    let node3 = simulator.add_node(Link::new(2, 1, Some(library_str), 3));
    let node4 = simulator.add_node(Link::new(2, 1, Some(library_str), 4));

    let _edge5 = simulator.add_one_direction_edge(Channel::new(1, 5), tgen, node1);
    let _edge6 = simulator.add_edge(Channel::new(2, 6), node1, node2);
    let _edge7 = simulator.add_edge(Channel::new(2, 7), node1, node3);
    let _edge8 = simulator.add_one_direction_edge(Channel::new(1, 8), node1, node4); // one way rpc sink
    
    // Print the graph
    simulator.print_graph();

    // Execute the simulator
    for tick in 0..20 {
        simulator.tick(tick);
    }
}
