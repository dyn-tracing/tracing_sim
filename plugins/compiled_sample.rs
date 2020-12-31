mod rpc;
mod codelet;
use codelet::CodeletType;

// udf_type: Scalar
// id: count
// return_type: int

fn count(counter: u32) -> u32 {
    counter + 1
}


#[no_mangle]
pub fn codelet (x : &rpc::Rpc) -> Option<rpc::Rpc> {
    // 1. Regardless of whether or not this is the root node, we need to find the 
    //    find the node attributes that need to be collected
     



    
    // TODO:  Make a subgraph, check isomorphism, and do return calls based on that info
    if x.uid == 0 {
        // we need to create the graph given by the query
        let vertices = vec![ "n", "m",   ];
        let edges = vec![  ( "n", "m",  ),  ];
    }
    Some(rpc::Rpc{ 
        data: count(x.data), uid: x.uid 
         }   ) 
}

// While the code fragment below does nothing useful at run time,
// it forces rustc to check that codelet_function is of type CodeletType.
// This avoids inscrutable run-time errors and turns them into compiler errors.
#[allow(dead_code)]
static CODELET_FUNCTION : CodeletType = codelet;
