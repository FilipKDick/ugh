use std::sync::Arc;

use crate::config::AppConfig;
use crate::services::{IssueTrackerService, LanguageModelService, VersionControlService};

#[derive(Clone)]
pub struct AppContext {
    pub config: AppConfig,
    pub version_control: Arc<dyn VersionControlService>,
    pub issue_tracker: Arc<dyn IssueTrackerService>,
    pub language_model: Arc<dyn LanguageModelService>,
}

impl AppContext {
    pub fn new(
        config: AppConfig,
        version_control: Arc<dyn VersionControlService>,
        issue_tracker: Arc<dyn IssueTrackerService>,
        language_model: Arc<dyn LanguageModelService>,
    ) -> Self {
        Self {
            config,
            version_control,
            issue_tracker,
            language_model,
        }
    }
}
