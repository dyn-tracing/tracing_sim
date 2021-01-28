# The Tracing Simulator

Tracing Simulator is a simulator for WebAssembly extensions.  With it you can define your own microservices architecture, 
and run user-defined extensions on each service.

## Quickstart

### Build
```cargo +nightly build```

### Run
Without a filter:
```cargo +nightly run```

With a filter:
```cargo +nightly run -- -p target/debug/libfilter_example.dylib```

## Customization

### Making your own microservices architecture

### Making your own extension
The recommended way to make an extension is by using the tracing compiler found [here](https://github.com/dyn-tracing/tracing_compiler) with the *-c sim* option.
Then run:
```cp tracing_compiler/rust_filter/filter.rs tracing_sim/libs/filter_lib/src/filter.rs```
```cd tracing_sim/libs/filter_lib/```
```cargo +nightly build```
```cd ../..```
```cargo +nightly build```
```cargo +nightly run -- -p target/debug/libfilter_example.dylib```

### Making your own microservice architecture
The microservice architecture is defined in sim/src/main.rs.  You must first create a simulator object through
```let mut simulator: Simulator = Simulator::new();```
Then you are free to add nodes and edges through simulator.add_node and simulator.add_edge.  Node types currently include
a TrafficGenerator and Link.  TrafficGenerator generates RPCs and Link is meant to represent a service.  Edges are by default 
bidirectional, and have one direction only if you specify that through simulator.add_one_direction_edge.

Writing the architecture you have in mind by naming nodes and edges can be tricky.  If you want to make a pdf of the 
graph you are making for debugging purposes, install graphviz (https://graphviz.org/download/)
and run the simulator with command line option"-g", eg, ```cargo +nightly run -- -g```  After running this command, your
graph will be in graph.pdf, and you can see the graph that you have made.

## Architecture
TODO




