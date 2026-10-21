use crate::models::PageType;
use crate::parser::extract_frontmatter;

pub const ACCEPTANCE_CRITERIA_KEY: &str = "acceptance_criteria";

pub fn validate_required_keys(frontmatter: &str) -> Result<(), String> {
    let content = format!("---\n{}\n---\n", frontmatter.trim());
    let Some(meta) = extract_frontmatter(&content).0 else {
        return Ok(());
    };
    let enforced = matches!(meta.page_type, PageType::Task | PageType::Spec);
    let missing = meta
        .task_data
        .as_ref()
        .is_none_or(|data| data.acceptance_criteria.is_empty());
    match enforced && missing {
        true => Err(format!(
            "{} page requires at least one '{}' frontmatter entry",
            meta.page_type.as_str(),
            ACCEPTANCE_CRITERIA_KEY
        )),
        false => Ok(()),
    }
}
