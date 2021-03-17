//! This file is used for the plugin wrapper to understand the types of the
//! functions and objects in the external library.

use indexmap::map::IndexMap;
use rpc_lib::rpc::Rpc;

pub type CodeletType = fn(*mut Filter, &Rpc) -> Vec<Rpc>;

// This represents a piece of state of the filter
// it either contains a user defined function, or some sort of
// other persistent state

extern "Rust" {
    pub type Filter;
}

pub type NewWithEnvoyProperties = fn(IndexMap<String, String>) -> *mut Filter;
