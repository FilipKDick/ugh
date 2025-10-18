#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchCategory {
    Feature,
    Fix,
    Quality,
}

impl BranchCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            BranchCategory::Feature => "feature",
            BranchCategory::Fix => "fix",
            BranchCategory::Quality => "quality",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "feature" => Some(BranchCategory::Feature),
            "fix" => Some(BranchCategory::Fix),
            "quality" => Some(BranchCategory::Quality),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BranchName(pub String);

impl BranchName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_parts(category: &BranchCategory, ticket_key: &str, summary: &str) -> Self {
        let clean_ticket = ticket_key.trim();
        let slug = slugify(summary);
        Self(format!("{}/{}/{}", category.as_str(), clean_ticket, slug))
    }
}

fn slugify(input: &str) -> String {
    let clean = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '-' || c == '_' || c == '/' {
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
        "summary".to_string()
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_branch_name() {
        let name = BranchName::from_parts(
            &BranchCategory::Feature,
            "TCK-12",
            "Add Git integration for checkout",
        );
        assert_eq!(
            name.as_str(),
            "feature/TCK-12/add-git-integration-for-checkout"
        );
    }

    #[test]
    fn parses_branch_category() {
        assert_eq!(
            BranchCategory::from_str("feature"),
            Some(BranchCategory::Feature)
        );
        assert_eq!(BranchCategory::from_str("FIX"), Some(BranchCategory::Fix));
        assert_eq!(BranchCategory::from_str("unknown"), None);
    }
}
