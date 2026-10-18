//! QUIC & Resilient Video Streaming Engine for Lossy and Low-Speed Networks (`erp-cms::resilient_streaming`).
//!
//! Provides:
//! - Network-adaptive ABR ladder presets tailored for lossy (5%-25% packet drop) and constrained (100-500 kbps) networks.
//! - RFC 9000 & RFC 9114 QUIC stream prioritization headers (RFC 9218 HTTP Priority).
//! - Dynamic User Streaming Cache tiers (`Standard`, `Increased`, `LossyResilient`, `OfflineBurst`) to buffer up to 1GB / 160s.
//! - Server-side and edge prefetch window computation for zero-stall playback over high-jitter cellular and satellite links.
//! - Header injectors for `Alt-Svc`, `Cache-Control: immutable`, and `X-Prefetch-Segments`.

use crate::hls_engine::HlsVariant;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Detected or reported client network condition profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum NetworkCondition {
    /// High-speed fiber, broadband, or 5G connection (> 5 Mbps, < 1% packet loss).
    #[default]
    HighSpeed,
    /// Low-bandwidth network (e.g. 2G/3G/EDGE or congested ADSL, 100-500 kbps).
    LowSpeed { bandwidth_kbps: u32 },
    /// Lossy wireless / satellite link with significant random or burst packet drop (5% - 30%).
    Lossy { packet_loss_pct: f32 },
    /// High latency or jitter connection (e.g. GEO satellite, long haul, 300ms - 1500ms RTT).
    JitteryHighLatency { rtt_ms: u32 },
    /// Severe edge constraint: both low bandwidth and elevated packet loss.
    ConstrainedEdge {
        bandwidth_kbps: u32,
        packet_loss_pct: f32,
    },
}

impl NetworkCondition {
    /// Resolves network condition from client query hints and network telemetry.
    #[must_use]
    pub fn from_hints(
        profile: Option<&str>,
        downlink_mbps: Option<f32>,
        rtt_ms: Option<u32>,
        packet_loss: Option<f32>,
    ) -> Self {
        if let Some(p) = profile {
            match p.to_ascii_lowercase().as_str() {
                "lossy" => {
                    return Self::Lossy {
                        packet_loss_pct: packet_loss.unwrap_or(12.5),
                    };
                }
                "low_speed" | "lowspeed" | "2g" | "3g" => {
                    let kbps = downlink_mbps.map(|d| (d * 1000.0) as u32).unwrap_or(250);
                    return Self::LowSpeed {
                        bandwidth_kbps: kbps,
                    };
                }
                "jittery" | "satellite" => {
                    return Self::JitteryHighLatency {
                        rtt_ms: rtt_ms.unwrap_or(650),
                    };
                }
                "constrained" | "edge" => {
                    return Self::ConstrainedEdge {
                        bandwidth_kbps: downlink_mbps.map(|d| (d * 1000.0) as u32).unwrap_or(200),
                        packet_loss_pct: packet_loss.unwrap_or(15.0),
                    };
                }
                _ => {}
            }
        }

        // Automatic heuristics based on downlink and packet loss
        if let Some(loss) = packet_loss
            && loss >= 5.0
        {
            if let Some(downlink) = downlink_mbps
                && downlink <= 0.6
            {
                return Self::ConstrainedEdge {
                    bandwidth_kbps: (downlink * 1000.0) as u32,
                    packet_loss_pct: loss,
                };
            }
            return Self::Lossy {
                packet_loss_pct: loss,
            };
        }

        if let Some(downlink) = downlink_mbps
            && downlink <= 0.5
        {
            return Self::LowSpeed {
                bandwidth_kbps: (downlink * 1000.0) as u32,
            };
        }

        if let Some(rtt) = rtt_ms
            && rtt >= 400
        {
            return Self::JitteryHighLatency { rtt_ms: rtt };
        }

        Self::HighSpeed
    }

    /// Whether this condition requires aggressive low-bitrate ladders and prefetching.
    #[must_use]
    pub fn is_constrained(&self) -> bool {
        !matches!(self, Self::HighSpeed)
    }

