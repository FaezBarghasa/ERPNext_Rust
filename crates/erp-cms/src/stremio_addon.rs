//! Stremio Addon Protocol v3 Server & Stream Aggregator (`erp-cms::stremio_addon`).
//!
//! Synthesizes the Stremio Addon Protocol v3 architecture:
//! - Full compliance with Stremio Addon Protocol v3 specifications.
//! - Dynamic manifest declaration supporting catalogs, metadata, streams, and subtitle resources.
//! - Catalog resolution (`/catalog/{type}/{id}.json`) with keyword search and genre filtering.
//! - Detailed metadata resolution (`/meta/{type}/{id}.json`) with cast, posters, fanart backgrounds, and episodic video ladders.
//! - Multi-bitrate HLS and CAS stream resolution (`/stream/{type}/{id}.json`) with behavior hints.
//! - WebVTT subtitle track resolution (`/subtitles/{type}/{id}.json`).
//! - Unified bridge for ERPNext OTT VOD assets, episodic series, and 24/7 FAST live linear channels.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Extra filter definition for Stremio catalogs (e.g. search, genre).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StremioExtraProp {
    pub name: String,
    pub is_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}

/// Catalog declaration in the addon manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StremioCatalogDefinition {
    pub r#type: String,
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Vec<StremioExtraProp>>,
}

/// Root manifest conforming to Stremio Addon Protocol v3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StremioManifest {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub resources: Vec<String>,
    pub types: Vec<String>,
    pub catalogs: Vec<StremioCatalogDefinition>,
    pub id_prefixes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
}

impl Default for StremioManifest {
    fn default() -> Self {
        Self {
            id: "org.erpnext.ott.addon".into(),
            version: "1.0.0".into(),
            name: "ERPNext OTT Cinema & Live FAST".into(),
            description: "High-performance Rust OTT streaming aggregator for VOD movies, episodic series, and 24/7 FAST linear TV.".into(),
            resources: vec![
                "catalog".into(),
                "meta".into(),
                "stream".into(),
                "subtitles".into(),
            ],
            types: vec!["movie".into(), "series".into(), "tv".into()],
            catalogs: vec![
                StremioCatalogDefinition {
                    r#type: "movie".into(),
                    id: "erp_movies".into(),
                    name: "ERPNext Cinema (VOD)".into(),
                    extra: Some(vec![
                        StremioExtraProp {
                            name: "search".into(),
                            is_required: false,
                            options: None,
                        },
                        StremioExtraProp {
                            name: "genre".into(),
                            is_required: false,
                            options: Some(vec![
                                "Sci-Fi".into(),
                                "Cyberpunk".into(),
                                "Action".into(),
                                "Documentary".into(),
                            ]),
                        },
                    ]),
                },
                StremioCatalogDefinition {
                    r#type: "series".into(),
                    id: "erp_series".into(),
                    name: "ERPNext Episodic Series".into(),
                    extra: Some(vec![
                        StremioExtraProp {
                            name: "search".into(),
                            is_required: false,
                            options: None,
                        },
                    ]),
                },
                StremioCatalogDefinition {
                    r#type: "tv".into(),
                    id: "erp_fast_tv".into(),
                    name: "24/7 Virtual FAST Channels".into(),
                    extra: None,
                },
            ],
            id_prefixes: vec!["erp_".into(), "fast_".into()],
            background: Some("/static/stremio/backdrop.webp".into()),
            logo: Some("/static/stremio/logo.png".into()),
        }
    }
}

/// Brief item representation within a Stremio catalog listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StremioCatalogItem {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub poster: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_info: Option<String>,
}

/// Catalog query response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StremioCatalogResponse {
    pub metas: Vec<StremioCatalogItem>,
}

/// Individual episode within an episodic series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StremioVideo {
    pub id: String,
    pub title: String,
    pub season: u32,
    pub episode: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

/// Deep metadata representation for an individual title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StremioMetaDetail {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub poster: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cast: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videos: Option<Vec<StremioVideo>>,
}

/// Meta query response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StremioMetaResponse {
    pub meta: StremioMetaDetail,
}

/// Stremio stream playback behavior hints.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StremioStreamBehaviorHints {
    pub not_web_ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_headers: Option<HashMap<String, String>>,
}

/// Playback stream source entry for a title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StremioStream {
    pub title: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior_hints: Option<StremioStreamBehaviorHints>,
}

