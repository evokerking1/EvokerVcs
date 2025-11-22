use anyhow::Result;
use std::path::PathBuf;

use crate::git_compat::{GitCommitInfo, GitCompat};
use crate::vcs::{operations, Repository};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Status,
    Log,
    Staging,
    Help,
}

/// Unified commit info that works for both git and native formats
#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub id: String,
    pub author: String,
    pub message: String,
    pub timestamp: i64,
}

impl From<GitCommitInfo> for CommitEntry {
    fn from(info: GitCommitInfo) -> Self {
        CommitEntry {
            id: info.id,
            author: info.author,
            message: info.message,
            timestamp: info.timestamp,
        }
    }
}

impl From<operations::CommitInfo> for CommitEntry {
    fn from(info: operations::CommitInfo) -> Self {
        CommitEntry {
            id: info.id,
            author: info.author,
            message: info.message,
            timestamp: info.timestamp,
        }
    }
}

pub struct App {
    pub mode: AppMode,
    pub repo_path: PathBuf,
    pub status_lines: Vec<String>,
    pub log_entries: Vec<CommitEntry>,
    pub selected_index: usize,
    pub should_quit: bool,
    pub git_compat_mode: bool,
    pub message: Option<String>,
}

impl App {
    pub fn new(repo_path: PathBuf) -> Self {
        let git_compat_mode = GitCompat::is_git_repo(&repo_path);
        
        App {
            mode: AppMode::Status,
            repo_path,
            status_lines: Vec::new(),
            log_entries: Vec::new(),
            selected_index: 0,
            should_quit: false,
            git_compat_mode,
            message: None,
        }
    }

    pub fn refresh_status(&mut self) -> Result<()> {
        if self.git_compat_mode {
            self.status_lines = GitCompat::git_status(&self.repo_path)?;
        } else {
            self.status_lines = operations::status(&self.repo_path)?;
        }
        Ok(())
    }

    pub fn refresh_log(&mut self) -> Result<()> {
        if self.git_compat_mode {
            let git_log = GitCompat::git_log(&self.repo_path, 50)?;
            self.log_entries = git_log.into_iter().map(CommitEntry::from).collect();
        } else {
            // Use native EvokerVcs log
            let evk_log = operations::log(&self.repo_path, 50)?;
            self.log_entries = evk_log.into_iter().map(CommitEntry::from).collect();
        }
        Ok(())
    }

    pub fn switch_mode(&mut self, mode: AppMode) {
        self.mode = mode;
        self.selected_index = 0;
        self.message = None;
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        let max_index = match self.mode {
            AppMode::Status => self.status_lines.len().saturating_sub(1),
            AppMode::Log => self.log_entries.len().saturating_sub(1),
            _ => 0,
        };
        
        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn set_message(&mut self, msg: String) {
        self.message = Some(msg);
    }

    pub fn get_current_branch(&self) -> String {
        if self.git_compat_mode {
            GitCompat::git_branch(&self.repo_path).unwrap_or_else(|_| "unknown".to_string())
        } else {
            Repository::open(&self.repo_path)
                .and_then(|r| r.current_branch())
                .unwrap_or_else(|_| "unknown".to_string())
        }
    }
}
