use std::env;
use std::path::{Path, PathBuf};

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub jira_base_url: Option<String>,
    pub jira_token: Option<String>,
    pub jira_email: Option<String>,
    pub default_board: Option<String>,
    pub llm_provider: LlmProvider,
    pub workspace_root: PathBuf,
    pub gemini_api_key: Option<String>,
    pub gemini_model: String,
    pub jira_issue_type: String,
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

        let jira_base_url = env::var("UGH_JIRA_BASE_URL").ok();
        let jira_token = env::var("UGH_JIRA_TOKEN").ok();
        let jira_email = env::var("UGH_JIRA_EMAIL").ok();
        let default_board = env::var("UGH_JIRA_DEFAULT_BOARD").ok();

        let gemini_api_key = env::var("UGH_GEMINI_API_KEY").ok();
        let gemini_model =
            env::var("UGH_GEMINI_MODEL").unwrap_or_else(|_| "gemini-1.5-flash-latest".to_string());
        let jira_issue_type =
            env::var("UGH_JIRA_ISSUE_TYPE").unwrap_or_else(|_| "Task".to_string());

        Ok(Self {
            jira_base_url,
            jira_token,
            jira_email,
            default_board,
            llm_provider,
            workspace_root: workspace_hint.to_path_buf(),
            gemini_api_key,
            gemini_model,
            jira_issue_type,
        })
    }
}
