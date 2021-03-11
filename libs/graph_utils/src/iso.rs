use mcmf::{Capacity, Cost, GraphBuilder, Path, Vertex};
use petgraph::algo::dijkstra;
/// Implements subgraph isomorphism algorithms two ways:
/// as described in https://www.cs.bgu.ac.il/~dekelts/publications/subtree.pdf
/// Another thing to consider, but is not implemented here, is
/// http://chasewoerner.org/popl87.pdf
///
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::DfsPostOrder;
use petgraph::Incoming;
use std::collections::HashMap;
use std::collections::HashSet;

// ----------------- Graph Convenience Functions ------------------
// TODO:  this entire section should migrate to graph utils once
// we stop using strings and instead use real graphs
fn find_leaves(node: NodeIndex, graph: &Graph<String, String>) -> Vec<NodeIndex> {
    let mut post_order = DfsPostOrder::new(&graph, node);
    let mut to_return = Vec::new();
    while let Some(visited) = post_order.next(&graph) {
        let neighbors: Vec<NodeIndex> = graph.neighbors(visited).collect();
        if neighbors.len() == 0 {
            to_return.push(visited);
        }
    }
    return to_return;
}

fn find_root(graph: &Graph<String, String>) -> NodeIndex {
    for node in graph.node_indices() {
        let neighbors: Vec<NodeIndex> = graph.neighbors_directed(node, Incoming).collect();
        if neighbors.len() == 0 {
            return node;
        }
    }
    panic!("no root found");
}

// TODO: once this works, stop making everything a string and use the graph
// utils version
fn find_node_with_weight(graph: &Graph<String, String>, weight: String) -> NodeIndex {
    for node in graph.node_indices() {
        if graph.node_weight(node).unwrap() == &weight {
            return node;
        }
    }
    panic!("could not find node with weight {0}", weight);
}

fn get_height(graph: &Graph<String, String>, node: NodeIndex) -> u32 {
    let distances = dijkstra(graph, node, None, |_| 1);
    let mut max = 0;
    for key in distances.keys() {
        if distances[key] > max {
            max = distances[key];
        }
    }
    return max;
}

// ----------------- Shamir Isomorphism Algorithm ------------------

