use serde::{Deserialize, Serialize};

/// Interactive Video Chapter Marker on the playback scrubber HUD.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterMarker {
    pub timestamp_secs: f64,
    pub title: String,
}

/// SVoD Video HUD Controller State (Milestone 5.7).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoHudState {
    pub media_id: String,
    pub current_time_secs: f64,
    pub duration_secs: f64,
    pub is_playing: bool,
    pub chapters: Vec<ChapterMarker>,
    pub comments: Vec<String>,
}

impl VideoHudState {
    /// Creates a new Video HUD state.
    #[must_use]
    pub fn new(media_id: &str, duration_secs: f64, chapters: Vec<ChapterMarker>) -> Self {
        Self {
            media_id: media_id.to_string(),
            current_time_secs: 0.0,
            duration_secs,
            is_playing: false,
            chapters,
            comments: Vec::new(),
        }
    }

    /// Toggles play/pause state.
    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }

    /// Scrubs to a target playback position.
    pub fn seek_to(&mut self, target_secs: f64) {
        self.current_time_secs = target_secs.clamp(0.0, self.duration_secs);
    }

    /// Appends a live comment string to the feed overlay.
    pub fn push_comment(&mut self, comment: String) {
        self.comments.push(comment);
    }
}
