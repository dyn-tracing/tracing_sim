#![feature(test)]

mod channel;
mod plugin_wrapper;
mod rpc;
mod codelet;

use channel::Channel;
use plugin_wrapper::PluginWrapper;
use rpc::Rpc;
use rand::{StdRng, Rng, SeedableRng};

static LIBRARY : &str = "target/debug/libplugin_sample.dylib";
static FUNCTION: &str = "codelet";

fn main() {
    // Create a random number generator.
    let mut rng : StdRng = StdRng::from_seed(&[1, 2, 3, 4]);

    // Create plugins and a single channel for each plugin for all outgoing RPCs
    // regardless of destintion. This doesn't yet model capacity limits.
    let mut plugins  = vec![];
    let mut channels = vec![];
    for plugin_id in 0..10 {
        plugins.push(PluginWrapper::new(LIBRARY, FUNCTION, plugin_id));
        channels.push(Channel::new(0, 0, 10));
    }

    // Keep a vector of RPCs to be processed for each plugin.
    let mut rpcs_per_plugin : Vec<Option<Rpc>> = vec![];
    for plugin_id in 0..10 {
        rpcs_per_plugin.push(Some(Rpc::new(plugin_id)));
    }

    // Now execute all the plugins.
    for tick in 0..1000 {
        for plugin_id  in 0..10 {
            if !rpcs_per_plugin[plugin_id].is_none() {
                println!("Input RPC for plugin {}: {:?} at {}",
                         plugin_id, rpcs_per_plugin[plugin_id].as_ref().unwrap(), tick);
                let transformed_rpc = plugins[plugin_id].execute(rpcs_per_plugin[plugin_id].as_ref().unwrap());
                rpcs_per_plugin[plugin_id] = None;

                // put in channel, destination TBD later
                channels[plugin_id].enqueue(transformed_rpc, tick);
            }
            // dequeue if it's time
            let deq_rpc = channels[plugin_id].dequeue(tick);
            if deq_rpc.is_some() {
                let next_destination = rng.gen_range(0, 10);
                if !rpcs_per_plugin[next_destination].is_none() {
                    println!("Overwriting rpcs_per_plugin at {}", next_destination);
                }
                rpcs_per_plugin[next_destination] = deq_rpc;
            }
        }
    }
}
