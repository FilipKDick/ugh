use async_trait::async_trait;

use crate::domain::change::ChangeSummary;
use crate::domain::ticket::TicketDraft;
use crate::error::AppResult;

#[async_trait]
pub trait LanguageModelService: Send + Sync {
    async fn draft_ticket(&self, changes: &ChangeSummary) -> AppResult<TicketDraft>;
}
