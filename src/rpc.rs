#[derive(PartialEq, Clone, Debug)]
#[repr(C)]
pub struct Rpc {
   pub data    : u32,
   pub uid     : u64,
   pub path    : String,
}

impl Rpc {
    pub fn new_rpc(data : u32 ) -> Self {
        static mut COUNTER : u64 = 0;
        let ret = unsafe { Rpc { data : data, uid : COUNTER , path : String::new() } };
        unsafe { COUNTER = COUNTER + 1; }
        ret
    }
    pub fn add_to_path(&mut self, hop: &str) {
        self.path.push_str(" ");
        self.path.push_str(hop);
    }
}
