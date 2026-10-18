//! Client Capability Negotiation & Playback Decision Engine (`erp-cms::video_streaming`).
//!
//! Implements Jellyfin-style capability negotiation:
//! - Client `DeviceProfile` analysis (supported containers, video codecs, audio codecs, max bitrates, max resolution, audio channels, HDR/SDR).
//! - Subtitle capability evaluation (Direct WebVTT delivery vs Server-Side Burn-in for ASS/PGS).
//! - Audio channel downmix evaluation (Surround 5.1/7.1 to Stereo).
//! - Source media stream profiling & HDR10-to-SDR tonemapping detection.
//! - Deterministic `PlayMethod` decision:
//!   1. `DirectPlay`: Client supports container and codecs natively within bandwidth limits.
//!   2. `DirectStream` (Remux): Client supports codecs, but requires container remuxing (e.g., MKV -> MP4 / HLS).
//!   3. `Transcode`: Codec mismatch, resolution downscaling, audio downmixing, tonemapping, or subtitle burn-in requires active FFmpeg transcoding.
//! - Dynamic FFmpeg command construction tailored to the exact decision.

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

/// Audio channel configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioChannels {
    Mono = 1,
    Stereo = 2,
    Surround51 = 6,
    Surround71 = 8,
}

/// Color dynamic range and transfer characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorRange {
    Sdr,
    Hdr10,
    Hdr10Plus,
    DolbyVision,
}

/// Supported subtitle track formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubtitleFormat {
    Vtt,
    Srt,
    Ass,
    Pgs,
    SubRip,
}

/// Delivery method for subtitle rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubtitleDelivery {
    /// Delivered as a native external WebVTT/SRT text track parsed by the client player.
    DirectText,
    /// Image-based or complex styled subtitles hard-burned into video frames via FFmpeg filtergraph.
    BurnIn,
}

/// Metadata describing an embedded or external subtitle track.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubtitleTrackInfo {
    pub language: String,
    pub format: SubtitleFormat,
    pub is_forced: bool,
}

impl SubtitleTrackInfo {
    /// Determines whether the subtitle format can be delivered as direct text or must be burned in.
    #[must_use]
    pub fn resolve_delivery(&self, client_supports_text_vtt: bool) -> SubtitleDelivery {
        match self.format {
            SubtitleFormat::Vtt | SubtitleFormat::Srt | SubtitleFormat::SubRip => {
                if client_supports_text_vtt {
                    SubtitleDelivery::DirectText
                } else {
                    SubtitleDelivery::BurnIn
                }
            }
            SubtitleFormat::Ass | SubtitleFormat::Pgs => SubtitleDelivery::BurnIn,
        }
    }
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
    pub max_audio_channels: u8,
    pub supports_hdr10: bool,
    pub supports_text_subtitles: bool,
    pub supports_hls: bool,
}

impl ClientDeviceProfile {
    /// Creates a generic modern web browser profile (H.264/VP9/AV1, AAC/Opus, Stereo, SDR, VTT).
    #[must_use]
    pub fn modern_web_browser(max_bitrate_kbps: u32) -> Self {
        Self {
            device_name: "Modern Web Browser".into(),
            supported_containers: vec![
                MediaContainer::Mp4,
                MediaContainer::WebM,
                MediaContainer::Hls,
            ],
            supported_video_codecs: vec![VideoCodec::H264, VideoCodec::Vp9, VideoCodec::Av1],
            supported_audio_codecs: vec![AudioCodec::Aac, AudioCodec::Opus, AudioCodec::Mp3],
            max_bitrate_kbps,
            max_width: 1920,
            max_height: 1080,
            max_audio_channels: 2,
            supports_hdr10: false,
            supports_text_subtitles: true,
            supports_hls: true,
        }
    }

