use crate::rpc;
pub type CodeletType = fn(&rpc::Rpc) -> Option<rpc::Rpc>;
