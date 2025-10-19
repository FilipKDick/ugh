use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::branch::BranchCategory;
use crate::domain::change::ChangeSummary;
use crate::domain::ticket::TicketDraft;
use crate::error::{AppError, AppResult};
use crate::services::LanguageModelService;

const GEMINI_SYSTEM_PROMPT: &str = r#"
You are an assistant for a developer CLI. Given local git change summaries, draft a Jira ticket
and git branch metadata. Respond with VALID JSON only, no markdown, no commentary.

Rules:
- Keys: title, description, branch_category, branch_summary.
- branch_category must be one of: "feature", "fix", "quality".
- branch_summary must be a short, lower-case slug (hyphen-separated words <= 6 words).
- description should be concise Markdown (bullets or short paragraphs) that references the planned work.
- Keep title under 80 characters and actionable.
- Ignore test changes if non-test changes exist.
- Never invent work unrelated to the provided changes.
"#;

pub struct GeminiClient {
    http: Client,
    api_key: Option<String>,
    model: String,
}

impl GeminiClient {
    pub fn new(api_key: Option<String>, model: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .expect("failed to build HTTP client");
        Self {
            http,
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LanguageModelService for GeminiClient {
    async fn draft_ticket(&self, changes: &ChangeSummary) -> AppResult<TicketDraft> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Configuration("Gemini API key not configured".to_string()))?;

        let baseline_category = heuristic_category(changes);
        let baseline_summary = heuristic_summary(changes);
        let user_prompt = build_user_prompt(changes, &baseline_category, &baseline_summary);

        let request = GenerateContentRequest {
            system_instruction: Some(Instruction::new(GEMINI_SYSTEM_PROMPT)),
            contents: vec![Content::user(user_prompt)],
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, api_key
        );

        let response = match self.http.post(&url).json(&request).send().await {
            Ok(resp) => resp,
            Err(err) => {
                eprintln!("Warning: Gemini request failed ({err}); using heuristic ticket.");
                return Ok(heuristic_ticket(changes));
            }
        };

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".to_string());
            eprintln!(
                "Warning: Gemini request returned {status}; falling back to heuristic ticket. Body: {body}"
            );
            return Ok(heuristic_ticket(changes));
        }

        let payload: GenerateContentResponse = match response.json().await {
            Ok(payload) => payload,
            Err(err) => {
                eprintln!(
                    "Warning: failed to parse Gemini response ({err}); using heuristic ticket."
                );
                return Ok(heuristic_ticket(changes));
            }
        };

        let candidate_text = payload
            .candidates
            .into_iter()
            .filter_map(|candidate| candidate.content)
            .flat_map(|content| content.parts.into_iter())
            .filter_map(|part| part.text)
            .map(|text| text.trim().to_string())
            .find(|text| !text.is_empty())
            .ok_or_else(|| {
                AppError::LanguageModel("Gemini returned an empty response".to_string())
            })?;

        let normalized = normalize_json_blob(&candidate_text);
        let draft: GeminiDraft = match serde_json::from_str(&normalized) {
            Ok(draft) => draft,
            Err(err) => {
                eprintln!(
                    "Warning: Gemini produced invalid JSON ({err}); using heuristic ticket. Payload: {}",
                    candidate_text
                );
                return Ok(heuristic_ticket(changes));
            }
        };

        let branch_category = match BranchCategory::from_str(&draft.branch_category) {
            Some(category) => category,
            None => {
                eprintln!(
                    "Warning: Gemini returned invalid branch_category '{}'; using heuristic ticket.",
                    draft.branch_category
                );
                return Ok(heuristic_ticket(changes));
            }
        };

        let branch_summary = if draft.branch_summary.trim().is_empty() {
            baseline_summary
        } else {
            draft.branch_summary.trim().to_lowercase()
        };

        let title = draft.title.trim();
        if title.is_empty() {
            eprintln!("Warning: Gemini returned empty title; using heuristic ticket.");
            return Ok(heuristic_ticket(changes));
        }

        let description = draft.description.trim();
        if description.is_empty() {
            eprintln!("Warning: Gemini returned empty description; using heuristic ticket.");
            return Ok(heuristic_ticket(changes));
        }

        Ok(TicketDraft {
            title: title.to_string(),
            description: description.to_string(),
            branch_category,
            branch_summary,
        })
    }
}

