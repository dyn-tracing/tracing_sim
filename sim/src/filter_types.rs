use std::collections::HashMap;
use rpc_lib::rpc::Rpc;

pub type CodeletType = fn(&Filter, &Rpc) -> Option<Rpc>;


// This represents a piece of state of the filter
// it either contains a user defined function, or some sort of
// other persistent state

extern "Rust"{
    pub type Filter;
}

pub type NewWithEnvoyProperties = fn(HashMap<String, String>) -> *mut Filter;
