/// Implements subgraph isomorphism algorithms two ways:
/// as described in https://www.cs.bgu.ac.il/~dekelts/publications/subtree.pdf
/// Another thing to consider, but is not implemented here, is
/// http://chasewoerner.org/popl87.pdf
///
use crate::graph_utils::{find_leaves, find_root, has_property_subset, get_node_with_id};
use mcmf::{Capacity, Cost, GraphBuilder, Path, Vertex};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::DfsPostOrder;
use petgraph::Incoming;
use std::collections::HashMap;
use std::collections::HashSet;

// ----------------- Shamir Isomorphism Algorithm ------------------

// this performs lines 0-4 in the Shamir paper figure 3
fn initialize_s(
    graph_g: &Graph<(String, HashMap<String, String>), String>,
    graph_h: &Graph<(String, HashMap<String, String>), String>,
) -> HashMap<(NodeIndex, NodeIndex), HashMap<NodeIndex, Option<Vec<(NodeIndex, NodeIndex)>>>> {
    let mut s = HashMap::<(NodeIndex, NodeIndex), HashMap<NodeIndex, Option<Vec<(NodeIndex, NodeIndex)>>>>::new();
    for node_g in graph_g.node_indices() {
        for u in graph_h.node_indices() {
            // initialize S entry as empty set
            s.insert((node_g, u), HashMap::new());
        }
    }
    let root_g = find_root(&graph_g);
    let root_h = find_root(&graph_h);
    for leaf_g in find_leaves(root_g, &graph_g) {
        for leaf_h in find_leaves(root_h, &graph_h) {
            s.get_mut(&(leaf_g, leaf_h)).unwrap().insert(leaf_h, Some(vec!((leaf_h, leaf_g))));
            for neighbor in graph_h.neighbors_directed(leaf_h, Incoming) {
                s.get_mut(&(leaf_g, leaf_h)).unwrap().insert(neighbor, Some(vec!((leaf_h, leaf_g))));
            }
        }
    }
    return s;
}

/// Given two sets of nodes, set x from graph g, and set y from graph h,
/// creates a flow graph with the source connected to all nodes in x and
/// the sink connected to all nodes in y.  Edges between x and y are computed
/// based on if their set (in set_s) contains u_null.  Then we compute
/// the flow of that graph, which is equivalent to the maximum matching.
fn max_matching(
    set_x: &Vec<NodeIndex>,
    set_y: &Vec<NodeIndex>,
    graph_g: &Graph<(String, HashMap<String, String>), String>,
    graph_h: &Graph<(String, HashMap<String, String>), String>,
    set_s: &HashMap<(NodeIndex, NodeIndex), HashMap<NodeIndex, Option<Vec<(NodeIndex, NodeIndex)>>>>,
    u_null: NodeIndex,
) -> (i32, Vec<(NodeIndex, NodeIndex)>) {
    let mut graph_builder = GraphBuilder::new();
    let mut added_nodes = HashSet::new();
    for u in set_x {
        for v in set_y {
            if set_s[&(*v, *u)].contains_key(&u_null) && 
               has_property_subset(&graph_h.node_weight(*u).unwrap().1,
                                   &graph_g.node_weight(*v).unwrap().1) {
                // 1. add edge from source if applicable
                let mut u_str = graph_h.node_weight(*u).unwrap().0.clone();
                u_str.push_str("U");
                if !added_nodes.contains(&u_str) {
                    graph_builder.add_edge(Vertex::Source,
                                           u_str.to_string(),
                                           Capacity(1),
                                           Cost(0));
                    added_nodes.insert(u_str.to_string());
                }

                // 2. add edge to sink if applicable
                let mut v_str = graph_g.node_weight(*v).unwrap().0.clone();
                v_str.push_str("V");
                if !added_nodes.contains(&v_str) {
                    graph_builder.add_edge(v_str.to_string(),
                                           Vertex::Sink,
                                           Capacity(1),
                                           Cost(0));
                    added_nodes.insert(v_str.to_string());
                }

                // 2. add edge between v and u
                graph_builder.add_edge(u_str.to_string(), v_str.to_string(), Capacity(1), Cost(1));
            }
        }
    }
    let (cost, paths) = graph_builder.mcmf();
    let mut matching = Vec::new();
    for path in paths {
        // path is source + u_vertex + v_vertex + sink
        let mut u_vertex = path.vertices()[1].clone().as_option();
        let mut v_vertex = path.vertices()[2].clone().as_option();

        let mut u_vertex_unwrapped = u_vertex.unwrap();
        let mut v_vertex_unwrapped = v_vertex.unwrap();

        u_vertex_unwrapped.pop();
        v_vertex_unwrapped.pop();

        // we slice off the appending u and v;  those are only there so that
        // nodes don't have the same string values, because we might want to give
        // both graphs the same string identifiers for nodes
        let u_vertex_id = get_node_with_id(graph_h, u_vertex_unwrapped).unwrap();
        let v_vertex_id = get_node_with_id(graph_g, v_vertex_unwrapped).unwrap();
        matching.push((u_vertex_id, v_vertex_id));
    }
    return (cost, matching);
}