    /// Recommended user cache tier to survive network stalls and dropouts.
    #[must_use]
    pub fn recommended_cache_tier(&self) -> UserCacheTier {
        match self {
            Self::HighSpeed => UserCacheTier::Standard,
            Self::LowSpeed { .. } => UserCacheTier::Increased,
            Self::Lossy { .. } | Self::JitteryHighLatency { .. } => UserCacheTier::LossyResilient,
            Self::ConstrainedEdge { .. } => UserCacheTier::OfflineBurst,
        }
    }

    /// Recommended segment duration in seconds for optimal recovery.
    #[must_use]
    pub fn recommended_segment_duration(&self) -> f32 {
        match self {
            Self::HighSpeed => 4.0,
            Self::LowSpeed { .. } => 3.0,
            Self::Lossy { .. } | Self::ConstrainedEdge { .. } => 2.0, // smaller 2s chunks reduce loss recovery time
            Self::JitteryHighLatency { .. } => 4.0,
        }
    }
}

/// User streaming cache profile and allocation tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UserCacheTier {
    /// Standard buffer (3 segments, ~12s buffer, 64 MB maximum cache).
    #[default]
    Standard,
    /// Increased cache for low-speed lines (8 segments, ~32s buffer, 256 MB cache).
    Increased,
    /// Lossy & jitter-resilient cache (16 segments, ~64s buffer, 512 MB cache).
    LossyResilient,
    /// Offline-burst cache for tunnels and commuter disconnects (40+ segments, ~160s buffer, 1024 MB cache).
    OfflineBurst,
}

impl UserCacheTier {
    /// Maximum client cache size allocated in megabytes.
    #[must_use]
    pub fn max_cache_mb(&self) -> u32 {
        match self {
            Self::Standard => 64,
            Self::Increased => 256,
            Self::LossyResilient => 512,
            Self::OfflineBurst => 1024,
        }
    }

    /// Forward prefetch depth (number of upcoming segments to pre-buffer).
    #[must_use]
    pub fn prefetch_depth(&self) -> usize {
        match self {
            Self::Standard => 3,
            Self::Increased => 8,
            Self::LossyResilient => 16,
            Self::OfflineBurst => 40,
        }
    }

    /// Estimated playback buffer protection duration in seconds (assuming 4s segments).
    #[must_use]
    pub fn buffer_protection_seconds(&self) -> f32 {
        (self.prefetch_depth() as f32) * 4.0
    }
}

/// Per-user/client streaming cache configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCacheConfig {
    pub client_id: CompactString,
    pub tier: UserCacheTier,
    pub max_cache_mb: u32,
    pub prefetch_depth: usize,
    pub quic_priority_enabled: bool,
    pub burst_prefetch_enabled: bool,
    pub cache_control_max_age: u32,
}

impl UserCacheConfig {
    /// Creates a new configuration initialized with a specific tier.
    #[must_use]
    pub fn new(client_id: impl Into<CompactString>, tier: UserCacheTier) -> Self {
        Self {
            client_id: client_id.into(),
            tier,
            max_cache_mb: tier.max_cache_mb(),
            prefetch_depth: tier.prefetch_depth(),
            quic_priority_enabled: true,
            burst_prefetch_enabled: tier != UserCacheTier::Standard,
            cache_control_max_age: 31_536_000, // 1 year immutable for segments
        }
    }
}

/// Global cache performance and resilience statistics.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UserCacheStats {
    pub total_configured_users: usize,
    pub total_prefetch_batches: u64,
    pub total_segments_prefetched: u64,
    pub cache_hits: u64,
    pub bytes_saved_mb: f64,
    pub estimated_stalls_mitigated: u64,
}

/// Central in-memory registry for user caching configurations and prefetch metrics.
#[derive(Debug, Default)]
pub struct UserCacheRegistry {
    configs: RwLock<HashMap<CompactString, UserCacheConfig>>,
    stats: RwLock<UserCacheStats>,
}

impl UserCacheRegistry {
    /// Creates a new empty user cache registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves an existing client config or returns default `Standard` tier.
    #[must_use]
    pub fn get_or_default(&self, client_id: &str) -> UserCacheConfig {
        if let Ok(lock) = self.configs.read()
            && let Some(cfg) = lock.get(client_id)
        {
            return cfg.clone();
        }
        UserCacheConfig::new(client_id, UserCacheTier::Standard)
    }

