mod rpc;
mod codelet;
use codelet::CodeletType;

#[no_mangle]
pub fn codelet (x : &rpc::Rpc) -> Option<rpc::Rpc> { None }

#[allow(dead_code)]
static CODELET_FUNCTION : CodeletType = codelet;
