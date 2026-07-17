#[derive(Clone, Debug)]
pub enum Page {
    Task { meta: crate::WikiPageMeta, data: crate::TaskData },
    Spec { meta: crate::WikiPageMeta, data: crate::SpecData },
    Decision { meta: crate::WikiPageMeta, data: crate::DecisionData },
    Pattern { meta: crate::WikiPageMeta, data: crate::PatternData },
    Memory { meta: crate::WikiPageMeta, data: crate::MemoryData },
    Rule { meta: crate::WikiPageMeta, data: crate::RuleData },
    Concept { meta: crate::WikiPageMeta },
    HowTo { meta: crate::WikiPageMeta },
    Note { meta: crate::WikiPageMeta },
    Reference { meta: crate::WikiPageMeta },
}

impl Page {
    pub fn meta(&self) -> &crate::WikiPageMeta {
        match self {
            Page::Task { meta, .. } | Page::Spec { meta, .. }
            | Page::Decision { meta, .. } | Page::Pattern { meta, .. }
            | Page::Memory { meta, .. } | Page::Rule { meta, .. }
            | Page::Concept { meta }
            | Page::HowTo { meta } | Page::Note { meta }
            | Page::Reference { meta } => meta,
        }
    }

    pub fn meta_mut(&mut self) -> &mut crate::WikiPageMeta {
        match self {
            Page::Task { meta, .. } | Page::Spec { meta, .. }
            | Page::Decision { meta, .. } | Page::Pattern { meta, .. }
            | Page::Memory { meta, .. } | Page::Rule { meta, .. }
            | Page::Concept { meta }
            | Page::HowTo { meta } | Page::Note { meta }
            | Page::Reference { meta } => meta,
        }
    }

    pub fn page_type(&self) -> crate::PageType {
        match self {
            Page::Task { .. } => crate::PageType::Task,
            Page::Spec { .. } => crate::PageType::Spec,
            Page::Decision { .. } => crate::PageType::Decision,
            Page::Pattern { .. } => crate::PageType::Pattern,
            Page::Memory { .. } => crate::PageType::Memory,
            Page::Rule { .. } => crate::PageType::Rule,
            Page::Concept { .. } => crate::PageType::Concept,
            Page::HowTo { .. } => crate::PageType::Howto,
            Page::Note { .. } => crate::PageType::Note,
            Page::Reference { .. } => crate::PageType::Reference,
        }
    }
}

impl From<crate::WikiPageMeta> for Page {
    fn from(mut wpm: crate::WikiPageMeta) -> Self {
        let pt = wpm.page_type.clone();
        match pt {
            crate::PageType::Task => Page::Task {
                data: wpm.task_data.take().expect("TaskData missing for Task page"),
                meta: wpm,
            },
            crate::PageType::Spec => Page::Spec {
                data: wpm.spec_data.take().expect("SpecData missing for Spec page"),
                meta: wpm,
            },
            crate::PageType::Decision => Page::Decision {
                data: wpm.decision_data.take().expect("DecisionData missing for Decision page"),
                meta: wpm,
            },
            crate::PageType::Pattern => Page::Pattern {
                data: wpm.pattern_data.take().expect("PatternData missing for Pattern page"),
                meta: wpm,
            },
            crate::PageType::Memory => Page::Memory {
                data: wpm.memory_data.take().expect("MemoryData missing for Memory page"),
                meta: wpm,
            },
            crate::PageType::Rule => Page::Rule {
                data: wpm.rule_data.take().expect("RuleData missing for Rule page"),
                meta: wpm,
            },
            crate::PageType::Concept => Page::Concept { meta: wpm },
            crate::PageType::Howto => Page::HowTo { meta: wpm },
            crate::PageType::Note => Page::Note { meta: wpm },
            crate::PageType::Reference => Page::Reference { meta: wpm },
        }
    }
}

impl From<Page> for crate::WikiPageMeta {
    fn from(page: Page) -> Self {
        let (page_type, mut meta) = match page {
            Page::Task { mut meta, data } => {
                meta.task_data = Some(data);
                (crate::PageType::Task, meta)
            }
            Page::Spec { mut meta, data } => {
                meta.spec_data = Some(data);
                (crate::PageType::Spec, meta)
            }
            Page::Decision { mut meta, data } => {
                meta.decision_data = Some(data);
                (crate::PageType::Decision, meta)
            }
            Page::Pattern { mut meta, data } => {
                meta.pattern_data = Some(data);
                (crate::PageType::Pattern, meta)
            }
            Page::Memory { mut meta, data } => {
                meta.memory_data = Some(data);
                (crate::PageType::Memory, meta)
            }
            Page::Rule { mut meta, data } => {
                meta.rule_data = Some(data);
                (crate::PageType::Rule, meta)
            }
            Page::Concept { meta } => (crate::PageType::Concept, meta),
            Page::HowTo { meta } => (crate::PageType::Howto, meta),
            Page::Note { meta } => (crate::PageType::Note, meta),
            Page::Reference { meta } => (crate::PageType::Reference, meta),
        };
        meta.page_type = page_type;
        meta
    }
}
