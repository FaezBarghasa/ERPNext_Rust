//! Audio Streaming & Transcoding Engine (`erp-cms::audio_streaming`).
//!
//! Features:
//! - Codec detection & client capability matching (AAC, Opus, MP3, FLAC, Vorbis, ALAC, WAV).
//! - EBU R128 loudness normalization (-14.0 LUFS broadcast target, -1.0 dBFS true-peak ceiling).
//! - Seamless gapless playback trimming & mathematical crossfade curves (Linear, Equal Power, Logarithmic).
//! - FFmpeg audio transcoding filter pipeline generator.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Strongly typed errors for the audio streaming engine.
#[derive(Debug, Error, PartialEq)]
pub enum AudioStreamError {
    #[error("Unsupported audio codec: {0}")]
    UnsupportedCodec(String),
    #[error("Invalid duration: {0}")]
    InvalidDuration(String),
    #[error("Invalid crossfade duration {0}s exceeds minimum track duration {1}s")]
    CrossfadeDurationTooLong(f32, f32),
    #[error("Invalid loudness values: Integrated {0} LUFS, True Peak {1} dBFS")]
    InvalidLoudnessMetrics(f32, f32),
}

/// Supported audio codecs for streaming and local caching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioCodec {
    Aac,
    Opus,
    Mp3,
    Flac,
    Vorbis,
    Alac,
    Wav,
}

impl AudioCodec {
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Aac => "audio/aac",
            Self::Opus => "audio/ogg; codecs=opus",
            Self::Mp3 => "audio/mpeg",
            Self::Flac => "audio/flac",
            Self::Vorbis => "audio/ogg; codecs=vorbis",
            Self::Alac => "audio/mp4",
            Self::Wav => "audio/wav",
        }
    }

    #[must_use]
    pub const fn file_extension(&self) -> &'static str {
        match self {
            Self::Aac => "aac",
            Self::Opus => "opus",
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Vorbis => "ogg",
            Self::Alac => "m4a",
            Self::Wav => "wav",
        }
    }

    #[must_use]
    pub const fn is_lossless(&self) -> bool {
        matches!(self, Self::Flac | Self::Alac | Self::Wav)
    }
}

/// Channel layout configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AudioChannels {
    Mono = 1,
    #[default]
    Stereo = 2,
    Surround51 = 6,
    Surround71 = 8,
}

/// Detailed audio track technical metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioTrackMetadata {
    pub codec: AudioCodec,
    pub sample_rate: u32,
    pub bit_depth: u8,
    pub channels: AudioChannels,
    pub bitrate_kbps: u32,
    pub duration_ms: u32,
    pub lead_in_samples: u32,
    pub lead_out_samples: u32,
}

/// EBU R128 / ReplayGain loudness normalization profile.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LoudnessProfile {
    /// Integrated loudness in LUFS (e.g. -11.5 LUFS for hot master).
    pub integrated_lufs: f32,
    /// Loudness range in LU.
    pub loudness_range_lu: f32,
    /// True peak in dBFS (e.g. -0.2 dBFS).
    pub true_peak_dbfs: f32,
}

impl Default for LoudnessProfile {
    fn default() -> Self {
        Self {
            integrated_lufs: -14.0,
            loudness_range_lu: 7.0,
            true_peak_dbfs: -1.0,
        }
    }
}

/// EBU R128 Normalizer enforcing Spotify / YouTube -14.0 LUFS target.
pub struct EbuR128Normalizer;

impl EbuR128Normalizer {
    pub const TARGET_LUFS: f32 = -14.0;
    pub const TRUE_PEAK_CEILING_DBFS: f32 = -1.0;

    /// Calculates required gain adjustment in dB and linear multiplier.
    ///
    /// Algorithm:
    /// 1. `gain_db = TARGET_LUFS - measured_integrated_lufs`
    /// 2. If `measured_true_peak + gain_db > TRUE_PEAK_CEILING_DBFS`, clamp gain to prevent digital clipping.
    /// 3. Convert dB to linear scale: `10^(gain_db / 20)`.
    pub fn calculate_gain(profile: &LoudnessProfile) -> Result<(f32, f32), AudioStreamError> {
        if profile.integrated_lufs < -70.0 || profile.true_peak_dbfs > 10.0 {
            return Err(AudioStreamError::InvalidLoudnessMetrics(
                profile.integrated_lufs,
                profile.true_peak_dbfs,
            ));
        }

        let raw_gain_db = Self::TARGET_LUFS - profile.integrated_lufs;
        let peak_limit_gain_db = Self::TRUE_PEAK_CEILING_DBFS - profile.true_peak_dbfs;

        // Apply limiter clamping
        let effective_gain_db = if raw_gain_db > peak_limit_gain_db {
            peak_limit_gain_db
        } else {
            raw_gain_db
        };

        // Linear scaling factor
        let linear_factor = 10.0f32.powf(effective_gain_db / 20.0);
        Ok((effective_gain_db, linear_factor))
    }

