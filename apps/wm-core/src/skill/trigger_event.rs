#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TriggerEvent {
    SessionStart,
    SourceComplete,
    PageCreate,
    PageUpdate,
    IndexRebuild,
}

impl std::str::FromStr for TriggerEvent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str(s))
    }
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
            _ => TriggerEvent::SourceComplete,
        }
    }
}
