#[derive(PartialEq, Clone, Debug)]
#[repr(C)]
pub struct Rpc {
   pub id      : u32,
   pub uid     : u64,
}

impl Rpc {
    pub fn get_id(&self) -> u32 { self.id }
    pub fn new(id : u32) -> Self {
        static mut COUNTER : u64 = 0;
        unsafe { COUNTER = COUNTER + 1; }
        unsafe { Rpc { id : id, uid : COUNTER } }
    }
}
