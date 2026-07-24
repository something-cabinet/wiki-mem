use crate::mcp::prelude::*;
use serde_json::json;

use crate::skill::TriggerEvent;

#[derive(Deserialize, JsonSchema)]
struct WmSkillTriggerInput {
    #[schemars(description = "Event name: session_start, page_create, page_update, index_rebuild, source_complete")]
    event: String,
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    if let Ok(skill_engine) = engine.skill_engine.read() {
        for spec in skill_engine.tool_specs() {
            registry.register_with_schema(&spec.name, &spec.description, json!({
                "type": "object",
                "properties": {}
            }), spec.handler);
        }
    }

    let e = engine.clone();
    registry.register_typed(
        "wm_skill.trigger",
        "Fire skills triggered by a lifecycle event. Returns all triggered skill instructions.",
        move |input: WmSkillTriggerInput| {
            let event = match input.event.to_lowercase().replace('-', "_").as_str() {
                "session_start" | "session.start" => TriggerEvent::SessionStart,
                "source_complete" | "source.complete" => TriggerEvent::SourceComplete,
                "page_create" | "page.create" => TriggerEvent::PageCreate,
                "page_update" | "page.update" => TriggerEvent::PageUpdate,
                "index_rebuild" | "index.rebuild" => TriggerEvent::IndexRebuild,
                other => return Err(ToolError::internal(
                    format!("Unknown trigger event: '{}'. Valid: session_start, source_complete, page_create, page_update, index_rebuild", other)
                )),
            };
            fire_session_event(&e, &event)
        },
    );
}

pub fn fire_session_event(
    engine: &EngineState,
    event: &TriggerEvent,
) -> Result<serde_json::Value, ToolError> {
    let skill_engine = engine.skill_engine.read().map_err(|e| {
        ToolError::internal(format!("Skill engine lock poisoned: {}", e))
    })?;

    let triggered = skill_engine.fire_event(event);
    if triggered.is_empty() {
        return Ok(json!({
            "event": format!("{:?}", event),
            "triggered": [],
            "count": 0,
        }));
    }

    let skills: Vec<serde_json::Value> = triggered
        .iter()
        .map(|skill| {
            let steps = crate::skill::parse_steps_from_markdown(&skill.instructions);
            json!({
                "name": skill.name,
                "title": skill.title,
                "description": skill.description,
                "steps": steps,
                "instructions": skill.instructions,
                "trigger": skill.trigger.as_ref().map(|t| json!({
                    "event": t.event,
                    "condition": t.condition,
                    "priority": t.priority,
                })),
            })
        })
        .collect();

    Ok(json!({
        "event": format!("{:?}", event),
        "triggered": skills,
        "count": skills.len(),
    }))
}
