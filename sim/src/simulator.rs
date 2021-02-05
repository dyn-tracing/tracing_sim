//! This defines the simulator and coordinates all of the sim_elements.  It is a tick-based simulator, so at every tick,
//! each sim_element will produce some RPCs and where they should go, and receive any in its own buffer.

use crate::edge::Edge;
use crate::node::Node;
use crate::sim_element::SimElement;
use crate::storage::Storage;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{Graph, NodeIndex};
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
    graph: Graph<String, String>,
    node_index_to_node: HashMap<String, NodeIndex>,
    seed: u64,
}

impl Simulator {
    pub fn new(seed: u64) -> Self {
        Simulator {
            elements: HashMap::new(),
            graph: Graph::new(),
            node_index_to_node: HashMap::new(),
            seed,
        }
    }

    pub fn query_storage(&mut self, storage_id: &str) -> &str {
        self.elements[storage_id].type_specific_info().unwrap()
    }

    pub fn add_node(
        &mut self,
        id: &str,
        capacity: u32,
        egress_rate: u32,
        generation_rate: u32,
        plugin: Option<&str>,
    ) {
        let node = Node::new(
            id,
            capacity,
            egress_rate,
            generation_rate,
            plugin,
            self.seed,
        );
        self.add_element(id, node);
        self.node_index_to_node
            .insert(id.to_string(), self.graph.add_node(id.to_string()));
    }

    pub fn add_edge(&mut self, delay: u32, element1: &str, element2: &str, unidirectional: bool) {
        if !self.elements.contains_key(element1) {
            panic!("Tried to add an edge using {0};  that is not a valid node", element1);
        }
        if !self.elements.contains_key(element2) {
            panic!("Tried to add an edge using {0};  that is not a valid node", element2);
        }
        // 1. create the id, which will be the two nodes' ids put together with a _
        let id = element1.to_string() + "_" + element2;

        // 2. create the edge
        let edge = Edge::new(&id, delay.into());
        self.add_element(&id, edge);
        let e1_node = self.node_index_to_node[element1];
        let e2_node = self.node_index_to_node[element2];
        self.graph.add_edge(e1_node, e2_node, "".to_string());

        // 3. connect the edge to its nodes
        self.add_connection(element1, &id);
        self.add_connection(&id, element2);

        if !unidirectional {
            self.add_connection(&id, element1);
            self.add_connection(element2, &id);
        }
    }

    pub fn add_storage(&mut self, id: &str) {
        let storage = Storage::new(id);
        self.add_element(id, storage);
        self.node_index_to_node
            .insert(id.to_string(), self.graph.add_node(id.to_string()));
    }

    pub fn add_element<T: 'static + PrintableElement>(&mut self, id: &str, element: T) -> usize {
        self.elements.insert(id.to_string(), Box::new(element));
        return self.elements.len() - 1;
    }

    pub fn add_connection(&mut self, src: &str, dst: &str) {
        self.elements
            .get_mut(src)
            .unwrap()
            .add_connection(dst.to_string());
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
        let mut rpc_buffer = HashMap::new();
        // tick all elements to generate Rpcs
        // this is the send phase. collect all the rpcs
        for (elem_name, element_obj) in self.elements.iter_mut() {
            let rpcs = element_obj.tick(tick);
            let mut input_rpcs = vec![];
            for (rpc, dst) in rpcs {
                input_rpcs.push((rpc, dst));
            }
            rpc_buffer.insert(elem_name.clone(), input_rpcs);
        }
        print!("\n\n");

        // now start the receive phase
        for (elem_name, rpc_tuples) in rpc_buffer {
            for (rpc, dst) in rpc_tuples {
                let elem = self.elements.get_mut(&dst).unwrap();
                elem.recv(rpc, tick, &elem_name);
            }
        }

        // Send these elements to the next hops

    }
}
