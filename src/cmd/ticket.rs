use crate::context::AppContext;
use crate::error::AppResult;
use crate::workflow::ticket::{TicketWorkflowOutcome, create_ticket_from_changes};

#[derive(Debug, Clone)]
pub struct TicketCommandArgs {
    pub board: Option<String>,
}

pub async fn run(ctx: &AppContext, args: TicketCommandArgs) -> AppResult<TicketWorkflowOutcome> {
    create_ticket_from_changes(ctx, args.board).await
}
