use std::collections::HashMap;
use crate::rpc;

pub type CodeletType = fn(&Filter, &rpc::Rpc) -> Option<rpc::Rpc>;

pub struct Filter {
    pub filter_state: HashMap<String, String>,
    pub execute_func: CodeletType,
}
