/// Implements subgraph isomorphism algorithms two ways:
/// as described in https://www.cs.bgu.ac.il/~dekelts/publications/subtree.pdf
/// and as described in http://www.grantjenks.com/wiki/_media/ideas/patternmatch.pdf
/// Another thing to consider, but is not implemented here, is 
/// http://chasewoerner.org/popl87.pdf
///
/// The first algorithm does not care about the ordering of the children of a node,
/// and the second one does.

use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::DfsPostOrder;
use petgraph::Incoming;
use petgraph::algo::{dijkstra, toposort};
use std::collections::HashSet;
use std::collections::HashMap;

fn find_leaves(node: NodeIndex, graph: &Graph<String,String>) -> Vec<NodeIndex> {
    let mut post_order = DfsPostOrder::new(&graph, node);
    let mut to_return = Vec::new();
    while let Some(visited) = post_order.next(&graph) {
        let neighbors : Vec<NodeIndex> = graph.neighbors(visited).collect();
        if neighbors.len() == 0 { to_return.push(visited); }
    }
    return to_return;
}

fn find_root(graph: &Graph<String, String>) -> NodeIndex {
    for node in graph.node_indices() {
        let neighbors : Vec<NodeIndex> = graph.neighbors_directed(node, Incoming).collect();
        if neighbors.len() == 0 { return node; }
    }
    panic!("no root found");
}

