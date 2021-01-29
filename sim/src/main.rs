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
    simulator.add_node("traffic generator".to_string(), 10, 1, 1, plugin_str);
    simulator.add_node("node 1".to_string(), 10, 1, 0, plugin_str);
    simulator.add_node("node 2".to_string(), 10, 1, 0, plugin_str);
    simulator.add_node("node 3".to_string(), 10, 1, 0, plugin_str);
    simulator.add_node("node 4".to_string(), 10, 1, 0, plugin_str);

    // edge arguments go:  delay, endpoint1, endpoint2, unidirectional
    simulator.add_edge(
        1,
        "traffic generator".to_string(),
        "node 1".to_string(),
        true,
    );
    simulator.add_edge(1, "node 1".to_string(), "node 2".to_string(), false);
    simulator.add_edge(1, "node 1".to_string(), "node 3".to_string(), false);
    //one way rpc sink
    simulator.add_edge(1, "node 1".to_string(), "node 4".to_string(), true);

    // Print the graph
    if let Some(_argument) = matches.value_of("print_graph") {
        simulator.print_graph();
    }

    // Execute the simulator
    for tick in 0..20 {
        simulator.tick(tick);
    }
}
