use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const TODO: &str = "todo";
pub const IN_PROGRESS: &str = "in-progress";
pub const IN_REVIEW: &str = "in-review";
pub const DONE: &str = "done";
pub const BLOCKED: &str = "blocked";
pub const CANCELLED: &str = "cancelled";
pub const DRAFT: &str = "draft";
pub const REVIEWED: &str = "reviewed";
pub const SUPERSEDED: &str = "superseded";
pub const APPROVED: &str = "approved";
pub const ACCEPTED: &str = "accepted";
pub const REJECTED: &str = "rejected";
pub const ARCHIVED: &str = "archived";
pub const ACTIVE: &str = "active";
pub const STALE: &str = "stale";

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
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
    Accepted,
    Rejected,
    Archived,
    Active,
    Stale,
}

impl std::fmt::Display for PageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PageStatus {
    pub fn as_str(&self) -> &'static str {
        use PageStatus::*;
        match self {
            Todo => TODO,
            InProgress => IN_PROGRESS,
            InReview => IN_REVIEW,
            Done => DONE,
            Blocked => BLOCKED,
            Cancelled => CANCELLED,
            Draft => DRAFT,
            Reviewed => REVIEWED,
            Superseded => SUPERSEDED,
            Approved => APPROVED,
            Accepted => ACCEPTED,
            Rejected => REJECTED,
            Archived => ARCHIVED,
            Active => ACTIVE,
            Stale => STALE,
        }
    }

    /// Check whether a transition from the current status to `to` is allowed.
    ///
    pub fn can_transition_to(&self, to: &PageStatus) -> Result<(), String> {
        use PageStatus::*;
        if self == to {
            return Ok(());
        }
        let allowed: &[PageStatus] = match self {
            Draft => &[Todo, Cancelled],
            Todo => &[InProgress, Cancelled],
            InProgress => &[InReview, Blocked, Done, Cancelled],
            InReview => &[Done, InProgress, Cancelled],
            Blocked => &[InProgress, Cancelled],
            Done => &[Reviewed, InProgress, Todo, Cancelled],
            Reviewed => &[Approved, InProgress, Todo, Cancelled],
            Approved => &[Superseded, Cancelled, Archived],
            Superseded => &[Cancelled, Archived],
            Cancelled => &[Todo, Active],
            Accepted => &[Archived, Superseded, Cancelled],
            Rejected => &[Active, Todo, Cancelled],
            Archived => &[Active],
            Active => &[Stale, Archived],
            Stale => &[Active, Archived],
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
                    .fold(String::new(), |mut acc, s| {
                        if !acc.is_empty() {
                            acc.push_str(", ");
                        }
                        acc.push_str(s);
                        acc
                    },)
            ))
        }
    }

    pub fn task_board_columns() -> Vec<PageStatus> {
        use PageStatus::*;
        vec![
            Draft, Todo, InProgress, InReview, Blocked, Done, Reviewed, Approved, Accepted,
            Rejected, Superseded, Cancelled, Archived, Active, Stale,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_status_same_state_is_valid() {
        assert!(PageStatus::Todo
            .can_transition_to(&PageStatus::Todo)
            .is_ok());
        assert!(PageStatus::Done
            .can_transition_to(&PageStatus::Done)
            .is_ok());
        assert!(PageStatus::Cancelled
            .can_transition_to(&PageStatus::Cancelled)
            .is_ok());
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
        assert_eq!(PageStatus::Accepted.as_str(), "accepted");
        assert_eq!(PageStatus::Rejected.as_str(), "rejected");
        assert_eq!(PageStatus::Archived.as_str(), "archived");
        assert_eq!(PageStatus::Active.as_str(), "active");
        assert_eq!(PageStatus::Stale.as_str(), "stale");
    }

    #[test]
    fn test_task_board_columns_count() {
        let cols = PageStatus::task_board_columns();
        assert_eq!(cols.len(), 15, "should have 15 status columns");
    }

    #[test]
    fn test_done_to_approved_rejected() {
        let result = PageStatus::Done.can_transition_to(&PageStatus::Approved);
        assert!(result.is_err(), "done -> approved should be rejected");
        assert!(result.unwrap_err().contains("Invalid transition"));
    }

    #[test]
    fn test_non_task_status_not_validated() {
        assert!(PageStatus::Draft
            .can_transition_to(&PageStatus::Todo)
            .is_ok());
        assert!(PageStatus::Reviewed
            .can_transition_to(&PageStatus::Approved)
            .is_ok());
    }
}
