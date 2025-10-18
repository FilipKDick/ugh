use async_trait::async_trait;

use crate::domain::ticket::{Ticket, TicketDraft};
use crate::error::{AppError, AppResult};
use crate::services::IssueTrackerService;

pub struct JiraClient;

impl JiraClient {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl IssueTrackerService for JiraClient {
    async fn create_ticket(&self, board: &str, draft: TicketDraft) -> AppResult<Ticket> {
        if board.is_empty() {
            return Err(AppError::IssueTracker(
                "board key must not be empty".to_string(),
            ));
        }
        if draft.title.trim().is_empty() {
            return Err(AppError::LanguageModel(
                "language model returned an empty title".to_string(),
            ));
        }

        Ok(Ticket {
            key: format!("{}-{}", board, 1),
            url: None,
        })
    }
}
