use crate::sim_element::SimElement;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{Graph, NodeIndex};
use rpc_lib::rpc::Rpc;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Display;
use std::fs;
use std::process::Command;
use crate::node::Node;

// Need to combine SimElement for simulation
// and Debug for printing.
// Uses this suggestion: https://stackoverflow.com/a/28898575
pub trait PrintableElement: SimElement + Display {}
impl<T: SimElement + Display> PrintableElement for T {}

#[derive(Default)]
pub struct Simulator {
    elements: Vec<Box<dyn PrintableElement>>,
    rpc_buffer: Vec<Vec<(Rpc, Option<u32>)>>,
    graph: Graph<usize, String>,
    node_index_to_node: HashMap<usize, NodeIndex>,
}

impl Simulator {
    pub fn new() -> Self {
        Simulator {
            ..Default::default()
        }
    }

    pub fn add_node(&mut self, element: Node) -> usize {
        let what_is_element = element.whoami();
        let node_index = self.graph.add_node(what_is_element.1 as usize);
        self.node_index_to_node
            .insert(self.elements.len(), node_index);
        self.add_element(element)
    }

    pub fn add_edge<T: 'static + PrintableElement>(
        &mut self,
        edge: T,
        element1: usize,
        element2: usize,
    ) -> usize {
        let edge_index = self.add_element(edge);
        self.add_connection(element1, edge_index);
        self.add_connection(edge_index, element1);
        self.add_connection(element2, edge_index);
        self.add_connection(edge_index, element2);
        let e1_node = self.node_index_to_node[&element1];
        let e2_node = self.node_index_to_node[&element2];
        self.graph.add_edge(e1_node, e2_node, "".to_string());
        return edge_index;
    }

    pub fn add_one_direction_edge<T: 'static + PrintableElement>(
        &mut self,
        edge: T,
        element1: usize,
        element2: usize,
    ) -> usize {
        self.add_connection(element1, element2);
        let e1_node = self.node_index_to_node[&element1];
        let e2_node = self.node_index_to_node[&element2];
        self.graph.add_edge(e1_node, e2_node, "".to_string());
        self.add_element(edge)
    }

    pub fn add_element<T: 'static + PrintableElement>(&mut self, element: T) -> usize {
        self.elements.push(Box::new(element));
        self.rpc_buffer.push(vec![]);
        return self.elements.len() - 1;
    }

    pub fn add_connection(&mut self, src: usize, dst: usize) {
        self.elements[src].add_connection(dst.try_into().unwrap());
    }

    pub fn print_graph(&mut self) {
        let dot_info = Dot::with_config(&self.graph, &[Config::EdgeNoLabel]).to_string();
        // print dot_info to a file
        let _ret = fs::write("graph.gv", dot_info);
        // render the dot file as a pdf dot -Tpdf graph.gv -o graph.pdf
        Command::new("dot")
            .arg("-Tpdf")
            .arg("graph.gv")
            .arg("-o")
            .arg("graph.pdf")
            .output()
            .expect("failed to execute process");
    }

    pub fn tick(&mut self, tick: u64) {
        // tick all elements to generate Rpcs
        for i in 0..self.elements.len() {
            self.rpc_buffer[i] = self.elements[i].tick(tick);
            println!(
                "After tick {:5}, {:45} outputs {:?}",
                tick, self.elements[i], self.rpc_buffer[i]
            );
        }

        // Send these elements to the next hops
        for src in 0..self.elements.len() {
            for (rpc, dst) in &self.rpc_buffer[src] {
                if (*dst).is_some() {
                    // Before we send this rpc on, we should update its path to include the most recently traversed node if applicable
                    // TODO: is cloning the best way to do this?
                    let mut new_rpc = rpc.clone();
                    if self.elements[src].whoami().0
                    {
                        new_rpc.add_to_path(&src.to_string());
                    }
                    self.elements[(*dst).unwrap() as usize].recv(new_rpc, tick, src as u32);
                }
            }
        }
        println!("");
    }
}
