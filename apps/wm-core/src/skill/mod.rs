use std::path::Path;

pub mod assets;
pub mod trigger_event;
pub mod trigger_config;
pub mod skill;
pub mod tool_spec;
pub mod engine;
pub(crate) mod frontmatter;

pub use assets::*;
pub use trigger_event::*;
pub use trigger_config::*;
pub use skill::*;
pub use tool_spec::*;
pub use engine::*;

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

fn parse_skill_file(path: &Path, content: &str) -> Option<Skill> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }

    let end = content[3..].find("\n---")?;
    let yaml_str = &content[3..3 + end];
    let body = &content[3 + end + 4..].trim();

    let fm: frontmatter::SkillFrontmatter = serde_yaml::from_str(yaml_str).ok()?;
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
    for path in SkillAssets::iter() {
        let path_str = path.as_ref();
        if !path_str.ends_with("SKILL.md") {
            continue;
        }
        if let Some(file) = SkillAssets::get(path_str) {
            let content = String::from_utf8_lossy(&file.data).to_string();
            let virtual_path = Path::new(path_str);
            if let Some(skill) = parse_skill_file(virtual_path, &content) {
                skills.push(skill);
            }
        }
    }
    skills
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_file_flat() {
        let content = r#"---
name: wm-test
description: Test skill
trigger:
  event: source.complete
  priority: 1
---

# Steps

1. Do this
2. Do that
"#;
        let path = Path::new("wm-test.md");
        let skill = parse_skill_file(path, content).unwrap();
        assert_eq!(skill.name, "wm-test");
        assert_eq!(skill.description, "Test skill");
        assert!(skill.trigger.is_some());
        assert_eq!(skill.trigger.as_ref().unwrap().event, "source.complete");
        assert!(skill.instructions.contains("Do this"));
    }

    #[test]
    fn test_parse_skill_file_subdirectory() {
        let content = r#"---
name: wm-init
description: Init skill
---

# Steps

1. Init
"#;
        let path = Path::new("skills/wm-init/SKILL.md");
        let skill = parse_skill_file(path, content).unwrap();
        assert_eq!(skill.name, "wm-init");
        assert_eq!(skill.description, "Init skill");
    }

    #[test]
    fn test_parse_skill_file_subdirectory_fallback() {
        let content = r#"---
description: Fallback name test
---

# Steps

1. Test
"#;
        let path = Path::new("skills/wm-fallback/SKILL.md");
        let skill = parse_skill_file(path, content).unwrap();
        assert_eq!(skill.name, "wm-fallback");
    }

    #[test]
    fn test_trigger_event_from_str() {
        assert_eq!(
            TriggerEvent::from_str("source.complete"),
            TriggerEvent::SourceComplete
        );
        assert_eq!(
            TriggerEvent::from_str("page.create"),
            TriggerEvent::PageCreate
        );
        assert_eq!(
            TriggerEvent::from_str("index.rebuild"),
            TriggerEvent::IndexRebuild
        );
    }

    #[test]
    fn test_trigger_event_from_str_all() {
        use crate::skill::TriggerEvent;
        assert_eq!("session_start".parse::<TriggerEvent>().unwrap(), TriggerEvent::SessionStart);
        assert_eq!("source.complete".parse::<TriggerEvent>().unwrap(), TriggerEvent::SourceComplete);
        assert_eq!("index.rebuild".parse::<TriggerEvent>().unwrap(), TriggerEvent::IndexRebuild);
        assert_eq!("page.create".parse::<TriggerEvent>().unwrap(), TriggerEvent::PageCreate);
        assert_eq!("page.update".parse::<TriggerEvent>().unwrap(), TriggerEvent::PageUpdate);
    }

    #[test]
    fn test_skill_engine_scan_empty_dir() {
        let mut engine = SkillEngine::new();
        let tmp = std::env::temp_dir();
        engine.scan(&tmp.join("nonexistent-skills-xyz"));
        assert!(engine.list().is_empty());
    }

    #[test]
    fn test_load_embedded_skills() {
        let skills = load_embedded_skills();
        assert_eq!(skills.len(), 15, "Expected 15 embedded skills");
        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"wm-init"), "Should contain wm-init");
        assert!(names.contains(&"wm-plan"), "Should contain wm-plan");
        assert!(names.contains(&"wm-implement"), "Should contain wm-implement");
        for skill in &skills {
            assert!(!skill.instructions.is_empty(), "Skill {} has empty instructions", skill.name);
        }
    }
}
