//! Adaptive Bitrate (ABR) HLS Playlist & Manifest Engine (`erp-cms::hls_engine`).
//!
//! Synthesizes MediaCMS, Disney+ BAMTECH, and Jellyfin ABR playlist generation:
//! - Master playlist (`#EXT-X-STREAM-INF`) compiling multiple resolution/bandwidth ladders (1080p, 720p, 480p, 360p).
//! - Multi-language audio tracks (`#EXT-X-MEDIA:TYPE=AUDIO`) and WebVTT subtitle tracks (`#EXT-X-MEDIA:TYPE=SUBTITLES`).
//! - CMAF fragmented MP4 initialization header mapping (`#EXT-X-MAP:URI="init.mp4"`).
//! - Media variant playlist (`#EXTINF`) with sequence tracking, target duration, and live/VOD boundaries.
//! - Secure tokenized segment URL formatting integrating with `HmacStreamingSigner`.
//! - High-performance in-memory thread-safe `HlsSegmentCache` with bounded capacity.

use crate::security::HmacStreamingSigner;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Adaptive Bitrate (ABR) stream variant entry for HLS Master Playlists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HlsVariant {
    pub name: String,
    pub bandwidth: u64,
    pub width: u32,
    pub height: u32,
    pub codecs: String,
    pub audio_group: Option<String>,
    pub subtitle_group: Option<String>,
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
            audio_group: Some("audio-default".into()),
            subtitle_group: Some("subs-default".into()),
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
            audio_group: Some("audio-default".into()),
            subtitle_group: Some("subs-default".into()),
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
            audio_group: Some("audio-default".into()),
            subtitle_group: Some("subs-default".into()),
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
            audio_group: Some("audio-default".into()),
            subtitle_group: Some("subs-default".into()),
            playlist_uri: playlist_uri.into(),
        }
    }
}

/// Alternative multi-language audio rendition track (Disney+ BAMTECH pattern).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HlsAudioRendition {
    pub group_id: String,
    pub name: String,
    pub language: String,
    pub is_default: bool,
    pub auto_select: bool,
    pub uri: String,
}

/// External WebVTT subtitle track rendition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HlsSubtitleRendition {
    pub group_id: String,
    pub name: String,
    pub language: String,
    pub is_default: bool,
    pub is_forced: bool,
    pub uri: String,
}

/// Master HLS Playlist compiler aggregating variants, audio tracks, and subtitles.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HlsMasterPlaylist {
    pub variants: Vec<HlsVariant>,
    pub audio_renditions: Vec<HlsAudioRendition>,
    pub subtitle_renditions: Vec<HlsSubtitleRendition>,
}

impl HlsMasterPlaylist {
    /// Creates a new empty master playlist.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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
            audio_renditions: vec![HlsAudioRendition {
                group_id: "audio-default".into(),
                name: "English (Default)".into(),
                language: "en".into(),
                is_default: true,
                auto_select: true,
                uri: format!("{media_id}/audio/en/index.m3u8"),
            }],
            subtitle_renditions: vec![HlsSubtitleRendition {
                group_id: "subs-default".into(),
                name: "English CC".into(),
                language: "en".into(),
                is_default: false,
                is_forced: false,
                uri: format!("{media_id}/subs/en/index.m3u8"),
            }],
        }
    }

    /// Appends a custom variant to the master playlist.
    pub fn add_variant(&mut self, variant: HlsVariant) {
        self.variants.push(variant);
    }

    /// Appends an audio rendition track.
    pub fn add_audio_rendition(&mut self, audio: HlsAudioRendition) {
        self.audio_renditions.push(audio);
    }

    /// Appends a subtitle rendition track.
    pub fn add_subtitle_rendition(&mut self, sub: HlsSubtitleRendition) {
        self.subtitle_renditions.push(sub);
    }

    /// Compiles into an RFC 8216 compliant HLS Master Playlist string.
    #[must_use]
    pub fn compile(&self) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str("#EXTM3U\n");
        out.push_str("#EXT-X-VERSION:4\n");
        out.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");

        // 1. Audio renditions
        for a in &self.audio_renditions {
            let def_str = if a.is_default { "YES" } else { "NO" };
            let auto_str = if a.auto_select { "YES" } else { "NO" };
            out.push_str(&format!(
                "#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"{}\",NAME=\"{}\",DEFAULT={},AUTOSELECT={},LANGUAGE=\"{}\",URI=\"{}\"\n",
                a.group_id, a.name, def_str, auto_str, a.language, a.uri
            ));
        }

        // 2. Subtitle renditions
        for s in &self.subtitle_renditions {
            let def_str = if s.is_default { "YES" } else { "NO" };
            let forced_str = if s.is_forced { "YES" } else { "NO" };
            out.push_str(&format!(
                "#EXT-X-MEDIA:TYPE=SUBTITLES,GROUP-ID=\"{}\",NAME=\"{}\",DEFAULT={},FORCED={},LANGUAGE=\"{}\",URI=\"{}\"\n",
                s.group_id, s.name, def_str, forced_str, s.language, s.uri
            ));
        }

        // 3. Stream variants
        for v in &self.variants {
            let mut attrs = format!(
                "#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}x{},NAME=\"{}\",CODECS=\"{}\"",
                v.bandwidth, v.width, v.height, v.name, v.codecs
            );
            if let Some(ref ag) = v.audio_group {
                attrs.push_str(&format!(",AUDIO=\"{ag}\""));
            }
            if let Some(ref sg) = v.subtitle_group {
                attrs.push_str(&format!(",SUBTITLES=\"{sg}\""));
            }
            out.push_str(&format!("{attrs}\n{}\n", v.playlist_uri));
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
    pub has_discontinuity: bool,
}

