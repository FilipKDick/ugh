use async_trait::async_trait;

use crate::domain::change::ChangeSummary;
use crate::domain::ticket::TicketDraft;
use crate::error::AppResult;
use crate::services::LanguageModelService;

pub struct GeminiClient;

impl GeminiClient {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LanguageModelService for GeminiClient {
    async fn draft_ticket(&self, changes: &ChangeSummary) -> AppResult<TicketDraft> {
        let title = if changes.summary.is_empty() {
            if changes.files_changed == 0 {
                "Review pending work".to_string()
            } else {
                format!("Review {} updated files", changes.files_changed)
            }
        } else {
            format!("Review changes: {}", changes.summary)
        };
        Ok(TicketDraft {
            title,
            description: "Fill in details via Gemini integration.".to_string(),
        })
    }
}
