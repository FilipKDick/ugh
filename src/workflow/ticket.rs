use crate::cache::TicketDraftCache;
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

    let cache_key =
        TicketDraftCache::compute_key(&changes.summary, changes.files_changed, Some(&board));
    let mut cache = match TicketDraftCache::load() {
        Ok(cache) => Some(cache),
        Err(err) => {
            eprintln!(
                "Warning: could not load ticket draft cache ({err}). Continuing without cache."
            );
            None
        }
    };

    let draft = match cache.as_mut().and_then(|c| c.get(&cache_key)) {
        Some(cached) => cached,
        None => {
            let generated = ctx.language_model.draft_ticket(&changes).await?;
            if let Some(cache_ref) = cache.as_mut() {
                cache_ref.insert(cache_key.clone(), &generated);
                if let Err(err) = cache_ref.save() {
                    eprintln!("Warning: failed to persist ticket draft cache ({err}).");
                }
            }
            generated
        }
    };

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
