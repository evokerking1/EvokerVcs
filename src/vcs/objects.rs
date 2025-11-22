use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use hex;
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

        // Parse object header to determine type
        let null_pos = content.iter().position(|&b| b == 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid object format: no null byte in header"))?;
        
        let header = std::str::from_utf8(&content[..null_pos])
            .with_context(|| "Invalid UTF-8 in object header")?;
        
        let parts: Vec<&str> = header.split(' ').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid object header format"));
        }
        
        let obj_type = parts[0];
        let _size: usize = parts[1].parse()
            .with_context(|| "Invalid size in object header")?;
        
        let data = &content[null_pos + 1..];
        
        match obj_type {
            "blob" => {
                Ok(Object::Blob(Blob::new(data.to_vec())))
            }
            "tree" => {
                // Parse tree entries
                let mut tree = Tree::new();
                let mut pos = 0;
                
                while pos < data.len() {
                    // Read mode
                    let space_pos = data[pos..].iter().position(|&b| b == b' ')
                        .ok_or_else(|| anyhow::anyhow!("Invalid tree entry: no space after mode"))?;
                    let mode = std::str::from_utf8(&data[pos..pos + space_pos])?.to_string();
                    pos += space_pos + 1;
                    
                    // Read name
                    let null_pos = data[pos..].iter().position(|&b| b == 0)
                        .ok_or_else(|| anyhow::anyhow!("Invalid tree entry: no null after name"))?;
                    let name = std::str::from_utf8(&data[pos..pos + null_pos])?.to_string();
                    pos += null_pos + 1;
                    
                    // Read hash (20 bytes for SHA-1)
                    if pos + 20 > data.len() {
                        return Err(anyhow::anyhow!("Invalid tree entry: incomplete hash"));
                    }
                    let hash_bytes = &data[pos..pos + 20];
                    let hash = hex::encode(hash_bytes);
                    pos += 20;
                    
                    tree.add_entry(mode, name, ObjectId::new(hash));
                }
                
                Ok(Object::Tree(tree))
            }
            "commit" => {
                // Parse commit data
                let content_str = std::str::from_utf8(data)
                    .with_context(|| "Invalid UTF-8 in commit object")?;
                
                let mut lines = content_str.lines();
                let mut tree_id = None;
                let mut parent_id = None;
                let mut author = String::new();
                let mut committer = String::new();
                let mut message = String::new();
                let mut in_message = false;
                
                for line in lines {
                    if in_message {
                        if !message.is_empty() {
                            message.push('\n');
                        }
                        message.push_str(line);
                    } else if line.is_empty() {
                        in_message = true;
                    } else if line.starts_with("tree ") {
                        tree_id = Some(ObjectId::new(line[5..].to_string()));
                    } else if line.starts_with("parent ") {
                        parent_id = Some(ObjectId::new(line[7..].to_string()));
                    } else if line.starts_with("author ") {
                        // Format: "author Name Timestamp"
                        let parts: Vec<&str> = line[7..].split_whitespace().collect();
                        if parts.len() >= 2 {
                            // Join all parts except the last one (which is timestamp)
                            author = parts[..parts.len() - 1].join(" ");
                        } else {
                            author = "unknown".to_string();
                        }
                    } else if line.starts_with("committer ") {
                        // Format: "committer Name Timestamp"
                        let parts: Vec<&str> = line[10..].split_whitespace().collect();
                        if parts.len() >= 2 {
                            // Join all parts except the last one (which is timestamp)
                            committer = parts[..parts.len() - 1].join(" ");
                        } else {
                            committer = "unknown".to_string();
                        }
                    }
                }
                
                let tree = tree_id.ok_or_else(|| anyhow::anyhow!("Commit missing tree reference"))?;
                
                let commit = Commit {
                    tree,
                    parent: parent_id,
                    author,
                    committer,
                    message,
                    timestamp: chrono::Utc::now(), // Note: timestamp parsing would need more work
                };
                
                Ok(Object::Commit(commit))
            }
            _ => Err(anyhow::anyhow!("Unknown object type: {}", obj_type))
        }
    }
}
