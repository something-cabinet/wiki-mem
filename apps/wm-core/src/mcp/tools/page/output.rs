use serde::Serialize;

#[derive(Serialize)]
pub struct WmPageGetOutput {
    pub id: String,
    pub content: String,
    pub sections: Vec<PageSectionOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Serialize)]
pub struct PageSectionOutput {
    pub header: String,
    pub body: String,
}

#[derive(Serialize)]
pub struct WmPageCreateOutput {
    pub id: String,
    pub path: String,
    pub r#type: String,
}

#[derive(Serialize)]
pub struct WmPageListOutput {
    pub pages: Vec<serde_json::Value>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct WmPageUpdateOutput {
    pub id: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct WmPageDeleteOutput {
    pub id: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct WmPageLinkOutput {
    pub id: String,
    pub target: String,
    pub r#type: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct WmPageUnlinkOutput {
    pub id: String,
    pub target: String,
    pub status: String,
}
