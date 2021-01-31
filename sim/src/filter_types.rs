//! This file is used for the plugin wrapper to understand the types of the
//! functions and objects in the external library.

use rpc_lib::rpc::Rpc;
use std::collections::HashMap;

pub type CodeletType = fn(*mut Filter, &Rpc) -> Option<Rpc>;

// This represents a piece of state of the filter
// it either contains a user defined function, or some sort of
// other persistent state

extern "Rust" {
    pub type Filter;
}

pub type NewWithEnvoyProperties = fn(HashMap<String, String>) -> *mut Filter;
