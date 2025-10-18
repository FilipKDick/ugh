use crate::context::AppContext;
use crate::domain::branch::BranchName;
use crate::domain::ticket::Ticket;
use crate::error::{AppError, AppResult};

pub struct TicketWorkflowOutcome {
    pub ticket: Ticket,
    pub branch: BranchName,
}

pub async fn create_ticket_from_changes(
    ctx: &AppContext,
    board_override: Option<String>,
) -> AppResult<TicketWorkflowOutcome> {
    let board = board_override
        .or_else(|| ctx.config.default_board.clone())
        .ok_or_else(|| AppError::Configuration("no board configured".to_string()))?;

    let changes = ctx.version_control.summarize_changes().await?;
    let draft = ctx.language_model.draft_ticket(&changes).await?;

    if draft.description.trim().is_empty() {
        return Err(AppError::LanguageModel(
            "language model returned an empty description".to_string(),
        ));
    }

    let ticket = ctx
        .issue_tracker
        .create_ticket(&board, draft.clone())
        .await?;

    let slug = slugify(&draft.title);
    let branch_name = match &ctx.config.branch_strategy {
        strategy => strategy.format_branch(&ticket.key, &slug),
    };

    ctx.version_control.checkout_branch(&branch_name).await?;

    Ok(TicketWorkflowOutcome {
        ticket,
        branch: branch_name,
    })
}

fn slugify(input: &str) -> String {
    let clean = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>();

    let trimmed = clean.trim_matches('-');
    let mut result = String::with_capacity(trimmed.len());
    let mut prev_dash = false;
    for ch in trimmed.chars() {
        if ch == '-' {
            if !prev_dash {
                result.push(ch);
            }
            prev_dash = true;
        } else {
            result.push(ch);
            prev_dash = false;
        }
    }
    if result.is_empty() {
        "work-item".to_string()
    } else {
        result
    }
}
