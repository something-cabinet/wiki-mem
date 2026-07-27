#[derive(Debug)]
pub enum Page {
    Task {
        meta: crate::models::WikiPageMeta,
        data: crate::models::TaskData,
    },
    Spec {
        meta: crate::models::WikiPageMeta,
        data: crate::models::SpecData,
    },
    Decision {
        meta: crate::models::WikiPageMeta,
        data: crate::models::DecisionData,
    },
    Pattern {
        meta: crate::models::WikiPageMeta,
        data: crate::models::PatternData,
    },
    Memory {
        meta: crate::models::WikiPageMeta,
        data: crate::models::MemoryData,
    },
    Core {
        meta: crate::models::WikiPageMeta,
    },
    Rule {
        meta: crate::models::WikiPageMeta,
        data: crate::models::RuleData,
    },
    Concept {
        meta: crate::models::WikiPageMeta,
    },
    HowTo {
        meta: crate::models::WikiPageMeta,
    },
    Note {
        meta: crate::models::WikiPageMeta,
    },
    Reference {
        meta: crate::models::WikiPageMeta,
    },
}

impl Page {
    pub fn meta(&self) -> &crate::models::WikiPageMeta {
        match self {
            Page::Task { meta, .. }
            | Page::Spec { meta, .. }
            | Page::Decision { meta, .. }
            | Page::Pattern { meta, .. }
            | Page::Memory { meta, .. }
            | Page::Core { meta }
            | Page::Rule { meta, .. }
            | Page::Concept { meta }
            | Page::HowTo { meta }
            | Page::Note { meta }
            | Page::Reference { meta } => meta,
        }
    }

    pub fn meta_mut(&mut self) -> &mut crate::models::WikiPageMeta {
        match self {
            Page::Task { meta, .. }
            | Page::Spec { meta, .. }
            | Page::Decision { meta, .. }
            | Page::Pattern { meta, .. }
            | Page::Memory { meta, .. }
            | Page::Core { meta }
            | Page::Rule { meta, .. }
            | Page::Concept { meta }
            | Page::HowTo { meta }
            | Page::Note { meta }
            | Page::Reference { meta } => meta,
        }
    }

    pub fn page_type(&self) -> crate::models::PageType {
        match self {
            Page::Task { .. } => crate::models::PageType::Task,
            Page::Spec { .. } => crate::models::PageType::Spec,
            Page::Decision { .. } => crate::models::PageType::Decision,
            Page::Pattern { .. } => crate::models::PageType::Pattern,
            Page::Memory { .. } => crate::models::PageType::Memory,
            Page::Core { .. } => crate::models::PageType::Core,
            Page::Rule { .. } => crate::models::PageType::Rule,
            Page::Concept { .. } => crate::models::PageType::Concept,
            Page::HowTo { .. } => crate::models::PageType::Howto,
            Page::Note { .. } => crate::models::PageType::Note,
            Page::Reference { .. } => crate::models::PageType::Reference,
        }
    }
}

impl From<crate::models::WikiPageMeta> for Page {
    fn from(mut wpm: crate::models::WikiPageMeta) -> Self {
        let pt = wpm.page_type.clone();
        match pt {
            crate::models::PageType::Task => Page::Task {
                data: wpm
                    .task_data
                    .take()
                    .expect("TaskData missing for Task page"),
                meta: wpm,
            },
            crate::models::PageType::Spec => Page::Spec {
                data: wpm
                    .spec_data
                    .take()
                    .expect("SpecData missing for Spec page"),
                meta: wpm,
            },
            crate::models::PageType::Decision => Page::Decision {
                data: wpm
                    .decision_data
                    .take()
                    .expect("DecisionData missing for Decision page"),
                meta: wpm,
            },
            crate::models::PageType::Pattern => Page::Pattern {
                data: wpm
                    .pattern_data
                    .take()
                    .expect("PatternData missing for Pattern page"),
                meta: wpm,
            },
            crate::models::PageType::Memory => Page::Memory {
                data: wpm
                    .memory_data
                    .take()
                    .expect("MemoryData missing for Memory page"),
                meta: wpm,
            },
            crate::models::PageType::Core => Page::Core { meta: wpm },
            crate::models::PageType::Rule => Page::Rule {
                data: wpm
                    .rule_data
                    .take()
                    .expect("RuleData missing for Rule page"),
                meta: wpm,
            },
            crate::models::PageType::Concept => Page::Concept { meta: wpm },
            crate::models::PageType::Howto => Page::HowTo { meta: wpm },
            crate::models::PageType::Note => Page::Note { meta: wpm },
            crate::models::PageType::Reference => Page::Reference { meta: wpm },
        }
    }
}

impl From<Page> for crate::models::WikiPageMeta {
    fn from(page: Page) -> Self {
        let (page_type, mut meta) = match page {
            Page::Task { mut meta, data } => {
                meta.task_data = Some(data);
                (crate::models::PageType::Task, meta)
            }
            Page::Spec { mut meta, data } => {
                meta.spec_data = Some(data);
                (crate::models::PageType::Spec, meta)
            }
            Page::Decision { mut meta, data } => {
                meta.decision_data = Some(data);
                (crate::models::PageType::Decision, meta)
            }
            Page::Pattern { mut meta, data } => {
                meta.pattern_data = Some(data);
                (crate::models::PageType::Pattern, meta)
            }
            Page::Memory { mut meta, data } => {
                meta.memory_data = Some(data);
                (crate::models::PageType::Memory, meta)
            }
            Page::Rule { mut meta, data } => {
                meta.rule_data = Some(data);
                (crate::models::PageType::Rule, meta)
            }
            Page::Core { meta } => (crate::models::PageType::Core, meta),
            Page::Concept { meta } => (crate::models::PageType::Concept, meta),
            Page::HowTo { meta } => (crate::models::PageType::Howto, meta),
            Page::Note { meta } => (crate::models::PageType::Note, meta),
            Page::Reference { meta } => (crate::models::PageType::Reference, meta),
        };
        meta.page_type = page_type;
        meta
    }
}
