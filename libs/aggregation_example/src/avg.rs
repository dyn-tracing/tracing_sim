use indexmap::map::IndexMap;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::Incoming;
use rpc_lib::rpc::Rpc;
use serde::{Deserialize, Serialize};
use utils::graph::graph_utils;
use utils::graph::iso::find_mapping_shamir_centralized;
use utils::graph::serde::FerriedData;
use utils::graph::serde::Property;
use utils::misc::headers::*;
extern crate serde_yaml;

pub type CodeletType = fn(&Filter, &Rpc) -> Option<Rpc>;

fn log_setup() {
    // Build a stderr logger.
    let stderr = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{h({l})}: {m}\n")))
        .target(Target::Stderr)
        .build();
    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{l}: {m}\n")))
        .append(false)
        .build("sim.log")
        .unwrap();
    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Info)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(log::LevelFilter::Trace),
        )
        .unwrap();
    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config);
}

fn put_ferried_data_in_hdrs(fd: &mut FerriedData, hdr: &mut IndexMap<String, String>) {
    match serde_yaml::to_string(fd) {
        Ok(stored_data_string) => {
            hdr.insert("ferried_data".to_string(), stored_data_string);
        }
        Err(e) => {
            log::error!(
                "ERROR:  could not translate stored data to json string: {0}\n",
                e
            );
        }
    }
}

// user defined functions:

// udf_type: Aggregation
// init_func: new
// exec_func: execute
// return_func: get_avg
// struct_name: Avg
// id: avg

#[derive(Clone, Copy, Debug)]
pub struct Avg {
    avg: u64,
    total: u64,
    num_instances: u64,
}

impl Avg {
    fn new() -> Avg {
        Avg {
            avg: 0,
            total: 0,
            num_instances: 0,
        }
    }
    fn execute(&mut self, _trace_id: u64, instance: String) {
        self.total += instance.parse::<u64>().unwrap();
        self.num_instances += 1;
    }
    fn get_avg(&mut self) -> String {
        self.avg = self.total / self.num_instances;
        self.avg.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct Filter {
    avg: Avg,

    pub data: Vec<String>,
}

impl Filter {
    #[no_mangle]
    pub fn new() -> *mut Filter {
        log_setup();
        Box::into_raw(Box::new(Filter {
            avg: Avg::new(),

            data: Vec::new(),
        }))
    }

    #[no_mangle]
    pub fn new_with_envoy_properties(string_data: IndexMap<String, String>) -> *mut Filter {
        log_setup();
        Box::into_raw(Box::new(Filter {
            avg: Avg::new(),

            data: Vec::new(),
        }))
    }

    pub fn on_incoming_requests(&mut self, mut x: Rpc) -> Vec<Rpc> {
        self.avg.execute(x.uid, x.data.clone());

        self.data.push(x.data.clone());
        return vec![x];
    }

    pub fn on_outgoing_responses(&mut self, mut x: Rpc) -> Vec<Rpc> {
        // 1. if there is an aggregation function, find its answer

        x.data = self.avg.get_avg();
        return vec![x];

        // 2. else just return data
        x.data = self.data.join(".");
        return vec![x];
    }

    pub fn on_outgoing_requests(&mut self, mut x: Rpc) -> Vec<Rpc> {
        // this should never happen to storage
        return vec![x];
    }

    pub fn on_incoming_responses(&mut self, mut x: Rpc) -> Vec<Rpc> {
        // this should never happen to storage
        return vec![x];
    }

    #[no_mangle]
    pub fn execute(&mut self, x: &Rpc) -> Vec<Rpc> {
        match x.headers["direction"].as_str() {
            "request" => match x.headers["location"].as_str() {
                "ingress" => {
                    return self.on_incoming_requests(x.clone());
                }
                "egress" => {
                    return self.on_outgoing_requests(x.clone());
                }
                _ => {
                    panic!("Filter got an rpc with no location\n");
                }
            },
            "response" => match x.headers["location"].as_str() {
                "ingress" => {
                    return self.on_incoming_responses(x.clone());
                }
                "egress" => {
                    return self.on_outgoing_responses(x.clone());
                }
                _ => {
                    panic!("Filter got an rpc with no location\n");
                }
            },
            _ => {
                panic!("Filter got an rpc with no direction\n");
            }
        }
    }
}