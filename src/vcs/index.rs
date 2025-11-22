use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::vcs::objects::ObjectId;

/// Represents an entry in the index (staging area)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub path: PathBuf,
    pub object_id: ObjectId,
    pub mode: String,
}

/// The index/staging area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub entries: HashMap<PathBuf, IndexEntry>,
}

impl Index {
    pub fn new() -> Self {
        Index {
            entries: HashMap::new(),
        }
    }

    /// Load index from file
    pub fn load(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Index::new());
        }

        let content = fs::read_to_string(path)?;
        let index: Index = serde_json::from_str(&content)?;
        Ok(index)
    }

    /// Save index to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Add or update an entry in the index
    pub fn add(&mut self, path: PathBuf, object_id: ObjectId, mode: String) {
        self.entries.insert(
            path.clone(),
            IndexEntry {
                path,
                object_id,
                mode,
            },
        );
    }

    /// Remove an entry from the index
    pub fn remove(&mut self, path: &PathBuf) {
        self.entries.remove(path);
    }

    /// Get all entries
    pub fn get_entries(&self) -> Vec<&IndexEntry> {
        self.entries.values().collect()
    }
}
