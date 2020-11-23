use crate::rpc;
pub type CodeletType = fn(rpc::Rpc) -> rpc::Rpc;
