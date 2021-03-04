//! This defines the simulator and coordinates all of the sim_elements.  It is a tick-based simulator, so at every tick,
//! each sim_element will produce some RPCs and where they should go, and receive any in its own buffer.

use crate::edge::Edge;
use crate::node::Node;
use crate::sim_element::SimElement;
use crate::storage::Storage;
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
    graph: Graph<String, String>,
    node_index_to_node: HashMap<String, NodeIndex>,
    edge_matrix: HashMap<(String, String), Edge>,
    seed: u64,
}

impl<'a> Simulator {
    pub fn new(seed: u64) -> Self {
        Simulator {
            elements: HashMap::new(),
            graph: Graph::new(),
            node_index_to_node: HashMap::new(),
            edge_matrix: HashMap::new(),
            seed,
        }
    }

    pub fn query_storage(&mut self, storage_id: &str) -> &str {
        let storage_box = &self.elements[storage_id];
        return match storage_box.as_any().downcast_ref::<Storage>() {
            Some(storage) => storage.query(),
            None => panic!("Expected storage element but got {0}", storage_box),
        };
    }

    pub fn add_node<T: 'static + PrintableElement>(&mut self, id: &str, node: T) {
        self.add_element(id, node);
        self.node_index_to_node
            .insert(id.to_string(), self.graph.add_node(id.to_string()));
    }

    pub fn add_random_node(
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
        self.add_node(id, node);
    }

    pub fn insert_rpc(&mut self, target: &str, rpc: Rpc) {
        let node = self.elements.get_mut(target).unwrap();
        node.recv(rpc, 0);
    }

    fn add_to_edge_matrix(&mut self, left: &str, right: &str, edge: Edge) {
        self.edge_matrix
            .insert((left.to_string(), right.to_string()), edge);
    }
    fn get_from_edge_matrix(&mut self, left: String, right: String) -> &mut Edge {
        let key_tuple = &(left.clone(), right.clone());
        if self.edge_matrix.contains_key(&key_tuple) {
            return self.edge_matrix.get_mut(&key_tuple).unwrap();
        } else {
            panic!("Edge connecting {:?} and {:?} not found", left, right);
        }
    }

    pub fn add_edge(&mut self, delay: u32, left: &str, right: &str, bidirectional: bool) {
        if !self.elements.contains_key(left) {
            panic!(
                "Tried to add an edge using {:?};  that is not a valid node",
                left
            );
        }
        if !self.elements.contains_key(right) {
            panic!(
                "Tried to add an edge using {:?};  that is not a valid node",
                right
            );
        }
        // 1. create the id, which will be the two nodes' ids put together with a _

        // 2. create the edge
        let edge = Edge::new(left.to_string(), right.to_string(), delay.into());
        let e1_node = self.node_index_to_node[left];
        let e2_node = self.node_index_to_node[right];
        self.graph.add_edge(e1_node, e2_node, "".to_string());
        // 3. add a connection between nodes
        self.add_connection(left, right);
        self.add_to_edge_matrix(left, right, edge);
        if bidirectional {
            // 2. create the edge
            let ret_edge = Edge::new(left.to_string(), right.to_string(), delay.into());
            let e1_node = self.node_index_to_node[left];
            let e2_node = self.node_index_to_node[right];
            self.graph.add_edge(e1_node, e2_node, "".to_string());
            // 3. add a connection between nodes
            self.add_connection(right, left);
            self.add_to_edge_matrix(right, left, ret_edge);
        }
    }

    pub fn add_storage(&mut self, id: &str) {
        let storage = Storage::new(id);
        self.add_element(id, storage);
        self.node_index_to_node
            .insert(id.to_string(), self.graph.add_node(id.to_string()));
    }

    fn add_element<T: 'static + PrintableElement>(&mut self, id: &str, element: T) -> usize {
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
         log::info!("################# TICK {0} START #################", tick);
        let mut rpc_buffer = vec![];
        // tick all elements to generate Rpcs
        // this is the send phase. collect all the rpcs
        for (_elem_name, element_obj) in self.elements.iter_mut() {
            let rpcs = element_obj.tick(tick);
            for rpc in &rpcs {
                rpc_buffer.push(rpc.clone());
            }
             log::info!("{:45}", element_obj);
             log::info!("\toutputs {:?}", rpcs);
        }

        for rpc in rpc_buffer {
            let edge = self.get_from_edge_matrix(
                rpc.headers["src"].to_string(),
                rpc.headers["dest"].to_string(),
            );
            edge.recv(rpc, tick);
        }
        let mut edge_buffer = vec![];
        for (_, edge) in self.edge_matrix.iter_mut() {
            edge_buffer.extend(edge.tick(tick));
        }
        // now start the receive phase
        for rpc in edge_buffer {
            let dst = &rpc.headers["dest"];
            match self.elements.get_mut(dst) {
                Some(elem) => elem.recv(rpc, tick),
                None => panic!("expected {0} to be in elements, but it was not", dst),
            }
        }
         log::info!("################# TICK {0} END #################", tick);
    }
}
