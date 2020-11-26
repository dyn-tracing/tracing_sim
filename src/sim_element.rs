use crate::rpc::Rpc;

pub trait SimElement {
    fn tick(&mut self, tick : u64) -> Option<Rpc>;

    fn recv(&mut self, rpc : Rpc, tick : u64);
}
