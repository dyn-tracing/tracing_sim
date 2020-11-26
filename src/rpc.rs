#[derive(PartialEq, Clone, Debug, Default, Copy)]
#[repr(C)]
pub struct Rpc {
   pub id      : u32,
   pub uid     : u64,
}

impl Rpc {
    pub fn new(id : u32) -> Self {
        static mut COUNTER : u64 = 0;
        let ret = unsafe { Rpc { id : id, uid : COUNTER } };
        unsafe { COUNTER = COUNTER + 1; }
        ret
    }
}
