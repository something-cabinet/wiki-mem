use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::engine::{IndexScheduler, WriteChannel};
use crate::error::{ToolError, ToolResult};
use crate::skill::{Skill, SkillEngine, SkillToolSpec, TriggerEvent};

pub struct LifecycleState {
    skill_engine: RwLock<SkillEngine>,
    write_channel: WriteChannel,
    index_scheduler: IndexScheduler,
}

impl LifecycleState {
    pub fn new(write_channel: WriteChannel, debounce_ms: u64) -> Self {
        Self {
            skill_engine: RwLock::new(SkillEngine::new()),
            write_channel,
            index_scheduler: IndexScheduler::new(debounce_ms),
        }
    }

    pub fn scan_skills(&self, skills_dir: &Path) {
        if let Ok(mut engine) = self.skill_engine.write() {
            engine.scan(skills_dir);
            tracing::info!("Loaded {} skill(s)", engine.list().len());
        }
    }

    pub fn fire_skill_event(&self, event: &TriggerEvent) -> ToolResult<Vec<Skill>> {
        let engine = self
            .skill_engine
            .read()
            .map_err(|_| ToolError::lock_poisoned("skill_engine"))?;
        Ok(engine.fire_event(event).into_iter().cloned().collect())
    }

    pub fn skill_tool_specs(&self) -> Vec<SkillToolSpec> {
        match self.skill_engine.read() {
            Ok(engine) => engine.tool_specs(),
            Err(_) => Vec::new(),
        }
    }

    pub fn write(&self, path: PathBuf, content: Vec<u8>) -> Result<(), String> {
        self.write_channel.write(path, content)
    }

    pub fn submit_index<F>(&self, job_type: &str, rebuild_fn: F)
    where
        F: Fn() + Send + 'static,
    {
        self.index_scheduler.submit(job_type, rebuild_fn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::WriteOp;

    const LIFECYCLE_FIELDS: [&str; 3] = ["skill_engine", "write_channel", "index_scheduler"];
    const TEST_DEBOUNCE_MS: u64 = 0;
    const TEST_JOB_TYPE: &str = "page";

    fn lifecycle_struct_body() -> &'static str {
        const SOURCE: &str = include_str!("lifecycle_state_model.rs");
        let start = SOURCE
            .find("pub struct LifecycleState {")
            .expect("LifecycleState declaration exists");
        let body = &SOURCE[start..];
        let end = body.find("\n}").expect("LifecycleState body terminates");
        &body[..end]
    }

    fn declared_field_line(field: &str) -> &'static str {
        let needles = [format!("{field}:"), format!("pub {field}:")];
        let matches: Vec<&str> = lifecycle_struct_body()
            .lines()
            .map(str::trim)
            .filter(|line| {
                needles
                    .iter()
                    .any(|needle| line.starts_with(needle.as_str()))
            })
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "expected exactly one declaration of {field}"
        );
        matches[0]
    }

    fn sample_skill() -> &'static str {
        "---\nname: wm-demo\ntitle: Demo\ndescription: Demo skill\ntrigger:\n  event: page.create\n---\n\n# Steps\n"
    }

    #[test]
    fn test_lifecycle_state_declares_its_three_fields_private() {
        for field in LIFECYCLE_FIELDS {
            let declaration = declared_field_line(field);
            assert!(
                !declaration.starts_with("pub"),
                "{field} must stay private so no sibling handle can read it"
            );
        }
    }

    #[test]
    fn test_scan_skills_feeds_the_owned_engine() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("demo.md"), sample_skill()).expect("skill file");
        let (channel, _receiver) = WriteChannel::new();
        let lifecycle = LifecycleState::new(channel, TEST_DEBOUNCE_MS);

        lifecycle.scan_skills(dir.path());

        let triggered = lifecycle
            .fire_skill_event(&TriggerEvent::PageCreate)
            .expect("event fires");
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].name, "wm-demo");
    }

    #[test]
    fn test_skill_tool_specs_come_from_the_owned_engine() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("demo.md"), sample_skill()).expect("skill file");
        let (channel, _receiver) = WriteChannel::new();
        let lifecycle = LifecycleState::new(channel, TEST_DEBOUNCE_MS);

        lifecycle.scan_skills(dir.path());

        let specs = lifecycle.skill_tool_specs();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "wm_skill.wm-demo");
    }

    #[test]
    fn test_write_routes_through_the_owned_channel() {
        let (channel, mut receiver) = WriteChannel::new();
        let lifecycle = LifecycleState::new(channel, TEST_DEBOUNCE_MS);
        let path = PathBuf::from("/tmp/lifecycle-write.md");

        lifecycle
            .write(path.clone(), b"body".to_vec())
            .expect("write queues");

        match receiver.try_recv().expect("queued op") {
            WriteOp::Write {
                path: queued_path,
                content,
            } => {
                assert_eq!(queued_path, path);
                assert_eq!(content, b"body");
            }
            _ => panic!("expected a write op"),
        }
    }

    #[tokio::test]
    async fn test_submit_index_runs_the_owned_scheduler() {
        let (channel, _receiver) = WriteChannel::new();
        let lifecycle = LifecycleState::new(channel, TEST_DEBOUNCE_MS);
        let notify = std::sync::Arc::new(tokio::sync::Notify::new());
        let signal = notify.clone();

        lifecycle.submit_index(TEST_JOB_TYPE, move || {
            signal.notify_one();
        });

        notify.notified().await;
    }
}
