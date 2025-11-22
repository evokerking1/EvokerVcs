use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::vcs::hooks::{HookManager, HookType};
use crate::vcs::index::Index;
use crate::vcs::objects::{Blob, Commit, Object, ObjectId, Tree};
use crate::vcs::repository::Repository;

/// Initialize a new repository
pub fn init<P: AsRef<Path>>(path: P) -> Result<()> {
    let repo = Repository::init(path)?;
    
    // Install sample hooks
    let hook_manager = HookManager::new(repo.hooks_dir.clone());
    hook_manager.install_default_samples()?;
    
    println!("Initialized empty EvokerVcs repository in {:?}", repo.evk_dir);
    Ok(())
}

/// Add files to the staging area
pub fn add<P: AsRef<Path>>(repo_path: P, files: Vec<PathBuf>) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let mut index = Index::load(&repo.index_file)?;

    for file_path in files {
        let full_path = repo.root.join(&file_path);
        
        if !full_path.exists() {
            eprintln!("Warning: {} does not exist, skipping", file_path.display());
            continue;
        }

        if full_path.is_file() {
            add_file(&repo, &mut index, &file_path)?;
        } else if full_path.is_dir() {
            // Recursively add all files in the directory
            add_directory(&repo, &mut index, &file_path)?;
        }
    }

    index.save(&repo.index_file)?;
    Ok(())
}

/// Add a single file to the index
fn add_file(repo: &Repository, index: &mut Index, file_path: &PathBuf) -> Result<()> {
    let full_path = repo.root.join(file_path);
    
    // Read file content
    let content = fs::read(&full_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    // Create blob object
    let blob = Blob::new(content);
    let object = Object::Blob(blob);

    // Write object to database
    let object_id = object.write_to_file(&repo.objects_dir)?;

    // Add to index
    let mode = "100644".to_string(); // Regular file
    index.add(file_path.clone(), object_id.clone(), mode);

    println!("Added {} ({})", file_path.display(), object_id);
    Ok(())
}

/// Recursively add all files in a directory
fn add_directory(repo: &Repository, index: &mut Index, dir_path: &PathBuf) -> Result<()> {
    let full_path = repo.root.join(dir_path);
    
    let entries = fs::read_dir(&full_path)
        .with_context(|| format!("Failed to read directory {}", dir_path.display()))?;

    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        
        // Skip .evk and .git directories
        if let Some(name) = entry_path.file_name() {
            if name == ".evk" || name == ".git" {
                continue;
            }
        }
        
        // Get relative path from repo root
        let relative_path = entry_path.strip_prefix(&repo.root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| entry_path.clone());

        if entry_path.is_file() {
            add_file(repo, index, &relative_path)?;
        } else if entry_path.is_dir() {
            // Recursively add subdirectory
            add_directory(repo, index, &relative_path)?;
        }
    }

    Ok(())
}

/// Commit staged changes
pub fn commit<P: AsRef<Path>>(repo_path: P, message: String, author: String) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let index = Index::load(&repo.index_file)?;

    if index.entries.is_empty() {
        return Err(anyhow::anyhow!("Nothing to commit"));
    }

    // Run pre-commit hook
    let hook_manager = HookManager::new(repo.hooks_dir.clone());
    if !hook_manager.run_hook(HookType::PreCommit, &[])? {
        return Err(anyhow::anyhow!("Pre-commit hook failed"));
    }

    // Create tree from index
    let mut tree = Tree::new();
    for entry in index.get_entries() {
        tree.add_entry(
            entry.mode.clone(),
            entry.path.to_string_lossy().to_string(),
            entry.object_id.clone(),
        );
    }

    // Write tree object
    let tree_object = Object::Tree(tree);
    let tree_id = tree_object.write_to_file(&repo.objects_dir)?;

    // Get parent commit if exists
    let parent = repo.current_commit()?.map(ObjectId::new);

    // Create commit
    let commit = Commit::new(tree_id.clone(), parent, author, message);
    let commit_object = Object::Commit(commit);
    let commit_id = commit_object.write_to_file(&repo.objects_dir)?;

    // Update branch reference
    repo.update_ref(commit_id.as_str())?;

    println!("Created commit {}", commit_id);

    // Run post-commit hook
    let _ = hook_manager.run_hook(HookType::PostCommit, &[]);

    Ok(())
}

/// Show repository status
pub fn status<P: AsRef<Path>>(repo_path: P) -> Result<Vec<String>> {
    let repo = Repository::open(repo_path)?;
    let index = Index::load(&repo.index_file)?;

    let mut status_lines = Vec::new();

    // Show current branch
    let branch = repo.current_branch()?;
    status_lines.push(format!("On branch {}", branch));

    // Show commit info
    if let Ok(Some(commit_id)) = repo.current_commit() {
        status_lines.push(format!("Current commit: {}", commit_id));
    } else {
        status_lines.push("No commits yet".to_string());
    }

    // Show staged files
    if !index.entries.is_empty() {
        status_lines.push(String::new());
        status_lines.push("Changes to be committed:".to_string());
        
        // Check if files existed in previous commit to distinguish new vs modified
        let has_commits = repo.current_commit()?.is_some();
        
        for entry in index.get_entries() {
            // For now, mark as new if no commits exist, otherwise as modified
            let status_type = if has_commits { "modified" } else { "new file" };
            status_lines.push(format!("  {}: {}", status_type, entry.path.display()));
        }
    } else {
        status_lines.push(String::new());
        status_lines.push("Nothing staged for commit".to_string());
    }

    Ok(status_lines)
}

/// Get commit log history
pub fn log<P: AsRef<Path>>(repo_path: P, limit: usize) -> Result<Vec<CommitInfo>> {
    let repo = Repository::open(repo_path)?;
    let mut commits = Vec::new();
    
    // Start from current commit
    let mut current_commit_id = match repo.current_commit()? {
        Some(id) => id,
        None => return Ok(commits), // No commits yet
    };
    
    // Walk through commit history
    for _ in 0..limit {
        let commit_oid = ObjectId::new(current_commit_id.clone());
        
        // Read commit object
        let obj = Object::read_from_file(&repo.objects_dir, &commit_oid)?;
        
        match obj {
            Object::Commit(commit) => {
                commits.push(CommitInfo {
                    id: current_commit_id.clone(),
                    author: commit.author.clone(),
                    message: commit.message.clone(),
                    timestamp: commit.timestamp.timestamp(),
                });
                
                // Move to parent commit if exists
                if let Some(parent_id) = commit.parent {
                    current_commit_id = parent_id.as_str().to_string();
                } else {
                    break; // No more parents
                }
            }
            _ => return Err(anyhow::anyhow!("Expected commit object, got different type")),
        }
    }
    
    Ok(commits)
}

/// Information about a commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub author: String,
    pub message: String,
    pub timestamp: i64,
}
