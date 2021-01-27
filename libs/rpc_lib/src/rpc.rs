use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
#[repr(C)]
pub struct Rpc {
    pub data: u32,                        // application data
    pub uid: u64,                         // number of hops the message has taken
    pub path: String,                     // the path that the request has taken thus far
    pub headers: HashMap<String, String>, // the "http" headers of the rpc, ie, filter-defined book keeping
}

impl Rpc {
    pub fn new_rpc(data: u32) -> Self {
        static mut COUNTER: u64 = 0;
        let ret = unsafe {
            Rpc {
                data,
                uid: COUNTER,
                path: String::new(),
                headers: HashMap::new(),
            }
        };
        unsafe {
            COUNTER += 1;
        }
        ret
    }
    pub fn add_to_path(&mut self, hop: &str) {
        self.path.push(' ');
        self.path.push_str(hop);
    }
}
