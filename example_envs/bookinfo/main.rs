#![feature(test)]
#![feature(extern_types)]

mod gateway;
mod leafnode;
mod productpage;
mod reviews;

pub mod bookinfo;

use crate::bookinfo::new_bookinfo;
use crate::gateway::Gateway;
use clap::{App, Arg};
use rand::Rng;
use rpc_lib::rpc::Rpc;

use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};

fn log_setup() {
    // Build a stderr logger.
    let stderr = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{h({l})}: {m}\n")))
        .target(Target::Stderr)
        .build();
    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{l}: {m}\n")))
        .append(false)
        .build("sim.log")
        .unwrap();
    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Info)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(log::LevelFilter::Trace),
        )
        .unwrap();
    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config);
}

fn main() {
    // Set up logging
    log_setup();

    let matches = App::new("Tracing Simulator")
        .arg(
            Arg::with_name("print_graph")
                .short("g")
                .long("print_graph")
                .value_name("PRINT_GRAPH")
                .help("Set if you want ot produce a pdf of the graph you create"),
        )
        .arg(
            Arg::with_name("record_network_usage")
                .short("n")
                .long("record_network_usage")
                .value_name("RECORD_NETWORK_USAGE")
                .help("Whether to record how much data the application sends over the network."),
        )
        .arg(
            Arg::with_name("plugin")
                .short("p")
                .long("plugin")
                .value_name("PLUGIN")
                .help("Path to the plugin."),
        )
        .arg(
            Arg::with_name("aggr_func")
                .short("a")
                .long("aggregation_function")
                .value_name("AGGREGATION_FUNCTION")
                .help("Path to the aggregation function implementation."),
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
    let aggr_str = matches.value_of("aggr_func");
    let seed_arg = matches.value_of("random_num_seed");

    let seed;
    if seed_arg.is_none() {
        let mut rng = rand::thread_rng();
        seed = rng.gen::<u64>();
        log::info!("Using seed {0}\n", seed);
    } else {
        seed = seed_arg.unwrap().parse::<u64>().unwrap();
    }

    // Create simulator object.
    let mut simulator;
    if let Some(_argument) = matches.value_of("record_network_usage") {
        simulator = new_bookinfo(seed, true, plugin_str, aggr_str);
    } else {
        simulator = new_bookinfo(seed, false, plugin_str, aggr_str);
    }

    // Print the graph
    if let Some(_argument) = matches.value_of("print_graph") {
        simulator.print_graph();
    }

    // Execute the simulator
    simulator.insert_rpc("gateway", Rpc::new("0"));
    for tick in 0..7 {
        simulator.tick(tick);
        log::info!("Filter results:\n {0}", simulator.query_storage("storage"));
    }
    let gateway = simulator.get_element::<Gateway>("gateway");
    log::info!("Gateway collected RPCS:");
    for rpc in gateway.get_collected_responses() {
        log::info!("{:?}", rpc);
    }
    let storage_result = simulator.query_storage("storage");
    log::info!("Final filter results:\n {0}", storage_result);
}
