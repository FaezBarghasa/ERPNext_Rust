//! Client Capability Negotiation & Playback Decision Engine (`erp-cms::video_streaming`).
//!
//! Implements Jellyfin-style capability negotiation:
//! - Client `DeviceProfile` analysis (supported containers, video codecs, audio codecs, max bitrates, max resolution).
//! - Source media stream profiling.
//! - Deterministic `PlayMethod` decision:
//!   1. `DirectPlay`: Client supports container and codecs natively within bandwidth limits.
//!   2. `DirectStream` (Remux): Client supports codecs, but requires container remuxing (e.g., MKV -> MP4 / HLS).
//!   3. `Transcode`: Codec mismatch, resolution downscaling, or bandwidth limitation requires active ABR transcoding.

use serde::{Deserialize, Serialize};

/// Supported media container formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaContainer {
    Mp4,
    Mkv,
    WebM,
    Hls,
    Ts,
    Flv,
}

impl MediaContainer {
    #[must_use]
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Mp4 => "video/mp4",
            Self::Mkv => "video/x-matroska",
            Self::WebM => "video/webm",
            Self::Hls => "application/vnd.apple.mpegurl",
            Self::Ts => "video/mp2t",
            Self::Flv => "video/x-flv",
        }
    }
}

/// Supported video codecs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    Hevc,
    Av1,
    Vp9,
    Other(String),
}

/// Supported audio codecs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioCodec {
    Aac,
    Opus,
    Mp3,
    Ac3,
    Eac3,
    Flac,
    Other(String),
}

/// Client device playback profile reported during session initiation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientDeviceProfile {
    pub device_name: String,
    pub supported_containers: Vec<MediaContainer>,
    pub supported_video_codecs: Vec<VideoCodec>,
    pub supported_audio_codecs: Vec<AudioCodec>,
    pub max_bitrate_kbps: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub supports_hls: bool,
}

impl ClientDeviceProfile {
    /// Creates a generic web browser profile (H.264, AAC, MP4/HLS).
    #[must_use]
    pub fn modern_web_browser(max_bitrate_kbps: u32) -> Self {
        Self {
            device_name: "Modern Web Browser".into(),
            supported_containers: vec![MediaContainer::Mp4, MediaContainer::WebM, MediaContainer::Hls],
            supported_video_codecs: vec![VideoCodec::H264, VideoCodec::Vp9, VideoCodec::Av1],
            supported_audio_codecs: vec![AudioCodec::Aac, AudioCodec::Opus, AudioCodec::Mp3],
            max_bitrate_kbps,
            max_width: 1920,
            max_height: 1080,
            supports_hls: true,
        }
    }

    /// Creates a constrained mobile or edge kiosk profile.
    #[must_use]
    pub fn constrained_edge_device() -> Self {
        Self {
            device_name: "Constrained Edge Terminal".into(),
            supported_containers: vec![MediaContainer::Mp4, MediaContainer::Hls],
            supported_video_codecs: vec![VideoCodec::H264],
            supported_audio_codecs: vec![AudioCodec::Aac],
            max_bitrate_kbps: 1500,
            max_width: 1280,
            max_height: 720,
            supports_hls: true,
        }
    }
}

/// Metadata describing source media stream assets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaStreamInfo {
    pub container: MediaContainer,
    pub video_codec: VideoCodec,
    pub audio_codec: AudioCodec,
    pub bitrate_kbps: u32,
    pub width: u32,
    pub height: u32,
    pub duration_seconds: f64,
}

/// The decided playback execution method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayMethod {
    /// Direct playback without server modification (HTTP 206 Byte Range partial content).
    DirectPlay,
    /// Remux container on the fly without re-encoding audio/video (`-c:v copy -c:a copy`).
    DirectStream { target_container: MediaContainer },
    /// Active multi-bitrate transcode required due to codec mismatch, bandwidth, or resolution limits.
    Transcode {
        target_resolution: (u32, u32),
        target_bitrate_kbps: u32,
        video_codec: VideoCodec,
        audio_codec: AudioCodec,
        reason: String,
    },
}

/// Decision engine evaluating incoming media against client capability profile.
pub struct PlaybackDecisionEngine;

