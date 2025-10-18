use async_trait::async_trait;

use crate::domain::branch::BranchName;
use crate::domain::change::ChangeSummary;
use crate::error::AppResult;

#[async_trait]
pub trait VersionControlService: Send + Sync {
    async fn summarize_changes(&self) -> AppResult<ChangeSummary>;
    async fn checkout_branch(&self, branch: &BranchName) -> AppResult<()>;
}
