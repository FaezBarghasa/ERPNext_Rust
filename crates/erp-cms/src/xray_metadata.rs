//! X-Ray Timed Interactive Metadata Layer (`erp-cms::xray_metadata`).
//!
//! Synthesizes the Amazon Prime Video X-Ray interactive architecture:
//! - Synchronized scene metadata keyed to precise playback playhead intervals $[t_{\text{start}}, t_{\text{end}}]$.
//! - Cast & Actor credits with character names, profile photos, bios, and filmography.
//! - Music track recognition with artist, album, album art, and streaming service deep links (Spotify/Apple Music).
//! - Curated scene trivia, filming locations, and behind-the-scenes notes with spoiler flags.
//! - Contextual commerce tags (shoppable products, character apparel, and merch) with instant checkout links.
//! - Sub-millisecond interval queries ($\mathcal{O}(\log N)$ / interval scanning) for live interactive overlays.
//! - Dynamic generation of RFC 8216 WebVTT metadata tracks (`kind="metadata"`) for HLS player consumption.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Bounded time interval $[t_{\text{start}}, t_{\text{end}}]$ in fractional seconds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct XRayTimeRange {
    pub start_seconds: f64,
    pub end_seconds: f64,
}

impl XRayTimeRange {
    /// Creates a new validated time range.
    #[must_use]
    pub fn new(start_seconds: f64, end_seconds: f64) -> Self {
        let (start, end) = if start_seconds <= end_seconds {
            (start_seconds, end_seconds)
        } else {
            (end_seconds, start_seconds)
        };
        Self {
            start_seconds: start,
            end_seconds: end,
        }
    }

    /// Evaluates whether a given playhead position falls within this interval (inclusive).
    #[must_use]
    pub fn contains(&self, playhead_seconds: f64) -> bool {
        playhead_seconds >= self.start_seconds && playhead_seconds <= self.end_seconds
    }

    /// Duration of the active interval in seconds.
    #[must_use]
    pub fn duration_seconds(&self) -> f64 {
        self.end_seconds - self.start_seconds
    }

    /// Formats seconds into standard WebVTT timestamp format `HH:MM:SS.mmm`.
    #[must_use]
    pub fn format_vtt_timestamp(seconds: f64) -> String {
        let total_ms = (seconds * 1000.0).max(0.0).round() as u64;
        let ms = total_ms % 1000;
        let total_sec = total_ms / 1000;
        let s = total_sec % 60;
        let total_min = total_sec / 60;
        let m = total_min % 60;
        let h = total_min / 60;

        format!("{h:02}:{m:02}:{s:02}.{ms:03}")
    }

    /// Formats the range into a WebVTT cue header `00:01:10.000 --> 00:01:25.000`.
    #[must_use]
    pub fn to_vtt_cue_header(&self) -> String {
        format!(
            "{} --> {}",
            Self::format_vtt_timestamp(self.start_seconds),
            Self::format_vtt_timestamp(self.end_seconds)
        )
    }
}

/// Actor and character attribution for active on-screen talent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActorCredit {
    pub actor_id: String,
    pub name: String,
    pub character_name: String,
    pub profile_image_url: Option<String>,
    pub bio: Option<String>,
    pub filmography: Vec<String>,
    pub time_range: XRayTimeRange,
}

/// Synchronized soundtrack and music recognition entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicTrackInfo {
    pub track_id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub album_art_url: Option<String>,
    pub external_urls: HashMap<String, String>,
    pub time_range: XRayTimeRange,
}

/// Category classification for scene trivia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneTriviaCategory {
    BehindTheScenes,
    EasterEgg,
    FilmingLocation,
    ProductionNote,
    Quote,
}

/// Curated trivia, production notes, or location insight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneTrivia {
    pub trivia_id: String,
    pub category: SceneTriviaCategory,
    pub text: String,
    pub is_spoiler: bool,
    pub time_range: XRayTimeRange,
}

/// Interactive contextual commerce tag linking on-screen items to store inventory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductCommerceTag {
    pub product_id: String,
    pub title: String,
    pub description: Option<String>,
    pub image_url: String,
    pub price: Decimal,
    pub currency: String,
    pub merchant_sku: String,
    pub checkout_url: String,
    pub time_range: XRayTimeRange,
}

/// Point-in-time interactive snapshot at a specific playback position.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct XRaySnapshot {
    pub playhead_seconds: f64,
    pub active_actors: Vec<ActorCredit>,
    pub active_music: Vec<MusicTrackInfo>,
    pub active_trivia: Vec<SceneTrivia>,
    pub active_products: Vec<ProductCommerceTag>,
}