impl PlaybackDecisionEngine {
    /// Evaluates client profile and media stream info to determine optimal playback strategy.
    #[must_use]
    pub fn evaluate_playback(profile: &ClientDeviceProfile, media: &MediaStreamInfo) -> PlayMethod {
        let container_supported = profile.supported_containers.contains(&media.container);
        let video_supported = profile.supported_video_codecs.contains(&media.video_codec);
        let audio_supported = profile.supported_audio_codecs.contains(&media.audio_codec);
        let bitrate_acceptable = media.bitrate_kbps <= profile.max_bitrate_kbps;
        let resolution_acceptable =
            media.width <= profile.max_width && media.height <= profile.max_height;

        // 1. Direct Play: All parameters match client hardware capability
        if container_supported
            && video_supported
            && audio_supported
            && bitrate_acceptable
            && resolution_acceptable
        {
            return PlayMethod::DirectPlay;
        }

        // 2. Direct Stream (Remux): Codecs & bandwidth match, only container needs repackaging
        if video_supported && audio_supported && bitrate_acceptable && resolution_acceptable {
            let target_container = if profile.supports_hls {
                MediaContainer::Hls
            } else if profile.supported_containers.contains(&MediaContainer::Mp4) {
                MediaContainer::Mp4
            } else {
                profile
                    .supported_containers
                    .first()
                    .copied()
                    .unwrap_or(MediaContainer::Mp4)
            };
            return PlayMethod::DirectStream { target_container };
        }

        // 3. Transcode: Incompatible codecs or exceeds bandwidth / resolution limits
        let target_bitrate = media.bitrate_kbps.min(profile.max_bitrate_kbps);
        let target_width = media.width.min(profile.max_width);
        let target_height = media.height.min(profile.max_height);

        let target_video_codec = if profile.supported_video_codecs.contains(&VideoCodec::H264) {
            VideoCodec::H264
        } else {
            profile
                .supported_video_codecs
                .first()
                .cloned()
                .unwrap_or(VideoCodec::H264)
        };

        let target_audio_codec = if profile.supported_audio_codecs.contains(&AudioCodec::Aac) {
            AudioCodec::Aac
        } else {
            profile
                .supported_audio_codecs
                .first()
                .cloned()
                .unwrap_or(AudioCodec::Aac)
        };

        let mut reasons = Vec::new();
        if !video_supported {
            reasons.push(format!("Unsupported video codec: {:?}", media.video_codec));
        }
        if !audio_supported {
            reasons.push(format!("Unsupported audio codec: {:?}", media.audio_codec));
        }
        if !bitrate_acceptable {
            reasons.push(format!(
                "Bitrate {}k exceeds limit {}k",
                media.bitrate_kbps, profile.max_bitrate_kbps
            ));
        }
        if !resolution_acceptable {
            reasons.push(format!(
                "Resolution {}x{} exceeds limit {}x{}",
                media.width, media.height, profile.max_width, profile.max_height
            ));
        }

        PlayMethod::Transcode {
            target_resolution: (target_width, target_height),
            target_bitrate_kbps: target_bitrate,
            video_codec: target_video_codec,
            audio_codec: target_audio_codec,
            reason: reasons.join("; "),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_play_decision() {
        let profile = ClientDeviceProfile::modern_web_browser(8000);
        let media = MediaStreamInfo {
            container: MediaContainer::Mp4,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,
            bitrate_kbps: 4500,
            width: 1920,
            height: 1080,
            duration_seconds: 3600.0,
        };

        let decision = PlaybackDecisionEngine::evaluate_playback(&profile, &media);
        assert_eq!(decision, PlayMethod::DirectPlay);
    }

    #[test]
    fn test_direct_stream_remux_decision() {
        let profile = ClientDeviceProfile::modern_web_browser(8000);
        // Source is MKV, which web browsers cannot play natively, but codecs (H.264 / AAC) are supported
        let media = MediaStreamInfo {
            container: MediaContainer::Mkv,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,
            bitrate_kbps: 4500,
            width: 1920,
            height: 1080,
            duration_seconds: 3600.0,
        };

        let decision = PlaybackDecisionEngine::evaluate_playback(&profile, &media);
        assert_eq!(
            decision,
            PlayMethod::DirectStream {
                target_container: MediaContainer::Hls
            }
        );
    }

    #[test]
    fn test_transcode_decision_bandwidth_limit() {
        let profile = ClientDeviceProfile::constrained_edge_device(); // max 1500 kbps, 720p
        let media = MediaStreamInfo {
            container: MediaContainer::Mp4,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,
            bitrate_kbps: 6000,
            width: 1920,
            height: 1080,
            duration_seconds: 120.0,
        };

        let decision = PlaybackDecisionEngine::evaluate_playback(&profile, &media);
        match decision {
            PlayMethod::Transcode {
                target_resolution,
                target_bitrate_kbps,
                video_codec,
                audio_codec,
                reason,
            } => {
                assert_eq!(target_resolution, (1280, 720));
                assert_eq!(target_bitrate_kbps, 1500);
                assert_eq!(video_codec, VideoCodec::H264);
                assert_eq!(audio_codec, AudioCodec::Aac);
                assert!(reason.contains("Bitrate 6000k exceeds limit 1500k"));
            }
            other => panic!("Expected Transcode, got {:?}", other),
        }
    }

    #[test]
    fn test_transcode_decision_codec_mismatch() {
        let profile = ClientDeviceProfile::constrained_edge_device(); // only supports H264
        let media = MediaStreamInfo {
            container: MediaContainer::Mkv,
            video_codec: VideoCodec::Hevc, // HEVC unsupported on this edge device
            audio_codec: AudioCodec::Flac, // FLAC unsupported
            bitrate_kbps: 1200,
            width: 1280,
            height: 720,
            duration_seconds: 60.0,
        };

        let decision = PlaybackDecisionEngine::evaluate_playback(&profile, &media);
        match decision {
            PlayMethod::Transcode {
                video_codec,
                audio_codec,
                reason,
                ..
            } => {
                assert_eq!(video_codec, VideoCodec::H264);
                assert_eq!(audio_codec, AudioCodec::Aac);
                assert!(reason.contains("Unsupported video codec"));
                assert!(reason.contains("Unsupported audio codec"));
            }
            other => panic!("Expected Transcode, got {:?}", other),
        }
    }
}
