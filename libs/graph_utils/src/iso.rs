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
use std::collections::HashSet;
use std::collections::HashMap;

fn find_leaves(node: NodeIndex, graph: &Graph<(),()>) -> Vec<NodeIndex> {
    let mut post_order = DfsPostOrder::new(&graph, node);
    let mut to_return = Vec::new();
    while let Some(visited) = post_order.next(&graph) {
        let neighbors : Vec<NodeIndex> = graph.neighbors(visited).collect();
        if neighbors.len() == 0 { to_return.push(visited); }
    }
    return to_return;
}

fn find_root(graph: &Graph<(), ()>) -> NodeIndex {
    for node in graph.node_indices() {
        let neighbors : Vec<NodeIndex> = graph.neighbors_directed(node, Incoming).collect();
        if neighbors.len() == 0 { return node; }
    }
    panic!("no root found");
}

// this performs lines 0-4 in the Shamir paper figure 3
fn initialize_s(graph_g: &Graph<(), ()>, graph_h: &Graph<(), ()>) -> HashMap::<(NodeIndex, NodeIndex), HashSet<NodeIndex>> {
    let mut S = HashMap::<(NodeIndex, NodeIndex), HashSet<NodeIndex>>::new();
    for node_g in graph_g.node_indices() {
        for node_h in graph_h.node_indices() {
            // initialize S entry as empty set
            S.insert((node_g, node_h), HashSet::new());
        }
    }
    let root_g = find_root(&graph_g);
    let root_h = find_root(&graph_h);
    for leaf_g in find_leaves(root_g, &graph_g) {
        for leaf_h in find_leaves(root_h, &graph_h) {
            for neighbor in graph_h.neighbors_directed(leaf_h, Incoming) {
                S.get_mut(&(leaf_g, leaf_h)).unwrap().insert(neighbor);
            }
        }
    }
    return S;
}


fn construct_bipartite_graph(set_x: Vec<NodeIndex>, set_y: Vec<NodeIndex>, edge_set: Vec<(&NodeIndex, &NodeIndex)>) -> Graph<usize, ()> {
    let mut graph = Graph::<usize,()>::default();
    return graph;
}

fn maximum_matching_size(set_x: &Vec<NodeIndex>, set_y: &Vec<NodeIndex>) -> u32 {
    return 0;
}

fn find_mapping(graph_g: Graph<(), ()>, graph_h: Graph<(), ()>) -> bool {
    // initialize S with all N(u) sets
    let mut S = initialize_s(&graph_g, &graph_h);
    let root_g = find_root(&graph_g); 
    let mut post_order = DfsPostOrder::new(&graph_g, root_g);
    while let Some(node) = post_order.next(&graph_g) {
        let v_children : Vec<NodeIndex> = graph_g.neighbors(node).collect();
        let v_children_len = v_children.len();
        for node_h in graph_h.node_indices() {
	    let u_neighbors : Vec<NodeIndex> = graph_h.neighbors(node_h).collect();
            if u_neighbors.len() < v_children_len+1 {
                let mut edge_set = Vec::new();
                for u in &u_neighbors {
                    for v in &v_children {
                        if S[&(*u,*v)].contains(&node_h) { edge_set.push((u,v)); }
                    }
                }
                let bipartite = construct_bipartite_graph(u_neighbors.clone(), v_children.clone(), edge_set);
                for i in 0..u_neighbors.len() {
                    let mut x_i = u_neighbors.clone();
                    if i != 0 { x_i.remove(i); }
                    let maximum_matching = maximum_matching_size(&x_i, &v_children);
                    if maximum_matching == x_i.len() as u32 {
                        S.get_mut(&(node, node_h)).unwrap().insert(u_neighbors[i]);
                    }
                    if S[&(node, node_h)].contains(&node_h) { return true; }
                }
            }
        }
    }
    return false;

}


#[cfg(test)]
mod tests {
    use super::*;


    fn little_branching_graph() -> Graph<(),()> {
        let mut graph = Graph::<(),()>::default();
        graph.extend_with_edges(&[
            (0, 1), (0, 2), (0, 3), (1, 4), (3, 5)
        ]);
        return graph;
    }

    // from figure 2 in shamir paper
    fn g_figure_2() -> Graph<(), ()> {
        let mut graph = Graph::<(), ()>::new();
        let r = graph.add_node();
        let v = graph.add_node();
        let v1 = graph.add_node();
        let v2 = graph.add_node();
        let v3 = graph.add_node();
        let left_unnamed_child = graph.add_node();
        let right_unnamed_child = graph.add_node();
        graph.add_edge(r, v, ());
        graph.add_edge(v, v1, ());
        graph.add_edge(v, v2, ());
        graph.add_edge(v, v3, ());
        graph.add_edge(v1, left_unnamed_child, ());
        graph.add_edge(v1, right_unnamed_child, ());
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
}
