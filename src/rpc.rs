#[derive(PartialEq)]
#[repr(C)]
pub struct Rpc {
   pub id      : u32,
}

impl Rpc {
    pub fn get_id(self) -> u32 { self.id }
}
