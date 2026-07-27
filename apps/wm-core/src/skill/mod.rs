pub mod helpers;
pub mod models;
pub mod services;
pub use helpers::*;
pub use models::*;
pub use services::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
        let skill =
            helpers::skill_frontmatter_parser_helper::parse_skill_file(path, content).unwrap();
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
        let skill =
            helpers::skill_frontmatter_parser_helper::parse_skill_file(path, content).unwrap();
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
        let skill =
            helpers::skill_frontmatter_parser_helper::parse_skill_file(path, content).unwrap();
        assert_eq!(skill.name, "wm-fallback");
    }

    #[test]
    fn test_trigger_event_from_str() {
        use std::str::FromStr;
        assert_eq!(
            TriggerEvent::from_str("source.complete").unwrap(),
            TriggerEvent::SourceComplete
        );
        assert_eq!(
            TriggerEvent::from_str("page.create").unwrap(),
            TriggerEvent::PageCreate
        );
        assert_eq!(
            TriggerEvent::from_str("index.rebuild").unwrap(),
            TriggerEvent::IndexRebuild
        );
    }

    #[test]
    fn test_trigger_event_from_str_all() {
        use crate::skill::TriggerEvent;
        assert_eq!(
            "session_start".parse::<TriggerEvent>().unwrap(),
            TriggerEvent::SessionStart
        );
        assert_eq!(
            "source.complete".parse::<TriggerEvent>().unwrap(),
            TriggerEvent::SourceComplete
        );
        assert_eq!(
            "index.rebuild".parse::<TriggerEvent>().unwrap(),
            TriggerEvent::IndexRebuild
        );
        assert_eq!(
            "page.create".parse::<TriggerEvent>().unwrap(),
            TriggerEvent::PageCreate
        );
        assert_eq!(
            "page.update".parse::<TriggerEvent>().unwrap(),
            TriggerEvent::PageUpdate
        );
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
        assert!(
            names.contains(&"wm-implement"),
            "Should contain wm-implement"
        );
        for skill in &skills {
            assert!(
                !skill.instructions.is_empty(),
                "Skill {} has empty instructions",
                skill.name
            );
        }
    }
}
