// ─── Skill System — Agent Workflow Skills with Fire Triggers ─

use rust_embed::RustEmbed;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Embedded skill assets (wm-*/SKILL.md files compiled into the binary)
#[derive(RustEmbed)]
#[folder = "src/skills/"]
pub struct SkillAssets;

/// A lifecycle event that can trigger a skill
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TriggerEvent {
    SessionStart,
    SourceComplete,
    PageCreate,
    PageUpdate,
    IndexRebuild,
}

impl TriggerEvent {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().replace('-', "_").as_str() {
            "session_start" | "session.start" => TriggerEvent::SessionStart,
            "source_complete" | "source.complete" => TriggerEvent::SourceComplete,
            "page_create" | "page.create" => TriggerEvent::PageCreate,
            "page_update" | "page.update" => TriggerEvent::PageUpdate,
            "index_rebuild" | "index.rebuild" => TriggerEvent::IndexRebuild,
            _ => TriggerEvent::SourceComplete, // default
        }
    }
}

/// Trigger configuration from skill frontmatter
#[derive(Debug, Deserialize, Clone)]
pub struct TriggerConfig {
    pub event: String,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub priority: Option<u32>,
}

/// A parsed skill from .agent/skills/*.md
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub title: String,
    pub description: String,
    pub trigger: Option<TriggerConfig>,
    pub instructions: String,
    pub file_path: PathBuf,
}

/// A skill tool specification for MCP registration (inverted dependency).
pub struct SkillToolSpec {
    pub name: String,
    pub description: String,
    pub handler: Arc<dyn Fn(serde_json::Value) -> Result<serde_json::Value, crate::error::ToolError> + Send + Sync>,
}

/// Skill engine — parses, registers, and dispatches skills
pub struct SkillEngine {
    /// All parsed skills, keyed by name
    skills: HashMap<String, Skill>,
}

impl Default for SkillEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillEngine {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Scan the skills directory and parse all skill files
    pub fn scan(&mut self, skills_dir: &Path) {
        self.skills.clear();
        if !skills_dir.exists() {
            return;
        }

        for entry in walkdir::WalkDir::new(skills_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        {
            let path = entry.path().to_path_buf();
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if let Some(skill) = parse_skill_file(&path, &content) {
                self.skills.insert(skill.name.clone(), skill);
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    /// Fire skills triggered by an event
    pub fn fire_event<F>(&self, event: &TriggerEvent, emit_audit: &F)
    where
        F: Fn(&str, &str, &str, i64, Option<String>, Vec<String>),
    {
        for skill in self.skills.values() {
            let trigger = match &skill.trigger {
                Some(t) => t,
                None => continue,
            };
            if &TriggerEvent::from_str(&trigger.event) != event {
                continue;
            }
            // Evaluate condition if present (future: LLM-based or expression eval)
            if let Some(ref _condition) = trigger.condition {
                // Condition field is parsed but not yet evaluated.
                // Future: evaluate condition expression against event context.
                // For now: fire unconditionally if event matches, log condition for reference.
                tracing::debug!("Skill '{}' has condition '{}' — not evaluated yet", skill.name, _condition);
            }
            // Best-effort execution: log trigger, don't block
            tracing::info!("Skill trigger: {} → {}", skill.name, trigger.event);
            emit_audit(
                &format!("wm_skill.{}", skill.name),
                "trigger",
                "ok",
                0,
                None,
                vec![skill.name.clone()],
            );
        }
    }

    /// Return tool specifications for MCP registration (inverted dependency).
    pub fn tool_specs(&self) -> Vec<SkillToolSpec> {
        self.skills.values().map(|skill| {
            let name = skill.name.clone();
            let instructions = skill.instructions.clone();
            let description = skill.description.clone();
            let handler = {
                let name = name.clone();
                let description = description.clone();
                Arc::new(move |_params| {
                    Ok(serde_json::json!({
                        "skill": name,
                        "description": description,
                        "instructions": instructions,
                    }))
                })
            };
            SkillToolSpec {
                name: format!("wm_skill.{}", name),
                description,
                handler,
            }
        }).collect()
    }
}

/// Parse a skill file from its markdown content
fn parse_skill_file(path: &Path, content: &str) -> Option<Skill> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }

    let end = content[3..].find("\n---")?;
    let yaml_str = &content[3..3 + end];
    let body = &content[3 + end + 4..].trim();

    #[derive(Debug, Deserialize)]
    struct SkillFrontmatter {
        name: Option<String>,
        title: Option<String>,
        description: Option<String>,
        trigger: Option<TriggerConfig>,
    }

    let fm: SkillFrontmatter = serde_yaml::from_str(yaml_str).ok()?;
    let file_stem = path.file_stem()?.to_string_lossy().to_string();

    // Determine skill name: prefer frontmatter `name:` field, then parent dir name
    // for SKILL.md in subdirectory (wm-*/SKILL.md), then file_stem as fallback
    let name = if let Some(ref n) = fm.name {
        n.clone()
    } else {
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if file_name.eq_ignore_ascii_case("SKILL.md") {
            // Subdirectory format: use parent directory name
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

/// Load embedded skill files from the binary (via rust-embed)
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
        // SKILL.md inside a subdirectory — name should come from parent dir
        let content = r#"---
name: wm-init
description: Init skill
---

# Steps

1. Init
"#;
        let path = Path::new("skills/wm-init/SKILL.md");
        let skill = parse_skill_file(path, content).unwrap();
        assert_eq!(skill.name, "wm-init"); // from name: field
        assert_eq!(skill.description, "Init skill");
    }

    #[test]
    fn test_parse_skill_file_subdirectory_fallback() {
        // SKILL.md without name: field — name should come from parent dir
        let content = r#"---
description: Fallback name test
---

# Steps

1. Test
"#;
        let path = Path::new("skills/wm-fallback/SKILL.md");
        let skill = parse_skill_file(path, content).unwrap();
        assert_eq!(skill.name, "wm-fallback"); // from parent dir
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
    fn test_skill_engine_scan_empty_dir() {
        let mut engine = SkillEngine::new();
        let tmp = std::env::temp_dir();
        engine.scan(&tmp.join("nonexistent-skills-xyz"));
        assert!(engine.list().is_empty());
    }

    #[test]
    fn test_load_embedded_skills() {
        let skills = load_embedded_skills();
        // We expect 15 embedded wm-* skills (13 SDD + 1 flow + 1 validate)
        assert_eq!(skills.len(), 15, "Expected 15 embedded skills");
        // Check a few expected skills exist
        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"wm-init"), "Should contain wm-init");
        assert!(names.contains(&"wm-plan"), "Should contain wm-plan");
        assert!(names.contains(&"wm-implement"), "Should contain wm-implement");
        // Each skill should have non-empty instructions
        for skill in &skills {
            assert!(!skill.instructions.is_empty(), "Skill {} has empty instructions", skill.name);
        }
    }
}
