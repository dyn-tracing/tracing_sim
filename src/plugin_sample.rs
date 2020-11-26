mod rpc;
mod codelet;
use codelet::CodeletType;

#[no_mangle]
pub fn codelet (x : &rpc::Rpc) -> rpc::Rpc { rpc::Rpc{ id : x.id + 5, uid : x.uid } }

// While the code fragment below does nothing useful at run time,
// it forces rustc to check that codelet_function is of type CodeletType.
// This avoids inscrutable run-time errors and turns them into compiler errors.
#[allow(dead_code)]
static CODELET_FUNCTION : CodeletType = codelet;