/// Media variant playlist representing a single quality level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HlsMediaPlaylist {
    pub target_duration: u32,
    pub media_sequence: u64,
    pub init_segment_uri: Option<String>, // CMAF fMP4 #EXT-X-MAP:URI="init.mp4"
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
            init_segment_uri: None,
            segments: Vec::new(),
            is_live,
        }
    }

    /// Sets the CMAF initialization segment header (fMP4 #EXT-X-MAP).
    pub fn set_init_segment(&mut self, uri: impl Into<String>) {
        self.init_segment_uri = Some(uri.into());
    }

    /// Appends a segment to the playlist.
    pub fn add_segment(&mut self, segment: HlsMediaSegment) {
        self.segments.push(segment);
    }

    /// Automatically generates a tokenized VOD playlist with HMAC signed URLs for each segment.
    #[must_use]
    pub fn generate_tokenized(
        media_id: &str,
        variant_name: &str,
        segment_count: usize,
        segment_duration_sec: f64,
        signer: &HmacStreamingSigner,
        expiry_timestamp: u64,
    ) -> Self {
        let mut pl = Self::new(segment_duration_sec.ceil() as u32, false);

        for i in 0..segment_count {
            let seg_token =
                signer.generate_token(&format!("{media_id}:{variant_name}:{i}"), expiry_timestamp);
            let uri = format!("{variant_name}/seg_{i}.ts?token={seg_token}&exp={expiry_timestamp}");
            pl.add_segment(HlsMediaSegment {
                sequence_number: i as u64,
                duration_seconds: segment_duration_sec,
                uri,
                has_discontinuity: false,
            });
        }

        pl
    }

    /// Compiles into an RFC 8216 compliant HLS Media Playlist string.
    #[must_use]
    pub fn compile(&self) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str("#EXTM3U\n");
        out.push_str("#EXT-X-VERSION:4\n");
        out.push_str(&format!("#EXT-X-TARGETDURATION:{}\n", self.target_duration));
        out.push_str(&format!("#EXT-X-MEDIA-SEQUENCE:{}\n", self.media_sequence));

        if !self.is_live {
            out.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
        }

        if let Some(ref init) = self.init_segment_uri {
            out.push_str(&format!("#EXT-X-MAP:URI=\"{}\"\n", init));
        }

        for seg in &self.segments {
            if seg.has_discontinuity {
                out.push_str("#EXT-X-DISCONTINUITY\n");
            }
            out.push_str(&format!(
                "#EXTINF:{:.3},\n{}\n",
                seg.duration_seconds, seg.uri
            ));
        }

        if !self.is_live {
            out.push_str("#EXT-X-ENDLIST\n");
        }

        out
    }
}

/// Key tuple identifying a cached media segment: `(Media ID, Variant Name, Sequence Number)`.
pub type HlsSegmentKey = (String, String, u64);

/// Inner state for bounded in-memory segment cache with automatic FIFO eviction.
#[derive(Default)]
pub struct BoundedSegmentStore {
    segments: HashMap<HlsSegmentKey, Vec<u8>>,
    order: std::collections::VecDeque<HlsSegmentKey>,
    current_bytes: usize,
    max_bytes: usize,
}

/// In-memory thread-safe segment cache with bounded memory capacity for low-latency HLS delivery.
#[derive(Clone)]
pub struct HlsSegmentCache {
    store: Arc<RwLock<BoundedSegmentStore>>,
}

impl Default for HlsSegmentCache {
    fn default() -> Self {
        Self::with_max_bytes(128 * 1024 * 1024) // 128 MB default ceiling
    }
}

impl HlsSegmentCache {
    /// Creates a new segment cache with default 128 MB capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new segment cache with a custom memory ceiling in bytes.
    #[must_use]
    pub fn with_max_bytes(max_bytes: usize) -> Self {
        Self {
            store: Arc::new(RwLock::new(BoundedSegmentStore {
                segments: HashMap::new(),
                order: std::collections::VecDeque::new(),
                current_bytes: 0,
                max_bytes,
            })),
        }
    }

