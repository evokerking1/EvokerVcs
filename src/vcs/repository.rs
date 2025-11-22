use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a VCS repository
pub struct Repository {
    pub root: PathBuf,
    pub evk_dir: PathBuf,
    pub objects_dir: PathBuf,
    pub refs_dir: PathBuf,
    pub head_file: PathBuf,
    pub index_file: PathBuf,
    pub hooks_dir: PathBuf,
}

impl Repository {
    /// Initialize a new repository
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        let evk_dir = root.join(".evk");
        
        if evk_dir.exists() {
            return Err(anyhow!("Repository already initialized"));
        }

        // Create directory structure
        fs::create_dir_all(&evk_dir)?;
        
        let objects_dir = evk_dir.join("objects");
        fs::create_dir_all(&objects_dir)?;

        let refs_dir = evk_dir.join("refs");
        fs::create_dir_all(&refs_dir)?;
        fs::create_dir_all(refs_dir.join("heads"))?;
        fs::create_dir_all(refs_dir.join("tags"))?;

        let hooks_dir = evk_dir.join("hooks");
        fs::create_dir_all(&hooks_dir)?;

        let head_file = evk_dir.join("HEAD");
        fs::write(&head_file, "ref: refs/heads/main\n")?;

        let index_file = evk_dir.join("index");

        Ok(Repository {
            root,
            evk_dir,
            objects_dir,
            refs_dir,
            head_file,
            index_file,
            hooks_dir,
        })
    }

    /// Open an existing repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        let evk_dir = root.join(".evk");

        if !evk_dir.exists() {
            return Err(anyhow!("Not a valid EvokerVcs repository"));
        }

        let objects_dir = evk_dir.join("objects");
        let refs_dir = evk_dir.join("refs");
        let head_file = evk_dir.join("HEAD");
        let index_file = evk_dir.join("index");
        let hooks_dir = evk_dir.join("hooks");

        Ok(Repository {
            root,
            evk_dir,
            objects_dir,
            refs_dir,
            head_file,
            index_file,
            hooks_dir,
        })
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head_content = fs::read_to_string(&self.head_file)?;
        if let Some(branch) = head_content.strip_prefix("ref: refs/heads/") {
            Ok(branch.trim().to_string())
        } else {
            Err(anyhow!("HEAD is detached"))
        }
    }

    /// Get the current commit ID
    pub fn current_commit(&self) -> Result<Option<String>> {
        let branch = self.current_branch()?;
        let ref_path = self.refs_dir.join("heads").join(&branch);
        
        if ref_path.exists() {
            let commit_id = fs::read_to_string(ref_path)?.trim().to_string();
            Ok(Some(commit_id))
        } else {
            Ok(None)
        }
    }

    /// Update the current branch to point to a commit
    pub fn update_ref(&self, commit_id: &str) -> Result<()> {
        let branch = self.current_branch()?;
        let ref_path = self.refs_dir.join("heads").join(&branch);
        fs::write(ref_path, format!("{}\n", commit_id))?;
        Ok(())
    }

    /// Create a new branch
    pub fn create_branch(&self, branch_name: &str, start_point: Option<&str>) -> Result<()> {
        let branch_path = self.refs_dir.join("heads").join(branch_name);
        
        if branch_path.exists() {
            return Err(anyhow!("Branch '{}' already exists", branch_name));
        }

        let commit_id = if let Some(commit) = start_point {
            commit.to_string()
        } else {
            self.current_commit()?.ok_or_else(|| anyhow!("No commits yet"))?
        };

        fs::write(branch_path, format!("{}\n", commit_id))?;
        Ok(())
    }

    /// Delete a branch
    pub fn delete_branch(&self, branch_name: &str, force: bool) -> Result<()> {
        let current = self.current_branch()?;
        if current == branch_name {
            return Err(anyhow!("Cannot delete the current branch"));
        }

        let branch_path = self.refs_dir.join("heads").join(branch_name);
        if !branch_path.exists() {
            return Err(anyhow!("Branch '{}' not found", branch_name));
        }

        if !force {
            // TODO: Check if branch is merged
            // For now, just delete
        }

        fs::remove_file(branch_path)?;
        Ok(())
    }

    /// List all branches
    pub fn list_branches(&self) -> Result<Vec<String>> {
        let heads_dir = self.refs_dir.join("heads");
        let mut branches = Vec::new();

        if heads_dir.exists() {
            for entry in fs::read_dir(heads_dir)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    branches.push(name.to_string());
                }
            }
        }

        branches.sort();
        Ok(branches)
    }

    /// Switch to a different branch
    pub fn checkout_branch(&self, branch_name: &str) -> Result<()> {
        let branch_path = self.refs_dir.join("heads").join(branch_name);
        
        if !branch_path.exists() {
            return Err(anyhow!("Branch '{}' does not exist", branch_name));
        }

        // Update HEAD to point to the new branch
        fs::write(&self.head_file, format!("ref: refs/heads/{}\n", branch_name))?;
        
        Ok(())
    }

    /// Create and checkout a new branch
    pub fn checkout_new_branch(&self, branch_name: &str) -> Result<()> {
        self.create_branch(branch_name, None)?;
        self.checkout_branch(branch_name)?;
        Ok(())
    }

    /// Rename a branch
    pub fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<()> {
        let old_path = self.refs_dir.join("heads").join(old_name);
        let new_path = self.refs_dir.join("heads").join(new_name);

        if !old_path.exists() {
            return Err(anyhow!("Branch '{}' not found", old_name));
        }

        if new_path.exists() {
            return Err(anyhow!("Branch '{}' already exists", new_name));
        }

        fs::rename(old_path, new_path)?;

        // Update HEAD if currently on the renamed branch
        let current = self.current_branch().ok();
        if current.as_deref() == Some(old_name) {
            fs::write(&self.head_file, format!("ref: refs/heads/{}\n", new_name))?;
        }

        Ok(())
    }

    /// Create a tag
    pub fn create_tag(&self, tag_name: &str, commit_id: Option<&str>, message: Option<&str>) -> Result<()> {
        let tag_path = self.refs_dir.join("tags").join(tag_name);
        
        if tag_path.exists() {
            return Err(anyhow!("Tag '{}' already exists", tag_name));
        }

        let commit = if let Some(id) = commit_id {
            id.to_string()
        } else {
            self.current_commit()?.ok_or_else(|| anyhow!("No commits yet"))?
        };

        // For lightweight tags, just store the commit ID
        // For annotated tags (with message), we'd create a tag object
        if message.is_some() {
            // TODO: Create annotated tag object
            fs::write(tag_path, format!("{}\n", commit))?;
        } else {
            fs::write(tag_path, format!("{}\n", commit))?;
        }

        Ok(())
    }

    /// Delete a tag
    pub fn delete_tag(&self, tag_name: &str) -> Result<()> {
        let tag_path = self.refs_dir.join("tags").join(tag_name);
        
        if !tag_path.exists() {
            return Err(anyhow!("Tag '{}' not found", tag_name));
        }

        fs::remove_file(tag_path)?;
        Ok(())
    }

    /// List all tags
    pub fn list_tags(&self) -> Result<Vec<String>> {
        let tags_dir = self.refs_dir.join("tags");
        let mut tags = Vec::new();

        if tags_dir.exists() {
            for entry in fs::read_dir(tags_dir)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    tags.push(name.to_string());
                }
            }
        }

        tags.sort();
        Ok(tags)
    }
}
