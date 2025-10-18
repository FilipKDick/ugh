use async_trait::async_trait;

use crate::domain::branch::BranchCategory;
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
        let description = if changes.summary.is_empty() {
            "Summarize the local modifications before creating the ticket.".to_string()
        } else {
            format!("Summary of uncommitted work:\n{}", changes.summary)
        };

        let branch_category = infer_category(changes);
        let branch_summary = infer_branch_summary(changes);

        let title = match branch_category {
            BranchCategory::Feature => format!("Add {}", branch_summary.replace('-', " ")),
            BranchCategory::Fix => format!("Fix {}", branch_summary.replace('-', " ")),
            BranchCategory::Quality => format!("Improve {}", branch_summary.replace('-', " ")),
        };

        Ok(TicketDraft {
            title,
            description,
            branch_category,
            branch_summary,
        })
    }
}

fn infer_category(changes: &ChangeSummary) -> BranchCategory {
    let lower = changes.summary.to_lowercase();
    if lower.contains("fix") || lower.contains("bug") || lower.contains("error") {
        BranchCategory::Fix
    } else if lower.contains("refactor")
        || lower.contains("cleanup")
        || lower.contains("docs")
        || lower.contains("chore")
    {
        BranchCategory::Quality
    } else {
        BranchCategory::Feature
    }
}

fn infer_branch_summary(changes: &ChangeSummary) -> String {
    let summary = changes.summary.trim();
    if summary.is_empty() {
        return if changes.files_changed == 0 {
            "pending-update".to_string()
        } else {
            format!("update-{}-files", changes.files_changed)
        };
    }

    let words: Vec<String> = summary
        .split_whitespace()
        .take(8)
        .map(|word| {
            word.chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|word| !word.is_empty())
        .collect();

    if words.is_empty() {
        "pending-update".to_string()
    } else {
        words.join("-")
    }
}
