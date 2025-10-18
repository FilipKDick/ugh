use async_trait::async_trait;

use crate::domain::branch::BranchName;
use crate::domain::change::ChangeSummary;
use crate::error::AppResult;

#[derive(Debug, Clone)]
pub enum BranchingStrategy {
    TicketKeyPrefix { prefix: String },
    Raw,
}

impl BranchingStrategy {
    pub fn format_branch(&self, ticket_key: &str, slug: &str) -> BranchName {
        match self {
            BranchingStrategy::TicketKeyPrefix { prefix } => {
                BranchName(format!("{prefix}/{ticket_key}-{slug}"))
            }
            BranchingStrategy::Raw => BranchName(format!("{ticket_key}-{slug}")),
        }
    }
}

#[async_trait]
pub trait VersionControlService: Send + Sync {
    async fn summarize_changes(&self) -> AppResult<ChangeSummary>;
    async fn checkout_branch(&self, branch: &BranchName) -> AppResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_ticket_key_prefix() {
        let strategy = BranchingStrategy::TicketKeyPrefix {
            prefix: "feature".to_string(),
        };
        let branch = strategy.format_branch("TCK-101", "add-cli");
        assert_eq!(branch.as_str(), "feature/TCK-101-add-cli");
    }

    #[test]
    fn formats_raw_strategy() {
        let strategy = BranchingStrategy::Raw;
        let branch = strategy.format_branch("TCK-101", "add-cli");
        assert_eq!(branch.as_str(), "TCK-101-add-cli");
    }
}
