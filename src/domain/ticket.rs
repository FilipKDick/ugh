#[derive(Debug, Clone)]
pub struct TicketDraft {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Ticket {
    pub key: String,
    pub url: Option<String>,
}
