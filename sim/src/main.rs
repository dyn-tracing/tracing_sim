#![feature(test)]
#![feature(extern_types)]
mod edge;
mod filter_types;
mod node;
mod plugin_wrapper;
mod sim_element;
mod simulator;

use clap::{App, Arg};
use rand::Rng;
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
        .arg(
            Arg::with_name("random_num_seed")
                .short("r")
                .long("random_num_seed")
                .value_name("RANDOM_NUM_SEED")
                .help("A seed for all the random routing decisions."),
        )
        .get_matches();

    // Set up library access
    let plugin_str = matches.value_of("plugin");
    let seed_arg = matches.value_of("random_num_seed");
    let seed;
    if seed_arg.is_none() {
        let mut rng = rand::thread_rng();
        seed = rng.gen::<u64>();
        print!("Using seed {0}\n", seed);
    } else {
        seed = seed_arg.unwrap().parse::<u64>().unwrap();
    }

    // Create simulator object.
    let mut simulator: Simulator = Simulator::new(seed);

    // node arguments go:  id, capacity, egress_rate, generation_rate, plugin
    simulator.add_node("traffic-gen", 1, 1, 1, plugin_str);
    simulator.add_node("node-1", 10, 1, 0, plugin_str);
    simulator.add_node("node-2", 10, 1, 0, plugin_str);
    simulator.add_node("node-3", 10, 1, 0, plugin_str);
    simulator.add_node("node-4", 1, 1, 0, plugin_str); // in setting egress rate to 0, we are making a sink
    simulator.add_node("node-5", 1, 1, 0, plugin_str); // in setting egress rate to 0, we are making a sink

    // edge arguments go:  delay, endpoint1, endpoint2, unidirectional
    simulator.add_edge(1, "traffic-gen", "node-1", true);
    simulator.add_edge(1, "node-1", "node-2", false);
    simulator.add_edge(1, "node-1", "node-3", false);
    //one way rpc sinks
    simulator.add_edge(1, "node-1", "node-4", true);
    simulator.add_edge(1, "node-2", "node-5", true);

    // Print the graph
    if let Some(_argument) = matches.value_of("print_graph") {
        simulator.print_graph();
    }

    // Execute the simulator
    for tick in 0..20 {
        simulator.tick(tick);
    }
}
