use rpc_lib::rpc::Rpc;

pub trait SimElement {
    fn add_connection(&mut self, neighbor: u32);

    fn tick(&mut self, tick: u64) -> Vec<(Rpc, Option<u32>)>;

    fn recv(&mut self, rpc: Rpc, tick: u64, sender: u32);

    // This returns the following information about a simulator element
    // 1. whether it should be included in the path
    // 2. what its ID is
    // 3. who its neighbors are
    fn whoami(&self) -> (bool, u32, Vec<u32>);
}

pub trait Node {}
