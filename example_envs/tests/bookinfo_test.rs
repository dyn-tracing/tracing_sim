use example_envs::bookinfo::new_bookinfo;
use example_envs::gateway::Gateway;
use example_envs::leafnode::LeafNode;
use example_envs::productpage::ProductPage;
use example_envs::reviews::Reviews;
use queues::IsQueue;
use rpc_lib::rpc::Rpc;
use std::path::PathBuf;

#[test]
fn check_bookinfo() {
    // Set up plugin name
    let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    cargo_dir.push("../target/debug/libfilter_example");
    let plugin_str = cargo_dir.to_str().unwrap();

    // Create simulator object.
    let mut simulator = new_bookinfo(0, None, Some(plugin_str), None);

    // Execute the simulator
    simulator.insert_rpc("gateway", Rpc::new("0"));

    // After the first tick, Productpage should receive a request
    simulator.tick(0);
    let productpage = simulator.get_element::<ProductPage>("productpage-v1");
    let product_ingress_size = productpage.get_ingress_queue().size();
    assert!(
        product_ingress_size == 1,
        "Expected 1 RPC in productpage queue, received {}",
        product_ingress_size
    );

    // Productpage sends to both Reviews and Ratings
    // For this seed the RPC is in Reviews-v1
    simulator.tick(1);
    let details = simulator.get_element::<LeafNode>("details-v1");
    let details_ingress_size = details.get_ingress_queue().size();
    assert!(
        details_ingress_size == 1,
        "Expected 1 RPC in details ingress queue, received {}",
        details_ingress_size
    );
    let reviews_v1 = simulator.get_element::<Reviews>("reviews-v1");
    let reviews_ingress_size = reviews_v1.get_ingress_queue().size();
    assert!(
        reviews_ingress_size == 1,
        "Expected 1 RPC in reviews-v1 queue, received {}",
        reviews_ingress_size
    );

    // Reviews-v1 forwards the request to Ratings
    simulator.tick(2);
    let ratings = simulator.get_element::<LeafNode>("ratings-v1");
    let ratings_ingress_size = ratings.get_ingress_queue().size();
    assert!(
        ratings_ingress_size == 1,
        "Expected 1 RPC in ratings ingress queue, received {}",
        ratings_ingress_size
    );

    // The answer from Ratings should be in the reviews queue now
    simulator.tick(3);
    let reviews_v1 = simulator.get_element::<Reviews>("reviews-v1");
    let reviews_ingress_size = reviews_v1.get_ingress_queue().size();
    assert!(
        reviews_ingress_size == 1,
        "Expected 1 RPC in reviews-v1 queue, received {}",
        reviews_ingress_size
    );

    // Productpage received a reply from both Details and Reviews
    // It should now place an RPC in its queue
    simulator.tick(4);
    let productpage = simulator.get_element::<ProductPage>("productpage-v1");
    let product_ingress_size = productpage.get_ingress_queue().size();
    assert!(
        product_ingress_size == 1,
        "Expected 1 RPC in productpage ingress queue, received {}",
        product_ingress_size
    );

    // After 6 ticks we should have stored 1 response
    // Check this in the gateway
    simulator.tick(5);
    let gateway = simulator.get_element::<Gateway>("gateway");
    let collected_rpcs = gateway.get_collected_responses();
    let response_num = collected_rpcs.len();
    assert!(
        response_num == 1,
        "Number of responses was {}",
        response_num
    );

    // Because of the filter, we should have a return value of 2 in storage
    // after one more tick
    simulator.tick(6);
    let storage_val = simulator.query_storage("storage");
    assert!(storage_val == "2\n", "storage contains {0}", storage_val);

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
