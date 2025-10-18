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

    let cwd = std::env::current_dir()?;
    let config = AppConfig::load(&cwd)?;

    if config.jira_base_url.is_none() {
        eprintln!("Warning: Jira base URL not configured; ticket links may be missing.");
    }
    if config.jira_token.is_none() {
        eprintln!("Warning: Jira token not configured; ticket creation will fail.");
    }

    let language_model: Arc<dyn LanguageModelService> = match &config.llm_provider {
        LlmProvider::Gemini => Arc::new(GeminiClient::new()),
        LlmProvider::Custom(provider) => {
            eprintln!(
                "Warning: custom LLM provider '{provider}' not yet implemented, using Gemini fallback."
            );
            Arc::new(GeminiClient::new())
        }
    };

    let git = Arc::new(GitCli::new(config.workspace_root.clone()));
    let issue_tracker = Arc::new(JiraClient::new());

    let context = AppContext::new(config, git, issue_tracker, language_model);

    match cli.command {
        Commands::Ticket(args) => {
            let outcome = ticket::run(&context, TicketCommandArgs { board: args.board }).await?;

            println!(
                "Ticket {} created. Branch ready: {}",
                outcome.ticket.key,
                outcome.branch.as_str()
            );
            if let Some(url) = &outcome.ticket.url {
                println!("View ticket: {url}");
            }
        }
    }

    Ok(())
}
