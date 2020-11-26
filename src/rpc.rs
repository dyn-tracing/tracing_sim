#[derive(PartialEq, Clone, Debug, Copy)]
#[repr(C)]
pub struct Rpc {
   pub data    : u32,
   pub uid     : u64,
}

impl Rpc {
    pub fn new_rpc(data : u32) -> Self {
        static mut COUNTER : u64 = 0;
        let ret = unsafe { Rpc { data : data, uid : COUNTER } };
        unsafe { COUNTER = COUNTER + 1; }
        ret
    }
}
