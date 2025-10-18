#[derive(Debug, Clone)]
pub struct ChangeSummary {
    pub files_changed: usize,
    pub summary: String,
}

impl ChangeSummary {
    pub fn empty() -> Self {
        Self {
            files_changed: 0,
            summary: String::new(),
        }
    }
}
