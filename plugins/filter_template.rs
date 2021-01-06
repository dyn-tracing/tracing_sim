mod rpc;
use std::collections::HashMap;

pub type CodeletType = fn(&Filter, &rpc::Rpc) -> Option<rpc::Rpc>;

pub struct Filter {
    filter_state: HashMap<String, String>,
    execute_func: CodeletType,
}

impl Filter {
    fn new() -> Filter {
        Filter { 
            filter_state: HashMap::new(),
            execute_func: codelet
        }
    }

    fn execute(&mut self, my_rpc: &rpc::Rpc) -> Option<rpc::Rpc> {
        (self.execute_func)(self, my_rpc)
    }
}

#[no_mangle]
pub fn codelet (_filter : &Filter, x : &rpc::Rpc) -> Option<rpc::Rpc> { 
Some(rpc::Rpc{ data : x.data + 5, uid : x.uid , path: x.path.clone()}) 
}

#[allow(dead_code)]
static CODELET_FUNCTION : CodeletType = codelet;