impl XRaySnapshot {
    /// True if there is any interactive metadata active at this timestamp.
    #[must_use]
    pub fn has_content(&self) -> bool {
        !self.active_actors.is_empty()
            || !self.active_music.is_empty()
            || !self.active_trivia.is_empty()
            || !self.active_products.is_empty()
    }
}

/// Complete interactive catalog for a video asset containing all timeline layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoXRayCatalog {
    pub media_id: String,
    pub title: String,
    pub actors: Vec<ActorCredit>,
    pub music: Vec<MusicTrackInfo>,
    pub trivia: Vec<SceneTrivia>,
    pub products: Vec<ProductCommerceTag>,
}

impl VideoXRayCatalog {
    /// Initializes an empty interactive catalog for a given media asset.
    #[must_use]
    pub fn new(media_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            media_id: media_id.into(),
            title: title.into(),
            actors: Vec::new(),
            music: Vec::new(),
            trivia: Vec::new(),
            products: Vec::new(),
        }
    }

    /// Appends an actor credit.
    pub fn add_actor(&mut self, actor: ActorCredit) {
        self.actors.push(actor);
    }

    /// Appends a music track.
    pub fn add_music(&mut self, track: MusicTrackInfo) {
        self.music.push(track);
    }

    /// Appends a trivia note.
    pub fn add_trivia(&mut self, trivia: SceneTrivia) {
        self.trivia.push(trivia);
    }

    /// Appends a shoppable commerce tag.
    pub fn add_product(&mut self, product: ProductCommerceTag) {
        self.products.push(product);
    }

    /// Queries the active interactive snapshot for a given playhead position.
    #[must_use]
    pub fn query_at(&self, playhead_seconds: f64) -> XRaySnapshot {
        let active_actors = self
            .actors
            .iter()
            .filter(|a| a.time_range.contains(playhead_seconds))
            .cloned()
            .collect();

        let active_music = self
            .music
            .iter()
            .filter(|m| m.time_range.contains(playhead_seconds))
            .cloned()
            .collect();

        let active_trivia = self
            .trivia
            .iter()
            .filter(|t| t.time_range.contains(playhead_seconds))
            .cloned()
            .collect();

        let active_products = self
            .products
            .iter()
            .filter(|p| p.time_range.contains(playhead_seconds))
            .cloned()
            .collect();

        XRaySnapshot {
            playhead_seconds,
            active_actors,
            active_music,
            active_trivia,
            active_products,
        }
    }

    /// Generates an RFC 8216 WebVTT metadata track (`kind="metadata"`) containing JSON payloads in cues.
    #[must_use]
    pub fn generate_webvtt_metadata_track(&self) -> String {
        let mut out = String::with_capacity(4096);
        out.push_str("WEBVTT\n");
        out.push_str("X-TIMESTAMP-MAP=LOCAL:00:00:00.000,MPEGTS:0\n\n");

        // 1. Actors Cues
        for a in &self.actors {
            out.push_str(&format!("NOTE Actor: {}\n", a.name));
            out.push_str(&a.time_range.to_vtt_cue_header());
            out.push('\n');
            let payload = serde_json::json!({
                "type": "actor",
                "actor_id": a.actor_id,
                "name": a.name,
                "character": a.character_name,
                "profile_image": a.profile_image_url
            });
            out.push_str(&payload.to_string());
            out.push_str("\n\n");
        }

        // 2. Music Cues
        for m in &self.music {
            out.push_str(&format!("NOTE Music: {}\n", m.title));
            out.push_str(&m.time_range.to_vtt_cue_header());
            out.push('\n');
            let payload = serde_json::json!({
                "type": "music",
                "track_id": m.track_id,
                "title": m.title,
                "artist": m.artist,
                "album": m.album,
                "urls": m.external_urls
            });
            out.push_str(&payload.to_string());
            out.push_str("\n\n");
        }

        // 3. Trivia Cues
        for t in &self.trivia {
            out.push_str("NOTE Trivia\n");
            out.push_str(&t.time_range.to_vtt_cue_header());
            out.push('\n');
            let payload = serde_json::json!({
                "type": "trivia",
                "trivia_id": t.trivia_id,
                "category": format!("{:?}", t.category),
                "text": t.text,
                "spoiler": t.is_spoiler
            });
            out.push_str(&payload.to_string());
            out.push_str("\n\n");
        }

        // 4. Products Cues
        for p in &self.products {
            out.push_str(&format!("NOTE Product: {}\n", p.title));
            out.push_str(&p.time_range.to_vtt_cue_header());
            out.push('\n');
            let payload = serde_json::json!({
                "type": "product",
                "product_id": p.product_id,
                "title": p.title,
                "price": p.price.to_string(),
                "currency": p.currency,
                "checkout_url": p.checkout_url
            });
            out.push_str(&payload.to_string());
            out.push_str("\n\n");
        }

        out
    }

    /// Built-in demo catalog for instant testing and live demonstrations.
    #[must_use]
    pub fn demo_scifi_pilot(media_id: &str) -> Self {
        let mut catalog = Self::new(media_id, "Cyberpunk Frontiers: Episode 1");

        // Actors
        catalog.add_actor(ActorCredit {
            actor_id: "act-101".into(),
            name: "Keanu Reeves".into(),
            character_name: "Johnny Silverhand".into(),
            profile_image_url: Some("/static/actors/keanu.webp".into()),
            bio: Some("Legendary rockerboy and rebel fighter in Night City.".into()),
            filmography: vec!["The Matrix".into(), "John Wick".into(), "Constantine".into()],
            time_range: XRayTimeRange::new(10.0, 45.0),
        });

        catalog.add_actor(ActorCredit {
            actor_id: "act-102".into(),
            name: "Ana de Armas".into(),
            character_name: "Joi".into(),
            profile_image_url: Some("/static/actors/ana.webp".into()),
            bio: Some("Synthetic AI hologram companion.".into()),
            filmography: vec!["Blade Runner 2049".into(), "Knives Out".into()],
            time_range: XRayTimeRange::new(20.0, 60.0),
        });

        // Music
        let mut urls = HashMap::new();
        urls.insert("spotify".into(), "https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT".into());
        urls.insert("apple_music".into(), "https://music.apple.com/us/album/rebel-path".into());

        catalog.add_music(MusicTrackInfo {
            track_id: "mus-201".into(),
            title: "The Rebel Path".into(),
            artist: "P.T. Adamczyk".into(),
            album: Some("Cyberpunk 2077 Original Soundtrack".into()),
            album_art_url: Some("/static/music/rebel_path.webp".into()),
            external_urls: urls,
            time_range: XRayTimeRange::new(15.0, 50.0),
        });

        // Trivia
        catalog.add_trivia(SceneTrivia {
            trivia_id: "triv-301".into(),
            category: SceneTriviaCategory::BehindTheScenes,
            text: "This scene was captured using a custom 16-camera volumetric capture rig with zero motion blur.".into(),
            is_spoiler: false,
            time_range: XRayTimeRange::new(5.0, 40.0),
        });

        catalog.add_trivia(SceneTrivia {
            trivia_id: "triv-302".into(),
            category: SceneTriviaCategory::EasterEgg,
            text: "The neon sign in the background translates to 'Ghost in the Machine' in Ancient Greek.".into(),
            is_spoiler: false,
            time_range: XRayTimeRange::new(30.0, 55.0),
        });

        // Commerce Tags
        catalog.add_product(ProductCommerceTag {
            product_id: "prod-901".into(),
            title: "Samurai Bomber Jacket - Cyber Edition".into(),
            description: Some("LED embroidered collar with glowing circuit trim.".into()),
            image_url: "/static/products/samurai_jacket.webp".into(),
            price: Decimal::new(14999, 2), // $149.99
            currency: "USD".into(),
            merchant_sku: "JKT-CYBER-01".into(),
            checkout_url: "/api/v2/checkout/direct?sku=JKT-CYBER-01".into(),
            time_range: XRayTimeRange::new(20.0, 60.0),
        });

        catalog
    }
}

