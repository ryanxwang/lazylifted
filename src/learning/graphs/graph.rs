use petgraph::{graph::Graph, Undirected};

pub type CGraph = Graph<i32, i32, Undirected, u32>;
pub type NodeID = petgraph::graph::NodeIndex<u32>;
