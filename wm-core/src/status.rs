// ─── Page Status ────────────────────────────────────────────

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PageStatus {
    Todo,
    InProgress,
    InReview,
    Done,
    Blocked,
    Cancelled,
    Draft,
    Reviewed,
    Superseded,
    Approved,
}

impl PageStatus {
    /// Return the canonical string representation (kebab-case).
    pub fn as_str(&self) -> &'static str {
        use PageStatus::*;
        match self {
            Todo => "todo",
            InProgress => "in-progress",
            InReview => "in-review",
            Done => "done",
            Blocked => "blocked",
            Cancelled => "cancelled",
            Draft => "draft",
            Reviewed => "reviewed",
            Superseded => "superseded",
            Approved => "approved",
        }
    }

    /// Check if a transition from `self` to `to` is valid.
    /// Returns Ok(()) if valid, Err with a message explaining why not.
    pub fn can_transition_to(&self, to: &PageStatus) -> Result<(), String> {
        use PageStatus::*;
        if self == to {
            return Ok(()); // same state is always allowed (no-op)
        }
        let allowed: &[PageStatus] = match self {
            Draft => &[Todo, Cancelled],
            Todo => &[InProgress, Cancelled],
            InProgress => &[InReview, Blocked, Done, Cancelled],
            InReview => &[Done, InProgress, Cancelled],
            Blocked => &[InProgress, Cancelled],
            Done => &[Reviewed, InProgress, Todo, Cancelled],
            Reviewed => &[Approved, InProgress, Todo, Cancelled],
            Approved => &[Superseded, Cancelled],
            Superseded => &[Cancelled],
            Cancelled => &[Todo],
        };
        if allowed.contains(to) {
            Ok(())
        } else {
            Err(format!(
                "Invalid transition: {} → {}. Allowed: {}",
                self.as_str(),
                to.as_str(),
                allowed
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
    }

    /// Return all statuses that should appear as task board columns.
    pub fn task_board_columns() -> Vec<PageStatus> {
        use PageStatus::*;
        vec![
            Draft,
            Todo,
            InProgress,
            InReview,
            Blocked,
            Done,
            Reviewed,
            Approved,
            Superseded,
            Cancelled,
        ]
    }
}

// ─── Priorities & Confidence ────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
    /// Return the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Urgent => "urgent",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

// ─── Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_status_same_state_is_valid() {
        assert!(PageStatus::Todo.can_transition_to(&PageStatus::Todo).is_ok());
        assert!(PageStatus::Done.can_transition_to(&PageStatus::Done).is_ok());
        assert!(PageStatus::Cancelled.can_transition_to(&PageStatus::Cancelled).is_ok());
    }

    #[test]
    fn test_draft_transitions() {
        let s = PageStatus::Draft;
        assert!(s.can_transition_to(&PageStatus::Todo).is_ok());
        assert!(s.can_transition_to(&PageStatus::Cancelled).is_ok());
        assert!(s.can_transition_to(&PageStatus::Done).is_err());
        assert!(s.can_transition_to(&PageStatus::InProgress).is_err());
    }

    #[test]
    fn test_todo_transitions() {
        let s = PageStatus::Todo;
        assert!(s.can_transition_to(&PageStatus::InProgress).is_ok());
        assert!(s.can_transition_to(&PageStatus::Cancelled).is_ok());
        assert!(s.can_transition_to(&PageStatus::Done).is_err());
        assert!(s.can_transition_to(&PageStatus::InReview).is_err());
    }

    #[test]
    fn test_in_progress_transitions() {
        let s = PageStatus::InProgress;
        assert!(s.can_transition_to(&PageStatus::InReview).is_ok());
        assert!(s.can_transition_to(&PageStatus::Blocked).is_ok());
        assert!(s.can_transition_to(&PageStatus::Done).is_ok());
        assert!(s.can_transition_to(&PageStatus::Cancelled).is_ok());
        assert!(s.can_transition_to(&PageStatus::Approved).is_err());
    }

    #[test]
    fn test_in_review_transitions() {
        let s = PageStatus::InReview;
        assert!(s.can_transition_to(&PageStatus::Done).is_ok());
        assert!(s.can_transition_to(&PageStatus::InProgress).is_ok());
        assert!(s.can_transition_to(&PageStatus::Cancelled).is_ok());
        assert!(s.can_transition_to(&PageStatus::Approved).is_err());
        assert!(s.can_transition_to(&PageStatus::Todo).is_err());
    }

    #[test]
    fn test_blocked_transitions() {
        let s = PageStatus::Blocked;
        assert!(s.can_transition_to(&PageStatus::InProgress).is_ok());
        assert!(s.can_transition_to(&PageStatus::Cancelled).is_ok());
        assert!(s.can_transition_to(&PageStatus::Done).is_err());
    }

    #[test]
    fn test_done_transitions() {
        let s = PageStatus::Done;
        assert!(s.can_transition_to(&PageStatus::Reviewed).is_ok());
        assert!(s.can_transition_to(&PageStatus::InProgress).is_ok());
        assert!(s.can_transition_to(&PageStatus::Todo).is_ok());
        assert!(s.can_transition_to(&PageStatus::Cancelled).is_ok());
        assert!(s.can_transition_to(&PageStatus::Approved).is_err());
    }

    #[test]
    fn test_reviewed_transitions() {
        let s = PageStatus::Reviewed;
        assert!(s.can_transition_to(&PageStatus::Approved).is_ok());
        assert!(s.can_transition_to(&PageStatus::InProgress).is_ok());
        assert!(s.can_transition_to(&PageStatus::Todo).is_ok());
        assert!(s.can_transition_to(&PageStatus::Cancelled).is_ok());
        assert!(s.can_transition_to(&PageStatus::Done).is_err());
    }

    #[test]
    fn test_approved_transitions() {
        let s = PageStatus::Approved;
        assert!(s.can_transition_to(&PageStatus::Superseded).is_ok());
        assert!(s.can_transition_to(&PageStatus::Cancelled).is_ok());
        assert!(s.can_transition_to(&PageStatus::Done).is_err());
    }

    #[test]
    fn test_superseded_transitions() {
        let s = PageStatus::Superseded;
        assert!(s.can_transition_to(&PageStatus::Cancelled).is_ok());
        assert!(s.can_transition_to(&PageStatus::Todo).is_err());
        assert!(s.can_transition_to(&PageStatus::Done).is_err());
    }

    #[test]
    fn test_cancelled_reopen() {
        let s = PageStatus::Cancelled;
        assert!(s.can_transition_to(&PageStatus::Todo).is_ok());
        assert!(s.can_transition_to(&PageStatus::Done).is_err());
        assert!(s.can_transition_to(&PageStatus::InProgress).is_err());
    }

    #[test]
    fn test_page_status_as_str() {
        assert_eq!(PageStatus::Todo.as_str(), "todo");
        assert_eq!(PageStatus::InProgress.as_str(), "in-progress");
        assert_eq!(PageStatus::InReview.as_str(), "in-review");
        assert_eq!(PageStatus::Done.as_str(), "done");
        assert_eq!(PageStatus::Blocked.as_str(), "blocked");
        assert_eq!(PageStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(PageStatus::Draft.as_str(), "draft");
        assert_eq!(PageStatus::Reviewed.as_str(), "reviewed");
        assert_eq!(PageStatus::Approved.as_str(), "approved");
        assert_eq!(PageStatus::Superseded.as_str(), "superseded");
    }

    #[test]
    fn test_priority_as_str() {
        assert_eq!(Priority::Low.as_str(), "low");
        assert_eq!(Priority::Medium.as_str(), "medium");
        assert_eq!(Priority::High.as_str(), "high");
        assert_eq!(Priority::Urgent.as_str(), "urgent");
    }

    #[test]
    fn test_task_board_columns_count() {
        let cols = PageStatus::task_board_columns();
        assert_eq!(cols.len(), 10, "should have 10 status columns");
    }
}
