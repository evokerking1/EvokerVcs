use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

/// Represents a Git-like object ID (SHA-1 hash)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(String);

impl ObjectId {
    pub fn new(hash: String) -> Self {
        ObjectId(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_content(content: &[u8]) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(content);
        let result = hasher.finalize();
        ObjectId(format!("{:x}", result))
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Object types in the VCS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Blob => write!(f, "blob"),
            ObjectType::Tree => write!(f, "tree"),
            ObjectType::Commit => write!(f, "commit"),
        }
    }
}

/// Represents a blob object (file content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blob {
    pub content: Vec<u8>,
}

impl Blob {
    pub fn new(content: Vec<u8>) -> Self {
        Blob { content }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let header = format!("blob {}\0", self.content.len());
        let mut result = header.into_bytes();
        result.extend_from_slice(&self.content);
        result
    }

    pub fn id(&self) -> ObjectId {
        ObjectId::from_content(&self.to_bytes())
    }
}

/// Represents a tree entry (file or directory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub id: ObjectId,
}

/// Represents a tree object (directory listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, mode: String, name: String, id: ObjectId) {
        self.entries.push(TreeEntry { mode, name, id });
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut content = Vec::new();
        for entry in &self.entries {
            content.extend_from_slice(entry.mode.as_bytes());
            content.push(b' ');
            content.extend_from_slice(entry.name.as_bytes());
            content.push(b'\0');
            // Store hash as binary (20 bytes), not as string
            let hash_bytes = hex::decode(entry.id.as_str()).expect("Invalid hash");
            content.extend_from_slice(&hash_bytes);
        }

        let header = format!("tree {}\0", content.len());
        let mut result = header.into_bytes();
        result.extend_from_slice(&content);
        result
    }

    pub fn id(&self) -> ObjectId {
        ObjectId::from_content(&self.to_bytes())
    }
}

/// Represents a commit object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub tree: ObjectId,
    pub parent: Option<ObjectId>,
    pub author: String,
    pub committer: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl Commit {
    pub fn new(
        tree: ObjectId,
        parent: Option<ObjectId>,
        author: String,
        message: String,
    ) -> Self {
        let timestamp = Utc::now();
        Commit {
            tree,
            parent,
            author: author.clone(),
            committer: author,
            message,
            timestamp,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut content = String::new();
        content.push_str(&format!("tree {}\n", self.tree));
        if let Some(parent) = &self.parent {
            content.push_str(&format!("parent {}\n", parent));
        }
        content.push_str(&format!("author {} {}\n", self.author, self.timestamp.timestamp()));
        content.push_str(&format!(
            "committer {} {}\n",
            self.committer,
            self.timestamp.timestamp()
        ));
        content.push_str(&format!("\n{}", self.message));

        let header = format!("commit {}\0", content.len());
        let mut result = header.into_bytes();
        result.extend_from_slice(content.as_bytes());
        result
    }

    pub fn id(&self) -> ObjectId {
        ObjectId::from_content(&self.to_bytes())
    }
}

/// Generic object that can be any type
#[derive(Debug, Clone)]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Commit),
}

impl Object {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Object::Blob(blob) => blob.to_bytes(),
            Object::Tree(tree) => tree.to_bytes(),
            Object::Commit(commit) => commit.to_bytes(),
        }
    }

    pub fn id(&self) -> ObjectId {
        match self {
            Object::Blob(blob) => blob.id(),
            Object::Tree(tree) => tree.id(),
            Object::Commit(commit) => commit.id(),
        }
    }

    pub fn write_to_file(&self, objects_dir: &Path) -> Result<ObjectId> {
        let id = self.id();
        let content = self.to_bytes();

        // Compress the content
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&content)?;
        let compressed = encoder.finish()?;

        // Create object directory
        let hash_str = id.as_str();
        let dir = objects_dir.join(&hash_str[0..2]);
        fs::create_dir_all(&dir)?;

        // Write object file
        let file_path = dir.join(&hash_str[2..]);
        fs::write(file_path, compressed)?;

        Ok(id)
    }

    pub fn read_from_file(objects_dir: &Path, id: &ObjectId) -> Result<Self> {
        let hash_str = id.as_str();
        let file_path = objects_dir.join(&hash_str[0..2]).join(&hash_str[2..]);

        let compressed = fs::read(file_path)
            .with_context(|| format!("Failed to read object {}", id))?;

        // Decompress
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut content = Vec::new();
        decoder.read_to_end(&mut content)?;

        // Parse object (simplified for now)
        // In a real implementation, we'd parse the header and content properly
        Ok(Object::Blob(Blob::new(content)))
    }
}
