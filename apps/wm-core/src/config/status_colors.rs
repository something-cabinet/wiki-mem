use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusColors {
    pub colors: HashMap<String, String>,
}

impl Default for StatusColors {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("todo".into(), "gray".into());
        colors.insert("in-progress".into(), "blue".into());
        colors.insert("in-review".into(), "violet".into());
        colors.insert("done".into(), "green".into());
        colors.insert("blocked".into(), "red".into());
        colors.insert("on-hold".into(), "amber".into());
        colors.insert("urgent".into(), "rose".into());
        Self { colors }
    }
}
