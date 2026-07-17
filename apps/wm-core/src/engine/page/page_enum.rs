#[derive(Clone, Debug)]
pub enum Page {
    Task { meta: crate::engine::WikiPageMeta, data: crate::engine::TaskData },
    Spec { meta: crate::engine::WikiPageMeta, data: crate::engine::SpecData },
    Decision { meta: crate::engine::WikiPageMeta, data: crate::engine::DecisionData },
    Pattern { meta: crate::engine::WikiPageMeta, data: crate::engine::PatternData },
    Memory { meta: crate::engine::WikiPageMeta, data: crate::engine::MemoryData },
    Rule { meta: crate::engine::WikiPageMeta, data: crate::engine::RuleData },
    Concept { meta: crate::engine::WikiPageMeta },
    HowTo { meta: crate::engine::WikiPageMeta },
    Note { meta: crate::engine::WikiPageMeta },
    Reference { meta: crate::engine::WikiPageMeta },
}

impl Page {
    pub fn meta(&self) -> &crate::engine::WikiPageMeta {
        match self {
            Page::Task { meta, .. } | Page::Spec { meta, .. }
            | Page::Decision { meta, .. } | Page::Pattern { meta, .. }
            | Page::Memory { meta, .. } | Page::Rule { meta, .. }
            | Page::Concept { meta }
            | Page::HowTo { meta } | Page::Note { meta }
            | Page::Reference { meta } => meta,
        }
    }

    pub fn meta_mut(&mut self) -> &mut crate::engine::WikiPageMeta {
        match self {
            Page::Task { meta, .. } | Page::Spec { meta, .. }
            | Page::Decision { meta, .. } | Page::Pattern { meta, .. }
            | Page::Memory { meta, .. } | Page::Rule { meta, .. }
            | Page::Concept { meta }
            | Page::HowTo { meta } | Page::Note { meta }
            | Page::Reference { meta } => meta,
        }
    }

    pub fn page_type(&self) -> crate::engine::PageType {
        match self {
            Page::Task { .. } => crate::engine::PageType::Task,
            Page::Spec { .. } => crate::engine::PageType::Spec,
            Page::Decision { .. } => crate::engine::PageType::Decision,
            Page::Pattern { .. } => crate::engine::PageType::Pattern,
            Page::Memory { .. } => crate::engine::PageType::Memory,
            Page::Rule { .. } => crate::engine::PageType::Rule,
            Page::Concept { .. } => crate::engine::PageType::Concept,
            Page::HowTo { .. } => crate::engine::PageType::Howto,
            Page::Note { .. } => crate::engine::PageType::Note,
            Page::Reference { .. } => crate::engine::PageType::Reference,
        }
    }
}

impl From<crate::engine::WikiPageMeta> for Page {
    fn from(mut wpm: crate::engine::WikiPageMeta) -> Self {
        let pt = wpm.page_type.clone();
        match pt {
            crate::engine::PageType::Task => Page::Task {
                data: wpm.task_data.take().expect("TaskData missing for Task page"),
                meta: wpm,
            },
            crate::engine::PageType::Spec => Page::Spec {
                data: wpm.spec_data.take().expect("SpecData missing for Spec page"),
                meta: wpm,
            },
            crate::engine::PageType::Decision => Page::Decision {
                data: wpm.decision_data.take().expect("DecisionData missing for Decision page"),
                meta: wpm,
            },
            crate::engine::PageType::Pattern => Page::Pattern {
                data: wpm.pattern_data.take().expect("PatternData missing for Pattern page"),
                meta: wpm,
            },
            crate::engine::PageType::Memory => Page::Memory {
                data: wpm.memory_data.take().expect("MemoryData missing for Memory page"),
                meta: wpm,
            },
            crate::engine::PageType::Rule => Page::Rule {
                data: wpm.rule_data.take().expect("RuleData missing for Rule page"),
                meta: wpm,
            },
            crate::engine::PageType::Concept => Page::Concept { meta: wpm },
            crate::engine::PageType::Howto => Page::HowTo { meta: wpm },
            crate::engine::PageType::Note => Page::Note { meta: wpm },
            crate::engine::PageType::Reference => Page::Reference { meta: wpm },
        }
    }
}

impl From<Page> for crate::engine::WikiPageMeta {
    fn from(page: Page) -> Self {
        let (page_type, mut meta) = match page {
            Page::Task { mut meta, data } => {
                meta.task_data = Some(data);
                (crate::engine::PageType::Task, meta)
            }
            Page::Spec { mut meta, data } => {
                meta.spec_data = Some(data);
                (crate::engine::PageType::Spec, meta)
            }
            Page::Decision { mut meta, data } => {
                meta.decision_data = Some(data);
                (crate::engine::PageType::Decision, meta)
            }
            Page::Pattern { mut meta, data } => {
                meta.pattern_data = Some(data);
                (crate::engine::PageType::Pattern, meta)
            }
            Page::Memory { mut meta, data } => {
                meta.memory_data = Some(data);
                (crate::engine::PageType::Memory, meta)
            }
            Page::Rule { mut meta, data } => {
                meta.rule_data = Some(data);
                (crate::engine::PageType::Rule, meta)
            }
            Page::Concept { meta } => (crate::engine::PageType::Concept, meta),
            Page::HowTo { meta } => (crate::engine::PageType::Howto, meta),
            Page::Note { meta } => (crate::engine::PageType::Note, meta),
            Page::Reference { meta } => (crate::engine::PageType::Reference, meta),
        };
        meta.page_type = page_type;
        meta
    }
}