    /// Stores a segment in the cache, evicting oldest segments if capacity is exceeded.
    pub fn insert_segment(
        &self,
        media_id: impl Into<String>,
        variant: impl Into<String>,
        seq: u64,
        data: Vec<u8>,
    ) {
        if let Ok(mut lock) = self.store.write() {
            let key = (media_id.into(), variant.into(), seq);
            let item_len = data.len();

            // Evict oldest entries while exceeding capacity
            while lock.current_bytes + item_len > lock.max_bytes && !lock.order.is_empty() {
                if let Some(oldest_key) = lock.order.pop_front()
                    && let Some(removed_bytes) = lock.segments.remove(&oldest_key)
                {
                    lock.current_bytes = lock.current_bytes.saturating_sub(removed_bytes.len());
                }
            }

            if let Some(prev) = lock.segments.insert(key.clone(), data) {
                lock.current_bytes = lock.current_bytes.saturating_sub(prev.len());
            }
            lock.current_bytes += item_len;
            lock.order.push_back(key);
        }
    }

    /// Retrieves a segment from the cache if present.
    #[must_use]
    pub fn get_segment(&self, media_id: &str, variant: &str, seq: u64) -> Option<Vec<u8>> {
        if let Ok(lock) = self.store.read() {
            lock.segments
                .get(&(media_id.to_string(), variant.to_string(), seq))
                .cloned()
        } else {
            None
        }
    }

    /// Returns the current total memory consumption in bytes.
    #[must_use]
    pub fn current_memory_bytes(&self) -> usize {
        self.store.read().map(|l| l.current_bytes).unwrap_or(0)
    }

    /// Returns the number of cached segments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.store.read().map(|l| l.segments.len()).unwrap_or(0)
    }

    /// Checks if the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_playlist_compilation_with_audio_and_subs() {
        let master = HlsMasterPlaylist::standard_ladder("vid-982");
        let compiled = master.compile();

        assert!(compiled.starts_with("#EXTM3U\n#EXT-X-VERSION:4\n"));
        assert!(compiled.contains("TYPE=AUDIO,GROUP-ID=\"audio-default\""));
        assert!(compiled.contains("TYPE=SUBTITLES,GROUP-ID=\"subs-default\""));
        assert!(compiled.contains("RESOLUTION=1920x1080"));
        assert!(compiled.contains("AUDIO=\"audio-default\""));
        assert!(compiled.contains("SUBTITLES=\"subs-default\""));
        assert!(compiled.contains("vid-982/1080p/index.m3u8"));
    }

    #[test]
    fn test_media_playlist_cmaf_and_discontinuity() {
        let mut media_pl = HlsMediaPlaylist::new(4, false);
        media_pl.set_init_segment("1080p/init.mp4");
        media_pl.add_segment(HlsMediaSegment {
            sequence_number: 0,
            duration_seconds: 4.0,
            uri: "seg_0.m4s".into(),
            has_discontinuity: false,
        });
        media_pl.add_segment(HlsMediaSegment {
            sequence_number: 1,
            duration_seconds: 4.0,
            uri: "seg_1.m4s".into(),
            has_discontinuity: true, // e.g. post-ad splice point
        });

        let compiled = media_pl.compile();
        assert!(compiled.contains("#EXT-X-MAP:URI=\"1080p/init.mp4\""));
        assert!(compiled.contains("#EXT-X-DISCONTINUITY"));
        assert!(compiled.contains("#EXTINF:4.000,\nseg_1.m4s"));
        assert!(compiled.ends_with("#EXT-X-ENDLIST\n"));
    }

    #[test]
    fn test_tokenized_playlist_generation() {
        let signer = HmacStreamingSigner::new(b"secret_key_123");
        let playlist =
            HlsMediaPlaylist::generate_tokenized("vid-101", "720p", 3, 4.0, &signer, 1799999999);

        assert_eq!(playlist.segments.len(), 3);
        assert!(playlist.segments[0].uri.contains("token="));
        assert!(playlist.segments[0].uri.contains("exp=1799999999"));

        let compiled = playlist.compile();
        assert!(compiled.contains("#EXT-X-TARGETDURATION:4"));
        assert!(compiled.contains("720p/seg_0.ts?token="));
    }

    #[test]
    fn test_hls_segment_cache() {
        let cache = HlsSegmentCache::new();
        assert!(cache.is_empty());

        cache.insert_segment("vid-1", "1080p", 0, vec![0x47, 0x40, 0x00, 0x10]);
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get_segment("vid-1", "1080p", 0).unwrap();
        assert_eq!(retrieved, vec![0x47, 0x40, 0x00, 0x10]);

        assert!(cache.get_segment("vid-1", "1080p", 99).is_none());
    }
}
