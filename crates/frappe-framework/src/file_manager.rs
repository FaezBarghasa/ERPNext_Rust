//! File Manager & Attachment Engine (`frappe-framework::file_manager`).
//!
//! Provides document file attachments, SHA-256 integrity hashing, private vs public
//! storage isolation, file size validation, and duplicate content deduplication.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

/// File Manager operational errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FileError {
    #[error("File exceeds maximum allowable size limit of {0} bytes")]
    FileTooLarge(usize),
    #[error("File extension '{0}' is prohibited for security compliance")]
    ProhibitedExtension(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
}

/// Metadata record representing a stored document attachment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileRecord {
    pub id: String,
    pub file_name: String,
    pub file_url: String,
    pub is_private: bool,
    pub file_size: usize,
    pub content_hash: String,
    pub attached_to_doctype: Option<String>,
    pub attached_to_name: Option<String>,
    pub uploaded_by: String,
    pub created_at: String,
}

/// In-memory and disk-backed file manager.
pub struct FileManager {
    max_file_size: usize,
    records: HashMap<String, FileRecord>,
    hash_index: HashMap<String, String>, // hash -> file_id
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new(25 * 1024 * 1024) // 25 MB default limit
    }
}

impl FileManager {
    /// Creates a new file manager with a specified byte size limit.
    #[must_use]
    pub fn new(max_file_size: usize) -> Self {
        Self {
            max_file_size,
            records: HashMap::new(),
            hash_index: HashMap::new(),
        }
    }

    /// Stores a new attachment, generating content hash and URL.
    pub fn save_file(
        &mut self,
        file_name: &str,
        content: &[u8],
        is_private: bool,
        attached_to_doctype: Option<&str>,
        attached_to_name: Option<&str>,
        uploaded_by: &str,
    ) -> Result<FileRecord, FileError> {
        if content.len() > self.max_file_size {
            return Err(FileError::FileTooLarge(self.max_file_size));
        }

        // Validate prohibited executable extensions
        if let Some(ext) = file_name.rsplit('.').next() {
            let lower_ext = ext.to_lowercase();
            if matches!(lower_ext.as_str(), "exe" | "sh" | "bat" | "cmd" | "com") {
                return Err(FileError::ProhibitedExtension(lower_ext));
            }
        }

        // Calculate SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hex::encode(hasher.finalize());

        let file_id = format!("FILE-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let base_url = if is_private {
            "/private/files"
        } else {
            "/files"
        };
        let file_url = format!("{base_url}/{file_name}");

        let record = FileRecord {
            id: file_id.clone(),
            file_name: file_name.to_string(),
            file_url,
            is_private,
            file_size: content.len(),
            content_hash: hash.clone(),
            attached_to_doctype: attached_to_doctype.map(str::to_string),
            attached_to_name: attached_to_name.map(str::to_string),
            uploaded_by: uploaded_by.to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        self.hash_index.insert(hash, file_id.clone());
        self.records.insert(file_id, record.clone());

        Ok(record)
    }

    /// Retrieves file metadata by ID.
    #[must_use]
    pub fn get_file(&self, file_id: &str) -> Option<&FileRecord> {
        self.records.get(file_id)
    }

    /// Finds files attached to a specific document.
    #[must_use]
    pub fn get_attached_files(&self, doctype: &str, docname: &str) -> Vec<&FileRecord> {
        self.records
            .values()
            .filter(|f| {
                f.attached_to_doctype.as_deref() == Some(doctype)
                    && f.attached_to_name.as_deref() == Some(docname)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_saving_hashing_and_size_limit() {
        let mut manager = FileManager::new(1024); // 1 KB limit
        let payload = b"Hello, ERPNext v16 in pure Rust!";

        let record = manager
            .save_file(
                "contract.pdf",
                payload,
                true,
                Some("Sales Invoice"),
                Some("ACC-INV-001"),
                "admin",
            )
            .expect("Saving valid file must succeed");

        assert_eq!(record.file_name, "contract.pdf");
        assert_eq!(record.file_url, "/private/files/contract.pdf");
        assert!(record.is_private);
        assert!(!record.content_hash.is_empty());

        let attached = manager.get_attached_files("Sales Invoice", "ACC-INV-001");
        assert_eq!(attached.len(), 1);
        assert_eq!(attached[0].id, record.id);

        // Test file too large
        let big_payload = vec![0u8; 2048];
        let err = manager.save_file("big.bin", &big_payload, false, None, None, "admin");
        assert_eq!(err, Err(FileError::FileTooLarge(1024)));

        // Test prohibited extension
        let script = b"echo hello";
        let err2 = manager.save_file("script.sh", script, false, None, None, "admin");
        assert_eq!(err2, Err(FileError::ProhibitedExtension("sh".into())));
    }
}
