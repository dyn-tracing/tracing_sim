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
use mcmf::{GraphBuilder, Vertex, Cost, Capacity, Path};

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
fn max_matching(set_x: &Vec<String>, set_y: &Vec<String>, edge_set: &Vec<&(String, String)>) -> (i32, Vec<Path<String>>) {
    let mut graph_builder = GraphBuilder::new();
    for node in set_x {
        graph_builder.add_edge(Vertex::Source, node.to_string(), Capacity(1), Cost(0));
    }
    for node in set_y {
        graph_builder.add_edge(node.to_string(), Vertex::Sink, Capacity(1), Cost(0));
    }
    for edge in edge_set {
        print!("adding edge in edge set");
        graph_builder.add_edge(edge.0.clone(), edge.1.clone(), Capacity(1), Cost(1));
    }
    let (cost, paths) = graph_builder.mcmf();
    print!("cost: {:?}\n", cost);
    return (cost, paths);
    //return graph_builder.mcmf();
}

fn find_mapping_shamir(graph_g: Graph<String, String>, graph_h: Graph<String, String>) -> bool {
    // initialize S with all N(u) sets, lines 1-4
    let mut s_set = initialize_s(&graph_g, &graph_h);
    let root_g = find_root(&graph_g); 

    // postorder traversal and filtering of children for degrees, lines 5-8
    let mut post_order = DfsPostOrder::new(&graph_g, root_g);
    while let Some(node) = post_order.next(&graph_g) {
        let v_children : Vec<NodeIndex> = graph_g.neighbors(node).collect();
        let v_children_len = v_children.len();
        for node_h in graph_h.node_indices() {
	    let u_neighbors : Vec<NodeIndex> = graph_h.neighbors(node_h).collect();
            if u_neighbors.len() <= v_children_len+1 {

                // construct bipartite graph, line 9
                let mut set_x_strings = Vec::new();
                let mut set_y_strings = Vec::new();

                let mut edge_set = Vec::new();
                for u in &u_neighbors {
                    for v in &v_children {
                        if s_set[&(*v,*u)].contains(&node_h) {
                            let mut u_str = u.index().to_string();
                            u_str.push_str("u");
                            set_x_strings.push(u_str.clone());
                            let mut v_str = v.index().to_string();
                            v_str.push_str("v");
                            set_y_strings.push(v_str.clone());
                            print!("hello\n\n\n");
                            edge_set.push((u_str,v_str));
                        }
                    }
                }
               
                // lines 10-11
                // TODO: include x_0 first, before loop
                let mut edge_set_addresses = Vec::new();
                for edge in &edge_set { edge_set_addresses.push(edge); }
                let (cost, paths) = max_matching(&set_x_strings, &set_y_strings, &edge_set_addresses);
                if cost == set_x_strings.len() as i32 {
                    s_set.get_mut(&(node, node_h)).unwrap().insert(node_h);
                }
                

                for i in 0..set_x_strings.len() {
                    let mut x_i = set_x_strings.clone();
                    let x_i_len_without_i : i32 = (x_i.len()-1) as i32;

                    // make edge set and x set without x_i
                    let mut edge_set_without_x_i = Vec::new();
                    let mut x_str = &x_i[i];
                    for edge in &edge_set {
                        if &edge.0 != x_str && &edge.1 != x_str {
                            edge_set_without_x_i.push(edge);
                        }
                    }
                    print!("len edge set is {:?}\n", edge_set_without_x_i.len());
                    let (cost, paths) = max_matching(&x_i, &set_y_strings, &edge_set_without_x_i);
                    if cost == x_i_len_without_i {
                        s_set.get_mut(&(node, node_h)).unwrap().insert(u_neighbors[i]);
                    }
                    
                    // lines 12-14
                    if s_set[&(node, node_h)].contains(&node_h) { return true; }
                }
            }
        }
    }
    // line 15
    print!("returning false, s_set is: \n");
    for key in s_set.keys() {
        print!("key: {:?}, value: ", key);
        for value in &s_set[key] {
            print!(" {:?} ", value);
        }
        print!("\n");

    }
    return false;

}
fn find_node_with_weight_shamir(graph: &Graph<String,String>, weight: String) -> NodeIndex {
    for node in graph.node_indices() {
        if graph.node_weight(node).unwrap() == &weight { return node; }
    }
    panic!("could not find node with weight {0}", weight);
}

