use std::io::{self, Write};

use clap::{Args, Subcommand};

use crate::config::{StoredConfig, config_file_path};
use crate::error::AppResult;

#[derive(Args, Debug, Clone)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommand {
    /// Run the interactive configuration wizard.
    Init,
    /// Show the stored configuration (secrets masked).
    Show,
}

pub fn run(command: ConfigCommand) -> AppResult<()> {
    match command {
        ConfigCommand::Init => run_init(),
        ConfigCommand::Show => run_show(),
    }
}

fn run_init() -> AppResult<()> {
    let mut cfg = StoredConfig::load()?;

    println!("Configuring ugh CLI.");
    println!("Press Enter to keep the current value, '-' to clear it.");
    println!("Secrets are stored in the local config file; protect your filesystem accordingly.");
    println!();

    apply_prompt(
        "Jira base URL (e.g., https://company.atlassian.net)",
        &mut cfg.jira_base_url,
        false,
    )?;
    apply_prompt("Jira email", &mut cfg.jira_email, false)?;
    apply_prompt("Jira API token", &mut cfg.jira_token, true)?;
    apply_prompt(
        "Default Jira board/project key",
        &mut cfg.default_board,
        false,
    )?;
    apply_prompt("Default Jira issue type", &mut cfg.jira_issue_type, false)?;

    apply_prompt("LLM provider (gemini/custom)", &mut cfg.llm_provider, false)?;
    apply_prompt("Gemini API key", &mut cfg.gemini_api_key, true)?;
    apply_prompt("Gemini model", &mut cfg.gemini_model, false)?;

    cfg.save()?;

    let path = config_file_path()?;
    println!("\nConfiguration saved to {}", path.display());
    Ok(())
}

fn run_show() -> AppResult<()> {
    let cfg = StoredConfig::load()?;
    let path = config_file_path()?;

    println!("Configuration file: {}", path.display());
    println!("Jira base URL: {}", display_value(&cfg.jira_base_url));
    println!("Jira email: {}", display_value(&cfg.jira_email));
    println!("Jira API token: {}", mask_secret(&cfg.jira_token));
    println!("Default board: {}", display_value(&cfg.default_board));
    println!(
        "Default issue type: {}",
        display_value(&cfg.jira_issue_type)
    );
    println!("LLM provider: {}", display_value(&cfg.llm_provider));
    println!("Gemini API key: {}", mask_secret(&cfg.gemini_api_key));
    println!("Gemini model: {}", display_value(&cfg.gemini_model));

    Ok(())
}

fn apply_prompt(field: &str, target: &mut Option<String>, secret: bool) -> AppResult<()> {
    match prompt(field, target.as_deref(), secret)? {
        PromptAction::Keep => {}
        PromptAction::Clear => *target = None,
        PromptAction::Set(value) => *target = Some(value),
    }
    Ok(())
}

fn prompt(field: &str, current: Option<&str>, secret: bool) -> AppResult<PromptAction> {
    let mut stdout = io::stdout();

    match (current, secret) {
        (Some(_), true) => write!(stdout, "{field} [****] (Enter to keep, '-' to clear): ")?,
        (Some(value), false) => {
            write!(stdout, "{field} [{value}] (Enter to keep, '-' to clear): ")?
        }
        (None, _) => write!(stdout, "{field} (Enter to skip): ")?,
    }
    stdout.flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    if trimmed.is_empty() {
        Ok(PromptAction::Keep)
    } else if trimmed == "-" {
        Ok(PromptAction::Clear)
    } else {
        Ok(PromptAction::Set(trimmed.to_string()))
    }
}

fn display_value(value: &Option<String>) -> String {
    value
        .as_deref()
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
        .unwrap_or_else(|| "<not set>".to_string())
}

fn mask_secret(value: &Option<String>) -> String {
    match value {
        Some(token) if token.len() > 6 => {
            let prefix = &token[..3];
            let suffix = &token[token.len() - 3..];
            format!("{prefix}***{suffix}")
        }
        Some(token) if !token.is_empty() => "***".to_string(),
        _ => "<not set>".to_string(),
    }
}

enum PromptAction {
    Keep,
    Clear,
    Set(String),
}