    /// Constructs the standard FFmpeg loudnorm audio filter argument.
    #[must_use]
    pub fn ffmpeg_loudnorm_filter() -> String {
        format!(
            "loudnorm=I={}:TP={}:LRA=11",
            Self::TARGET_LUFS,
            Self::TRUE_PEAK_CEILING_DBFS
        )
    }
}

/// Crossfade transition curve between consecutive tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CrossfadeCurve {
    #[default]
    EqualPower,
    Linear,
    Logarithmic,
}

impl CrossfadeCurve {
    /// Computes gain multipliers (fading_out, fading_in) at normalized progress t in [0.0, 1.0].
    #[must_use]
    pub fn compute_gains(&self, progress: f32) -> (f32, f32) {
        let t = progress.clamp(0.0, 1.0);
        match self {
            Self::Linear => (1.0 - t, t),
            Self::EqualPower => {
                // Equal-power crossfade prevents volume dip in the middle (constant sum of squares = 1.0)
                let angle = t * core::f32::consts::FRAC_PI_2;
                (angle.cos(), angle.sin())
            }
            Self::Logarithmic => {
                let out = if t >= 1.0 { 0.0 } else { 1.0 - (1.0 + 9.0 * t).log10() };
                let in_gain = if t <= 0.0 { 0.0 } else { (1.0 + 9.0 * t).log10() };
                (out.clamp(0.0, 1.0), in_gain.clamp(0.0, 1.0))
            }
        }
    }
}

/// Seamless gapless playback and crossfade descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaplessTransitionDescriptor {
    pub from_track_id: String,
    pub to_track_id: String,
    pub crossfade_duration_sec: f32,
    pub curve: CrossfadeCurve,
    pub from_lead_out_samples: u32,
    pub to_lead_in_samples: u32,
}

impl GaplessTransitionDescriptor {
    pub fn new(
        from_track_id: impl Into<String>,
        to_track_id: impl Into<String>,
        crossfade_duration_sec: f32,
        curve: CrossfadeCurve,
    ) -> Self {
        Self {
            from_track_id: from_track_id.into(),
            to_track_id: to_track_id.into(),
            crossfade_duration_sec,
            curve,
            from_lead_out_samples: 0,
            to_lead_in_samples: 0,
        }
    }
}

/// Transcoding decision for client device requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioDeliveryAction {
    /// Direct stream asset with byte ranges (zero CPU transcoding).
    DirectPlay {
        content_type: &'static str,
    },
    /// Transcode on the fly via FFmpeg.
    Transcode {
        target_codec: AudioCodec,
        target_bitrate_kbps: u32,
        apply_ebur128: bool,
        ffmpeg_command: String,
    },
}

/// Evaluator selecting optimal audio delivery action.
pub struct AudioDeliveryOptimizer;

