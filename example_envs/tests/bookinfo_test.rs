use example_envs::bookinfo::new_bookinfo;
use example_envs::gateway::Gateway;
use rpc_lib::rpc::Rpc;

#[test]
fn check_bookinfo() {
    // Set up library access

    // Create simulator object.
    let mut simulator = new_bookinfo(0, None);

    // Execute the simulator
    simulator.insert_rpc("gateway", Rpc::new("0"));
    for tick in 0..6 {
        simulator.tick(tick);
    }
    // After 6 ticks we should have stored 1 response
    // Check this in the gateway
    let gateway = simulator.get_element::<Gateway>("gateway");
    let collected_rpcs = gateway.get_collected_responses();
    let response_num = collected_rpcs.len();
    assert!(
        response_num == 1,
        "Number of responses was {}",
        response_num
    );
    // Also check that we stay at one response
    for tick in 0..10 {
        simulator.tick(tick);
    }
    // TODO: It should not be necessary to call this getter twice
    // Look into const fn in Rust
    let gateway = simulator.get_element::<Gateway>("gateway");
    let collected_rpcs = gateway.get_collected_responses();
    let response_num = collected_rpcs.len();
    assert!(
        response_num == 1,
        "Number of final responses was {}",
        response_num
    );
}
