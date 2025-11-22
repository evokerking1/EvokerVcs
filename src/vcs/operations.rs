use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::vcs::index::Index;
use crate::vcs::objects::{Blob, Commit, Object, ObjectId, Tree};
use crate::vcs::repository::Repository;

/// Initialize a new repository
pub fn init<P: AsRef<Path>>(path: P) -> Result<()> {
    let repo = Repository::init(path)?;
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
        } else if full_path.is_dir() {
            // TODO: Handle directories recursively
            eprintln!("Warning: Directory support not yet implemented, skipping {}", file_path.display());
        }
    }

    index.save(&repo.index_file)?;
    Ok(())
}

/// Commit staged changes
pub fn commit<P: AsRef<Path>>(repo_path: P, message: String, author: String) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let index = Index::load(&repo.index_file)?;

    if index.entries.is_empty() {
        return Err(anyhow::anyhow!("Nothing to commit"));
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
        for entry in index.get_entries() {
            status_lines.push(format!("  modified: {}", entry.path.display()));
        }
    } else {
        status_lines.push(String::new());
        status_lines.push("Nothing staged for commit".to_string());
    }

    Ok(status_lines)
}
