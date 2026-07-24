use std::path::Path;

use serde::Deserialize;

use crate::skill::models::trigger_config_model::TriggerConfig;
use crate::embed_files::EmbeddedFiles;
use crate::skill::models::skill_model::Skill;

#[derive(Debug, Deserialize)]
pub(crate) struct SkillFrontmatter {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub trigger: Option<TriggerConfig>,
}

pub fn parse_steps_from_markdown(md: &str) -> Vec<serde_json::Value> {
    let mut steps = Vec::new();
    let mut current_title = String::new();
    let mut current_body = String::new();

    for line in md.lines() {
        if let Some(stripped) = line.strip_prefix("## ") {
            if !current_title.is_empty() {
                steps.push(serde_json::json!({
                    "title": current_title,
                    "detail": current_body.trim(),
                }));
            }
            current_title = stripped.trim().to_string();
            current_body.clear();
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    if !current_title.is_empty() {
        steps.push(serde_json::json!({
            "title": current_title,
            "detail": current_body.trim(),
        }));
    }

    steps
}

pub(crate) fn parse_skill_file(path: &Path, content: &str) -> Option<Skill> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }

    let end = content[3..].find("\n---")?;
    let yaml_str = &content[3..3 + end];
    let body = &content[3 + end + 4..].trim();

    let fm: SkillFrontmatter = serde_yaml::from_str(yaml_str).ok()?;
    let file_stem = path.file_stem()?.to_string_lossy().to_string();

    let name = if let Some(ref n) = fm.name {
        n.clone()
    } else {
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if file_name.eq_ignore_ascii_case("SKILL.md") {
            path.parent()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(file_stem)
        } else {
            file_stem
        }
    };

    Some(Skill {
        name,
        title: fm.title.unwrap_or_default(),
        description: fm.description.unwrap_or_default(),
        trigger: fm.trigger,
        instructions: body.to_string(),
        file_path: path.to_path_buf(),
    })
}

pub fn load_embedded_skills() -> Vec<Skill> {
    let mut skills = Vec::new();
    for path in EmbeddedFiles::iter() {
        let path_str = path.as_ref();
        if !path_str.ends_with("SKILL.md") {
            continue;
        }
        if let Some(file) = EmbeddedFiles::get(path_str) {
            let content = String::from_utf8_lossy(&file.data).to_string();
            let virtual_path = Path::new(path_str);
            if let Some(skill) = parse_skill_file(virtual_path, &content) {
                skills.push(skill);
            }
        }
    }
    skills
}
