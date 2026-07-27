pub enum OutputFormat {
    Json,
    Text,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        if s == "json" {
            return OutputFormat::Json;
        }
        OutputFormat::Text
    }
}