fn find_node_with_weight_hoffman(graph: &Graph<String,()>, weight: String) -> NodeIndex {
    for node in graph.node_indices() {
        if graph.node_weight(node).unwrap() == &weight { return node; }
    }
    panic!("could not find node with weight {0}", weight);
}

// only works for trees
fn get_height(graph: &Graph<String, String>, node: NodeIndex) -> u32 {
    let distances = dijkstra(graph, node, None, |_| 1);
    let mut max = 0;
    for key in distances.keys() {
        if distances[key] > max { max = distances[key]; }
    }
    return max;
}
// Creates the subsumption graph gs
fn algorithm_a_hoffman(graph_h: &Graph<String, String>) -> Graph<String,()> {
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
    level_node_pairs.sort_by(|a, b| b.0.cmp(&a.0));
    println!("first node {:?} at level {:?}", graph_h.node_weight(*level_node_pairs[0].1), level_node_pairs[0].0);
    
    // 2. Initialize the graph Gs (the "immediate subsumption graph")
    let mut gs = Graph::<String, ()>::new();

    let mut node_weight_to_gs_node = HashMap::new();
    for (level, node) in &level_node_pairs {
        // initializing gs with nodes in PF.  here each node represents the pattern of that node's children
        let node_weight = graph_h.node_weight(**node).unwrap();
        let node = gs.add_node(node_weight.to_string());
        node_weight_to_gs_node.insert(node_weight, node);
    }
    
    // 3. For each pattern, which here is represented by the parent of the children in the pattern,
    //    check subsumption and add edges if relevant
    //    We look by increasing order of height
    for i in 0..level_node_pairs.len() {
        let node_p_in_graph_h = *level_node_pairs[i].1;
        let node_p = find_node_with_weight_hoffman(&gs, graph_h.node_weight(node_p_in_graph_h).unwrap().to_string());
        println!("\nlooking at node {:?}", gs.node_weight(node_p));
        for j in 0..level_node_pairs.len() {
            if get_height(graph_h, *level_node_pairs[j].1) > get_height(graph_h, *level_node_pairs[i].1) { continue; }
            let node_p_prime_in_graph_h = *level_node_pairs[j].1;
            let node_p_prime = find_node_with_weight_hoffman(&gs, graph_h.node_weight(node_p_prime_in_graph_h).unwrap().to_string());
            println!("comparing with node {:?}", gs.node_weight(node_p_prime));
            // we need separate patterns
            if gs.node_weight(node_p_prime).unwrap() == "*" {
                let children: Vec<NodeIndex> = graph_h.neighbors(node_p_in_graph_h).collect();
                for node_p_child_in_graph_h in children {
                    if graph_h.node_weight(node_p_child_in_graph_h).unwrap() == "*" {
                        print!("adding star edge between {:?} and {:?}\n", gs.node_weight(node_p), gs.node_weight(node_p_prime));
                        gs.add_edge(node_p, node_p_prime, ());
                    }
                }
            } else {
                let mut subsumes = true;
                if graph_h.neighbors(node_p_in_graph_h).count() != graph_h.neighbors(node_p_prime_in_graph_h).count() {
                    subsumes = false;
                }
                for p_child_in_graph_h in graph_h.neighbors(node_p_in_graph_h) {
                    for p_prime_child_in_graph_h in graph_h.neighbors(node_p_prime_in_graph_h) {
                        let p_child = find_node_with_weight_hoffman(&gs, graph_h.node_weight(p_child_in_graph_h).unwrap().to_string());
                        let p_prime_child = find_node_with_weight_hoffman(&gs, graph_h.node_weight(p_prime_child_in_graph_h).unwrap().to_string());
                        if !gs.contains_edge(p_child, p_prime_child) { subsumes = false; }
                    }
                }
                if subsumes {
                    if !gs.contains_edge(node_p, node_p_prime) {
                        println!("adding nonstar edge from {:?} to {:?}", gs.node_weight(node_p), gs.node_weight(node_p_prime));
                        gs.add_edge(node_p, node_p_prime, ());
                    }
                }
            } 
        }
    } 
    return gs;

}

