#![feature(test)]
#![feature(extern_types)]
mod edge;
mod filter_types;
mod node;
mod plugin_wrapper;
mod sim_element;
mod simulator;

use clap::{App, Arg};
use simulator::Simulator;

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

    // node arguments go:  id, capacity, egress_rate, generation_rate, plugin
    simulator.add_node("traffic-gen", 10, 1, 1, plugin_str);
    simulator.add_node("node-1", 10, 1, 0, plugin_str);
    simulator.add_node("node-2", 10, 1, 0, plugin_str);
    simulator.add_node("node-3", 10, 1, 0, plugin_str);
    simulator.add_node("node-4", 10, 1, 0, plugin_str);

    // edge arguments go:  delay, endpoint1, endpoint2, unidirectional
    simulator.add_edge(1, "traffic-gen", "node-1", true);
    simulator.add_edge(1, "node-1", "node-2", false);
    simulator.add_edge(1, "node-1", "node-3", false);
    //one way rpc sink
    simulator.add_edge(1, "node-1", "node-4", true);

    // Print the graph
    if let Some(_argument) = matches.value_of("print_graph") {
        simulator.print_graph();
    }

    // Execute the simulator
    for tick in 0..20 {
        simulator.tick(tick);
    }
}
