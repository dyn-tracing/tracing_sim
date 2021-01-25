use crate::graph_utils::{generate_target_graph, generate_trace_graph_from_headers};
use petgraph::algo::isomorphic_subgraph_mapping;
use rpc_lib::rpc::Rpc;
use std::collections::HashMap;
use std::fs;

pub type CodeletType = fn(&Filter, &Rpc) -> Option<Rpc>;

// user defined functions:
// init_func: new
// exec_func: execute
// struct_name: Count
// id: count

#[derive(Clone, Copy, Debug)]
pub struct Count {
    counter: u32,
}

impl Count {
    fn new() -> Count {
        Count { counter: 0 }
    }
    fn execute(&mut self) -> u32 {
        self.counter = self.counter + 1;
        self.counter
    }
}

// This represents a piece of state of the filter
// it either contains a user defined function, or some sort of
// other persistent state
#[derive(Clone, Debug)]
pub struct State {
    pub type_of_state: Option<String>,
    pub string_data: Option<String>,
    pub udf_count: Option<Count>,
}

impl State {
    pub fn new() -> State {
        State {
            type_of_state: None,
            string_data: None,
            udf_count: None,
        }
    }

    pub fn new_with_str(str_data: String) -> State {
        State {
            type_of_state: Some(String::from("String")),
            string_data: Some(str_data),
            udf_count: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Filter {
    pub filter_state: HashMap<String, State>,
}

impl Filter {
    #[no_mangle]
    pub fn new() -> *mut Filter {
         Box::into_raw(Box::new(Filter {
            filter_state: HashMap::new(),
        }))
    }

    #[no_mangle]
    pub fn new_with_envoy_properties(string_data: HashMap<String, String>) -> *mut Filter {
        let mut hash = HashMap::new();
        for key in string_data.keys() {
            hash.insert(key.clone(), State::new_with_str(string_data[key].clone()));
        }
        Box::into_raw(Box::new(Filter { filter_state: hash }))
    }

    #[no_mangle]
    pub fn execute(&mut self, x: &Rpc) -> Option<Rpc> {
        // 0. Who am I?
        let my_node = self
            .filter_state
            .get("WORKLOAD_NAME")
            .unwrap()
            .string_data
            .clone()
            .unwrap();

        // 1. Do I need to put any udf variables/objects in?

        if !self.filter_state.contains_key(&String::from("count")) {
            let mut new_state = State::new();
            new_state.type_of_state = Some(String::from("count"));
            new_state.udf_count = Some(Count::new());
            self.filter_state.insert(String::from("count"), new_state);
        }

        // 2. TODO: Find the node attributes to be collected

        // 3.  Make a subgraph representing the query, check isomorphism compared to the
        //     observed trace, and do return calls based on that info
        if my_node == String::from("0") {
            // we need to create the graph given by the query
            let vertices = vec![String::from("n"), String::from("m")];
            let edges = vec![(String::from("n"), String::from("m"))];
            let mut ids_to_properties: HashMap<String, Vec<String>> = HashMap::new();

            ids_to_properties.insert(
                String::from("a"),
                vec![
                    String::from("node"),
                    String::from("metadata"),
                    String::from("WORKLOAD_NAME"),
                ],
            );

            let target_graph = generate_target_graph(vertices, edges, ids_to_properties);
            let trace_graph = generate_trace_graph_from_headers(x.path.clone());
            let mapping = isomorphic_subgraph_mapping(&trace_graph, &target_graph);
            if !mapping.is_none() {
                // In the non-simulator version, we will send the result to storage.  Given this is
                // a simulation, we will write it to a file.

                let state_ptr = self.filter_state.get_mut("count").unwrap();
                let count_ptr = state_ptr.udf_count.as_mut().unwrap();
                let value = count_ptr.execute().to_string();
                fs::write("result.txt", value).expect("Unable to write file");
            }
        }
        let state_ptr = self.filter_state.get_mut("count").unwrap();
        let count_ptr = state_ptr.udf_count.as_mut().unwrap();
        count_ptr.execute();

        // 4.  Pass the rpc on
        Some(Rpc {
            data: x.data,
            uid: x.uid,
            path: x.path.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_is_persistent() {
        let mut envoy_prop = HashMap::new();
        envoy_prop.insert("WORKLOAD_NAME".to_string(), "HI".to_string());
        let my_filter = unsafe {
            &mut *Filter::new_with_envoy_properties(envoy_prop)
        };

        let incoming_rpc = Rpc {
            data: 1,
            uid: 1,
            path: String::from("ok"),
        };
        my_filter.execute(&incoming_rpc);
        assert!(my_filter.filter_state.len() == 2);

        let state_ptr = my_filter.filter_state.get_mut("count").unwrap();
        let count_ptr = state_ptr.udf_count.as_mut().unwrap();
        count_ptr.execute();
        count_ptr.execute();
        count_ptr.execute();
        let udf_counter = my_filter.filter_state
            .get("count")
            .unwrap()
            .udf_count
            .unwrap()
            .counter;
        assert!(udf_counter == 4, "Counter {} was not 4", udf_counter);
    }

    #[test]
    fn test_count_is_persistent_hashmap_level() {
        let mut map = HashMap::new();
        let mut state = State::new();
        state.udf_count = Some(Count::new());
        map.insert("hi".to_string(), state);
        assert!(map.len() == 1);
        let state_ptr = map.get_mut("hi").unwrap();
        let count_ptr = state_ptr.udf_count.as_mut().unwrap();
        count_ptr.execute();
        count_ptr.execute();
        let udf_counter = map.get("hi").unwrap().udf_count.unwrap().counter;
        assert!(udf_counter == 2, "Counter {} was not 2", udf_counter);
    }
}