    /// Creates an Apple TV / Smart TV 4K profile (HEVC/H.264, 5.1 Surround, HDR10, HLS).
    #[must_use]
    pub fn smart_tv_4k() -> Self {
        Self {
            device_name: "Smart TV 4K".into(),
            supported_containers: vec![MediaContainer::Mp4, MediaContainer::Hls],
            supported_video_codecs: vec![VideoCodec::Hevc, VideoCodec::H264],
            supported_audio_codecs: vec![AudioCodec::Eac3, AudioCodec::Ac3, AudioCodec::Aac],
            max_bitrate_kbps: 25_000,
            max_width: 3840,
            max_height: 2160,
            max_audio_channels: 6,
            supports_hdr10: true,
            supports_text_subtitles: true,
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
            max_audio_channels: 2,
            supports_hdr10: false,
            supports_text_subtitles: true,
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
    pub audio_channels: AudioChannels,
    pub color_range: ColorRange,
    pub bitrate_kbps: u32,
    pub width: u32,
    pub height: u32,
    pub duration_seconds: f64,
    pub active_subtitle: Option<SubtitleTrackInfo>,
}

/// The decided playback execution method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayMethod {
    /// Direct playback without server modification (HTTP 206 Byte Range partial content).
    DirectPlay,
    /// Remux container on the fly without re-encoding audio/video (`-c:v copy -c:a copy`).
    DirectStream { target_container: MediaContainer },
    /// Active multi-bitrate transcode required due to codec mismatch, bandwidth, resolution, downmixing, or subtitle burn-in.
    Transcode {
        target_resolution: (u32, u32),
        target_bitrate_kbps: u32,
        video_codec: VideoCodec,
        audio_codec: AudioCodec,
        downmix_to_channels: Option<u8>,
        requires_tonemapping: bool,
        burn_subtitle: Option<SubtitleFormat>,
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
        let channels_acceptable = (media.audio_channels as u8) <= profile.max_audio_channels;
        let hdr_acceptable = match media.color_range {
            ColorRange::Hdr10 | ColorRange::Hdr10Plus | ColorRange::DolbyVision => {
                profile.supports_hdr10
            }
            ColorRange::Sdr => true,
        };

        let subtitle_burn_needed = if let Some(ref sub) = media.active_subtitle {
            sub.resolve_delivery(profile.supports_text_subtitles) == SubtitleDelivery::BurnIn
        } else {
            false
        };

        // 1. Direct Play: All parameters match client hardware capability natively
        if container_supported
            && video_supported
            && audio_supported
            && bitrate_acceptable
            && resolution_acceptable
            && channels_acceptable
            && hdr_acceptable
            && !subtitle_burn_needed
        {
            return PlayMethod::DirectPlay;
        }

        // 2. Direct Stream (Remux): Codecs, channels, HDR, and subtitles match; only container needs repackaging
        if video_supported
            && audio_supported
            && bitrate_acceptable
            && resolution_acceptable
            && channels_acceptable
            && hdr_acceptable
            && !subtitle_burn_needed
        {
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

        // 3. Transcode: Incompatible codecs, bandwidth limits, audio downmixing, tonemapping, or subtitle burn-in
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

        let downmix_to_channels = if !channels_acceptable {
            Some(profile.max_audio_channels)
        } else {
            None
        };

        let requires_tonemapping = !hdr_acceptable;

        let burn_subtitle = if subtitle_burn_needed {
            media.active_subtitle.as_ref().map(|s| s.format)
        } else {
            None
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
        if !channels_acceptable {
            reasons.push(format!(
                "Audio channels {:?} exceeds limit {}",
                media.audio_channels, profile.max_audio_channels
            ));
        }
        if !hdr_acceptable {
            reasons.push(format!(
                "HDR format {:?} requires SDR tonemapping",
                media.color_range
            ));
        }
        if subtitle_burn_needed {
            reasons.push("Subtitle requires server-side burn-in".into());
        }

        PlayMethod::Transcode {
            target_resolution: (target_width, target_height),
            target_bitrate_kbps: target_bitrate,
            video_codec: target_video_codec,
            audio_codec: target_audio_codec,
            downmix_to_channels,
            requires_tonemapping,
            burn_subtitle,
            reason: reasons.join("; "),
        }
    }

    /// Constructs the exact optimized FFmpeg command line invocation for a given decision.
    #[must_use]
    pub fn build_ffmpeg_command(
        input_path: &str,
        output_path: &str,
        method: &PlayMethod,
    ) -> Option<String> {
        match method {
            PlayMethod::DirectPlay => None,
            PlayMethod::DirectStream { .. } => Some(format!(
                "ffmpeg -i \"{input_path}\" -c copy -movflags +faststart \"{output_path}\""
            )),
            PlayMethod::Transcode {
                target_resolution,
                target_bitrate_kbps,
                downmix_to_channels,
                requires_tonemapping,
                burn_subtitle,
                ..
            } => {
                let mut v_filter = format!("scale={}:{}", target_resolution.0, target_resolution.1);

                if *requires_tonemapping {
                    v_filter.push_str(",zscale=t=linear:npl=100,tonemap=tonemap=mobius");
                }

                if let Some(sub_fmt) = burn_subtitle {
                    match sub_fmt {
                        SubtitleFormat::Ass | SubtitleFormat::Srt => {
                            v_filter.push_str(&format!(",subtitles=\"{input_path}\""));
                        }
                        SubtitleFormat::Pgs => {
                            v_filter.push_str(",filter_complex=\"[0:v][0:s]overlay\"");
                        }
                        _ => {}
                    }
                }

                let audio_downmix = if let Some(ch) = downmix_to_channels {
                    format!("-ac {ch}")
                } else {
                    "-ac 2".into()
                };

                Some(format!(
                    "ffmpeg -i \"{input_path}\" -vf \"{v_filter}\" -c:v libx264 -b:v {target_bitrate_kbps}k \
                    -c:a aac -b:a 192k {audio_downmix} -movflags +faststart \"{output_path}\""
                ))
            }
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
            audio_channels: AudioChannels::Stereo,
            color_range: ColorRange::Sdr,
            bitrate_kbps: 4500,
            width: 1920,
            height: 1080,
            duration_seconds: 3600.0,
            active_subtitle: None,
        };

        let decision = PlaybackDecisionEngine::evaluate_playback(&profile, &media);
        assert_eq!(decision, PlayMethod::DirectPlay);
        assert!(PlaybackDecisionEngine::build_ffmpeg_command("in.mp4", "out.mp4", &decision).is_none());
    }

    #[test]
    fn test_direct_stream_remux_decision() {
        let profile = ClientDeviceProfile::modern_web_browser(8000);
        // Source is MKV (unsupported natively in browser), but H.264/AAC/Stereo match
        let media = MediaStreamInfo {
            container: MediaContainer::Mkv,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,
            audio_channels: AudioChannels::Stereo,
            color_range: ColorRange::Sdr,
            bitrate_kbps: 4500,
            width: 1920,
            height: 1080,
            duration_seconds: 3600.0,
            active_subtitle: None,
        };

        let decision = PlaybackDecisionEngine::evaluate_playback(&profile, &media);
        assert_eq!(
            decision,
            PlayMethod::DirectStream {
                target_container: MediaContainer::Hls
            }
        );

        let cmd = PlaybackDecisionEngine::build_ffmpeg_command("in.mkv", "out.mp4", &decision).unwrap();
        assert!(cmd.contains("-c copy"));
        assert!(cmd.contains("-movflags +faststart"));
    }

    #[test]
    fn test_transcode_audio_downmixing_and_tonemapping() {
        let profile = ClientDeviceProfile::modern_web_browser(5000); // Max 2 channels, SDR only
        let media = MediaStreamInfo {
            container: MediaContainer::Mp4,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,
            audio_channels: AudioChannels::Surround51, // 6 channels requires downmixing
            color_range: ColorRange::Hdr10,            // HDR10 requires tonemapping
            bitrate_kbps: 4500,
            width: 1920,
            height: 1080,
            duration_seconds: 120.0,
            active_subtitle: None,
        };

        let decision = PlaybackDecisionEngine::evaluate_playback(&profile, &media);
        match decision {
            PlayMethod::Transcode {
                downmix_to_channels,
                requires_tonemapping,
                ref reason,
                ..
            } => {
                assert_eq!(downmix_to_channels, Some(2));
                assert!(requires_tonemapping);
                assert!(reason.contains("Audio channels Surround51 exceeds limit 2"));
                assert!(reason.contains("HDR format Hdr10 requires SDR tonemapping"));
            }
            other => panic!("Expected Transcode, got {:?}", other),
        }

        let cmd = PlaybackDecisionEngine::build_ffmpeg_command("in.mp4", "out.mp4", &decision).unwrap();
        assert!(cmd.contains("-ac 2"));
        assert!(cmd.contains("tonemap=mobius"));
    }

    #[test]
    fn test_transcode_subtitle_burn_in() {
        let profile = ClientDeviceProfile::modern_web_browser(8000);
        let media = MediaStreamInfo {
            container: MediaContainer::Mp4,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,
            audio_channels: AudioChannels::Stereo,
            color_range: ColorRange::Sdr,
            bitrate_kbps: 4500,
            width: 1920,
            height: 1080,
            duration_seconds: 3600.0,
            active_subtitle: Some(SubtitleTrackInfo {
                language: "jpn".into(),
                format: SubtitleFormat::Ass, // Styled ASS requires burn-in
                is_forced: false,
            }),
        };

        let decision = PlaybackDecisionEngine::evaluate_playback(&profile, &media);
        match decision {
            PlayMethod::Transcode {
                burn_subtitle,
                ref reason,
                ..
            } => {
                assert_eq!(burn_subtitle, Some(SubtitleFormat::Ass));
                assert!(reason.contains("Subtitle requires server-side burn-in"));
            }
            other => panic!("Expected Transcode, got {:?}", other),
        }

        let cmd = PlaybackDecisionEngine::build_ffmpeg_command("in.mp4", "out.mp4", &decision).unwrap();
        assert!(cmd.contains("subtitles=\"in.mp4\""));
    }
}
