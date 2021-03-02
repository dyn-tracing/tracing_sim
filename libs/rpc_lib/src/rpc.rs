use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
#[repr(C)]
pub struct Rpc {
    pub data: String,                     // application data
    pub uid: u64,                         // number of hops the message has taken
    pub headers: HashMap<String, String>, // the "http" headers of the rpc, ie, filter-defined book keeping
}

impl Rpc {
    pub fn new_rpc(data: &str) -> Self {
        static mut COUNTER: u64 = 0;
        let ret = unsafe {
            Rpc {
                data: data.to_string(),
                uid: COUNTER,
                headers: HashMap::new(),
            }
        };
        unsafe {
            COUNTER += 1;
        }
        ret
    }

    pub fn new_with_src(data: &str, src: &str) -> Self {
        let mut rpc = Rpc::new_rpc(data);
        rpc.headers.insert("src".to_string(), src.to_string());
        rpc
    }

    pub fn new_with_src_dest(data: &str, src: &str, dst: &str) -> Self {
        let mut rpc = Rpc::new_rpc(data);
        rpc.headers.insert("src".to_string(), src.to_string());
        rpc.headers.insert("dst".to_string(), dst.to_string());
        rpc
    }
}