fn build_user_prompt(
    changes: &ChangeSummary,
    baseline_category: &BranchCategory,
    baseline_summary: &str,
) -> String {
    let summary = if changes.summary.trim().is_empty() {
        "(no diff summary provided)".to_string()
    } else {
        changes.summary.trim().to_string()
    };

    format!(
        concat!(
            "Git status summary:\n{}\n\n",
            "Files changed: {}\n\n",
            "Return only JSON with keys: title, description, branch_category, branch_summary.\n",
            "branch_category must be feature, fix, or quality.\n",
            "branch_summary must be a short hyphenated slug (<=6 words).\n",
            "Use concise Markdown in the description. Do not list changed files in the description.\n",
            "The description should be a backward engineered Jira ticket, not a changelog.\n",
            "Ignore pure test-only changes when other files are touched; mention tests as follow-up if needed.\n",
            "Heuristic hint -> category: {}, summary: {}.\n",
            "If information is missing, make conservative assumptions and mention follow-up items."
        ),
        summary,
        changes.files_changed,
        baseline_category.as_str(),
        baseline_summary
    )
}

fn heuristic_category(changes: &ChangeSummary) -> BranchCategory {
    let lower = changes.summary.to_lowercase();
    if lower.contains("fix") || lower.contains("bug") || lower.contains("error") {
        BranchCategory::Fix
    } else if lower.contains("refactor")
        || lower.contains("cleanup")
        || lower.contains("docs")
        || lower.contains("chore")
    {
        BranchCategory::Quality
    } else {
        BranchCategory::Feature
    }
}

fn heuristic_summary(changes: &ChangeSummary) -> String {
    let summary = changes.summary.trim();
    if summary.is_empty() {
        return if changes.files_changed == 0 {
            "pending-update".to_string()
        } else {
            format!("update-{}-files", changes.files_changed)
        };
    }

    let words: Vec<String> = summary
        .split_whitespace()
        .take(8)
        .map(|word| {
            word.chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|word| !word.is_empty())
        .collect();

    if words.is_empty() {
        "pending-update".to_string()
    } else {
        words.join("-")
    }
}

fn heuristic_ticket(changes: &ChangeSummary) -> TicketDraft {
    let branch_category = heuristic_category(changes);
    let branch_summary = heuristic_summary(changes);
    let description = if changes.summary.is_empty() {
        "Summarize the local modifications before creating the ticket.".to_string()
    } else {
        format!("Summary of uncommitted work:\n{}", changes.summary)
    };

    let title = match branch_category {
        BranchCategory::Feature => format!("Add {}", branch_summary.replace('-', " ")),
        BranchCategory::Fix => format!("Fix {}", branch_summary.replace('-', " ")),
        BranchCategory::Quality => format!("Improve {}", branch_summary.replace('-', " ")),
    };

    TicketDraft {
        title,
        description,
        branch_category,
        branch_summary,
    }
}

#[derive(Serialize)]
struct GenerateContentRequest {
    #[serde(rename = "system_instruction")]
    system_instruction: Option<Instruction>,
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Instruction {
    parts: Vec<Part>,
}

impl Instruction {
    fn new(text: &str) -> Self {
        Self {
            parts: vec![Part {
                text: text.to_string(),
            }],
        }
    }
}

#[derive(Serialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

impl Content {
    fn user(text: String) -> Self {
        Self {
            role: "user".to_string(),
            parts: vec![Part { text }],
        }
    }
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GenerateContentResponse {
    #[serde(default)]
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
}

#[derive(Deserialize)]
struct CandidateContent {
    #[serde(default)]
    parts: Vec<CandidatePart>,
}

#[derive(Deserialize)]
struct CandidatePart {
    text: Option<String>,
}

#[derive(Deserialize)]
struct GeminiDraft {
    title: String,
    description: String,
    branch_category: String,
    branch_summary: String,
}

fn normalize_json_blob(input: &str) -> String {
    let mut trimmed = input.trim();
    if trimmed.starts_with("```") {
        trimmed = trimmed.trim_start_matches("```");
        trimmed = trimmed.trim_start_matches(|c: char| c.is_whitespace());
        if trimmed.len() >= 4 && trimmed[..4].eq_ignore_ascii_case("json") {
            trimmed = &trimmed[4..];
            trimmed = trimmed.trim_start_matches(|c: char| c.is_whitespace());
        }
        trimmed = trimmed.trim_end();
        if let Some(stripped) = trimmed.strip_suffix("```") {
            trimmed = stripped.trim_end();
        }
    }

    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        trimmed[start..=end].to_string()
    } else {
        trimmed.to_string()
    }
}
