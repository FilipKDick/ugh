use async_trait::async_trait;
use base64::prelude::{BASE64_STANDARD, Engine as _};
use reqwest::{
    Client,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};

use crate::domain::ticket::{Ticket, TicketDraft};
use crate::error::{AppError, AppResult};
use crate::services::IssueTrackerService;

pub struct JiraClient {
    http: Client,
    base_url: Option<String>,
    email: Option<String>,
    token: Option<String>,
    issue_type: String,
}

impl JiraClient {
    pub fn new(
        base_url: Option<String>,
        email: Option<String>,
        token: Option<String>,
        issue_type: String,
    ) -> Self {
        Self {
            http: Client::new(),
            base_url,
            email,
            token,
            issue_type,
        }
    }

    fn api_details(&self) -> AppResult<(&str, &str, &str)> {
        let base_url = self
            .base_url
            .as_deref()
            .ok_or_else(|| AppError::Configuration("Jira base URL not configured".to_string()))?;
        let email = self
            .email
            .as_deref()
            .ok_or_else(|| AppError::Configuration("Jira email not configured".to_string()))?;
        let token = self
            .token
            .as_deref()
            .ok_or_else(|| AppError::Configuration("Jira API token not configured".to_string()))?;
        Ok((base_url, email, token))
    }

    fn auth_header(email: &str, token: &str) -> String {
        let credentials = format!("{email}:{token}");
        let encoded = BASE64_STANDARD.encode(credentials);
        format!("Basic {encoded}")
    }

    fn issue_endpoint(base_url: &str) -> String {
        format!(
            "{}/rest/api/3/issue",
            base_url.trim_end_matches('/').to_string()
        )
    }

    fn browse_url(base_url: &str, key: &str) -> String {
        format!("{}/browse/{}", base_url.trim_end_matches('/'), key)
    }
}

#[async_trait]
impl IssueTrackerService for JiraClient {
    async fn create_ticket(&self, board: &str, draft: TicketDraft) -> AppResult<Ticket> {
        let board_key = board.trim();
        if board_key.is_empty() {
            return Err(AppError::IssueTracker(
                "board key must not be empty".to_string(),
            ));
        }
        if draft.title.trim().is_empty() {
            return Err(AppError::LanguageModel(
                "language model returned an empty title".to_string(),
            ));
        }
        if draft.branch_summary.trim().is_empty() {
            return Err(AppError::LanguageModel(
                "language model returned an empty branch summary".to_string(),
            ));
        }

        let (base_url, email, token) = self.api_details()?;
        let request_body = JiraCreateIssueRequest::new(
            board_key,
            &self.issue_type,
            draft.title.trim(),
            draft.description.trim(),
        );

        let response = self
            .http
            .post(Self::issue_endpoint(base_url))
            .header(AUTHORIZATION, Self::auth_header(email, token))
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|err| AppError::IssueTracker(format!("failed to call Jira: {err}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unable to read response>".to_string());
            return Err(AppError::IssueTracker(format!(
                "Jira responded with {status}: {body}"
            )));
        }

        let payload: JiraCreateIssueResponse = response.json().await.map_err(|err| {
            AppError::IssueTracker(format!("failed to parse Jira response: {err}"))
        })?;

        let key = payload.key;
        let url = payload
            .self_url
            .unwrap_or_else(|| Self::browse_url(base_url, &key));

        Ok(Ticket {
            key,
            url: Some(url),
        })
    }
}

#[derive(Serialize)]
struct JiraCreateIssueRequest {
    fields: JiraCreateIssueFields,
}

impl JiraCreateIssueRequest {
    fn new(project_key: &str, issue_type: &str, summary: &str, description: &str) -> Self {
        Self {
            fields: JiraCreateIssueFields {
                project: JiraProject {
                    key: project_key.to_string(),
                },
                summary: summary.to_string(),
                description: JiraDescription::from_markdown(description),
                issuetype: JiraIssueType {
                    name: issue_type.to_string(),
                },
            },
        }
    }
}

#[derive(Serialize)]
struct JiraCreateIssueFields {
    project: JiraProject,
    summary: String,
    description: JiraDescription,
    issuetype: JiraIssueType,
}

#[derive(Serialize)]
struct JiraProject {
    key: String,
}

#[derive(Serialize)]
struct JiraIssueType {
    name: String,
}

#[derive(Serialize)]
struct JiraDescription {
    #[serde(rename = "type")]
    doc_type: &'static str,
    version: u8,
    content: Vec<JiraDocNode>,
}

impl JiraDescription {
    fn from_markdown(description: &str) -> Self {
        let cleaned = description.replace('\r', "");
        let mut sections = cleaned
            .split("\n\n")
            .map(|section| section.trim())
            .filter(|section| !section.is_empty())
            .collect::<Vec<_>>();

        if sections.is_empty() {
            sections.push("Describe the planned work.");
        }

        let content = sections
            .into_iter()
            .map(|section| {
                let paragraph_text = section.replace('\n', " ").trim().to_string();
                JiraDocNode::paragraph(paragraph_text)
            })
            .collect();

        Self {
            doc_type: "doc",
            version: 1,
            content,
        }
    }
}

#[derive(Serialize)]
struct JiraDocNode {
    #[serde(rename = "type")]
    node_type: &'static str,
    content: Vec<JiraDocText>,
}

impl JiraDocNode {
    fn paragraph(text: String) -> Self {
        Self {
            node_type: "paragraph",
            content: vec![JiraDocText::text(text)],
        }
    }
}

#[derive(Serialize)]
struct JiraDocText {
    #[serde(rename = "type")]
    text_type: &'static str,
    text: String,
}

impl JiraDocText {
    fn text(text: String) -> Self {
        Self {
            text_type: "text",
            text,
        }
    }
}

#[derive(Deserialize)]
struct JiraCreateIssueResponse {
    key: String,
    #[serde(rename = "self")]
    self_url: Option<String>,
}
