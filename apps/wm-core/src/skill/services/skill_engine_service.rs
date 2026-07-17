use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::skill::models::skill_model::Skill;
use crate::skill::models::trigger_event_model::TriggerEvent;
use crate::skill::models::skill_tool_spec_model::SkillToolSpec;

pub struct SkillEngine {
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

            if let Some(skill) = crate::skill::helpers::skill_frontmatter_parser_helper::parse_skill_file(&path, &content) {
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

    pub fn fire_event(&self, event: &TriggerEvent) -> Vec<&Skill> {
        let mut triggered = Vec::new();
        for skill in self.skills.values() {
            let trigger = match &skill.trigger {
                Some(t) => t,
                None => continue,
            };
            if &TriggerEvent::from_str(&trigger.event) != event {
                continue;
            }
            if let Some(ref _condition) = trigger.condition {
                tracing::debug!(
                    "Skill '{}' has condition '{}' — not evaluated yet",
                    skill.name,
                    _condition
                );
            }
            tracing::info!("Skill trigger: {} → {}", skill.name, trigger.event);
            triggered.push(skill);
        }
        triggered
    }

    pub fn tool_specs(&self) -> Vec<SkillToolSpec> {
        self.skills.values().map(|skill| {
            let name = skill.name.clone();
            let instructions = skill.instructions.clone();
            let steps = crate::skill::helpers::skill_frontmatter_parser_helper::parse_steps_from_markdown(&instructions);
            let description = skill.description.clone();
            let trigger_info = skill.trigger.as_ref().map(|t| serde_json::json!({
                "event": t.event,
                "condition": t.condition,
                "priority": t.priority,
            }));
            let handler = {
                let name = name.clone();
                let description = description.clone();
                let steps = steps.clone();
                let trigger_info = trigger_info.clone();
                Arc::new(move |_params| {
                    Ok(serde_json::json!({
                        "skill": name,
                        "description": description,
                        "steps": steps,
                        "instructions": instructions,
                        "trigger_info": trigger_info,
                        "type": "skill_instructions",
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
