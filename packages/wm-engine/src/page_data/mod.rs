use std::collections::HashMap;

use petgraph::stable_graph::StableGraph;

use super::edge_type::EdgeType;
use super::page::meta::WikiPageMeta;

pub mod task_data;
pub mod spec_data;
pub mod decision_data;
pub mod pattern_data;
pub mod memory_data;
pub mod rule;
pub mod spec_reqs;

pub use task_data::*;
pub use spec_data::*;
pub use decision_data::*;
pub use pattern_data::*;
pub use memory_data::*;
pub use rule::*;
pub use spec_reqs::*;

pub type GraphSnapshot = (
    StableGraph<WikiPageMeta, EdgeType>,
    HashMap<String, petgraph::stable_graph::NodeIndex>,
);
