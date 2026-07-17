use serde::{Deserialize, Serialize};

use super::meta::WikiPageMeta;
use super::section::SectionDoc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WikiPageContent {
    pub raw: String,
    pub sections: Vec<SectionDoc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<WikiPageMeta>,
}
