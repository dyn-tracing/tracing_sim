mod rpc;
use std::collections::HashMap;

pub type CodeletType = fn(&Filter, &rpc::Rpc) -> Option<rpc::Rpc>;

pub struct Filter {
    pub filter_state: HashMap<String, String>,
}

impl Filter {
    #[no_mangle]
    pub fn new() -> Filter {
        Filter { 
	    filter_state: HashMap::new(),
	}
    }

    #[no_mangle]
    pub fn execute(&mut self, x: &rpc::Rpc) -> Option<rpc::Rpc> {
        // NOTE: This is where the compiler-generated code goes
        Some(rpc::Rpc{ data : x.data + 5, uid : x.uid , path: x.path.clone()}) 
    }

}
