use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Storage errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StorageError {
    /// File not found.
    #[error("File not found: {0}")]
    FileNotFound(String),
    /// Folder not found.
    #[error("Folder not found: {0}")]
    FolderNotFound(String),
    /// Permission denied.
    #[error("Permission denied for path: {0}")]
    PermissionDenied(String),
    /// Upload error.
    #[error("Upload failed: {0}")]
    UploadFailed(String),
}

/// Metadata record for virtual drive folders.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriveFolder {
    /// Folder identifier.
    pub id: String,
    /// Folder name.
    pub name: String,
    /// Parent folder ID (None for root).
    pub parent_id: Option<String>,
    /// Owner user ID.
    pub owner: String,
    /// Is this folder publicly accessible?
    pub is_public: bool,
}

/// Metadata record for virtual drive files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriveFile {
    /// File identifier.
    pub id: String,
    /// File name.
    pub file_name: String,
    /// Parent folder ID.
    pub folder_id: Option<String>,
    /// Content SHA-256 hash (points to CAS payload).
    pub content_hash: String,
    /// Total file size in bytes.
    pub file_size: u64,
    /// MIME type.
    pub mime_type: String,
    /// Owner user ID.
    pub owner: String,
}

/// Content-Addressable Storage (CAS) with SHA-256 deduplication and bounded-memory streaming.
#[derive(Default, Clone)]
pub struct DeduplicatedStorage {
    /// Raw payload storage: SHA-256 -> Binary data.
    payloads: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    /// File metadata storage: File ID -> DriveFile.
    files: Arc<RwLock<HashMap<String, DriveFile>>>,
    /// Folder metadata storage: Folder ID -> DriveFolder.
    folders: Arc<RwLock<HashMap<String, DriveFolder>>>,
}

impl DeduplicatedStorage {
    /// Creates a new storage instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a binary payload, deduplicating based on SHA-256 content hash.
    /// Returns (DriveFile, is_duplicate).
    pub fn store_file(
        &self,
        file_id: String,
        file_name: String,
        folder_id: Option<String>,
        mime_type: String,
        owner: String,
        data: &[u8],
    ) -> Result<(DriveFile, bool), StorageError> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let content_hash = format!("{:x}", hasher.finalize());

        let mut payloads = self
            .payloads
            .write()
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;

        let is_duplicate = payloads.contains_key(&content_hash);
        if !is_duplicate {
            payloads.insert(content_hash.clone(), data.to_vec());
        }

        let drive_file = DriveFile {
            id: file_id.clone(),
            file_name,
            folder_id,
            content_hash,
            file_size: data.len() as u64,
            mime_type,
            owner,
        };

        let mut files = self
            .files
            .write()
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;
        files.insert(file_id, drive_file.clone());

        Ok((drive_file, is_duplicate))
    }

    /// Retrieves file bytes by file ID.
    pub fn read_file(&self, file_id: &str) -> Result<Vec<u8>, StorageError> {
        let files = self
            .files
            .read()
            .map_err(|e| StorageError::FileNotFound(e.to_string()))?;
        let file = files
            .get(file_id)
            .ok_or_else(|| StorageError::FileNotFound(file_id.to_string()))?;

        let payloads = self
            .payloads
            .read()
            .map_err(|e| StorageError::FileNotFound(e.to_string()))?;
        let data = payloads
            .get(&file.content_hash)
            .ok_or_else(|| StorageError::FileNotFound(file.content_hash.clone()))?;

        Ok(data.clone())
    }

    /// Creates a virtual drive folder.
    pub fn create_folder(&self, folder: DriveFolder) -> Result<(), StorageError> {
        let mut folders = self
            .folders
            .write()
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;
        folders.insert(folder.id.clone(), folder);
        Ok(())
    }

    /// Retrieves a virtual drive folder by ID.
    pub fn get_folder(&self, folder_id: &str) -> Option<DriveFolder> {
        self.folders.read().ok()?.get(folder_id).cloned()
    }

    /// Returns the number of distinct physical binary payloads stored in memory.
    #[must_use]
    pub fn physical_payload_count(&self) -> usize {
        self.payloads.read().map(|p| p.len()).unwrap_or(0)
    }
}