fn find_mapping_shamir_centralized_inner_loop(
    v: NodeIndex,
    graph_g: &Graph<(String, HashMap<String, String>), String>,
    graph_h: &Graph<(String, HashMap<String, String>), String>,
    set_s: &mut HashMap<(NodeIndex, NodeIndex), HashMap<NodeIndex, Option<Vec<(NodeIndex, NodeIndex)>>>>,
) -> (bool, Option<NodeIndex>) {
    let root_h = find_root(&graph_h);
    let v_neighbors: Vec<NodeIndex> = graph_g.neighbors_undirected(v).collect();
    for u in graph_h.node_indices() {
        let u_neighbors: Vec<NodeIndex> = graph_h.neighbors_undirected(u).collect();
        // all vertices of degree at most t+1
        if u_neighbors.len() > v_neighbors.len() + 1 {
            continue;
        }

        // maximum matching where X0 = X
        let (cost, path) = max_matching(&u_neighbors,
                                         &v_neighbors,
                                         graph_g,
                                         graph_h,
                                         set_s,
                                         u);
        if cost == u_neighbors.len() as i32 {
                if set_s[&(v,u)].contains_key(&u) {
                } else {
                    set_s.get_mut(&(v, u)).unwrap().insert(u, Some(path));
                }
        }

        // maximum matching where X0 is X minus an element
        for vertex in 0..u_neighbors.len() {
            let mut new_x_set = u_neighbors.clone();
            let vertex_id = new_x_set.remove(vertex);
            let (cost, path) = max_matching(&new_x_set,
                                             &v_neighbors,
                                             graph_g,
                                             graph_h,
                                             set_s,
                                             u);
            if cost == new_x_set.len() as i32 {
                if set_s[&(v,u)].contains_key(&vertex_id) {
                } else {
                    set_s.get_mut(&(v, u)).unwrap().insert(vertex_id, Some(path));
                }
            }
        }


        // lines 12-14
        if set_s[&(v, root_h)].contains_key(&root_h) {
            if has_property_subset(&graph_g.node_weight(v).unwrap().1,
                                   &graph_h.node_weight(root_h).unwrap().1) {
                return (true, Some(v));
            }
        }
    }
    return (false, None);
}

// For debugging only
fn print_set_s(
    graph_g: &Graph<(String, HashMap<String, String>), String>,
    graph_h: &Graph<(String, HashMap<String, String>), String>,
    set_s: &HashMap<(NodeIndex, NodeIndex), HashMap<NodeIndex, Option<Vec<(NodeIndex, NodeIndex)>>>>,
) {
    for key in set_s.keys() {
        print!("key: {:?} {:?} ", graph_g.node_weight(key.0).unwrap(), graph_h.node_weight(key.1).unwrap());
        for value_key in set_s[key].keys() {
            print!("inner key: {:?} ", graph_h.node_weight(*value_key).unwrap());
            for mapping in &set_s[key][value_key] {
                for map in mapping {
                    print!("maps {:?} to {:?} ", graph_h.node_weight(map.0).unwrap(), graph_g.node_weight(map.1).unwrap());
                }
            }
        }
        print!("\n\n");
    }
}