    /// Configures or upgrades a user's cache tier and settings.
    pub fn configure_user(
        &self,
        client_id: &str,
        tier: UserCacheTier,
        custom_max_cache_mb: Option<u32>,
        custom_prefetch_depth: Option<usize>,
    ) -> UserCacheConfig {
        let mut cfg = UserCacheConfig::new(client_id, tier);
        if let Some(mb) = custom_max_cache_mb {
            cfg.max_cache_mb = mb;
        }
        if let Some(depth) = custom_prefetch_depth {
            cfg.prefetch_depth = depth;
        }

        if let Ok(mut lock) = self.configs.write() {
            lock.insert(cfg.client_id.clone(), cfg.clone());
        }

        if let Ok(mut stats) = self.stats.write()
            && let Ok(lock) = self.configs.read()
        {
            stats.total_configured_users = lock.len();
        }

        cfg
    }

    /// Computes the list of forward segment filenames to prefetch based on user cache tier.
    #[must_use]
    pub fn calculate_prefetch_segments(
        &self,
        current_seq: u64,
        max_seq: u64,
        config: &UserCacheConfig,
    ) -> Vec<String> {
        let depth = config.prefetch_depth as u64;
        let mut prefetch_list = Vec::new();

        let start = current_seq + 1;
        let end = (current_seq + depth).min(max_seq);

        for seq in start..=end {
            prefetch_list.push(format!("seg_{seq}.ts"));
        }

        if let Ok(mut stats) = self.stats.write() {
            stats.total_prefetch_batches += 1;
            stats.total_segments_prefetched += prefetch_list.len() as u64;
        }

        prefetch_list
    }

    /// Records a successful cache hit and bandwidth savings.
    pub fn record_cache_hit(&self, bytes: u64) {
        if let Ok(mut stats) = self.stats.write() {
            stats.cache_hits += 1;
            stats.bytes_saved_mb += (bytes as f64) / (1024.0 * 1024.0);
            stats.estimated_stalls_mitigated += 1;
        }
    }

    /// Returns a snapshot of the current cache statistics.
    #[must_use]
    pub fn get_stats(&self) -> UserCacheStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }
}

/// Helper methods for QUIC and HTTP/3 stream prioritization (RFC 9218) and headers.
pub struct QuicStreamOptimizer;

impl QuicStreamOptimizer {
    /// Generates the standard `Alt-Svc` response header to advertise QUIC / HTTP/3.
    #[must_use]
    pub fn alt_svc_header(h3_port: u16, max_age_seconds: u32) -> String {
        format!(
            "h3=\":{h3_port}\"; ma={max_age_seconds}; persist=1, h3-29=\":{h3_port}\"; ma={max_age_seconds}"
        )
    }

    /// Generates RFC 9218 HTTP Priority headers (`Priority: u=..., i`).
    /// - `u=0` / `u=1`: urgent video playback stream.
    /// - `u=4`: speculative forward prefetch stream (avoids starving active playback).
    #[must_use]
    pub fn priority_header_value(is_prefetch: bool, is_urgent_low_speed: bool) -> &'static str {
        if is_urgent_low_speed {
            "u=0, i"
        } else if is_prefetch {
            "u=4, i"
        } else {
            "u=1, i"
        }
    }

    /// Formats the comma-separated `X-Prefetch-Segments` header value.
    #[must_use]
    pub fn format_prefetch_header(segments: &[String]) -> String {
        segments.join(",")
    }
}

/// Builder and compiler for resilient ABR Master Playlists under lossy / low-speed networks.
pub struct ResilientPlaylistEngine;

impl ResilientPlaylistEngine {
    /// Ultra-low bitrate 180p preset for heavily lossy / rural networks (120 kbps).
    #[must_use]
    pub fn preset_180p_lossy(playlist_uri: impl Into<String>) -> HlsVariant {
        HlsVariant {
            name: "180p-lossy".into(),
            bandwidth: 120_000,
            width: 320,
            height: 180,
            codecs: "avc1.42e00c,mp4a.40.2".into(),
            audio_group: Some("audio-default".into()),
            subtitle_group: Some("subs-default".into()),
            playlist_uri: playlist_uri.into(),
        }
    }