// this performs lines 0-4 in the Shamir paper figure 3
fn initialize_s(graph_g: &Graph<String, String>, graph_h: &Graph<String, String>) -> HashMap::<(NodeIndex, NodeIndex), HashSet<NodeIndex>> {
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
/*
fn construct_bipartite_graph(edge_set: Vec<(String, String)>) -> Graph<String, ()> {
    let mut graph = Graph::<String,()>::new();
    let mut added_nodes = HashMap::new();
    for edge in edge_set {
        if !added_nodes.contains_key(&edge.0) {
            let node = graph.add_node(edge.0.clone());
            added_nodes.insert(&edge.0.clone(), node);
        }
        if !added_nodes.contains_key(&edge.1) {
            let node = graph.add_node(edge.1.clone());
            added_nodes.insert(&edge.1, node);
        }
        graph.add_edge(added_nodes[&edge.0], added_nodes[&edge.1], ());
    }
    return graph;
}
*/
fn maximum_matching_size(set_x: &Vec<NodeIndex>, set_y: &Vec<NodeIndex>) -> u32 {
    return 0;
}

fn find_mapping_shamir(graph_g: Graph<String, String>, graph_h: Graph<String, String>) -> bool {
    // initialize S with all N(u) sets, lines 1-4
    let mut s_set = initialize_s(&graph_g, &graph_h);
    let root_g = find_root(&graph_g); 

    // postorder traversal and filtering of children for degrees, ines 5-8
    let mut post_order = DfsPostOrder::new(&graph_g, root_g);
    while let Some(node) = post_order.next(&graph_g) {
        let v_children : Vec<NodeIndex> = graph_g.neighbors(node).collect();
        let v_children_len = v_children.len();
        for node_h in graph_h.node_indices() {
	    let u_neighbors : Vec<NodeIndex> = graph_h.neighbors(node_h).collect();
            if u_neighbors.len() <= v_children_len+1 {

                // construct bipartite graph, line 9
                let mut edge_set = Vec::new();
                for u in &u_neighbors {
                    for v in &v_children {
                        if s_set[&(*u,*v)].contains(&node_h) {
                            let mut u_str = u.index().to_string();
                            u_str.push_str("u");
                            let mut v_str = v.index().to_string();
                            v_str.push_str("v");
                            edge_set.push((u_str,v_str));
                        }
                    }
                }
                //let bipartite = construct_bipartite_graph(edge_set);

                // lines 10-11
                for i in 0..u_neighbors.len() {
                    let mut x_i = u_neighbors.clone();
                    if i != 0 { x_i.remove(i); }
                    let maximum_matching = maximum_matching_size(&x_i, &v_children);
                    if maximum_matching == x_i.len() as u32 {
                        s_set.get_mut(&(node, node_h)).unwrap().insert(u_neighbors[i]);
                    }
                    
                    // lines 12-14
                    if s_set[&(node, node_h)].contains(&node_h) { return true; }
                }
            }
        }
    }
    // line 15
    return false;

}

// Creates the subsumption graph gs
fn algorithm_a_hoffman(graph_h: &Graph<String, String>) -> Graph<NodeIndex,()> {
    // 1. List subtrees by increasing height (this would be equivalent to listing nodes by height
    //    and then assuming everything below it is a subtree) in trace graph
    //    Note that height is just length from the root node so we can use dijkstra's
    let node_id_to_level = dijkstra(&graph_h, find_root(&graph_h), None, |e| 1);

    // dijkstra gives us a node ID to level mapping, but we want to sort by level
    let mut level_node_pairs = Vec::new();
    for node_id in node_id_to_level.keys() {
        let level = node_id_to_level[node_id];
        level_node_pairs.push((level, node_id));
    }
    level_node_pairs.sort_by_key(|&(x,y)| x);

    // 2. Initialize the graph Gs (the "immediate subsumption graph")
    let gs = Graph::<NodeIndex, ()>::new();

    for (level, node) in level_node_pairs {
        // initializing gs with nodes in PF
        
    }
    

    // 3. For each pattern, check subsumption and add edges if relevant 


    return gs;

}

fn algorithm_b_hoffman(gs: &Graph<NodeIndex, ()>, graph_h: &Graph<String, String>) -> HashMap<NodeIndex, Vec<String>> {
    // 4. Use algorithm B, as described in the paper, to get tables
    let top_sort = toposort(&gs, None).unwrap();
    let mut tables = HashMap::<NodeIndex, Vec<String>>::new(); // hashmap of pattern (as rep by node) to patterns
    // 4a. initialize to *
    for node in top_sort {
        let mut vec = Vec::new();
        vec.push("*".to_string());
        tables.insert(*gs.node_weight(node).unwrap(), vec);
    }
    for node_as_pattern in graph_h.node_indices() {
        for other_node_as_pattern in graph_h.node_indices() {
            // TODO: if node_as_pattern's subtree is subsumed by other_node_as_pattern's subtree for all children
            if graph_h.neighbors(node_as_pattern).count() == graph_h.neighbors(other_node_as_pattern).count() {
                let mut subsumed = false;
                for neighbor in graph_h.neighbors(node_as_pattern) {
                    for other_neighbor in graph_h.neighbors(other_node_as_pattern) {
                        let reachable = dijkstra(&gs, neighbor, Some(other_neighbor), |e| 1);
                        if reachable.contains_key(&other_neighbor) {
                            tables.get_mut(&node_as_pattern).unwrap().push(graph_h.node_weight(other_node_as_pattern).unwrap().to_string());
                        }
                    }
                }
            }
        }
    }
    return tables;
}

fn precompute_hoffman(graph_h: &Graph<String, String>) -> HashMap<NodeIndex, Vec<String>> {
    let gs = algorithm_a_hoffman(graph_h);
    let table = algorithm_b_hoffman(&gs, graph_h);
    return table;
}


fn compute_hoffman(precompute_output: HashMap<NodeIndex, Vec<String>>, graph_g: Graph<String,String>, graph_h: Graph<String, String>) -> Vec<(String, String)> {
    // TODO
    let mut post_order = DfsPostOrder::new(&graph_g, find_root(&graph_g));
    let mut matchings = HashMap::<NodeIndex, Vec<String>>::new();
    while let Some(node) = post_order.next(&graph_g) {
        // TODO:  assign node symbols
        let mut node_symbols = Vec::new();
        matchings.insert(node, node_symbols);
    }
    return Vec::new();

}

fn find_mapping_hoffman(graph_g: Graph<String, String>, graph_h: Graph<String, String>) -> bool {
    let precompute_output = precompute_hoffman(&graph_h);
    let mapping = compute_hoffman(precompute_output, graph_g, graph_h);
    if mapping.len() > 0 { return true; }
    return false;
}












#[cfg(test)]
mod tests {
    use super::*;


    fn little_branching_graph() -> Graph<String,String> {
        let mut graph = Graph::<String,String>::default();
        graph.extend_with_edges(&[
            (0, 1), (0, 2), (0, 3), (1, 4), (3, 5)
        ]);
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
    fn test_precompute_hoffman() {
        let graph = little_branching_graph();
        let table = precompute_hoffman(h_figure_2);
        for key in table.keys() {
            println!("key: {:?}", key);
            for value in table[key] {
                print!("value: {:?} ", value);
            }
        }

    }
}