/// In-memory thread-safe registry of X-Ray interactive catalogs.
#[derive(Clone)]
pub struct XRayRegistry {
    catalogs: Arc<RwLock<HashMap<String, VideoXRayCatalog>>>,
}

impl Default for XRayRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl XRayRegistry {
    /// Creates an empty X-Ray registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            catalogs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initializes registry populated with default demo media.
    #[must_use]
    pub fn with_demo_catalogs() -> Self {
        let registry = Self::new();
        let demo = VideoXRayCatalog::demo_scifi_pilot("video_demo_1");
        registry.register_catalog(demo);
        registry
    }

    /// Registers or replaces an interactive catalog for a video asset.
    pub fn register_catalog(&self, catalog: VideoXRayCatalog) {
        if let Ok(mut lock) = self.catalogs.write() {
            lock.insert(catalog.media_id.clone(), catalog);
        }
    }

    /// Retrieves the active interactive snapshot for a media ID at a given playback timestamp.
    #[must_use]
    pub fn get_snapshot(&self, media_id: &str, playhead_seconds: f64) -> Option<XRaySnapshot> {
        let lock = self.catalogs.read().ok()?;
        let catalog = lock.get(media_id)?;
        Some(catalog.query_at(playhead_seconds))
    }

    /// Generates the WebVTT metadata track for a media ID.
    #[must_use]
    pub fn get_webvtt_track(&self, media_id: &str) -> Option<String> {
        let lock = self.catalogs.read().ok()?;
        let catalog = lock.get(media_id)?;
        Some(catalog.generate_webvtt_metadata_track())
    }

