//! Audioserve-Grade Audiobook Chapters & Cross-Device Playback Position Sync
//!
//! Provides chapter marker management (for M4B, ID3v2 CHAP frames, and podcasts),
//! lightweight client heartbeat sync, and Last-Write-Wins (LWW) cross-device
//! bookmark reconciliation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// An audiobook or podcast chapter marker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioChapter {
    pub id: String,
    pub title: String,
    pub start_ms: u32,
    pub end_ms: u32,
}

/// A playback position bookmark for cross-device resume.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioPlaybackBookmark {
    pub media_id: String,
    pub user_id: String,
    pub device_id: String,
    pub position_ms: u32,
    pub duration_ms: u32,
    pub playback_rate: f32, // e.g. 1.0, 1.25, 1.5, 2.0
    pub updated_at: i64,    // Unix timestamp in seconds
}

/// Manager reconciling cross-device bookmarks.
#[derive(Debug, Default)]
pub struct AudioBookmarkRegistry {
    // Key: (user_id, media_id) -> Bookmark
    pub bookmarks: RwLock<HashMap<(String, String), AudioPlaybackBookmark>>,
    // Key: media_id -> Chapters
    pub chapters: RwLock<HashMap<String, Vec<AudioChapter>>>,
}

impl AudioBookmarkRegistry {
    /// Creates a new registry initialized with sample demo chapters.
    #[must_use]
    pub fn with_demo_chapters() -> Self {
        let registry = Self::default();
        let demo_chapters = vec![
            AudioChapter {
                id: "ch_01".into(),
                title: "Chapter 1: The Rust Revolution".into(),
                start_ms: 0,
                end_ms: 300_000,
            },
            AudioChapter {
                id: "ch_02".into(),
                title: "Chapter 2: Embedded Systems & RTIC".into(),
                start_ms: 300_000,
                end_ms: 720_000,
            },
            AudioChapter {
                id: "ch_03".into(),
                title: "Chapter 3: Zero-Allocation Telemetry".into(),
                start_ms: 720_000,
                end_ms: 1_200_000,
            },
        ];

        {
            let mut chapters_map = registry.chapters.write().unwrap();
            chapters_map.insert("podcast_ep_42".into(), demo_chapters.clone());
            chapters_map.insert("book_rust_concurrency".into(), demo_chapters);
        }

        registry
    }

    /// Saves or updates a bookmark using Last-Write-Wins (LWW) conflict resolution.
    pub fn save_bookmark(&self, bookmark: AudioPlaybackBookmark) -> AudioPlaybackBookmark {
        let key = (bookmark.user_id.clone(), bookmark.media_id.clone());
        let mut map = self.bookmarks.write().unwrap();

        match map.get_mut(&key) {
            Some(existing) => {
                if bookmark.updated_at >= existing.updated_at {
                    *existing = bookmark.clone();
                }
                existing.clone()
            }
            None => {
                map.insert(key, bookmark.clone());
                bookmark
            }
        }
    }

    /// Retrieves the latest playback bookmark for a given user and media.
    #[must_use]
    pub fn get_bookmark(&self, user_id: &str, media_id: &str) -> Option<AudioPlaybackBookmark> {
        let key = (user_id.to_string(), media_id.to_string());
        let map = self.bookmarks.read().unwrap();
        map.get(&key).cloned()
    }

    /// Returns chapters for an audiobook or podcast episode.
    #[must_use]
    pub fn get_chapters(&self, media_id: &str) -> Vec<AudioChapter> {
        let map = self.chapters.read().unwrap();
        map.get(media_id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chapters_and_lww_bookmark_sync() {
        let registry = AudioBookmarkRegistry::with_demo_chapters();
        let chs = registry.get_chapters("podcast_ep_42");
        assert_eq!(chs.len(), 3);
        assert_eq!(chs[0].title, "Chapter 1: The Rust Revolution");

        // Bookmark 1 on phone
        let b1 = AudioPlaybackBookmark {
            media_id: "podcast_ep_42".into(),
            user_id: "alice".into(),
            device_id: "phone".into(),
            position_ms: 150_000,
            duration_ms: 1_200_000,
            playback_rate: 1.5,
            updated_at: 1000,
        };
        registry.save_bookmark(b1);

        // Bookmark 2 on desktop (newer timestamp)
        let b2 = AudioPlaybackBookmark {
            media_id: "podcast_ep_42".into(),
            user_id: "alice".into(),
            device_id: "desktop".into(),
            position_ms: 320_000,
            duration_ms: 1_200_000,
            playback_rate: 1.25,
            updated_at: 1050,
        };
        let saved = registry.save_bookmark(b2);
        assert_eq!(saved.position_ms, 320_000);
        assert_eq!(saved.device_id, "desktop");

        // Stale phone update rejected
        let stale_b = AudioPlaybackBookmark {
            media_id: "podcast_ep_42".into(),
            user_id: "alice".into(),
            device_id: "phone".into(),
            position_ms: 200_000,
            duration_ms: 1_200_000,
            playback_rate: 1.5,
            updated_at: 1020, // older than 1050
        };
        let current = registry.save_bookmark(stale_b);
        assert_eq!(current.position_ms, 320_000);
    }
}
