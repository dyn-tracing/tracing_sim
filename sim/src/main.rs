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
    let mut simulator = sim::simulator::Simulator::new(1); // always run with the seed 0 while we are checking tests

    let regular_nodes = [
        "frontend-v1",
        "cartservice-v1",
        "productcatalogservice-v1",
        "currencyservice-v1",
        "paymentservice-v1",
        "shippingservice-v1",
        "emailservice-v1",
        "checkoutservice-v1",
        "recomendationservice-v1",
        "adservice-v1",
    ]
    .to_vec();
    for node in &regular_nodes {
        // node: ID, capacity, egress rate, generation rate, plugin
        simulator.add_node(node, 10, 5, 0, plugin_str);
    }
    simulator.add_node("loadgenerator-v1", 10, 1, 1, None);
    simulator.add_node("sink", 10, 1, 0, None); // rpc sink
    simulator.add_storage(storage_name);

    // add all connections to storage and to the sink - we want traces to be able to end arbitrarily
    for node in &regular_nodes {
        simulator.add_edge(1, node, storage_name, true);
        simulator.add_edge(1, node, "sink", true);
    }

    // src: load generator
    simulator.add_edge(1, "loadgenerator-v1", "frontend-v1", true);

    // src: frontend
    simulator.add_edge(1, "frontend-v1", "cartservice-v1", false);
    simulator.add_edge(1, "frontend-v1", "recomendationservice-v1", false);
    simulator.add_edge(1, "frontend-v1", "productcatalogservice-v1", false);

    simulator.add_edge(1, "frontend-v1", "shippingservice-v1", false);
    simulator.add_edge(1, "frontend-v1", "checkoutservice-v1", false);
    simulator.add_edge(1, "frontend-v1", "adservice-v1", false);

    // src: recomendation service
    simulator.add_edge(
        1,
        "recomendationservice-v1",
        "productcatalogservice-v1",
        false,
    );

    // src: checkout service
    simulator.add_edge(1, "checkoutservice-v1", "shippingservice-v1", false);
    simulator.add_edge(1, "checkoutservice-v1", "paymentservice-v1", false);
    simulator.add_edge(1, "checkoutservice-v1", "emailservice-v1", false);

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