    /// Bandwidth-efficient 240p preset for low-speed connections (260 kbps).
    #[must_use]
    pub fn preset_240p_lowspeed(playlist_uri: impl Into<String>) -> HlsVariant {
        HlsVariant {
            name: "240p-lowspeed".into(),
            bandwidth: 260_000,
            width: 426,
            height: 240,
            codecs: "avc1.42e015,mp4a.40.2".into(),
            audio_group: Some("audio-default".into()),
            subtitle_group: Some("subs-default".into()),
            playlist_uri: playlist_uri.into(),
        }
    }

    /// Compiles an adaptive master playlist re-prioritized for the detected network condition.
    #[must_use]
    pub fn compile_resilient_master_playlist(
        video_id: &str,
        condition: &NetworkCondition,
    ) -> String {
        let mut variants = Vec::new();

        match condition {
            NetworkCondition::LowSpeed { .. } => {
                // Prioritize 240p, 360p, 180p at top of manifest for immediate smooth startup
                variants.push(Self::preset_240p_lowspeed(format!(
                    "{video_id}/240p/index.m3u8"
                )));
                variants.push(Self::preset_180p_lossy(format!(
                    "{video_id}/180p/index.m3u8"
                )));
                variants.push(HlsVariant::preset_360p(format!(
                    "{video_id}/360p/index.m3u8"
                )));
                variants.push(HlsVariant::preset_480p(format!(
                    "{video_id}/480p/index.m3u8"
                )));
            }
            NetworkCondition::Lossy { .. } | NetworkCondition::ConstrainedEdge { .. } => {
                // Under packet loss, short GOP ultra-low overhead 180p and 240p guarantee playback
                variants.push(Self::preset_180p_lossy(format!(
                    "{video_id}/180p/index.m3u8"
                )));
                variants.push(Self::preset_240p_lowspeed(format!(
                    "{video_id}/240p/index.m3u8"
                )));
                variants.push(HlsVariant::preset_360p(format!(
                    "{video_id}/360p/index.m3u8"
                )));
                variants.push(HlsVariant::preset_480p(format!(
                    "{video_id}/480p/index.m3u8"
                )));
            }
            NetworkCondition::JitteryHighLatency { .. } => {
                variants.push(HlsVariant::preset_360p(format!(
                    "{video_id}/360p/index.m3u8"
                )));
                variants.push(HlsVariant::preset_480p(format!(
                    "{video_id}/480p/index.m3u8"
                )));
                variants.push(Self::preset_240p_lowspeed(format!(
                    "{video_id}/240p/index.m3u8"
                )));
                variants.push(HlsVariant::preset_720p(format!(
                    "{video_id}/720p/index.m3u8"
                )));
            }
            NetworkCondition::HighSpeed => {
                variants.push(HlsVariant::preset_1080p(format!(
                    "{video_id}/1080p/index.m3u8"
                )));
                variants.push(HlsVariant::preset_720p(format!(
                    "{video_id}/720p/index.m3u8"
                )));
                variants.push(HlsVariant::preset_480p(format!(
                    "{video_id}/480p/index.m3u8"
                )));
                variants.push(HlsVariant::preset_360p(format!(
                    "{video_id}/360p/index.m3u8"
                )));
            }
        }

        let mut manifest = String::with_capacity(1024);
        manifest.push_str("#EXTM3U\n");
        manifest.push_str("#EXT-X-VERSION:6\n");
        manifest.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");

        for v in variants {
            manifest.push_str(&format!(
                "#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}x{},CODECS=\"{}\"\n{}\n",
                v.bandwidth, v.width, v.height, v.codecs, v.playlist_uri
            ));
        }

        manifest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_condition_hint_resolution() {
        let cond_lossy = NetworkCondition::from_hints(Some("lossy"), None, None, Some(14.0));
        assert_eq!(
            cond_lossy,
            NetworkCondition::Lossy {
                packet_loss_pct: 14.0
            }
        );
        assert!(cond_lossy.is_constrained());
        assert_eq!(
            cond_lossy.recommended_cache_tier(),
            UserCacheTier::LossyResilient
        );

        let cond_low_speed = NetworkCondition::from_hints(Some("low_speed"), Some(0.3), None, None);
        assert_eq!(
            cond_low_speed,
            NetworkCondition::LowSpeed {
                bandwidth_kbps: 300
            }
        );
        assert_eq!(
            cond_low_speed.recommended_cache_tier(),
            UserCacheTier::Increased
        );

        let cond_high_speed = NetworkCondition::from_hints(None, Some(50.0), Some(20), Some(0.1));
        assert_eq!(cond_high_speed, NetworkCondition::HighSpeed);
        assert!(!cond_high_speed.is_constrained());
    }

    #[test]
    fn test_user_cache_tier_capacities_and_prefetch() {
        assert_eq!(UserCacheTier::Standard.max_cache_mb(), 64);
        assert_eq!(UserCacheTier::Standard.prefetch_depth(), 3);

        assert_eq!(UserCacheTier::Increased.max_cache_mb(), 256);
        assert_eq!(UserCacheTier::Increased.prefetch_depth(), 8);

        assert_eq!(UserCacheTier::LossyResilient.max_cache_mb(), 512);
        assert_eq!(UserCacheTier::LossyResilient.prefetch_depth(), 16);

        assert_eq!(UserCacheTier::OfflineBurst.max_cache_mb(), 1024);
        assert_eq!(UserCacheTier::OfflineBurst.prefetch_depth(), 40);
        assert_eq!(
            UserCacheTier::OfflineBurst.buffer_protection_seconds(),
            160.0
        );
    }

    #[test]
    fn test_user_cache_registry_and_prefetch_calculation() {
        let registry = UserCacheRegistry::new();
        let client_id = "client_mobile_ir_01";

        // Defaults to Standard tier
        let default_cfg = registry.get_or_default(client_id);
        assert_eq!(default_cfg.tier, UserCacheTier::Standard);

        // Upgrade user to Increased tier with custom 300MB
        let upgraded =
            registry.configure_user(client_id, UserCacheTier::Increased, Some(300), Some(10));
        assert_eq!(upgraded.tier, UserCacheTier::Increased);
        assert_eq!(upgraded.max_cache_mb, 300);
        assert_eq!(upgraded.prefetch_depth, 10);

        // Calculate forward prefetch segments from seq 2 up to max 20
        let prefetches = registry.calculate_prefetch_segments(2, 20, &upgraded);
        assert_eq!(prefetches.len(), 10);
        assert_eq!(prefetches[0], "seg_3.ts");
        assert_eq!(prefetches[9], "seg_12.ts");

        // Record cache hits
        registry.record_cache_hit(1024 * 1024 * 2); // 2 MB
        let stats = registry.get_stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.total_segments_prefetched, 10);
        assert!(stats.bytes_saved_mb >= 2.0);
    }

