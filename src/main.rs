#![feature(test)]

mod channel;
mod plugin_wrapper;
mod rpc;
mod codelet;

use channel::Channel;
use plugin_wrapper::PluginWrapper;
use rpc::Rpc;

static LIBRARY : &str = "target/debug/libplugin_sample.dylib";
static FUNCTION: &str = "codelet";

fn main() {
    let plugins  = vec![PluginWrapper::new(LIBRARY, FUNCTION, 0),
                        PluginWrapper::new(LIBRARY, FUNCTION, 1)];

    let mut channels = vec![Channel::new(0, 1, 10)];

    for tick in 0..100 {
        let input_rpc = Rpc::new(tick as u32);
        println!("Generated RPC: {:?} at {}", input_rpc, tick);
        let transformed_rpc = plugins[0].execute(input_rpc);
        channels[0].enqueue(transformed_rpc, tick);
        let deq_rpc = channels[0].dequeue(tick);
        if deq_rpc.is_some() {
            println!("Final RPC: {:?} at {}", plugins[1].execute(deq_rpc.unwrap()), tick);
        }
    }
}
