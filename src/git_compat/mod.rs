use anyhow::{Context, Result};
use git2::Repository as Git2Repository;
use std::path::Path;

pub struct GitCompat {
    pub enabled: bool,
}

impl GitCompat {
    pub fn new(enabled: bool) -> Self {
        GitCompat { enabled }
    }

    /// Check if a directory is a git repository
    pub fn is_git_repo<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().join(".git").exists()
    }

    /// Open a git repository in compatibility mode
    pub fn open_git_repo<P: AsRef<Path>>(path: P) -> Result<Git2Repository> {
        Git2Repository::open(path).context("Failed to open git repository")
    }

    /// Get git status
    pub fn git_status<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
        let repo = Self::open_git_repo(path)?;
        let mut status_lines = Vec::new();

        let statuses = repo.statuses(None)?;
        
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("unknown");
            let status = entry.status();

            if status.is_wt_modified() {
                status_lines.push(format!("  modified: {}", path));
            } else if status.is_wt_new() {
                status_lines.push(format!("  new file: {}", path));
            } else if status.is_wt_deleted() {
                status_lines.push(format!("  deleted: {}", path));
            }
        }

        Ok(status_lines)
    }

    /// Get current git branch
    pub fn git_branch<P: AsRef<Path>>(path: P) -> Result<String> {
        let repo = Self::open_git_repo(path)?;
        let head = repo.head()?;
        
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }

    /// Get git commit history
    pub fn git_log<P: AsRef<Path>>(path: P, limit: usize) -> Result<Vec<GitCommitInfo>> {
        let repo = Self::open_git_repo(path)?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        let mut commits = Vec::new();
        for (i, oid) in revwalk.enumerate() {
            if i >= limit {
                break;
            }

            let oid = oid?;
            let commit = repo.find_commit(oid)?;

            commits.push(GitCommitInfo {
                id: format!("{}", oid),
                author: commit.author().name().unwrap_or("unknown").to_string(),
                message: commit.message().unwrap_or("").to_string(),
                timestamp: commit.time().seconds(),
            });
        }

        Ok(commits)
    }
}

#[derive(Debug, Clone)]
pub struct GitCommitInfo {
    pub id: String,
    pub author: String,
    pub message: String,
    pub timestamp: i64,
}
