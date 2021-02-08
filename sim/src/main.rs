#![feature(test)]
#![feature(extern_types)]
mod edge;
mod filter_types;
mod node;
mod plugin_wrapper;
mod sim_element;
mod simulator;
mod storage;

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
    let storage_name = "storage";
    let mut simulator = sim::simulator::Simulator::new(seed); // always run with the seed 0 while we are checking tests

    let regular_nodes = [
        "productpage-v1",
        "ratings-v1",
        "reviews-v1",
        "reviews-v2",
        "reviews-v3",
        "details-v1",
    ]
    .to_vec();
    simulator.add_node("productpage-v1", 10, 5, 0, plugin_str);
    simulator.add_node("reviews-v1", 10, 5, 0, plugin_str);
    simulator.add_node("reviews-v2", 10, 5, 0, plugin_str);
    simulator.add_node("reviews-v3", 10, 5, 0, plugin_str);

    // ratings and details are dead ends
    simulator.add_node("ratings-v1", 10, 0, 0, plugin_str);
    simulator.add_node("details-v1", 10, 0, 0, plugin_str);
    simulator.add_node("loadgenerator-v1", 10, 1, 1, None);
    simulator.add_storage(storage_name);

    // add all connections to storage
    for node in &regular_nodes {
        simulator.add_edge(1, node, storage_name, true);
    }

    // src: traffic generator
    simulator.add_edge(1, "loadgenerator-v1", "productpage-v1", true);
    // src: product page
    simulator.add_edge(1, "productpage-v1", "reviews-v1", false);
    simulator.add_edge(1, "productpage-v1", "reviews-v2", false);
    simulator.add_edge(1, "productpage-v1", "reviews-v3", false);
    simulator.add_edge(1, "productpage-v1", "details-v1", true);
    // src: reviews
    simulator.add_edge(1, "reviews-v1", "ratings-v1", false);
    simulator.add_edge(1, "reviews-v2", "ratings-v1", false);
    simulator.add_edge(1, "reviews-v3", "ratings-v1", false);

    // Print the graph
    if let Some(_argument) = matches.value_of("print_graph") {
        simulator.print_graph();
    }

    // Execute the simulator
    for tick in 0..10 {
        simulator.tick(tick);
        print!(
            "Filter outputs:\n {0}\n\n\n\n",
            simulator.query_storage("storage")
        );
    }
    print!("Filter outputs:\n {0}", simulator.query_storage("storage"));
}