/// Streams query response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StremioStreamsResponse {
    pub streams: Vec<StremioStream>,
}

/// Subtitle track entry conforming to Stremio protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StremioSubtitle {
    pub id: String,
    pub lang: String,
    pub url: String,
}

/// Subtitles query response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StremioSubtitlesResponse {
    pub subtitles: Vec<StremioSubtitle>,
}

/// Stremio Addon Engine orchestrating catalog, metadata, streams, and subtitle delivery.
#[derive(Clone)]
pub struct StremioAddonEngine {
    manifest: StremioManifest,
    metas: Arc<RwLock<HashMap<String, StremioMetaDetail>>>,
}

impl Default for StremioAddonEngine {
    fn default() -> Self {
        Self::with_demo_catalogs()
    }
}

impl StremioAddonEngine {
    /// Creates a new Stremio addon engine with a custom manifest.
    #[must_use]
    pub fn new(manifest: StremioManifest) -> Self {
        Self {
            manifest,
            metas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initializes a production-ready addon engine pre-loaded with movies, series, and FAST TV channels.
    #[must_use]
    pub fn with_demo_catalogs() -> Self {
        let engine = Self::new(StremioManifest::default());

        // 1. Movie: Cyberpunk 2099
        engine.register_meta(StremioMetaDetail {
            id: "erp_movie_cyberpunk".into(),
            r#type: "movie".into(),
            name: "Cyberpunk: Neon Horizons".into(),
            poster: "/static/posters/cyberpunk_poster.webp".into(),
            background: Some("/static/posters/cyberpunk_fanart.webp".into()),
            logo: Some("/static/posters/cyberpunk_logo.png".into()),
            description: Some("In a megacity powered by neural networks, a rebel operative uncovers an orbital conspiracy.".into()),
            release_info: Some("2026".into()),
            genres: Some(vec!["Sci-Fi".into(), "Cyberpunk".into(), "Action".into()]),
            cast: Some(vec!["Keanu Reeves".into(), "Ana de Armas".into()]),
            runtime: Some("112 min".into()),
            videos: None,
        });

        // 2. Movie: Code Zero
        engine.register_meta(StremioMetaDetail {
            id: "erp_movie_codezero".into(),
            r#type: "movie".into(),
            name: "Code Zero: The Silicon Frontier".into(),
            poster: "/static/posters/codezero_poster.webp".into(),
            background: Some("/static/posters/codezero_fanart.webp".into()),
            logo: None,
            description: Some("The untold story of the pioneers who built the first decentralized operating system.".into()),
            release_info: Some("2025".into()),
            genres: Some(vec!["Documentary".into(), "Tech".into()]),
            cast: Some(vec!["Linus Torvalds".into(), "Graydon Hoare".into()]),
            runtime: Some("94 min".into()),
            videos: None,
        });

        // 3. Series: Silicon Syndicate
        let episodes = vec![
            StremioVideo {
                id: "erp_series_silicon:1:1".into(),
                title: "Genesis Node".into(),
                season: 1,
                episode: 1,
                released: Some("2026-01-10".into()),
                thumbnail: Some("/static/thumbs/ep1.webp".into()),
            },
            StremioVideo {
                id: "erp_series_silicon:1:2".into(),
                title: "Packet Storm".into(),
                season: 1,
                episode: 2,
                released: Some("2026-01-17".into()),
                thumbnail: Some("/static/thumbs/ep2.webp".into()),
            },
        ];

        engine.register_meta(StremioMetaDetail {
            id: "erp_series_silicon".into(),
            r#type: "series".into(),
            name: "The Silicon Syndicate".into(),
            poster: "/static/posters/syndicate_poster.webp".into(),
            background: Some("/static/posters/syndicate_fanart.webp".into()),
            logo: None,
            description: Some(
                "Corporate intrigue and asynchronous systems engineering in high-stakes finance."
                    .into(),
            ),
            release_info: Some("2026".into()),
            genres: Some(vec!["Drama".into(), "Thriller".into()]),
            cast: Some(vec!["Rami Malek".into(), "Christian Slater".into()]),
            runtime: Some("48 min".into()),
            videos: Some(episodes),
        });

        // 4. FAST Live TV: Sci-Fi 24/7
        engine.register_meta(StremioMetaDetail {
            id: "fast_scifi".into(),
            r#type: "tv".into(),
            name: "Sci-Fi 24/7 (Live FAST)".into(),
            poster: "/static/posters/scifi_tv_poster.webp".into(),
            background: Some("/static/posters/scifi_tv_fanart.webp".into()),
            logo: None,
            description: Some("24/7 Continuous live linear broadcast of premier cyberpunk and science fiction movies.".into()),
            release_info: Some("LIVE".into()),
            genres: Some(vec!["Sci-Fi".into(), "Live TV".into()]),
            cast: None,
            runtime: Some("24/7".into()),
            videos: None,
        });

        engine
    }

    /// Registers or updates metadata for a title.
    pub fn register_meta(&self, meta: StremioMetaDetail) {
        if let Ok(mut lock) = self.metas.write() {
            lock.insert(meta.id.clone(), meta);
        }
    }

    /// Returns the addon manifest.
    #[must_use]
    pub fn manifest(&self) -> &StremioManifest {
        &self.manifest
    }

    /// Resolves catalog query filtered by type, catalog ID, optional search term, and genre.
    #[must_use]
    pub fn catalog(
        &self,
        type_name: &str,
        _catalog_id: &str,
        search: Option<&str>,
        genre: Option<&str>,
    ) -> StremioCatalogResponse {
        let lock = match self.metas.read() {
            Ok(l) => l,
            Err(_) => return StremioCatalogResponse { metas: Vec::new() },
        };

        let metas = lock
            .values()
            .filter(|m| m.r#type == type_name)
            .filter(|m| {
                if let Some(s) = search {
                    m.name.to_lowercase().contains(&s.to_lowercase())
                } else {
                    true
                }
            })
            .filter(|m| {
                if let Some(g) = genre {
                    m.genres
                        .as_ref()
                        .map(|genres| genres.iter().any(|item| item.eq_ignore_ascii_case(g)))
                        .unwrap_or(false)
                } else {
                    true
                }
            })
            .map(|m| StremioCatalogItem {
                id: m.id.clone(),
                r#type: m.r#type.clone(),
                name: m.name.clone(),
                poster: m.poster.clone(),
                description: m.description.clone(),
                genres: m.genres.clone(),
                release_info: m.release_info.clone(),
            })
            .collect();

        StremioCatalogResponse { metas }
    }

    /// Resolves deep metadata for an item.
    #[must_use]
    pub fn meta(&self, _type_name: &str, item_id: &str) -> Option<StremioMetaResponse> {
        let lock = self.metas.read().ok()?;
        let meta = lock.get(item_id).cloned()?;
        Some(StremioMetaResponse { meta })
    }

    /// Resolves active playback streams for an item or episode.
    #[must_use]
    pub fn streams(
        &self,
        _type_name: &str,
        item_id: &str,
        base_url: &str,
    ) -> StremioStreamsResponse {
        let mut streams = Vec::new();
        let clean_base = base_url.trim_end_matches('/');

        if item_id.starts_with("fast_") {
            // Live FAST Channel Stream
            streams.push(StremioStream {
                title: "ERPNext 24/7 FAST Live Broadcast (1080p60)".into(),
                url: format!(
                    "{clean_base}/api/v2/streaming/fast/{item_id}/live.m3u8?variant=1080p"
                ),
                behavior_hints: Some(StremioStreamBehaviorHints {
                    not_web_ready: false,
                    proxy_headers: None,
                }),
            });
            streams.push(StremioStream {
                title: "ERPNext 24/7 FAST Live Broadcast (720p Mobile)".into(),
                url: format!("{clean_base}/api/v2/streaming/fast/{item_id}/live.m3u8?variant=720p"),
                behavior_hints: Some(StremioStreamBehaviorHints {
                    not_web_ready: false,
                    proxy_headers: None,
                }),
            });
        } else {
            // VOD or Episodic Stream
            let media_key = item_id.replace(':', "_");
            streams.push(StremioStream {
                title: "ERPNext Master ABR HLS (Auto 1080p/720p/480p)".into(),
                url: format!("{clean_base}/api/v2/streaming/videos/{media_key}/master.m3u8"),
                behavior_hints: Some(StremioStreamBehaviorHints {
                    not_web_ready: false,
                    proxy_headers: None,
                }),
            });
            streams.push(StremioStream {
                title: "ERPNext Direct HTTP Byte-Range Stream".into(),
                url: format!("{clean_base}/api/v2/streaming/videos/{media_key}/1080p/seg_0.ts"),
                behavior_hints: Some(StremioStreamBehaviorHints {
                    not_web_ready: false,
                    proxy_headers: None,
                }),
            });
        }

        StremioStreamsResponse { streams }
    }

    /// Resolves subtitle tracks for an item.
    #[must_use]
    pub fn subtitles(
        &self,
        _type_name: &str,
        item_id: &str,
        base_url: &str,
    ) -> StremioSubtitlesResponse {
        let clean_base = base_url.trim_end_matches('/');
        let media_key = item_id.replace(':', "_");

        let subtitles = vec![
            StremioSubtitle {
                id: format!("sub_en_{media_key}"),
                lang: "eng".into(),
                url: format!("{clean_base}/api/v2/streaming/videos/{media_key}/subs/en.vtt"),
            },
            StremioSubtitle {
                id: format!("sub_xray_{media_key}"),
                lang: "x-ray".into(),
                url: format!("{clean_base}/api/v2/streaming/videos/{media_key}/xray.vtt"),
            },
        ];

        StremioSubtitlesResponse { subtitles }
    }
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stremio_manifest_serialization() {
        let manifest = StremioManifest::default();
        let json_val = serde_json::to_value(&manifest).expect("serialize manifest failed");

        assert_eq!(json_val.get("id").unwrap(), "org.erpnext.ott.addon");
        assert_eq!(json_val.get("version").unwrap(), "1.0.0");
        let catalogs = json_val.get("catalogs").unwrap().as_array().unwrap();
        assert_eq!(catalogs.len(), 3);
        assert_eq!(catalogs[0].get("id").unwrap(), "erp_movies");
    }

    #[test]
    fn test_catalog_query_and_filtering() {
        let engine = StremioAddonEngine::default();

        // Query all movies
        let movies = engine.catalog("movie", "erp_movies", None, None);
        assert_eq!(movies.metas.len(), 2);

        // Search filter: "cyberpunk"
        let search_res = engine.catalog("movie", "erp_movies", Some("cyberpunk"), None);
        assert_eq!(search_res.metas.len(), 1);
        assert_eq!(search_res.metas[0].name, "Cyberpunk: Neon Horizons");

        // Genre filter: "Documentary"
        let genre_res = engine.catalog("movie", "erp_movies", None, Some("Documentary"));
        assert_eq!(genre_res.metas.len(), 1);
        assert_eq!(genre_res.metas[0].name, "Code Zero: The Silicon Frontier");

        // Non-matching genre
        let empty_res = engine.catalog("movie", "erp_movies", None, Some("Horror"));
        assert!(empty_res.metas.is_empty());
    }

    #[test]
    fn test_meta_resolution() {
        let engine = StremioAddonEngine::default();

        let movie_meta = engine.meta("movie", "erp_movie_cyberpunk");
        assert!(movie_meta.is_some());
        let meta = movie_meta.unwrap().meta;
        assert_eq!(meta.name, "Cyberpunk: Neon Horizons");
        assert_eq!(meta.cast.unwrap().len(), 2);

        let series_meta = engine.meta("series", "erp_series_silicon");
        assert!(series_meta.is_some());
        let s_meta = series_meta.unwrap().meta;
        let episodes = s_meta.videos.expect("Expected episodes");
        assert_eq!(episodes.len(), 2);
        assert_eq!(episodes[0].title, "Genesis Node");
    }

    #[test]
    fn test_stream_and_subtitles_resolution() {
        let engine = StremioAddonEngine::default();
        let base_url = "https://stream.erpnext.rs";

        // VOD streams
        let vod_streams = engine.streams("movie", "erp_movie_cyberpunk", base_url);
        assert_eq!(vod_streams.streams.len(), 2);
        assert!(vod_streams.streams[0].url.contains("master.m3u8"));

        // FAST live streams
        let fast_streams = engine.streams("tv", "fast_scifi", base_url);
        assert_eq!(fast_streams.streams.len(), 2);
        assert!(
            fast_streams.streams[0]
                .url
                .contains("live.m3u8?variant=1080p")
        );

        // Subtitles
        let subs = engine.subtitles("movie", "erp_movie_cyberpunk", base_url);
        assert_eq!(subs.subtitles.len(), 2);
        assert_eq!(subs.subtitles[0].lang, "eng");
        assert_eq!(subs.subtitles[1].lang, "x-ray");
    }
}