fn get_mapping_from_set_s(
    graph_g: &Graph<(String, HashMap<String, String>), String>,
    graph_h: &Graph<(String, HashMap<String, String>), String>,
    set_s: &HashMap<(NodeIndex, NodeIndex), HashMap<NodeIndex, Option<Vec<(NodeIndex, NodeIndex)>>>>,
    root_in_g: &NodeIndex,
) -> Vec<(NodeIndex, NodeIndex)> {
    let root_h = find_root(graph_h);
    let mut to_return = set_s[&(*root_in_g, root_h)][&root_h].as_ref().unwrap().to_vec();
    to_return.push((root_h, *root_in_g));
    return to_return;

}
pub fn find_mapping_shamir_centralized(
    graph_g: &Graph<(String, HashMap<String, String>), String>,
    graph_h: &Graph<(String, HashMap<String, String>), String>,
) -> Option<Vec<(NodeIndex, NodeIndex)>> {
    // TODO:  before even dealing with isomorphism, ask if breadth,
    // height, num nodes match up
    if graph_g.node_count() < graph_h.node_count() {
        return None;
    }

    // initialize S with all N(u) sets, lines 1-4
    let mut set_s = initialize_s(graph_g, graph_h);
    let root_g = find_root(graph_g);

    // postorder traversal and filtering of children for degrees, lines 5-8;
    let mut post_order = DfsPostOrder::new(graph_g, root_g);
    while let Some(node) = post_order.next(graph_g) {
        let v_children: Vec<NodeIndex> = graph_g.neighbors(node).collect();
        let v_children_len = v_children.len();
        let (mapping_found, mapping_root) =
        find_mapping_shamir_centralized_inner_loop(node, graph_g, graph_h, &mut set_s);
        if mapping_found {
            return Some(get_mapping_from_set_s(graph_g, graph_h, &set_s, &mapping_root.unwrap()));
        }
    }
    // line 15
    return None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_utils::get_node_with_id;

    /// --------------- Graph Creation Helper functions -------------------
    fn three_node_graph() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let a = graph.add_node(("a".to_string(), HashMap::new()));
        let b = graph.add_node(("b".to_string(), HashMap::new()));
        let c = graph.add_node(("c".to_string(), HashMap::new()));
        graph.add_edge(a, b, String::new());
        graph.add_edge(a, c, String::new());
        return graph;
    }

    fn two_node_graph() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let a = graph.add_node(("a".to_string(), HashMap::new()));
        let b = graph.add_node(("b".to_string(), HashMap::new()));

        graph.add_edge(a, b, String::new());
        return graph;
    }

    fn three_node_graph_with_properties() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let a_hashmap: HashMap<String, String> = [("height".to_string(), "100".to_string()), 
            ("breadth".to_string(), "5".to_string())].iter().cloned().collect();
        let a = graph.add_node(("a".to_string(), a_hashmap));
        let b = graph.add_node(("b".to_string(), HashMap::new()));
        let c = graph.add_node(("c".to_string(), HashMap::new()));
        graph.add_edge(a, b, String::new());
        graph.add_edge(a, c, String::new());
        return graph;
    }

    fn two_node_graph_with_properties() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let a_hashmap: HashMap<String, String> = [("height".to_string(), "100".to_string()), 
            ].iter().cloned().collect();
        let a = graph.add_node(("a".to_string(), a_hashmap));
        let b = graph.add_node(("b".to_string(), HashMap::new()));

        graph.add_edge(a, b, String::new());
        return graph;
    }

    fn two_node_graph_with_wrong_properties() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let a_hashmap: HashMap<String, String> = [("height".to_string(), "1".to_string()), 
            ].iter().cloned().collect();
        let a = graph.add_node(("a".to_string(), a_hashmap));
        let b = graph.add_node(("b".to_string(), HashMap::new()));

        graph.add_edge(a, b, String::new());
        return graph;
    }


    fn chain_graph() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let a = graph.add_node(("a".to_string(), HashMap::new()));
        let b = graph.add_node(("b".to_string(), HashMap::new()));
        let c = graph.add_node(("c".to_string(), HashMap::new()));
        let star = graph.add_node(("*".to_string(), HashMap::new()));

        graph.add_edge(a, b, String::new());
        graph.add_edge(b, c, String::new());
        graph.add_edge(c, star, String::new());
        return graph;
    }

    // from figure 2 in shamir paper
    fn g_figure_2() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let r = graph.add_node((String::from("r"), HashMap::new()));
        let v = graph.add_node((String::from("v"), HashMap::new()));
        let v1 = graph.add_node((String::from("v1"), HashMap::new()));
        let v2 = graph.add_node((String::from("v2"), HashMap::new()));
        let v3 = graph.add_node((String::from("v3"), HashMap::new()));

        let left_unnamed_child = graph.add_node((String::from("leftchild"), HashMap::new()));
        let right_unnamed_child = graph.add_node((String::from("rightchild"), HashMap::new()));

        graph.add_edge(r, v, String::new());
        graph.add_edge(v, v1, String::new());
        graph.add_edge(v, v2, String::new());
        graph.add_edge(v, v3, String::new());
        graph.add_edge(v1, left_unnamed_child, String::new());
        graph.add_edge(v1, right_unnamed_child, String::new());

        return graph;
    }

    // from figure 2 in shamir paper
    fn h_figure_2() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let u = graph.add_node((String::from("u"), HashMap::new()));
        let u1 = graph.add_node((String::from("u1"), HashMap::new()));
        let u2 = graph.add_node((String::from("u2"), HashMap::new()));
        let u3 = graph.add_node((String::from("u3"), HashMap::new()));
        let u1_left_child = graph.add_node((String::from("u1left"), HashMap::new()));
        let u1_right_child = graph.add_node((String::from("u1right"), HashMap::new()));
        let u3_child = graph.add_node((String::from("u3child"), HashMap::new()));

        graph.add_edge(u, u1, String::new());
        graph.add_edge(u, u2, String::new());
        graph.add_edge(u, u3, String::new());
        graph.add_edge(u1, u1_left_child, String::new());
        graph.add_edge(u1, u1_right_child, String::new());
        graph.add_edge(u3, u3_child, String::new());

        return graph;
    }

    fn three_child_graph() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let root = graph.add_node((String::from("root"), HashMap::new()));
        let child1 = graph.add_node((String::from("child1"), HashMap::new()));
        let child2 = graph.add_node((String::from("child2"), HashMap::new()));
        let child3 = graph.add_node((String::from("child3"), HashMap::new()));

        graph.add_edge(root, child1, String::new());
        graph.add_edge(root, child2, String::new());
        graph.add_edge(root, child3, String::new());

        return graph;
    }

    fn four_child_graph() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let root = graph.add_node((String::from("root"), HashMap::new()));
        let child1 = graph.add_node((String::from("child1"), HashMap::new()));
        let child2 = graph.add_node((String::from("child2"), HashMap::new()));
        let child3 = graph.add_node((String::from("child3"), HashMap::new()));
        let child4 = graph.add_node((String::from("child4"), HashMap::new()));

        graph.add_edge(root, child1, String::new());
        graph.add_edge(root, child2, String::new());
        graph.add_edge(root, child3, String::new());
        graph.add_edge(root, child4, String::new());

        return graph;
    }

    fn bookinfo_trace_graph() -> Graph<(String, HashMap<String, String>), String> {
        let mut graph = Graph::<(String, HashMap<String, String>), String>::new();
        let productpage = graph.add_node((String::from("productpage-v1"), HashMap::new()));
        let reviews = graph.add_node((String::from("reviews-v1"), HashMap::new()));
        let ratings = graph.add_node((String::from("ratings-v1"), HashMap::new()));
        let details = graph.add_node((String::from("details-v1"), HashMap::new()));

        graph.add_edge(productpage, reviews, String::new());
        graph.add_edge(productpage, details, String::new());
        graph.add_edge(reviews, ratings, String::new());

        return graph;
    }
    // ---------------------- Shamir Tests -------------------------

    #[test]
    fn test_initialize_s() {
        let graph_g = three_node_graph();
        let graph_h = two_node_graph();
        let s = initialize_s(&graph_g, &graph_h);
        assert!(s.keys().count() == 6);

        // useful debugging if this fails
        for key in s.keys() {
            print!(
                "key: {:?} weight: {:?}, {:?}\n",
                key,
                graph_g.node_weight(key.0),
                graph_h.node_weight(key.1)
            );
        }

        let aa = (
            get_node_with_id(&graph_g, "a".to_string()).unwrap(),
            get_node_with_id(&graph_h, "a".to_string()).unwrap(),
        );
        let ab = (
            get_node_with_id(&graph_g, "a".to_string()).unwrap(),
            get_node_with_id(&graph_h, "b".to_string()).unwrap(),
        );

        let ba = (
            get_node_with_id(&graph_g, "b".to_string()).unwrap(),
            get_node_with_id(&graph_h, "a".to_string()).unwrap(),
        );
        let bb = (
            get_node_with_id(&graph_g, "b".to_string()).unwrap(),
            get_node_with_id(&graph_h, "b".to_string()).unwrap(),
        );

        let ca = (
            get_node_with_id(&graph_g, "c".to_string()).unwrap(),
            get_node_with_id(&graph_h, "a".to_string()).unwrap(),
        );
        let cb = (
            get_node_with_id(&graph_g, "c".to_string()).unwrap(),
            get_node_with_id(&graph_h, "b".to_string()).unwrap(),
        );

        assert!(s.contains_key(&aa));
        assert!(s.contains_key(&ab));

        assert!(s.contains_key(&ba));
        assert!(s.contains_key(&bb));

        assert!(s.contains_key(&ca));
        assert!(s.contains_key(&cb));

        assert!(s[&aa].len() == 0);
        assert!(s[&ba].len() == 0);
        assert!(s[&ca].len() == 0);

        assert!(s[&bb].len() == 2, "bb len is {:?}", s[&bb].len());
        assert!(s[&cb].len() == 2, "cb len is {:?}", s[&cb].len());
    }

    #[test]
    fn test_shamir_small_graphs() {
        let graph_g = three_node_graph();
        let graph_h = two_node_graph();
        assert!(find_mapping_shamir_centralized(&graph_g, &graph_h).is_some());
    }
    #[test]
    fn test_shamir_figure_2() {
        let graph_g = g_figure_2();
        let graph_h = h_figure_2();
        assert!(find_mapping_shamir_centralized(&graph_g, &graph_h).is_none());
    }

    #[test]
    fn test_shamir_chain_graphs() {
        let graph_g = chain_graph();
        let graph_h_1 = two_node_graph();
        assert!(find_mapping_shamir_centralized(&graph_g, &graph_h_1).is_some());
    }

    #[test]
    fn test_shamir_branching_graphs() {
        let graph_g = four_child_graph();
        let graph_h = three_child_graph();
        assert!(find_mapping_shamir_centralized(&graph_g, &graph_h).is_some());

        let graph_g_2 = three_child_graph();
        let graph_h_2 = four_child_graph();
        assert!(find_mapping_shamir_centralized(&graph_g_2, &graph_h_2).is_none());
    }

    #[test]
    fn test_shamir_on_bookinfo() {
        let graph_g = bookinfo_trace_graph();
        let graph_h = three_node_graph();
        let mapping_wrapped = find_mapping_shamir_centralized(&graph_g, &graph_h);
        assert!(mapping_wrapped.is_some());
        let mapping = mapping_wrapped.unwrap();
        let a = get_node_with_id(&graph_h, "a".to_string()).unwrap();
        let b = get_node_with_id(&graph_h, "b".to_string()).unwrap();
        let c = get_node_with_id(&graph_h, "c".to_string()).unwrap();
        let prod = get_node_with_id(&graph_g, "productpage-v1".to_string()).unwrap();
        let det = get_node_with_id(&graph_g, "details-v1".to_string()).unwrap();
        let rev = get_node_with_id(&graph_g, "reviews-v1".to_string()).unwrap();
        assert!(mapping.contains(&(a,prod)));
        assert!(mapping.contains(&(b,det)) || mapping.contains(&(c,det)));
        assert!(mapping.contains(&(b,rev)) || mapping.contains(&(c,rev)));
        
    }

    #[test]
    fn test_shamir_full_match() {
        let graph_g = three_node_graph();
        let graph_h = three_node_graph();
        assert!(find_mapping_shamir_centralized(&graph_g, &graph_h).is_some());
    }

    #[test]
    fn test_property_matches() {
        let graph_g = three_node_graph_with_properties();
        let graph_h = two_node_graph_with_properties();
        assert!(find_mapping_shamir_centralized(&graph_g, &graph_h).is_some());
    }

    #[test]
    fn test_property_does_not_match() {
        let graph_g = three_node_graph_with_properties();
        let graph_h = two_node_graph_with_wrong_properties();
        assert!(find_mapping_shamir_centralized(&graph_g, &graph_h).is_none());

    }
}
