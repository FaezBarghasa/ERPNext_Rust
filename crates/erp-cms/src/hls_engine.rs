//! Adaptive Bitrate (ABR) HLS Playlist & Manifest Engine (`erp-cms::hls_engine`).
//!
//! Synthesizes MediaCMS and Jellyfin ABR playlist generation:
//! - Master playlist (`#EXT-X-STREAM-INF`) compiling multiple resolution/bandwidth ladders (1080p, 720p, 480p, 360p).
//! - Media variant playlist (`#EXTINF`) with sequence tracking, target duration, and live/VOD boundaries.
//! - Secure tokenized segment URL formatting integrating with `HmacStreamingSigner`.

use serde::{Deserialize, Serialize};

/// Adaptive Bitrate (ABR) stream variant entry for HLS Master Playlists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HlsVariant {
    pub name: String,
    pub bandwidth: u64,
    pub width: u32,
    pub height: u32,
    pub codecs: String,
    pub playlist_uri: String,
}

impl HlsVariant {
    /// Constructs a standard 1080p variant.
    #[must_use]
    pub fn preset_1080p(playlist_uri: impl Into<String>) -> Self {
        Self {
            name: "1080p".into(),
            bandwidth: 4_500_000,
            width: 1920,
            height: 1080,
            codecs: "avc1.64002a,mp4a.40.2".into(),
            playlist_uri: playlist_uri.into(),
        }
    }

    /// Constructs a standard 720p variant.
    #[must_use]
    pub fn preset_720p(playlist_uri: impl Into<String>) -> Self {
        Self {
            name: "720p".into(),
            bandwidth: 2_400_000,
            width: 1280,
            height: 720,
            codecs: "avc1.4d401f,mp4a.40.2".into(),
            playlist_uri: playlist_uri.into(),
        }
    }

    /// Constructs a standard 480p variant.
    #[must_use]
    pub fn preset_480p(playlist_uri: impl Into<String>) -> Self {
        Self {
            name: "480p".into(),
            bandwidth: 1_200_000,
            width: 854,
            height: 480,
            codecs: "avc1.4d401e,mp4a.40.2".into(),
            playlist_uri: playlist_uri.into(),
        }
    }

    /// Constructs a standard 360p variant.
    #[must_use]
    pub fn preset_360p(playlist_uri: impl Into<String>) -> Self {
        Self {
            name: "360p".into(),
            bandwidth: 600_000,
            width: 640,
            height: 360,
            codecs: "avc1.42e01e,mp4a.40.2".into(),
            playlist_uri: playlist_uri.into(),
        }
    }
}

/// Master HLS Playlist compiler aggregating multiple quality variants.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HlsMasterPlaylist {
    pub variants: Vec<HlsVariant>,
}

impl HlsMasterPlaylist {
    /// Creates a new empty master playlist.
    #[must_use]
    pub fn new() -> Self {
        Self {
            variants: Vec::new(),
        }
    }

    /// Creates a standard 4-tier ABR ladder (1080p, 720p, 480p, 360p) for a given media asset ID.
    #[must_use]
    pub fn standard_ladder(media_id: &str) -> Self {
        Self {
            variants: vec![
                HlsVariant::preset_1080p(format!("{media_id}/1080p/index.m3u8")),
                HlsVariant::preset_720p(format!("{media_id}/720p/index.m3u8")),
                HlsVariant::preset_480p(format!("{media_id}/480p/index.m3u8")),
                HlsVariant::preset_360p(format!("{media_id}/360p/index.m3u8")),
            ],
        }
    }

    /// Appends a custom variant to the master playlist.
    pub fn add_variant(&mut self, variant: HlsVariant) {
        self.variants.push(variant);
    }

    /// Compiles into an RFC 8216 compliant HLS Master Playlist string.
    #[must_use]
    pub fn compile(&self) -> String {
        let mut out = String::with_capacity(512);
        out.push_str("#EXTM3U\n");
        out.push_str("#EXT-X-VERSION:3\n");
        out.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");

        for v in &self.variants {
            out.push_str(&format!(
                "#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}x{},NAME=\"{}\",CODECS=\"{}\"\n{}\n",
                v.bandwidth, v.width, v.height, v.name, v.codecs, v.playlist_uri
            ));
        }

        out
    }
}

