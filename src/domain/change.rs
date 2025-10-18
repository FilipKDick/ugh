#[derive(Debug, Clone)]
pub struct ChangeSummary {
    pub files_changed: usize,
    pub summary: String,
}