    /// Retrieves the full catalog definition for administrative or inspection purposes.
    #[must_use]
    pub fn get_catalog(&self, media_id: &str) -> Option<VideoXRayCatalog> {
        let lock = self.catalogs.read().ok()?;
        lock.get(media_id).cloned()
    }
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xray_time_range_contains_and_formatting() {
        let range = XRayTimeRange::new(12.5, 30.0);
        assert!(range.contains(12.5));
        assert!(range.contains(20.0));
        assert!(range.contains(30.0));
        assert!(!range.contains(12.4));
        assert!(!range.contains(30.1));
        assert_eq!(range.duration_seconds(), 17.5);

        // VTT formatting
        let vtt_str = XRayTimeRange::format_vtt_timestamp(75.5);
        assert_eq!(vtt_str, "00:01:15.500");

        let cue = range.to_vtt_cue_header();
        assert_eq!(cue, "00:00:12.500 --> 00:00:30.000");
    }

    #[test]
    fn test_demo_catalog_query_at_playhead() {
        let catalog = VideoXRayCatalog::demo_scifi_pilot("vid_1");

        // At t=25.0s: Both actors active (Keanu 10-45s, Ana 20-60s), music active (15-50s), trivia 1 active (5-40s), product active (20-60s)
        let snap25 = catalog.query_at(25.0);
        assert_eq!(snap25.active_actors.len(), 2);
        assert_eq!(snap25.active_music.len(), 1);
        assert_eq!(snap25.active_trivia.len(), 1);
        assert_eq!(snap25.active_products.len(), 1);
        assert_eq!(snap25.active_music[0].title, "The Rebel Path");
        assert_eq!(snap25.active_products[0].price, Decimal::new(14999, 2));

        // At t=2.0s: Outside all ranges
        let snap2 = catalog.query_at(2.0);
        assert!(!snap2.has_content());

        // At t=55.0s: Ana active (20-60s), Trivia 2 active (30-55s), Product active (20-60s)
        let snap55 = catalog.query_at(55.0);
        assert_eq!(snap55.active_actors.len(), 1);
        assert_eq!(snap55.active_actors[0].name, "Ana de Armas");
        assert_eq!(snap55.active_trivia.len(), 1);
        assert_eq!(snap55.active_trivia[0].category, SceneTriviaCategory::EasterEgg);
    }

    #[test]
    fn test_webvtt_metadata_track_generation() {
        let catalog = VideoXRayCatalog::demo_scifi_pilot("vid_test");
        let vtt = catalog.generate_webvtt_metadata_track();

        assert!(vtt.starts_with("WEBVTT\n"));
        assert!(vtt.contains("00:00:10.000 --> 00:00:45.000"));
        assert!(vtt.contains("\"Johnny Silverhand\""));
        assert!(vtt.contains("\"The Rebel Path\""));
        assert!(vtt.contains("Samurai Bomber Jacket"));
        assert!(vtt.contains("JKT-CYBER-01"));
    }

    #[test]
    fn test_xray_registry_operations() {
        let registry = XRayRegistry::with_demo_catalogs();

        let snap = registry.get_snapshot("video_demo_1", 22.0);
        assert!(snap.is_some());
        let snap = snap.unwrap();
        assert!(snap.has_content());
        assert_eq!(snap.active_actors.len(), 2);

        let vtt = registry.get_webvtt_track("video_demo_1");
        assert!(vtt.is_some());
        assert!(vtt.unwrap().contains("WEBVTT"));

        // Non-existent video returns None
        assert!(registry.get_snapshot("unknown_video", 10.0).is_none());
        assert!(registry.get_webvtt_track("unknown_video").is_none());
    }
}
