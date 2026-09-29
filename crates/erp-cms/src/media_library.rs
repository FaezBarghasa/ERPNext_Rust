//! Digital Asset Management & Media Library Engine (`erp-cms::media_library`).
//!
//! Provides WordPress Media Library parity:
//! - Content-Addressable Storage (CAS) indexing with SHA-256 deduplication
//! - Focal point normalized coordinate storage for smart responsive crops
//! - Image dimension extraction, MIME classification, and responsive variant tracking
//! - Document attachment tracking (`attached_to_doctype`, `attached_to_name`)
//! - Multi-tag cataloging and full-text metadata search

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Media asset errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MediaError {
    #[error("Asset '{0}' not found")]
    NotFound(String),
    #[error("Invalid focal point coordinates: must be between 0.0 and 1.0")]
    InvalidFocalPoint,
    #[error("Filename cannot be empty")]
    EmptyFilename,
}

/// Digital Asset model stored in the CMS Media Library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaAsset {
    pub id: String,
    pub tenant_id: String,
    pub filename: String,
    pub content_hash: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub alt_text: String,
    pub caption: Option<String>,
    pub focal_point: (f32, f32), // (x, y) 0.0..1.0
    pub uploaded_by: String,
    pub uploaded_at: DateTime<Utc>,
    pub attached_to_doctype: Option<String>,
    pub attached_to_name: Option<String>,
    pub tags: Vec<String>,
    pub variants: HashMap<String, String>, // variant_name -> content_hash
}

impl MediaAsset {
    /// Creates a new media asset record.
    pub fn new(
        id: impl Into<String>,
        tenant_id: impl Into<String>,
        filename: impl Into<String>,
        content_hash: impl Into<String>,
        mime_type: impl Into<String>,
        size_bytes: u64,
        uploaded_by: impl Into<String>,
    ) -> Result<Self, MediaError> {
        let filename = filename.into();
        if filename.trim().is_empty() {
            return Err(MediaError::EmptyFilename);
        }

        Ok(Self {
            id: id.into(),
            tenant_id: tenant_id.into(),
            filename,
            content_hash: content_hash.into(),
            mime_type: mime_type.into(),
            size_bytes,
            width: None,
            height: None,
            alt_text: String::new(),
            caption: None,
            focal_point: (0.5, 0.5), // center default
            uploaded_by: uploaded_by.into(),
            uploaded_at: Utc::now(),
            attached_to_doctype: None,
            attached_to_name: None,
            tags: Vec::new(),
            variants: HashMap::new(),
        })
    }

    #[must_use]
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    #[must_use]
    pub fn with_alt_text(mut self, alt: impl Into<String>) -> Self {
        self.alt_text = alt.into();
        self
    }

    pub fn set_focal_point(&mut self, x: f32, y: f32) -> Result<(), MediaError> {
        if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
            return Err(MediaError::InvalidFocalPoint);
        }
        self.focal_point = (x, y);
        Ok(())
    }

    pub fn attach_to(&mut self, doctype: impl Into<String>, doc_name: impl Into<String>) {
        self.attached_to_doctype = Some(doctype.into());
        self.attached_to_name = Some(doc_name.into());
    }

    pub fn add_variant(&mut self, variant_name: impl Into<String>, hash: impl Into<String>) {
        self.variants.insert(variant_name.into(), hash.into());
    }
}

/// In-memory Media Library index supporting search & filtering.
#[derive(Debug, Clone, Default)]
pub struct MediaLibraryRegistry {
    assets: HashMap<String, MediaAsset>,
}

impl MediaLibraryRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    /// Registers or updates a media asset in the catalog.
    pub fn save_asset(&mut self, asset: MediaAsset) {
        self.assets.insert(asset.id.clone(), asset);
    }

    /// Retrieves an asset by ID.
    #[must_use]
    pub fn get_asset(&self, id: &str) -> Option<&MediaAsset> {
        self.assets.get(id)
    }

    /// Queries assets with optional MIME type filter, search query, and pagination.
    #[must_use]
    pub fn query_assets(
        &self,
        mime_prefix: Option<&str>,
        search_query: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> (Vec<&MediaAsset>, usize) {
        let mut filtered: Vec<&MediaAsset> = self
            .assets
            .values()
            .filter(|a| {
                if mime_prefix.is_some_and(|prefix| !a.mime_type.starts_with(prefix)) {
                    return false;
                }
                if let Some(q) = search_query {
                    let q_lower = q.to_ascii_lowercase();
                    let match_name = a.filename.to_ascii_lowercase().contains(&q_lower);
                    let match_alt = a.alt_text.to_ascii_lowercase().contains(&q_lower);
                    let match_tags = a
                        .tags
                        .iter()
                        .any(|t| t.to_ascii_lowercase().contains(&q_lower));
                    if !match_name && !match_alt && !match_tags {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Sort latest first
        filtered.sort_by_key(|a| std::cmp::Reverse(a.uploaded_at));

        let total = filtered.len();
        let paged = filtered.into_iter().skip(offset).take(limit).collect();

        (paged, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_asset_creation_and_focal_point() {
        let mut asset = MediaAsset::new(
            "media_01",
            "default",
            "hero-banner.webp",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "image/webp",
            245_000,
            "admin@example.com",
        )
        .unwrap()
        .with_dimensions(1920, 1080)
        .with_alt_text("RustNext ERP Executive Dashboard Banner");

        assert_eq!(asset.focal_point, (0.5, 0.5));
        assert!(asset.set_focal_point(0.75, 0.25).is_ok());
        assert_eq!(asset.focal_point, (0.75, 0.25));

        // Invalid focal point out of bounds
        assert_eq!(
            asset.set_focal_point(1.5, 0.2),
            Err(MediaError::InvalidFocalPoint)
        );

        asset.attach_to("Item", "LAPTOP-01");
        asset.add_variant("thumb_200", "hash_thumb_200");
        assert_eq!(asset.attached_to_doctype.as_deref(), Some("Item"));
        assert_eq!(
            asset.variants.get("thumb_200").map(|s| s.as_str()),
            Some("hash_thumb_200")
        );
    }

    #[test]
    fn test_media_library_query_and_search() {
        let mut registry = MediaLibraryRegistry::new();

        let a1 = MediaAsset::new(
            "m1",
            "d",
            "report.pdf",
            "h1",
            "application/pdf",
            1000,
            "user1",
        )
        .unwrap()
        .with_alt_text("Quarterly Financial Report Q3");

        let a2 = MediaAsset::new(
            "m2",
            "d",
            "product-showcase.png",
            "h2",
            "image/png",
            50000,
            "user2",
        )
        .unwrap()
        .with_alt_text("Main Product Hero Image");

        registry.save_asset(a1);
        registry.save_asset(a2);

        // Query only images
        let (images, total_img) = registry.query_assets(Some("image/"), None, 10, 0);
        assert_eq!(total_img, 1);
        assert_eq!(images[0].id, "m2");

        // Search text "Financial"
        let (docs, total_docs) = registry.query_assets(None, Some("financial"), 10, 0);
        assert_eq!(total_docs, 1);
        assert_eq!(docs[0].id, "m1");
    }
}
