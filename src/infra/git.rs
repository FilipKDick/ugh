use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};

use async_trait::async_trait;
use tokio::process::Command;

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

    async fn exec_git(&self, args: &[&str]) -> AppResult<GitCommandOutput> {
        let mut command = Command::new("git");
        command.current_dir(&self.workspace_root);
        command.args(args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let output = command
            .output()
            .await
            .map_err(|err| AppError::VersionControl(format!("failed to run git: {err}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(GitCommandOutput {
            stdout,
            stderr,
            status: output.status,
        })
    }

    async fn run_git_checked(&self, args: &[&str]) -> AppResult<String> {
        let output = self.exec_git(args).await?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(AppError::VersionControl(format!(
                "git {} failed: {}",
                args.join(" "),
                output.stderr.trim()
            )))
        }
    }

    async fn current_branch(&self) -> Option<String> {
        let output = self.exec_git(&["rev-parse", "--abbrev-ref", "HEAD"]).await;
        match output {
            Ok(result) if result.status.success() => Some(result.stdout.trim().to_string()),
            _ => None,
        }
    }

    async fn branch_exists(&self, branch: &str) -> AppResult<bool> {
        let ref_name = format!("refs/heads/{branch}");
        let args = ["show-ref", "--verify", "--quiet", ref_name.as_str()];
        let output = self.exec_git(&args).await?;
        Ok(output.status.success())
    }
}

#[async_trait]
impl VersionControlService for GitCli {
    async fn summarize_changes(&self) -> AppResult<ChangeSummary> {
        let status_output = self.run_git_checked(&["status", "--short"]).await?;

        let files_changed = status_output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();

        let diff_stat = if files_changed == 0 {
            String::new()
        } else {
            self.run_git_checked(&["diff", "--stat=200"])
                .await
                .unwrap_or_default()
        };

        let branch = self
            .current_branch()
            .await
            .unwrap_or_else(|| "HEAD".to_string());

        let summary = if files_changed == 0 {
            format!("Branch {branch} has no uncommitted changes.")
        } else {
            let mut lines = Vec::new();
            lines.push(format!(
                "Branch {branch} has {files_changed} file(s) with local changes."
            ));

            for entry in status_output
                .lines()
                .filter(|line| !line.trim().is_empty())
                .take(8)
            {
                lines.push(format!("  {entry}"));
            }

            if files_changed > 8 {
                lines.push("  …".to_string());
            }

            let diff_stat_lines: Vec<&str> = diff_stat
                .lines()
                .filter(|line| !line.trim().is_empty())
                .collect();

            if !diff_stat_lines.is_empty() {
                lines.push(String::new());
                lines.push("Diff summary:".to_string());
                lines.extend(
                    diff_stat_lines
                        .into_iter()
                        .take(8)
                        .map(|line| format!("  {line}")),
                );
            }

            lines.join("\n")
        };

        Ok(ChangeSummary {
            files_changed,
            summary,
        })
    }

    async fn checkout_branch(&self, branch: &BranchName) -> AppResult<()> {
        if branch.as_str().is_empty() {
            return Err(AppError::VersionControl(
                "branch name cannot be empty".to_string(),
            ));
        }

        if self.branch_exists(branch.as_str()).await? {
            self.run_git_checked(&["checkout", branch.as_str()]).await?;
        } else {
            self.run_git_checked(&["checkout", "-b", branch.as_str()])
                .await?;
        }

        Ok(())
    }
}

struct GitCommandOutput {
    stdout: String,
    stderr: String,
    status: ExitStatus,
}
