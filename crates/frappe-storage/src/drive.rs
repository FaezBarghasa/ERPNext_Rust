use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(String),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err.to_string())
    }
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
    /// Physical file path on disk (if stored in durable disk CAS).
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Content-Addressable Storage (CAS) with SHA-256 deduplication, disk durability, and streaming.
#[derive(Clone)]
pub struct DeduplicatedStorage {
    /// Optional durable disk storage root path (e.g. `./data/storage`).
    /// If None, operates purely in memory.
    root_path: Option<PathBuf>,
    /// Raw payload storage/cache: SHA-256 -> Binary data.
    payloads: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    /// File metadata storage: File ID -> DriveFile.
    files: Arc<RwLock<HashMap<String, DriveFile>>>,
    /// Folder metadata storage: Folder ID -> DriveFolder.
    folders: Arc<RwLock<HashMap<String, DriveFolder>>>,
}

impl Default for DeduplicatedStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl DeduplicatedStorage {
    /// Creates a new storage instance backed by disk storage at `./data/storage`.
    #[must_use]
    pub fn new() -> Self {
        Self::with_path("./data/storage")
    }

    /// Creates a storage instance backed by disk storage at the specified root directory.
    pub fn with_path(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: Some(root_path.into()),
            payloads: Arc::new(RwLock::new(HashMap::new())),
            files: Arc::new(RwLock::new(HashMap::new())),
            folders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates an ephemeral in-memory-only storage instance (useful for unit testing).
    #[must_use]
    pub fn in_memory() -> Self {
        Self {
            root_path: None,
            payloads: Arc::new(RwLock::new(HashMap::new())),
            files: Arc::new(RwLock::new(HashMap::new())),
            folders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the configured root path, if disk persistence is enabled.
    #[must_use]
    pub fn root_path(&self) -> Option<&Path> {
        self.root_path.as_deref()
    }

    /// Stores a binary payload, deduplicating based on SHA-256 content hash.
    /// If disk persistence is enabled, shards by `hash[0..2]` and writes atomically via temporary file rename.
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
        let content_hash: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();

        // 1. Physical disk persistence
        let (file_path_str, disk_duplicate) = if let Some(ref root) = self.root_path {
            let shard = if content_hash.len() >= 2 {
                &content_hash[0..2]
            } else {
                "00"
            };
            let shard_dir = root.join(shard);
            let target_path = shard_dir.join(&content_hash);

            let existed = target_path.exists();
            if !existed {
                std::fs::create_dir_all(&shard_dir)?;
                let tmp_filename = format!(
                    "{}.tmp.{}_{}",
                    content_hash,
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_nanos())
                        .unwrap_or(0)
                );
                let tmp_path = shard_dir.join(tmp_filename);
                std::fs::write(&tmp_path, data)?;
                if let Err(e) = std::fs::rename(&tmp_path, &target_path) {
                    let _ = std::fs::remove_file(&tmp_path);
                    if !target_path.exists() {
                        return Err(StorageError::Io(e.to_string()));
                    }
                }
            }
            let canonical = target_path
                .canonicalize()
                .unwrap_or_else(|_| target_path.clone())
                .to_string_lossy()
                .to_string();
            (Some(canonical), existed)
        } else {
            (None, false)
        };

        // 2. In-memory cache & deduplication check
        let mut payloads = self
            .payloads
            .write()
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;

        let is_duplicate = payloads.contains_key(&content_hash) || disk_duplicate;
        if !payloads.contains_key(&content_hash) {
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
            file_path: file_path_str,
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

        self.read_by_hash(&file.content_hash)
    }

    /// Retrieves file bytes by SHA-256 content hash (from memory cache or disk).
    pub fn read_by_hash(&self, content_hash: &str) -> Result<Vec<u8>, StorageError> {
        // Fast path: memory cache
        {
            let payloads = self
                .payloads
                .read()
                .map_err(|e| StorageError::FileNotFound(e.to_string()))?;
            if let Some(data) = payloads.get(content_hash) {
                return Ok(data.clone());
            }
        }

        // Slow path: check disk CAS
        if let Some(ref root) = self.root_path {
            let shard = if content_hash.len() >= 2 {
                &content_hash[0..2]
            } else {
                "00"
            };
            let target_path = root.join(shard).join(content_hash);
            if target_path.exists() {
                let data = std::fs::read(&target_path)?;
                // Populate memory cache
                if let Ok(mut payloads) = self.payloads.write() {
                    payloads.insert(content_hash.to_string(), data.clone());
                }
                return Ok(data);
            }
        }

        Err(StorageError::FileNotFound(content_hash.to_string()))
    }

    /// Returns the absolute path on disk for a given content hash, if it exists on disk.
    #[must_use]
    pub fn get_file_path(&self, content_hash: &str) -> Option<PathBuf> {
        let root = self.root_path.as_ref()?;
        let shard = if content_hash.len() >= 2 {
            &content_hash[0..2]
        } else {
            "00"
        };
        let target_path = root.join(shard).join(content_hash);
        if target_path.exists() {
            Some(target_path.canonicalize().unwrap_or(target_path))
        } else {
            None
        }
    }

    /// Opens an asynchronous streaming file handle for the target content hash.
    pub async fn open_read_stream(
        &self,
        content_hash: &str,
    ) -> Result<tokio::fs::File, StorageError> {
        if let Some(ref root) = self.root_path {
            let shard = if content_hash.len() >= 2 {
                &content_hash[0..2]
            } else {
                "00"
            };
            let target_path = root.join(shard).join(content_hash);
            if target_path.exists() {
                return tokio::fs::File::open(target_path)
                    .await
                    .map_err(|e| StorageError::Io(e.to_string()));
            }
        }
        Err(StorageError::FileNotFound(content_hash.to_string()))
    }

    /// Checks whether the given content hash exists either in memory or on disk.
    #[must_use]
    pub fn exists(&self, content_hash: &str) -> bool {
        if let Ok(payloads) = self.payloads.read() {
            if payloads.contains_key(content_hash) {
                return true;
            }
        }
        if let Some(ref root) = self.root_path {
            let shard = if content_hash.len() >= 2 {
                &content_hash[0..2]
            } else {
                "00"
            };
            return root.join(shard).join(content_hash).exists();
        }
        false
    }

    /// Retrieves file metadata by file ID.
    pub fn get_file(&self, file_id: &str) -> Option<DriveFile> {
        self.files.read().ok()?.get(file_id).cloned()
    }

    /// Removes file metadata from the storage catalog.
    pub fn delete_file(&self, file_id: &str) -> Result<(), StorageError> {
        let mut files = self
            .files
            .write()
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;
        files
            .remove(file_id)
            .ok_or_else(|| StorageError::FileNotFound(file_id.to_string()))?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;
    use tokio::io::AsyncReadExt;

    #[test]
    fn test_in_memory_deduplication() {
        let storage = DeduplicatedStorage::in_memory();
        assert!(storage.root_path().is_none());

        let data = b"common document payload for in-memory deduplication";
        let (file1, is_dup1) = storage
            .store_file(
                "file_1".into(),
                "doc1.txt".into(),
                None,
                "text/plain".into(),
                "user_a".into(),
                data,
            )
            .expect("store file 1");

        assert!(!is_dup1);
        assert_eq!(file1.file_size, data.len() as u64);
        assert!(file1.file_path.is_none());
        assert_eq!(storage.physical_payload_count(), 1);

        // Store same content under different file ID
        let (file2, is_dup2) = storage
            .store_file(
                "file_2".into(),
                "doc2.txt".into(),
                None,
                "text/plain".into(),
                "user_b".into(),
                data,
            )
            .expect("store file 2");

        assert!(is_dup2);
        assert_eq!(file1.content_hash, file2.content_hash);
        assert_eq!(storage.physical_payload_count(), 1);

        // Verify read by file ID and by hash
        let read1 = storage.read_file("file_1").expect("read file 1");
        assert_eq!(read1, data);
        let read2 = storage.read_file("file_2").expect("read file 2");
        assert_eq!(read2, data);
        let read_hash = storage
            .read_by_hash(&file1.content_hash)
            .expect("read by hash");
        assert_eq!(read_hash, data);

        assert!(storage.exists(&file1.content_hash));
        assert!(!storage.exists("non_existent_hash"));
    }

    #[tokio::test]
    async fn test_disk_durability_and_sharding() {
        let test_dir = std::env::temp_dir().join(format!(
            "erp_cas_test_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));

        let storage = DeduplicatedStorage::with_path(&test_dir);
        assert_eq!(storage.root_path(), Some(test_dir.as_path()));

        let data = b"durable CAS payload with on-disk sharded storage and streaming verification";
        let (file, is_dup) = storage
            .store_file(
                "file_disk_1".into(),
                "invoice.pdf".into(),
                None,
                "application/pdf".into(),
                "accountant".into(),
                data,
            )
            .expect("store file to disk");

        assert!(!is_dup);
        let hash = file.content_hash.clone();
        let shard = &hash[0..2];

        // Verify physical shard directory and file existence
        let shard_dir = test_dir.join(shard);
        assert!(shard_dir.exists(), "Shard directory must exist");
        let physical_file = shard_dir.join(&hash);
        assert!(physical_file.exists(), "Physical file must exist on disk");

        // Verify file path metadata was set
        assert!(file.file_path.is_some());
        let disk_path = storage.get_file_path(&hash).expect("path must exist");
        assert_eq!(disk_path, physical_file.canonicalize().unwrap());

        // Verify read streaming
        let mut stream = storage
            .open_read_stream(&hash)
            .await
            .expect("open read stream");
        let mut stream_bytes = Vec::new();
        stream
            .read_to_end(&mut stream_bytes)
            .await
            .expect("read stream to end");
        assert_eq!(stream_bytes, data);

        // Verify cross-session durability: create new storage instance pointing to same directory with cold memory
        let cold_storage = DeduplicatedStorage::with_path(&test_dir);
        assert_eq!(cold_storage.physical_payload_count(), 0);
        let cold_read = cold_storage
            .read_by_hash(&hash)
            .expect("cold read from disk");
        assert_eq!(cold_read, data);
        assert_eq!(
            cold_storage.physical_payload_count(),
            1,
            "Must be cached into memory after cold read"
        );

        // Verify duplicate detection on disk
        let (_, is_dup2) = cold_storage
            .store_file(
                "file_disk_2".into(),
                "invoice_copy.pdf".into(),
                None,
                "application/pdf".into(),
                "manager".into(),
                data,
            )
            .expect("store duplicate to disk");
        assert!(is_dup2, "Disk-backed file must detect duplicate");

        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_folder_hierarchy_and_file_management() {
        let storage = DeduplicatedStorage::in_memory();

        let folder = DriveFolder {
            id: "folder_finance".into(),
            name: "Finance".into(),
            parent_id: None,
            owner: "admin".into(),
            is_public: false,
        };
        storage.create_folder(folder.clone()).expect("create folder");
        let retrieved = storage.get_folder("folder_finance").expect("get folder");
        assert_eq!(retrieved, folder);

        let data = b"salary slip 2026-10";
        let (file, _) = storage
            .store_file(
                "file_slip".into(),
                "slip.txt".into(),
                Some("folder_finance".into()),
                "text/plain".into(),
                "hr".into(),
                data,
            )
            .expect("store file");
        assert_eq!(file.file_name, "slip.txt");

        let fetched_file = storage.get_file("file_slip").expect("get file");
        assert_eq!(fetched_file.folder_id, Some("folder_finance".into()));

        storage.delete_file("file_slip").expect("delete file");
        assert!(storage.get_file("file_slip").is_none());
    }
}
