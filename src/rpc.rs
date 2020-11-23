use uuid::Uuid;

#[derive(PartialEq, Clone, Debug)]
#[repr(C)]
pub struct Rpc {
   pub id      : u32,
   pub uuid    : Uuid,
}

impl Rpc {
    pub fn get_id(&self) -> u32 { self.id }
    pub fn new(id : u32) -> Self { Rpc { id : id, uuid : Uuid::new_v4() } }
}