// uses subsumption graph to make table, which will be used in actual matching step
fn algorithm_b_hoffman(gs: &Graph<String, ()>, graph_h: &Graph<String, String>) -> HashMap<String, String> {
    let top_sort_wrapped = toposort(&gs, None);
    if let Err(e) = top_sort_wrapped {
        println!("could not perform topological sort on gs because {:?}", e);
        panic!();
    }
    let top_sort = top_sort_wrapped.unwrap();

    // initialize tables
    let mut tables = HashMap::<String, String>::new(); // hashmap of pattern (as rep by node) to patterns
    for node in &top_sort {
        tables.insert(gs.node_weight(*node).unwrap().to_string(), "*".to_string());
    }

    // iterate through PF
    for node_as_pattern in top_sort {
        for other_node_as_pattern in graph_h.node_indices() {
            // 1. check that the pattern's children are subsumed for all children
            if graph_h.neighbors(node_as_pattern).count() == graph_h.neighbors(other_node_as_pattern).count() {
                let mut subsumed = true;
                for neighbor in graph_h.neighbors(node_as_pattern) {
                    for other_neighbor in graph_h.neighbors(other_node_as_pattern) {
                        let neighbor_in_gs = find_node_with_weight_hoffman(&gs, graph_h.node_weight(neighbor).unwrap().to_string());
                        let other_neighbor_in_gs = find_node_with_weight_hoffman(&gs, graph_h.node_weight(other_neighbor).unwrap().to_string());
                        let reachable = dijkstra(&gs, neighbor_in_gs, Some(other_neighbor_in_gs), |e| 1);
                        if !reachable.contains_key(&other_neighbor_in_gs) {
                            subsumed = false;
                        }
                    }
                }
                // if so, those children should be counted
                if subsumed {
                    for other_neighbor in graph_h.neighbors(other_node_as_pattern) {
                        tables.insert(graph_h.node_weight(other_neighbor).unwrap().to_string(), graph_h.node_weight(node_as_pattern).unwrap().to_string());
                    }
                }
            }
        }
    }
    // finally consider the root, which is left out since we iterate through and consider nodes' children
    return tables;
}

// does both algo a and b to complete preprocessing step
fn precompute_hoffman(graph_h: &Graph<String, String>) -> HashMap<String, String> {
    let mut gs = algorithm_a_hoffman(graph_h);
    let table = algorithm_b_hoffman(&gs, graph_h);
    return table;
}

