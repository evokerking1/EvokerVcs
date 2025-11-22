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

        Ok(Repository {
            root,
            evk_dir,
            objects_dir,
            refs_dir,
            head_file,
            index_file,
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
}
