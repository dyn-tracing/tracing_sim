pub fn codelet (_filter : &Filter, x : &rpc::Rpc) -> Option<rpc::Rpc> { 
Some(rpc::Rpc{ data : x.data + 5, uid : x.uid , path: x.path.clone()}) 
}
