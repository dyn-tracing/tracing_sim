use std::collections::HashMap;
use rpc_lib::rpc::Rpc;
use type_lib::filter_types::{Filter};

pub type CodeletType = fn(&Filter, &Rpc) -> Option<Rpc>;


// user defined functions:



// This represents a piece of state of the filter
// it either contains a user defined function, or some sort of
// other persistent state
pub struct State {
    pub type_of_state: Option<String>,
    pub string_data: Option<String>,

}

pub struct FilterImpl {
    pub filter_state: HashMap<String, State>,
}
