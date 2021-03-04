use assert_cmd::prelude::*; // Add methods on commands
use diffy;
use std::fs;
use std::path::Path; // Directory management
use std::process::Command; // Run programs

fn check_bookinfo() {

    // Set up library access
    let plugin_str = matches.value_of("plugin");

    // Create simulator object.
    let mut simulator = new_bookinfo(0, plugin_str);

    // Print the graph
    if let Some(_argument) = matches.value_of("print_graph") {
        simulator.print_graph();
    }

    // Execute the simulator
    simulator.insert_rpc("gateway", Rpc::new("0"));
    for tick in 0..6 {
        simulator.tick(tick);
    }
    let gateway = simulator.get_element::<Gateway>("gateway");
    let collected_rpcs = gateway.get_collected_responses();
    assert collect_rpcs.size() == 1
}
