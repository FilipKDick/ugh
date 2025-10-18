use crate::domain::branch::BranchCategory;

#[derive(Debug, Clone)]
pub struct TicketDraft {
    pub title: String,
    pub description: String,
    pub branch_category: BranchCategory,
    pub branch_summary: String,
}

#[derive(Debug, Clone)]
pub struct Ticket {
    pub key: String,
    pub url: Option<String>,
}
