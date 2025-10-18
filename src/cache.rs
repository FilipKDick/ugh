use std::fs;
use std::path::PathBuf;

use blake3::Hasher;
use serde::{Deserialize, Serialize};

use crate::config::config_directory;
use crate::domain::branch::BranchCategory;
use crate::domain::ticket::TicketDraft;
use crate::error::{AppError, AppResult};

const CACHE_FILE_NAME: &str = "draft_cache.json";
const CACHE_LIMIT: usize = 32;

#[derive(Default, Serialize, Deserialize)]
struct CacheFile {
    entries: Vec<CacheEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CacheEntry {
    key: String,
    title: String,
    description: String,
    branch_category: String,
    branch_summary: String,
}

pub struct TicketDraftCache {
    file_path: PathBuf,
    file: CacheFile,
}

impl TicketDraftCache {
    pub fn load() -> AppResult<Self> {
        let dir = config_directory()?;
        let path = dir.join(CACHE_FILE_NAME);
        let file = match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str::<CacheFile>(&contents)
                .map_err(|err| AppError::Configuration(format!("invalid cache file: {err}")))?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => CacheFile::default(),
            Err(err) => return Err(AppError::Io(err)),
        };

        Ok(Self {
            file_path: path,
            file,
        })
    }

    pub fn get(&self, key: &str) -> Option<TicketDraft> {
        self.file
            .entries
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| {
                let category = BranchCategory::from_str(&entry.branch_category)
                    .unwrap_or(BranchCategory::Feature);
                TicketDraft {
                    title: entry.title.clone(),
                    description: entry.description.clone(),
                    branch_category: category,
                    branch_summary: entry.branch_summary.clone(),
                }
            })
    }

    pub fn insert(&mut self, key: String, draft: &TicketDraft) {
        self.file.entries.retain(|entry| entry.key != key);
        self.file.entries.push(CacheEntry {
            key,
            title: draft.title.clone(),
            description: draft.description.clone(),
            branch_category: draft.branch_category.as_str().to_string(),
            branch_summary: draft.branch_summary.clone(),
        });

        if self.file.entries.len() > CACHE_LIMIT {
            let overflow = self.file.entries.len() - CACHE_LIMIT;
            self.file.entries.drain(0..overflow);
        }
    }

    pub fn save(&self) -> AppResult<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(&self.file)
            .map_err(|err| AppError::Configuration(format!("failed to write cache: {err}")))?;
        fs::write(&self.file_path, data)?;
        Ok(())
    }

    pub fn compute_key(summary: &str, files_changed: usize, board: Option<&str>) -> String {
        let mut hasher = Hasher::new();
        hasher.update(summary.as_bytes());
        hasher.update(files_changed.to_string().as_bytes());
        if let Some(board) = board {
            hasher.update(board.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }
}
