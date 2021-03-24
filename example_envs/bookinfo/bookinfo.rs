use crate::gateway::Gateway;
use crate::leafnode::LeafNode;
use crate::productpage::ProductPage;
use crate::reviews::Reviews;
use sim::simulator::Simulator;

pub fn new_bookinfo(seed: u64, record_network_usage: bool, plugin: Option<&str>, aggr_func: Option<&str>) -> Simulator {
    let mut sim = Simulator::new(seed, record_network_usage);

    let gateway = Gateway::new("gateway", 5, 5, 0, seed); // no plugins on a gateway
    let productpage = ProductPage::new("productpage-v1", 5, 5, plugin, seed);
    let reviews1 = Reviews::new("reviews-v1", 5, 5, plugin);
    let reviews2 = Reviews::new("reviews-v2", 5, 5, plugin);
    let reviews3 = Reviews::new("reviews-v3", 5, 5, plugin);
    let details = LeafNode::new("details-v1", 5, 5, plugin);
    let ratings = LeafNode::new("ratings-v1", 5, 5, plugin);
    sim.add_storage("storage", aggr_func);

    sim.add_node("gateway", gateway);
    sim.add_node("productpage-v1", productpage);
    sim.add_node("reviews-v1", reviews1);
    sim.add_node("reviews-v2", reviews2);
    sim.add_node("reviews-v3", reviews3);
    sim.add_node("details-v1", details);
    sim.add_node("ratings-v1", ratings);

    sim.add_edge(0, "gateway", "productpage-v1", true);
    sim.add_edge(0, "productpage-v1", "details-v1", true);
    sim.add_edge(0, "productpage-v1", "reviews-v1", true);
    sim.add_edge(0, "productpage-v1", "reviews-v2", true);
    sim.add_edge(0, "productpage-v1", "reviews-v3", true);
    sim.add_edge(0, "reviews-v1", "ratings-v1", true);
    sim.add_edge(0, "reviews-v2", "ratings-v1", true);
    sim.add_edge(0, "reviews-v3", "ratings-v1", true);
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
