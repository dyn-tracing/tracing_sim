#![feature(test)]
#![feature(extern_types)]

mod gateway;
mod leafnode;
mod productpage;
mod reviews;

use crate::gateway::Gateway;
use crate::leafnode::LeafNode;
use crate::productpage::ProductPage;
use crate::reviews::Reviews;
use clap::{App, Arg};
use rand::Rng;
use sim::simulator::Simulator;

pub fn new_bookinfo(seed: u64, plugin: Option<&str>) -> Simulator {
    let mut sim = Simulator::new(seed);

    let gateway = Gateway::new("gateway", 5, 5, 1, seed); // no plugins on a gateway
    let productpage = ProductPage::new("productpage-v1", 5, 5, plugin, seed);
    let reviews1 = Reviews::new("reviews-v1", 5, 5, plugin);
    let reviews2 = Reviews::new("reviews-v2", 5, 5, plugin);
    let reviews3 = Reviews::new("reviews-v3", 5, 5, plugin);
    let details = LeafNode::new("details-v1", 5, 5, plugin);
    let ratings = LeafNode::new("ratings-v1", 5, 5, plugin);
    sim.add_storage("storage");

    sim.add_node("gateway", gateway);
    sim.add_node("productpage-v1", productpage);
    sim.add_node("reviews-v1", reviews1);
    sim.add_node("reviews-v2", reviews2);
    sim.add_node("reviews-v3", reviews3);
    sim.add_node("details-v1", details);
    sim.add_node("ratings-v1", ratings);

    sim.add_edge(1, "gateway", "productpage-v1", false);
    sim.add_edge(1, "productpage-v1", "details-v1", false);
    sim.add_edge(1, "productpage-v1", "reviews-v1", false);
    sim.add_edge(1, "productpage-v1", "reviews-v2", false);
    sim.add_edge(1, "productpage-v1", "reviews-v3", false);
    sim.add_edge(1, "reviews-v1", "ratings-v1", false);
    sim.add_edge(1, "reviews-v2", "ratings-v1", false);
    sim.add_edge(1, "reviews-v3", "ratings-v1", false);
    let regular_nodes = [
        "productpage-v1",
        "reviews-v1",
        "reviews-v2",
        "reviews-v3",
        "details-v1",
        "ratings-v1",
    ];
    for node in &regular_nodes {
        sim.add_edge(1, node, "storage", true);
    }
    return sim;
}

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
    let mut simulator = new_bookinfo(seed, plugin_str);

    // Print the graph
    if let Some(_argument) = matches.value_of("print_graph") {
        simulator.print_graph();
    }

    // Execute the simulator
    for tick in 0..20 {
        simulator.tick(tick);
        print!(
            "Filter outputs:\n {0}\n\n\n\n",
            simulator.query_storage("storage")
        );
    }
    print!("Filter outputs:\n {0}", simulator.query_storage("storage"));
}
