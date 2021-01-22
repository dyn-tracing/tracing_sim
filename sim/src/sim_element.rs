use rpc_lib::rpc::Rpc;

pub trait SimElement {
    fn add_connection(&mut self, neighbor: u32);

    fn tick(&mut self, tick: u64) -> Vec<(Rpc, Option<u32>)>;

    fn recv(&mut self, rpc: Rpc, tick: u64, sender: u32);

    fn whoami(&self) -> (&str, u32, Vec<u32>);
}
