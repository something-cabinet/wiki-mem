use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::mpsc::Sender;

use crate::engine::AuditEvent;

pub struct AuditState {
    audit_sender: Sender<AuditEvent>,
    audit_drops: AtomicU64,
}

impl AuditState {
    pub fn new(audit_sender: Sender<AuditEvent>) -> Self {
        Self {
            audit_sender,
            audit_drops: AtomicU64::default(),
        }
    }

    pub fn emit(&self, event: AuditEvent) {
        if self.audit_sender.try_send(event).is_err() {
            self.audit_drops.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn dropped_count(&self) -> u64 {
        self.audit_drops.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wm_constants::AUDIT_CHANNEL_BUFFER;

    const AUDIT_FIELDS: [&str; 2] = ["audit_sender", "audit_drops"];

    fn sample_event(tool_name: &str) -> AuditEvent {
        AuditEvent {
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            tool_name: tool_name.to_string(),
            action: "invoke".to_string(),
            duration_ms: 1,
            result: "ok".to_string(),
            error_message: None,
            entity_refs: Vec::new(),
        }
    }

    fn audit_struct_body() -> &'static str {
        const SOURCE: &str = include_str!("audit_state_model.rs");
        let start = SOURCE
            .find("pub struct AuditState {")
            .expect("AuditState declaration exists");
        let body = &SOURCE[start..];
        let end = body.find("\n}").expect("AuditState body terminates");
        &body[..end]
    }

    fn declared_field_line(field: &str) -> &'static str {
        let needles = [format!("{field}:"), format!("pub {field}:")];
        let matches: Vec<&str> = audit_struct_body()
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

    #[test]
    fn test_audit_state_declares_its_two_fields_private() {
        for field in AUDIT_FIELDS {
            let declaration = declared_field_line(field);
            assert!(
                !declaration.starts_with("pub"),
                "{field} must stay private so no sibling handle can read it"
            );
        }
    }

    #[test]
    fn test_emit_routes_to_owned_sender_and_counts_drops() {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(AUDIT_CHANNEL_BUFFER);
        let audit = AuditState::new(sender);

        audit.emit(sample_event("page.read"));
        let received = receiver
            .try_recv()
            .expect("owned channel carries the event");
        assert_eq!(received.tool_name, "page.read");
        assert_eq!(received.action, "invoke");
        assert_eq!(audit.dropped_count(), 0);

        drop(receiver);
        audit.emit(sample_event("page.write"));
        assert_eq!(audit.dropped_count(), 1);
    }
}
