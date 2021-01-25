#![feature(test)]
#![feature(extern_types)]
mod channel;
mod link;
mod plugin_wrapper;
mod sim_element;
mod simulator;
mod traffic_generator;
mod filter_types;

use channel::Channel;
use clap::{App, Arg};
use link::Link;
use simulator::Simulator;
use traffic_generator::TrafficGenerator;

fn main() {
    let matches = App::new("Tracing Simulator")
        .arg(
            Arg::with_name("print_graph")
                .short("g")
                .long("print_graph")
                .value_name("PRINT_GRAPH")
                .help("Set if you want ot produce a pdf of the graph you create"),
        )
        .arg(
            Arg::with_name("plugin")
                .short("p")
                .long("plugin")
                .value_name("PLUGIN")
                .help("Path to the plugin."),
        )
        .get_matches();

    // Set up library access
    let plugin_str = matches.value_of("plugin");

    // Create simulator object.
    let mut simulator: Simulator = Simulator::new();

    let tgen = simulator.add_node(TrafficGenerator::new(1, 0));
    let node1 = simulator.add_node(Link::new(2, 1, plugin_str, 1));
    let node2 = simulator.add_node(Link::new(2, 1, plugin_str, 2));
    let node3 = simulator.add_node(Link::new(2, 1, plugin_str, 3));
    let node4 = simulator.add_node(Link::new(2, 1, plugin_str, 4));

    let _edge5 = simulator.add_one_direction_edge(Channel::new(1, 5), tgen, node1);
    let _edge6 = simulator.add_edge(Channel::new(2, 6), node1, node2);
    let _edge7 = simulator.add_edge(Channel::new(2, 7), node1, node3);
    // one way rpc sink
    let _edge8 = simulator.add_one_direction_edge(Channel::new(1, 8), node1, node4);

    // Print the graph
    if let Some(_argument) = matches.value_of("print_graph") {
        simulator.print_graph();
    }

    // Execute the simulator
    for tick in 0..20 {
        simulator.tick(tick);
    }
}
