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
- Never invent work unrelated to the provided changes.
"#;

pub struct GeminiClient {
    http: Client,
    api_key: Option<String>,
    model: String,
}

impl GeminiClient {
    pub fn new(api_key: Option<String>, model: String) -> Self {
        Self {
            http: Client::new(),
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

        let response = self
            .http
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|err| AppError::LanguageModel(format!("failed to call Gemini: {err}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".to_string());
            return Err(AppError::LanguageModel(format!(
                "Gemini request failed with status {status}: {body}"
            )));
        }

        let payload: GenerateContentResponse = response.json().await.map_err(|err| {
            AppError::LanguageModel(format!("failed to parse Gemini response: {err}"))
        })?;

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

        let draft: GeminiDraft = serde_json::from_str(&candidate_text).map_err(|err| {
            AppError::LanguageModel(format!(
                "Gemini produced invalid JSON ({err}): {}",
                candidate_text
            ))
        })?;

        let branch_category =
            BranchCategory::from_str(&draft.branch_category).ok_or_else(|| {
                AppError::LanguageModel(format!(
                    "invalid branch_category from Gemini: {}",
                    draft.branch_category
                ))
            })?;

        let branch_summary = if draft.branch_summary.trim().is_empty() {
            baseline_summary
        } else {
            draft.branch_summary.trim().to_lowercase()
        };

        let title = draft.title.trim();
        if title.is_empty() {
            return Err(AppError::LanguageModel(
                "Gemini returned an empty title".to_string(),
            ));
        }

        let description = draft.description.trim();
        if description.is_empty() {
            return Err(AppError::LanguageModel(
                "Gemini returned an empty description".to_string(),
            ));
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
            "Use concise Markdown in the description.\n",
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
