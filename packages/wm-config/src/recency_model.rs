use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RecencyModel {
    Fsrs,
    Linear,
    Exponential,
    None,
}

impl Default for RecencyModel {
    fn default() -> Self {
        RecencyModel::Fsrs
    }
}
