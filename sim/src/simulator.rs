//! This defines the simulator and coordinates all of the sim_elements.  It is a tick-based simulator, so at every tick,
//! each sim_element will produce some RPCs and where they should go, and receive any in its own buffer.

use crate::edge::Edge;
use crate::node::Node;
use crate::sim_element::SimElement;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{Graph, NodeIndex};
use rpc_lib::rpc::Rpc;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::process::Command;

// Need to combine SimElement for simulation
// and Debug for printing.
// Uses this suggestion: https://stackoverflow.com/a/28898575
pub trait PrintableElement: SimElement + Display {}
impl<T: SimElement + Display> PrintableElement for T {}

#[derive(Default)]
pub struct Simulator {
    elements: HashMap<String, Box<dyn PrintableElement>>,
    rpc_buffer: HashMap<String, Vec<(Rpc, Option<String>)>>,
    graph: Graph<&'static str, &'static str>,
    node_index_to_node: HashMap<&'static str, NodeIndex>,
}

impl Simulator {
    pub fn new() -> Self {
        Simulator {
            elements: HashMap::new(),
            rpc_buffer: HashMap::new(),
            graph: Graph::new(),
            node_index_to_node: HashMap::new(),
        }
    }

    pub fn add_node(
        &mut self,
        id: &'static str,
        capacity: u32,
        egress_rate: u32,
        generation_rate: u32,
        plugin: Option<&str>,
        plugin_id: Option<&'static str>,
    ) {
        let node = Node::new(id, capacity, egress_rate, generation_rate, plugin, plugin_id);
        self.add_element(id, node);
        self.node_index_to_node
            .insert(id, self.graph.add_node(id));
    }

    pub fn add_edge(
        &mut self,
        delay: u32,
        edge_name: &'static str,
        element1: &str,
        element2: &str,
        unidirectional: bool,
    ) {
        // 1. create the edge
        let edge = Edge::new(edge_name, delay.into());
        self.add_element(edge_name, edge);
        let e1_node = self.node_index_to_node[element1];
        let e2_node = self.node_index_to_node[element2];
        self.graph.add_edge(e1_node, e2_node, "");

        // 3. connect the edge to its nodes
        self.add_connection(element1, edge_name);
        self.add_connection(edge_name, element2);

        if !unidirectional {
            self.add_connection(edge_name, element1);
            self.add_connection(element2, edge_name);
        }
    }

    pub fn add_element<T: 'static + PrintableElement>(&mut self, id: &str, element: T) -> usize {
        self.elements.insert(id.to_string(), Box::new(element));
        self.rpc_buffer.insert(id.to_string(), vec![]);
        return self.elements.len() - 1;
    }

    pub fn add_connection(&mut self, src: &str, dst: &str) {
        self.elements.get_mut(src).unwrap().add_connection(dst.to_string());
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
        for (i, element) in self.elements.iter_mut() {
            let rpcs = element.tick(tick);
            self.rpc_buffer.insert(i.to_string(), rpcs);
            println!(
                "After tick {:5}, {:45} \n\toutputs {:?}\n",
                tick, element, self.rpc_buffer[i]
            );
        }
        print!("\n\n");

        // Send these elements to the next hops

        // We have to make this hashmap because if we don't, then we're iterating over and modifying the same hashmap self.elements
        // and Rust, understandably, does not like that at all for memory reasons
        let mut indices_to_sim_el = HashMap::new();
        let mut j = 0;
        for i in self.elements.keys() {
            indices_to_sim_el.insert(j, i.clone());
            j = j + 1;
        }
        for i in 0..self.elements.keys().count() {
            let src = &indices_to_sim_el[&i];
            for (rpc, dst) in &self.rpc_buffer[src] {
                if dst.is_some() {
                    // Before we send this rpc on, we should update its path to include the most recently traversed node if applicable
                    // TODO: is cloning the best way to do this?
                    let mut new_rpc = rpc.clone();
                    if self.elements[src].whoami().0 {
                        new_rpc.add_to_path(&src.to_string());
                    }

                    self.elements
                        .get_mut(dst.as_ref().clone().unwrap())
                        .unwrap()
                        .recv(new_rpc, tick, src.to_string());
                }
            }
        }
        println!("");
    }
}