impl AudioDeliveryOptimizer {
    /// Determines whether to stream original audio or transcode.
    #[must_use]
    pub fn resolve_delivery(
        meta: &AudioTrackMetadata,
        client_codecs: &[AudioCodec],
        client_max_bitrate_kbps: u32,
        normalize_audio: bool,
        source_path: &str,
        dest_path: &str,
    ) -> AudioDeliveryAction {
        let codec_supported = client_codecs.contains(&meta.codec);
        let bitrate_acceptable = meta.bitrate_kbps <= client_max_bitrate_kbps;

        if codec_supported && bitrate_acceptable && !normalize_audio {
            return AudioDeliveryAction::DirectPlay {
                content_type: meta.codec.mime_type(),
            };
        }

        // Choose optimal target codec
        let target_codec = if client_codecs.contains(&AudioCodec::Opus) {
            AudioCodec::Opus
        } else if client_codecs.contains(&AudioCodec::Aac) {
            AudioCodec::Aac
        } else {
            AudioCodec::Mp3
        };

        let target_bitrate = meta.bitrate_kbps.min(client_max_bitrate_kbps).max(64);
        let mut filters = Vec::new();
        if normalize_audio {
            filters.push(EbuR128Normalizer::ffmpeg_loudnorm_filter());
        }

        let filter_arg = if filters.is_empty() {
            String::new()
        } else {
            format!("-af \"{}\" ", filters.join(","))
        };

        let codec_arg = match target_codec {
            AudioCodec::Opus => "-c:a libopus",
            AudioCodec::Aac => "-c:a aac",
            AudioCodec::Mp3 => "-c:a libmp3lame",
            AudioCodec::Flac => "-c:a flac",
            _ => "-c:a aac",
        };

        let ffmpeg_command = format!(
            "ffmpeg -v error -i \"{source_path}\" {filter_arg}{codec_arg} -b:a {target_bitrate}k -vn \"{dest_path}\""
        );

        AudioDeliveryAction::Transcode {
            target_codec,
            target_bitrate_kbps: target_bitrate,
            apply_ebur128: normalize_audio,
            ffmpeg_command,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebu_r128_gain_calculation_attenuation() {
        // Track mastered hot at -9.0 LUFS with true peak at +0.5 dBFS
        let hot_track = LoudnessProfile {
            integrated_lufs: -9.0,
            loudness_range_lu: 6.0,
            true_peak_dbfs: 0.5,
        };

        let (gain_db, linear) = EbuR128Normalizer::calculate_gain(&hot_track).unwrap();
        // Target -14.0 LUFS requires -5.0 dB gain. True peak of 0.5 - 5.0 = -4.5 dBFS <= -1.0 dBFS ceiling.
        assert!((gain_db - (-5.0)).abs() < 0.01);
        assert!((linear - 0.5623).abs() < 0.01);
    }

    #[test]
    fn test_ebu_r128_gain_calculation_peak_limiting() {
        // Quiet track at -20.0 LUFS with high dynamic peak of -0.5 dBFS
        let dynamic_track = LoudnessProfile {
            integrated_lufs: -20.0,
            loudness_range_lu: 12.0,
            true_peak_dbfs: -0.5,
        };

        let (gain_db, _) = EbuR128Normalizer::calculate_gain(&dynamic_track).unwrap();
        // Unconstrained gain would be +6.0 dB, but peak ceiling is -1.0 dBFS.
        // Peak limit gain = -1.0 - (-0.5) = -0.5 dB. Clamped!
        assert!((gain_db - (-0.5)).abs() < 0.01);
    }

    #[test]
    fn test_crossfade_equal_power_gain_conservation() {
        let curve = CrossfadeCurve::EqualPower;
        // At midpoint t = 0.5, angle = 45 deg, sin = cos = sqrt(2)/2 ~ 0.7071
        let (out_gain, in_gain) = curve.compute_gains(0.5);
        assert!((out_gain - core::f32::consts::FRAC_1_SQRT_2).abs() < 0.001);
        assert!((in_gain - core::f32::consts::FRAC_1_SQRT_2).abs() < 0.001);

        // Constant power invariant: out^2 + in^2 == 1.0 across all t
        for step in 0..=10 {
            let t = step as f32 / 10.0;
            let (o, i) = curve.compute_gains(t);
            let power = o * o + i * i;
            assert!((power - 1.0).abs() < 0.001, "Power mismatch at t={t}: {power}");
        }
    }

    #[test]
    fn test_delivery_optimizer_direct_vs_transcode() {
        let meta = AudioTrackMetadata {
            codec: AudioCodec::Flac,
            sample_rate: 44100,
            bit_depth: 16,
            channels: AudioChannels::Stereo,
            bitrate_kbps: 900,
            duration_ms: 180000,
            lead_in_samples: 500,
            lead_out_samples: 500,
        };

        // Client supports FLAC with 1000k bandwidth and no normalization requested -> DirectPlay
        let action = AudioDeliveryOptimizer::resolve_delivery(
            &meta,
            &[AudioCodec::Flac, AudioCodec::Aac],
            1000,
            false,
            "track.flac",
            "out.flac",
        );
        assert_eq!(
            action,
            AudioDeliveryAction::DirectPlay {
                content_type: "audio/flac"
            }
        );

        // Client only supports Opus and requests EBU R128 normalization -> Transcode
        let action = AudioDeliveryOptimizer::resolve_delivery(
            &meta,
            &[AudioCodec::Opus, AudioCodec::Mp3],
            256,
            true,
            "track.flac",
            "out.opus",
        );

        match action {
            AudioDeliveryAction::Transcode {
                target_codec,
                target_bitrate_kbps,
                apply_ebur128,
                ffmpeg_command,
            } => {
                assert_eq!(target_codec, AudioCodec::Opus);
                assert_eq!(target_bitrate_kbps, 256);
                assert!(apply_ebur128);
                assert!(ffmpeg_command.contains("loudnorm=I=-14:TP=-1:LRA=11"));
                assert!(ffmpeg_command.contains("-c:a libopus"));
            }
            _ => panic!("Expected transcode action"),
        }
    }
}
