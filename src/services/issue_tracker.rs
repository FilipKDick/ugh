use async_trait::async_trait;

use crate::domain::ticket::{Ticket, TicketDraft};
use crate::error::AppResult;

#[async_trait]
pub trait IssueTrackerService: Send + Sync {
    async fn create_ticket(&self, board: &str, draft: TicketDraft) -> AppResult<Ticket>;
}
