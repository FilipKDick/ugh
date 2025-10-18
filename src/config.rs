use std::env;
use std::path::{Path, PathBuf};

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub jira_base_url: Option<String>,
    pub jira_token: Option<String>,
    pub default_board: Option<String>,
    pub llm_provider: LlmProvider,
    pub workspace_root: PathBuf,
}

#[derive(Debug, Clone)]
pub enum LlmProvider {
    Gemini,
    Custom(String),
}

impl AppConfig {
    pub fn load(workspace_hint: &Path) -> AppResult<Self> {
        // Placeholder loading logic; replace with actual config parsing.
        let llm_provider = env::var("UGH_LLM_PROVIDER")
            .ok()
            .map(|provider| match provider.to_lowercase().as_str() {
                "gemini" => LlmProvider::Gemini,
                other => LlmProvider::Custom(other.to_string()),
            })
            .unwrap_or(LlmProvider::Gemini);

        Ok(Self {
            // TODO: Load from environment variables or config file.
            jira_base_url: None,
            jira_token: None,
            default_board: None,
            llm_provider,
            workspace_root: workspace_hint.to_path_buf(),
        })
    }
}