// uses precompute output to do matching step
fn compute_hoffman(precompute_output: HashMap<String, String>, graph_g: Graph<String,String>, graph_h: Graph<String, String>) -> Vec<(String, String)> {
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

    /// ---------------------- Graph Creation Helper functions -------------------------
    fn three_node_graph() -> Graph<String,String> {
        let mut graph = Graph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());
        graph.add_edge(a,b, String::new());
        graph.add_edge(a,c, String::new());
        return graph;
        
    }

    fn two_node_graph_star() -> Graph<String,String> {
        let mut graph = Graph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("*".to_string());
        graph.add_edge(a,b, String::new());
        return graph;
    }

    fn two_node_graph() -> Graph<String,String> {
        let mut graph = Graph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        graph.add_edge(a,b, String::new());
        return graph;
    }

    fn chain_graph() -> Graph<String, String> {
        let mut graph = Graph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());
        let star = graph.add_node("*".to_string());
        graph.add_edge(a,b, String::new());
        graph.add_edge(b,c, String::new());
        graph.add_edge(c,star, String::new());
        return graph;
    }

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
        assert!(s.keys().count()==6);

        // useful debugging if this fails
        for key in s.keys() {
            print!("key: {:?} weight: {:?}, {:?}\n", key, graph_g.node_weight(key.0), graph_h.node_weight(key.1));
        }

        let aa = (find_node_with_weight_shamir(&graph_g, "a".to_string()), find_node_with_weight_shamir(&graph_h, "a".to_string()));
        let ab = (find_node_with_weight_shamir(&graph_g, "a".to_string()), find_node_with_weight_shamir(&graph_h, "b".to_string()));

        let ba = (find_node_with_weight_shamir(&graph_g, "b".to_string()), find_node_with_weight_shamir(&graph_h, "a".to_string()));
        let bb = (find_node_with_weight_shamir(&graph_g, "b".to_string()), find_node_with_weight_shamir(&graph_h, "b".to_string()));

        let ca = (find_node_with_weight_shamir(&graph_g, "c".to_string()), find_node_with_weight_shamir(&graph_h, "a".to_string()));
        let cb = (find_node_with_weight_shamir(&graph_g, "c".to_string()), find_node_with_weight_shamir(&graph_h, "b".to_string()));

        assert!(s.contains_key(&aa));
        assert!(s.contains_key(&ab));

        assert!(s.contains_key(&ba));
        assert!(s.contains_key(&bb));

        assert!(s.contains_key(&ca));
        assert!(s.contains_key(&cb));

        assert!(s[&aa].len()==0);
        assert!(s[&ba].len()==0);
        assert!(s[&ca].len()==0);

        assert!(s[&bb].len()==1, "bb len is {:?}", s[&bb].len());
        assert!(s[&cb].len()==1, "cb len is {:?}", s[&cb].len());
    }

    #[test]
    fn test_max_matching() {
        let mut set_x = Vec::new();
        set_x.push("Vancouver".to_string());
        let mut set_y = Vec::new();
        set_y.push("Toronto".to_string());
        let mut edge_set = Vec::new();
        let edge = ("Vancouver".to_string(), "Toronto".to_string());
        edge_set.push(&edge);

        let (cost, paths) = max_matching(&set_x, &set_y, &edge_set);
        assert!(cost==1, "cost is {:?}", cost);
    }

    #[test]
    fn test_shamir_small_graphs() {
        let graph_g = three_node_graph();
        let graph_h = two_node_graph();
        assert!(find_mapping_shamir(graph_g, graph_h));

    }

    // ---------------------- Hoffman Tests -------------------------

    /*
    #[test]
    fn test_precompute_hoffman_small_graph() {
        let graph = two_node_graph();
        let gs = algorithm_a_hoffman(&graph);
        assert!(gs.node_count()==2, "gs node count is {:?}", gs.node_count());
        assert!(gs.edge_count()==1, "gs edge count is {:?}", gs.edge_count());
        let table = algorithm_b_hoffman(&gs, &graph);
        assert!(table.contains_key("a"));
        assert!(table.contains_key("*"));
        assert!(table["a"].len()==2);
        assert!(table["*"].len()==1);
        assert!(table["a"].contains(&"a".to_string()));
        assert!(table["a"].contains(&"*".to_string()));
        assert!(table["*"].contains(&"*".to_string()));
    }
    */


    /*
    #[test]
    fn test_precompute_hoffman_chain_graph() {
        let graph = chain_graph();
        let gs = algorithm_a_hoffman(&graph);
        assert!(gs.node_count()==4);
        for node in gs.node_indices() {
            if gs.node_weight(node).unwrap() == "*" {
                assert!(gs.neighbors(node).count()==0);
            }
            if gs.node_weight(node).unwrap() == "c" {
                assert!(gs.neighbors(node).count()==1, "c neighbor num is {0}", gs.neighbors(node).count());
            }
            if gs.node_weight(node).unwrap() == "b" {
                assert!(gs.neighbors(node).count()==1, "b neighbor num is {0}", gs.neighbors(node).count());
            }
            if gs.node_weight(node).unwrap() == "a" {
                assert!(gs.neighbors(node).count()==1, "a neighbor num is {0}", gs.neighbors(node).count());
            }

        }
        let table = algorithm_b_hoffman(&gs, &graph);
        for key in table.keys() {
            print!("key: {:?}\n", key);
            print!("entry: {:?} ", table[key]);
            print!("\n");

        }
        assert!(table["a"].contains(&"a".to_string()));
        assert!(table["a"].contains(&"*".to_string()));
        assert!(table["a"].len()==2);

        assert!(table["b"].contains(&"b".to_string()));
        assert!(table["b"].contains(&"c".to_string()));

        assert!(table["c"].contains(&"c".to_string()));
        assert!(table["c"].contains(&"*".to_string()));

        assert!(table["*"].contains(&"*".to_string()));
    }

    #[test]
    fn test_compute_hoffman() {
        // TODO
        let graph_g = two_node_graph();
        let graph_h = three_node_graph();
        let table = precompute_hoffman(&graph_g);
        let maps = compute_hoffman(table, graph_g, graph_h);
        //assert!(maps.len()>0);

    }
    */

}
