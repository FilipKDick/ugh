use std::path::PathBuf;

use async_trait::async_trait;

use crate::domain::branch::BranchName;
use crate::domain::change::ChangeSummary;
use crate::error::{AppError, AppResult};
use crate::services::VersionControlService;

pub struct GitCli {
    workspace_root: PathBuf,
}

impl GitCli {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl VersionControlService for GitCli {
    async fn summarize_changes(&self) -> AppResult<ChangeSummary> {
        // Replace with actual git status parsing.
        let mut summary = ChangeSummary::empty();
        summary.summary = format!(
            "No changes summarized yet for workspace {}.",
            self.workspace_root.display()
        );
        Ok(summary)
    }

    async fn checkout_branch(&self, branch: &BranchName) -> AppResult<()> {
        // Replace with actual `git checkout -b`.
        if branch.as_str().is_empty() {
            return Err(AppError::VersionControl(
                "branch name cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}