    #[test]
    fn test_quic_stream_optimizer_headers() {
        let alt_svc = QuicStreamOptimizer::alt_svc_header(4433, 86400);
        assert!(alt_svc.contains("h3=\":4433\"; ma=86400"));

        let priority_urgent = QuicStreamOptimizer::priority_header_value(false, true);
        assert_eq!(priority_urgent, "u=0, i");

        let priority_prefetch = QuicStreamOptimizer::priority_header_value(true, false);
        assert_eq!(priority_prefetch, "u=4, i");

        let segments = vec!["seg_1.ts".into(), "seg_2.ts".into()];
        let header_val = QuicStreamOptimizer::format_prefetch_header(&segments);
        assert_eq!(header_val, "seg_1.ts,seg_2.ts");
    }

    #[test]
    fn test_resilient_master_playlist_compilation() {
        let cond_lossy = NetworkCondition::Lossy {
            packet_loss_pct: 12.0,
        };
        let manifest =
            ResilientPlaylistEngine::compile_resilient_master_playlist("vid-001", &cond_lossy);

        assert!(manifest.starts_with("#EXTM3U\n#EXT-X-VERSION:6\n#EXT-X-INDEPENDENT-SEGMENTS\n"));
        // 180p-lossy should be first
        assert!(manifest.contains("vid-001/180p/index.m3u8"));
        assert!(manifest.contains("BANDWIDTH=120000"));
        assert!(manifest.contains("BANDWIDTH=260000"));
    }
}
