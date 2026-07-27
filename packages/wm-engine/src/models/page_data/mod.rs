use std::collections::HashMap;

use petgraph::stable_graph::StableGraph;

use crate::models::edge_type_model::EdgeType;
use crate::models::page::meta_model::WikiPageMeta;

pub mod acceptance_criterion_model;
pub mod decision_data_model;
pub mod memory_data_model;
pub mod pattern_data_model;
pub mod rule_category_model;
pub mod rule_data_model;
pub mod spec_data_model;
pub mod spec_reqs;
pub mod task_data_model;

pub use acceptance_criterion_model::*;
pub use decision_data_model::*;
pub use memory_data_model::*;
pub use pattern_data_model::*;
pub use rule_category_model::*;
pub use rule_data_model::*;
pub use spec_data_model::*;
pub use spec_reqs::*;
pub use task_data_model::*;

pub type GraphSnapshot = (
    StableGraph<WikiPageMeta, EdgeType>,
    HashMap<String, petgraph::stable_graph::NodeIndex>,
);