// this performs lines 0-4 in the Shamir paper figure 3
fn initialize_s(
    graph_g: &Graph<String, String>,
    graph_h: &Graph<String, String>,
) -> HashMap<(NodeIndex, NodeIndex), HashSet<NodeIndex>> {
    let mut s = HashMap::<(NodeIndex, NodeIndex), HashSet<NodeIndex>>::new();
    for node_g in graph_g.node_indices() {
        for node_h in graph_h.node_indices() {
            // initialize S entry as empty set
            s.insert((node_g, node_h), HashSet::new());
        }
    }
    let root_g = find_root(&graph_g);
    let root_h = find_root(&graph_h);
    for leaf_g in find_leaves(root_g, &graph_g) {
        for leaf_h in find_leaves(root_h, &graph_h) {
            for neighbor in graph_h.neighbors_directed(leaf_h, Incoming) {
                s.get_mut(&(leaf_g, leaf_h)).unwrap().insert(neighbor);
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
    graph_g: &Graph<String, String>,
    graph_h: &Graph<String, String>,
    set_s: &HashMap<(NodeIndex, NodeIndex), HashSet<NodeIndex>>,
    u_null: NodeIndex,
) -> (i32, Vec<Path<String>>) {
    let mut graph_builder = GraphBuilder::new();
    let mut added_nodes = HashSet::new();
    for u in set_x {
        for v in set_y {
            if set_s[&(*v, *u)].contains(&u_null) {
                let mut u_str = graph_h.node_weight(*u).unwrap().clone();
                u_str.push_str("U");
                if !added_nodes.contains(&u_str) {
                    graph_builder.add_edge(Vertex::Source, u_str.to_string(), Capacity(1), Cost(0));
                    added_nodes.insert(u_str.to_string());
                }

                let mut v_str = graph_h.node_weight(*v).unwrap().clone();
                v_str.push_str("v");
                if !added_nodes.contains(&v_str) {
                    graph_builder.add_edge(v_str.to_string(), Vertex::Sink, Capacity(1), Cost(0));
                    added_nodes.insert(v_str.to_string());
                }

                graph_builder.add_edge(u_str.to_string(), v_str.to_string(), Capacity(1), Cost(1));
            }
        }
    }
    let (cost, paths) = graph_builder.mcmf();
    print!("cost: {:?}\n", cost);
    return (cost, paths);
    //return graph_builder.mcmf();
}

fn find_mapping_shamir_centralized_inner_loop(
    v: NodeIndex,
    graph_g: &Graph<String, String>,
    graph_h: &Graph<String, String>,
    set_s: &mut HashMap<(NodeIndex, NodeIndex), HashSet<NodeIndex>>,
) -> bool {
    let root_h = find_root(&graph_h);
    let v_neighbors: Vec<NodeIndex> = graph_g.neighbors(v).collect();
    for node_h in graph_h.node_indices() {
        let u_neighbors: Vec<NodeIndex> = graph_h.neighbors(node_h).collect();
        // all vertices of degree at most t+1
        if u_neighbors.len() > v_neighbors.len() + 1 {
            continue;
        }

        // maximum matching where X0 = X
        let (cost, path) =
            max_matching(&u_neighbors, &v_neighbors, graph_g, graph_h, set_s, node_h);
        if cost == u_neighbors.len() as i32 {
            set_s.get_mut(&(v, node_h)).unwrap().insert(node_h);
        }

        // maximum matching where X0 is X minus an element
        for vertex in 0..u_neighbors.len() {
            let mut new_x_set = u_neighbors.clone();
            new_x_set.remove(vertex);
            let (cost, path) =
                max_matching(&new_x_set, &v_neighbors, graph_g, graph_h, set_s, node_h);
            if cost == new_x_set.len() as i32 {
                set_s.get_mut(&(v, node_h)).unwrap().insert(node_h);
            }
        }

        // lines 12-14
        if set_s[&(v, node_h)].contains(&root_h) {
            return true;
        }
    }
    return false;
}

fn find_mapping_shamir_centralized(
    graph_g: Graph<String, String>,
    graph_h: Graph<String, String>,
) -> bool {
    // TODO:  before even dealing with isomorphism, ask if breadth,
    // height, num nodes match up

    // initialize S with all N(u) sets, lines 1-4
    let mut set_s = initialize_s(&graph_g, &graph_h);
    let root_g = find_root(&graph_g);

    // postorder traversal and filtering of children for degrees, lines 5-8;
    let mut post_order = DfsPostOrder::new(&graph_g, root_g);
    while let Some(node) = post_order.next(&graph_g) {
        let v_children: Vec<NodeIndex> = graph_g.neighbors(node).collect();
        let v_children_len = v_children.len();
        if v_children_len > 0 {
            let mapping_found =
                find_mapping_shamir_centralized_inner_loop(node, &graph_h, &graph_h, &mut set_s);
            if mapping_found {
                return true;
            }
        }
    }
    // line 15
    print!("returning false, set_s is: \n");
    for key in set_s.keys() {
        print!("key: {:?}, value: ", key);
        for value in &set_s[key] {
            print!(" {:?} ", value);
        }
        print!("\n");
    }
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// --------------- Graph Creation Helper functions -------------------
    fn three_node_graph() -> Graph<String, String> {
        let mut graph = Graph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());
        graph.add_edge(a, b, String::new());
        graph.add_edge(a, c, String::new());
        return graph;
    }

    fn two_node_graph_star() -> Graph<String, String> {
        let mut graph = Graph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("*".to_string());
        graph.add_edge(a, b, String::new());
        return graph;
    }

    fn two_node_graph() -> Graph<String, String> {
        let mut graph = Graph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        graph.add_edge(a, b, String::new());
        return graph;
    }

    fn chain_graph() -> Graph<String, String> {
        let mut graph = Graph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());
        let star = graph.add_node("*".to_string());
        graph.add_edge(a, b, String::new());
        graph.add_edge(b, c, String::new());
        graph.add_edge(c, star, String::new());
        return graph;
    }

    fn little_branching_graph() -> Graph<String, String> {
        let mut graph = Graph::<String, String>::default();
        graph.extend_with_edges(&[(0, 1), (0, 2), (0, 3), (1, 4), (3, 5)]);
        return graph;
    }

    // from figure 2 in shamir paper
    fn g_figure_2() -> Graph<String, String> {
        let mut graph = Graph::<String, String>::default();
        let r = graph.add_node(String::from("r"));
        let v = graph.add_node(String::from("v"));
        let v1 = graph.add_node(String::from("v1"));
        let v2 = graph.add_node(String::from("v2"));
        let v3 = graph.add_node(String::from("v3"));
        let left_unnamed_child = graph.add_node(String::from("leftchild"));
        let right_unnamed_child = graph.add_node(String::from("rightchild"));

        graph.add_edge(r, v, String::new());
        graph.add_edge(v, v1, String::new());
        graph.add_edge(v, v2, String::new());
        graph.add_edge(v, v3, String::new());
        graph.add_edge(v1, left_unnamed_child, String::new());
        graph.add_edge(v1, right_unnamed_child, String::new());

        return graph;
    }

    // from figure 2 in shamir paper
    fn h_figure_2() -> Graph<String, String> {
        let mut graph = Graph::<String, String>::default();
        let u = graph.add_node(String::from("u"));
        let u1 = graph.add_node(String::from("u1"));
        let u2 = graph.add_node(String::from("u2"));
        let u3 = graph.add_node(String::from("u3"));
        let u1_left_child = graph.add_node(String::from("u1left"));
        let u1_right_child = graph.add_node(String::from("u1right"));
        let u3_child = graph.add_node(String::from("u3child"));

        graph.add_edge(u, u1, String::new());
        graph.add_edge(u, u2, String::new());
        graph.add_edge(u, u3, String::new());
        graph.add_edge(u1, u1_left_child, String::new());
        graph.add_edge(u1, u1_right_child, String::new());
        graph.add_edge(u3, u3_child, String::new());

        return graph;
    }

    fn three_child_graph() -> Graph<String, String> {
        let mut graph = Graph::<String, String>::new();
        let root = graph.add_node(String::from("root"));
        let child1 = graph.add_node(String::from("child1"));
        let child2 = graph.add_node(String::from("child2"));
        let child3 = graph.add_node(String::from("child3"));

        graph.add_edge(root, child1, String::new());
        graph.add_edge(root, child2, String::new());
        graph.add_edge(root, child3, String::new());

        return graph;
    }

    fn four_child_graph() -> Graph<String, String> {
        let mut graph = Graph::<String, String>::new();
        let root = graph.add_node(String::from("root"));
        let child1 = graph.add_node(String::from("child1"));
        let child2 = graph.add_node(String::from("child2"));
        let child3 = graph.add_node(String::from("child3"));
        let child4 = graph.add_node(String::from("child4"));

        graph.add_edge(root, child1, String::new());
        graph.add_edge(root, child2, String::new());
        graph.add_edge(root, child3, String::new());
        graph.add_edge(root, child4, String::new());

        return graph;
    }
    // ---------------------- Shamir Tests -------------------------

    #[test]
    fn test_find_leaves() {
        let graph = little_branching_graph();
        let leaves = find_leaves(NodeIndex::new(0), &graph);
        let correct_leaves = vec![2, 4, 5];
        for leaf in &leaves {
            assert!(correct_leaves.contains(&leaf.index()));
            print!(" leaf : {0} ", leaf.index());
        }
    }

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
            find_node_with_weight(&graph_g, "a".to_string()),
            find_node_with_weight(&graph_h, "a".to_string()),
        );
        let ab = (
            find_node_with_weight(&graph_g, "a".to_string()),
            find_node_with_weight(&graph_h, "b".to_string()),
        );

        let ba = (
            find_node_with_weight(&graph_g, "b".to_string()),
            find_node_with_weight(&graph_h, "a".to_string()),
        );
        let bb = (
            find_node_with_weight(&graph_g, "b".to_string()),
            find_node_with_weight(&graph_h, "b".to_string()),
        );

        let ca = (
            find_node_with_weight(&graph_g, "c".to_string()),
            find_node_with_weight(&graph_h, "a".to_string()),
        );
        let cb = (
            find_node_with_weight(&graph_g, "c".to_string()),
            find_node_with_weight(&graph_h, "b".to_string()),
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

        assert!(s[&bb].len() == 1, "bb len is {:?}", s[&bb].len());
        assert!(s[&cb].len() == 1, "cb len is {:?}", s[&cb].len());
    }

    #[test]
    fn test_shamir_small_graphs() {
        let graph_g = three_node_graph();
        let graph_h = two_node_graph();
        assert!(find_mapping_shamir_centralized(graph_g, graph_h));
    }

    #[test]
    fn test_shamir_chain_graphs() {
        let graph_g = chain_graph();
        let graph_h_1 = two_node_graph();
        let graph_h_2 = three_node_graph();
        assert!(find_mapping_shamir_centralized(graph_g.clone(), graph_h_1));
        assert!(find_mapping_shamir_centralized(graph_g, graph_h_2));
    }

    #[test]
    fn test_shamir_branching_graphs() {
        let graph_g = h_figure_2();
        let graph_h = three_node_graph();
        assert!(find_mapping_shamir_centralized(graph_g, graph_h));

        let graph_g_2 = three_child_graph();
        let graph_h_2 = four_child_graph();
        assert!(!find_mapping_shamir_centralized(graph_g_2, graph_h_2));
    }

    #[test]
    fn test_shamir_figure_2() {
        let graph_g = g_figure_2();
        let graph_h = h_figure_2();
        assert!(!find_mapping_shamir_centralized(graph_g, graph_h));
    }
}
