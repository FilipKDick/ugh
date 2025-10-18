mod cmd;
mod config;
mod context;
mod domain;
mod error;
mod infra;
mod services;
mod workflow;

use std::sync::Arc;

use clap::{Args, Parser, Subcommand};

use crate::cmd::config::{self as config_cmd, ConfigArgs};
use crate::cmd::ticket::{self, TicketCommandArgs};
use crate::config::{AppConfig, LlmProvider};
use crate::context::AppContext;
use crate::error::AppResult;
use crate::infra::git::GitCli;
use crate::infra::jira::JiraClient;
use crate::infra::llm::GeminiClient;
use crate::services::LanguageModelService;

#[derive(Parser)]
#[command(name = "ugh", author, version, about = "Multi-agent developer CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a ticket from local changes and create a matching branch.
    Ticket(TicketArgs),
    /// Manage CLI configuration.
    Config(ConfigArgs),
}

#[derive(Args)]
struct TicketArgs {
    /// Override the default board configured in the CLI.
    #[arg(short, long)]
    board: Option<String>,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

async fn run() -> AppResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config(args) => {
            config_cmd::run(args.command)?;
            Ok(())
        }
        Commands::Ticket(args) => run_ticket(args).await,
    }
}

async fn run_ticket(args: TicketArgs) -> AppResult<()> {
    let cwd = std::env::current_dir()?;
    let config = AppConfig::load(&cwd)?;

    let gemini_api_key = config.gemini_api_key.clone();
    let gemini_model = config.gemini_model.clone();
    let jira_base_url = config.jira_base_url.clone();
    let jira_email = config.jira_email.clone();
    let jira_token = config.jira_token.clone();
    let jira_issue_type = config.jira_issue_type.clone();

    if jira_base_url.is_none() {
        eprintln!("Warning: Jira base URL not configured; ticket creation and links will fail.");
    }
    if jira_email.is_none() {
        eprintln!("Warning: Jira email not configured; ticket creation will fail.");
    }
    if jira_token.is_none() {
        eprintln!("Warning: Jira token not configured; ticket creation will fail.");
    }
    if config.gemini_api_key.is_none() {
        eprintln!("Warning: Gemini API key not configured; ticket drafting will fail.");
    }

    let language_model: Arc<dyn LanguageModelService> = match &config.llm_provider {
        LlmProvider::Gemini => Arc::new(GeminiClient::new(
            gemini_api_key.clone(),
            gemini_model.clone(),
        )),
        LlmProvider::Custom(provider) => {
            eprintln!(
                "Warning: custom LLM provider '{provider}' not yet implemented, using Gemini fallback."
            );
            Arc::new(GeminiClient::new(
                gemini_api_key.clone(),
                gemini_model.clone(),
            ))
        }
    };

    let git = Arc::new(GitCli::new(config.workspace_root.clone()));
    let issue_tracker = Arc::new(JiraClient::new(
        jira_base_url,
        jira_email,
        jira_token,
        jira_issue_type,
    ));

    let context = AppContext::new(config, git, issue_tracker, language_model);

    let outcome = ticket::run(&context, TicketCommandArgs { board: args.board }).await?;

    println!(
        "Ticket {} created. Branch ready: {}",
        outcome.ticket.key,
        outcome.branch.as_str()
    );
    if let Some(url) = &outcome.ticket.url {
        println!("View ticket: {url}");
    }

    Ok(())
}
