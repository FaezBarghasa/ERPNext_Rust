use serde::{Deserialize, Serialize};

/// Adaptive Bitrate (ABR) Video Variant profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VideoVariant {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub bandwidth: u64,
}

/// Standard ABR profiles (Milestone 5.2).
pub const STANDARD_VARIANTS: &[VideoVariant] = &[
    VideoVariant {
        name: "1080p",
        width: 1920,
        height: 1080,
        bandwidth: 5_000_000,
    },
    VideoVariant {
        name: "720p",
        width: 1280,
        height: 720,
        bandwidth: 2_800_000,
    },
    VideoVariant {
        name: "480p",
        width: 854,
        height: 480,
        bandwidth: 1_200_000,
    },
];

/// HLS Adaptive Bitrate (ABR) Playlist Generator (Milestone 5.2).
pub struct HlsPlaylistGenerator;

impl HlsPlaylistGenerator {
    /// Generates valid master.m3u8 playlist indexing all variants.
    #[must_use]
    pub fn generate_master_playlist(variants: &[VideoVariant]) -> String {
        let mut lines = vec![
            "#EXTM3U".to_string(),
            "#EXT-X-VERSION:3".to_string(),
        ];

        for v in variants {
            lines.push(format!(
                "#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}x{}",
                v.bandwidth, v.width, v.height
            ));
            lines.push(format!("{}.m3u8", v.name));
        }

        lines.join("\n")
    }

    /// Generates an individual variant segment index playlist.
    #[must_use]
    pub fn generate_variant_playlist(variant_name: &str, segment_count: usize, segment_duration_secs: u32) -> String {
        let mut lines = vec![
            "#EXTM3U".to_string(),
            "#EXT-X-VERSION:3".to_string(),
            format!("#EXT-X-TARGETDURATION:{segment_duration_secs}"),
            "#EXT-X-MEDIA-SEQUENCE:0".to_string(),
        ];

        for i in 0..segment_count {
            lines.push(format!("#EXTINF:{segment_duration_secs}.0,"));
            lines.push(format!("{variant_name}_segment_{i:03}.ts"));
        }

        lines.push("#EXT-X-ENDLIST".to_string());
        lines.join("\n")
    }
}
