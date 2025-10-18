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

    let branch_summary = draft.branch_summary.trim();
    if branch_summary.is_empty() {
        return Err(AppError::LanguageModel(
            "language model returned an empty branch summary".to_string(),
        ));
    }

    let branch_name = BranchName::from_parts(&draft.branch_category, &ticket.key, branch_summary);

    ctx.version_control.checkout_branch(&branch_name).await?;

    Ok(TicketWorkflowOutcome {
        ticket,
        branch: branch_name,
    })
}