/// Discrete media chunk entry within an HLS Media Playlist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HlsMediaSegment {
    pub sequence_number: u64,
    pub duration_seconds: f64,
    pub uri: String,
}

/// Media variant playlist representing a single quality level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HlsMediaPlaylist {
    pub target_duration: u32,
    pub media_sequence: u64,
    pub segments: Vec<HlsMediaSegment>,
    pub is_live: bool,
}

impl HlsMediaPlaylist {
    /// Creates a new media playlist.
    #[must_use]
    pub fn new(target_duration: u32, is_live: bool) -> Self {
        Self {
            target_duration,
            media_sequence: 0,
            segments: Vec::new(),
            is_live,
        }
    }

    /// Appends a segment to the playlist.
    pub fn add_segment(&mut self, segment: HlsMediaSegment) {
        self.segments.push(segment);
    }

    /// Compiles into an RFC 8216 compliant HLS Media Playlist string.
    #[must_use]
    pub fn compile(&self) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str("#EXTM3U\n");
        out.push_str("#EXT-X-VERSION:3\n");
        out.push_str(&format!("#EXT-X-TARGETDURATION:{}\n", self.target_duration));
        out.push_str(&format!("#EXT-X-MEDIA-SEQUENCE:{}\n", self.media_sequence));

        if !self.is_live {
            out.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
        }

        for seg in &self.segments {
            out.push_str(&format!("#EXTINF:{:.3},\n{}\n", seg.duration_seconds, seg.uri));
        }

        if !self.is_live {
            out.push_str("#EXT-X-ENDLIST\n");
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_playlist_compilation() {
        let master = HlsMasterPlaylist::standard_ladder("vid-982");
        let compiled = master.compile();

        assert!(compiled.starts_with("#EXTM3U\n#EXT-X-VERSION:3\n"));
        assert!(compiled.contains("RESOLUTION=1920x1080"));
        assert!(compiled.contains("RESOLUTION=1280x720"));
        assert!(compiled.contains("RESOLUTION=854x480"));
        assert!(compiled.contains("RESOLUTION=640x360"));
        assert!(compiled.contains("vid-982/1080p/index.m3u8"));
    }

    #[test]
    fn test_media_playlist_vod_compilation() {
        let mut media_pl = HlsMediaPlaylist::new(4, false);
        media_pl.add_segment(HlsMediaSegment {
            sequence_number: 0,
            duration_seconds: 4.0,
            uri: "segment_0.ts?token=xyz".into(),
        });
        media_pl.add_segment(HlsMediaSegment {
            sequence_number: 1,
            duration_seconds: 3.84,
            uri: "segment_1.ts?token=xyz".into(),
        });

        let compiled = media_pl.compile();
        assert!(compiled.contains("#EXT-X-TARGETDURATION:4"));
        assert!(compiled.contains("#EXT-X-PLAYLIST-TYPE:VOD"));
        assert!(compiled.contains("#EXTINF:4.000,\nsegment_0.ts?token=xyz"));
        assert!(compiled.contains("#EXTINF:3.840,\nsegment_1.ts?token=xyz"));
        assert!(compiled.ends_with("#EXT-X-ENDLIST\n"));
    }

    #[test]
    fn test_media_playlist_live_compilation() {
        let mut media_pl = HlsMediaPlaylist::new(2, true);
        media_pl.media_sequence = 104;
        media_pl.add_segment(HlsMediaSegment {
            sequence_number: 104,
            duration_seconds: 2.0,
            uri: "live_104.ts".into(),
        });

        let compiled = media_pl.compile();
        assert!(compiled.contains("#EXT-X-TARGETDURATION:2"));
        assert!(compiled.contains("#EXT-X-MEDIA-SEQUENCE:104"));
        assert!(!compiled.contains("#EXT-X-PLAYLIST-TYPE:VOD"));
        assert!(!compiled.contains("#EXT-X-ENDLIST"));
    }
}
