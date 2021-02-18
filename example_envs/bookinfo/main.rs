#![feature(test)]
#![feature(extern_types)]

mod gateway;
mod leafnode;
mod productpage;
mod reviews;

pub mod bookinfo;

use crate::bookinfo::new_bookinfo;
use clap::{App, Arg};
use rand::Rng;

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
