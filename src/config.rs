use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const CONFIG_FILE_NAME: &str = "config.json";

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredConfig {
    pub jira_base_url: Option<String>,
    pub jira_token: Option<String>,
    pub jira_email: Option<String>,
    pub default_board: Option<String>,
    pub llm_provider: Option<String>,
    pub gemini_api_key: Option<String>,
    pub gemini_model: Option<String>,
    pub jira_issue_type: Option<String>,
}

impl StoredConfig {
    pub fn load() -> AppResult<Self> {
        let path = config_file_path()?;
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents)
                .map_err(|err| AppError::Configuration(format!("invalid config file: {err}"))),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(AppError::Io(err)),
        }
    }

    pub fn save(&self) -> AppResult<()> {
        let path = config_file_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|err| AppError::Configuration(format!("failed to serialize config: {err}")))?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum LlmProvider {
    Gemini,
    Custom(String),
}

impl LlmProvider {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "gemini" => Some(LlmProvider::Gemini),
            other if !other.is_empty() => Some(LlmProvider::Custom(other.to_string())),
            _ => None,
        }
    }
}

impl AppConfig {
    pub fn load(workspace_hint: &Path) -> AppResult<Self> {
        let stored = StoredConfig::load()?;

        let jira_base_url = env::var("UGH_JIRA_BASE_URL")
            .ok()
            .or(stored.jira_base_url.clone());
        let jira_token = env::var("UGH_JIRA_TOKEN")
            .ok()
            .or(stored.jira_token.clone());
        let jira_email = env::var("UGH_JIRA_EMAIL")
            .ok()
            .or(stored.jira_email.clone());
        let default_board = env::var("UGH_JIRA_DEFAULT_BOARD")
            .ok()
            .or(stored.default_board.clone());

        let llm_provider = env::var("UGH_LLM_PROVIDER")
            .ok()
            .or(stored.llm_provider.clone())
            .and_then(|value| LlmProvider::from_str(&value))
            .unwrap_or(LlmProvider::Gemini);

        let gemini_api_key = env::var("UGH_GEMINI_API_KEY")
            .ok()
            .or(stored.gemini_api_key.clone());
        let gemini_model = env::var("UGH_GEMINI_MODEL")
            .ok()
            .or(stored.gemini_model.clone())
            .unwrap_or_else(|| "gemini-2.5-flash".to_string());
        let jira_issue_type = env::var("UGH_JIRA_ISSUE_TYPE")
            .ok()
            .or(stored.jira_issue_type.clone())
            .unwrap_or_else(|| "Task".to_string());

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

pub fn config_file_path() -> AppResult<PathBuf> {
    let dir = config_directory()?;
    Ok(dir.join(CONFIG_FILE_NAME))
}

pub fn config_directory() -> AppResult<PathBuf> {
    if let Ok(custom) = env::var("UGH_CONFIG_DIR") {
        return Ok(PathBuf::from(custom));
    }

    if let Some(dir) = platform_config_dir() {
        return Ok(dir);
    }

    Err(AppError::Configuration(
        "could not determine configuration directory".to_string(),
    ))
}

fn platform_config_dir() -> Option<PathBuf> {
    if let Ok(path) = env::var("XDG_CONFIG_HOME") {
        if !path.trim().is_empty() {
            return Some(PathBuf::from(path).join("ugh"));
        }
    }

    if cfg!(target_os = "macos") {
        if let Some(home) = home_dir() {
            return Some(home.join("Library").join("Application Support").join("ugh"));
        }
    } else if cfg!(target_os = "windows") {
        if let Ok(appdata) = env::var("APPDATA") {
            if !appdata.trim().is_empty() {
                return Some(PathBuf::from(appdata).join("ugh"));
            }
        }
    } else if let Some(home) = home_dir() {
        return Some(home.join(".config").join("ugh"));
    }

    home_dir().map(|home| home.join(".ugh"))
}

fn home_dir() -> Option<PathBuf> {
    if let Ok(path) = env::var("HOME") {
        if !path.trim().is_empty() {
            return Some(PathBuf::from(path));
        }
    }

    #[cfg(windows)]
    {
        if let Ok(path) = env::var("USERPROFILE") {
            if !path.trim().is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    None
}
