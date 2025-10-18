#[derive(Debug, Clone)]
pub struct BranchName(pub String);

impl BranchName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
