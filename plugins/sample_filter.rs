mod rpc;
mod filter;
use filter::{Filter, CodeletType};
use std::collections::HashMap;

#[no_mangle]
pub fn new() -> Filter {
    Filter { 
        filter_state: HashMap::new(),
        execute_func: codelet
    }
}

#[no_mangle]
pub fn execute(filter: &mut Filter, my_rpc: &rpc::Rpc) -> Option<rpc::Rpc> {
    (filter.execute_func)(filter, my_rpc)
}

#[no_mangle]
pub fn codelet (_filter : &Filter, x : &rpc::Rpc) -> Option<rpc::Rpc> { 
Some(rpc::Rpc{ data : x.data + 5, uid : x.uid , path: x.path.clone()}) 
}

#[allow(dead_code)]
static CODELET_FUNCTION : CodeletType = codelet;
