//! High-Performance Lossless Audio Pipeline & FLAC Compression Engine
//!
//! Provides automated ingestion, transcoding, and compression of any uploaded
//! audio format (WAV, AIFF, MP3, AAC, OGG, ALAC) into optimized FLAC archives
//! with level 0..8 compression, 16/24-bit studio quality profiling, MD5 integrity
//! verification, and embedded VorbisComment tags.

use serde::{Deserialize, Serialize};

/// High-resolution audio tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FlacQualityTier {
    /// 16-bit / 44.1 kHz or 48 kHz (Red Book CD Quality).
    CdQuality,
    /// 24-bit / 44.1 kHz or 48 kHz (Studio Master).
    StudioMaster24,
    /// 24-bit / 96 kHz or 192 kHz (Ultra Hi-Res Audiophile).
    HiResAudiophile,
    /// Transcoded from lossy source (MP3/AAC/OGG) into lossless FLAC container.
    TranscodedFromLossy,
}

impl FlacQualityTier {
    #[must_use]
    pub const fn badge(&self) -> &'static str {
        match self {
            Self::CdQuality => "FLAC 16-bit/44.1kHz [Lossless]",
            Self::StudioMaster24 => "FLAC 24-bit/48kHz [Studio Master]",
            Self::HiResAudiophile => "FLAC 24-bit/96kHz+ [Hi-Res Lossless]",
            Self::TranscodedFromLossy => "FLAC Container [Lossy Source]",
        }
    }
}

/// Parameters for FLAC encoding & compression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlacCompressionConfig {
    /// FLAC compression level (0 = fastest, 5 = balanced default, 8 = highest compression).
    pub compression_level: u8,
    /// Target sample rate (e.g. 44100, 48000, 96000, 192000), or None to preserve source.
    pub target_sample_rate: Option<u32>,
    /// Target bit depth (16 or 24), or None to preserve source.
    pub target_bit_depth: Option<u8>,
    /// Whether to embed MD5 audio checksum inside the FLAC header.
    pub verify_md5: bool,
    /// Block size (e.g. 4096 or 4608 for 44.1kHz).
    pub block_size: u32,
}

impl Default for FlacCompressionConfig {
    fn default() -> Self {
        Self {
            compression_level: 8, // Maximum compression by default for storage efficiency
            target_sample_rate: None,
            target_bit_depth: None,
            verify_md5: true,
            block_size: 4096,
        }
    }
}

/// Metadata tags to embed into the FLAC VorbisComment block.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlacMetadataTags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub isrc: Option<String>,
    pub lyrics: Option<String>,
    pub comment: Option<String>,
}

/// Plan for transcoding an uploaded audio file into compressed FLAC.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlacTranscodeJobPlan {
    pub input_path: String,
    pub output_flac_path: String,
    pub input_format: String,
    pub quality_tier: FlacQualityTier,
    pub config: FlacCompressionConfig,
    pub ffmpeg_command: String,
    pub estimated_compression_ratio: f32,
}

/// Engine managing automated upload compression to FLAC.
#[derive(Debug, Default, Clone)]
pub struct FlacPipelineEngine;

impl FlacPipelineEngine {
    /// Determines the audio quality tier from bit depth, sample rate, and codec format.
    #[must_use]
    pub fn classify_tier(input_format: &str, sample_rate: u32, bit_depth: u8) -> FlacQualityTier {
        let fmt = input_format.to_lowercase();
        let is_lossy = matches!(fmt.as_str(), "mp3" | "aac" | "ogg" | "opus" | "m4a" | "wma");

        if is_lossy {
            FlacQualityTier::TranscodedFromLossy
        } else if bit_depth >= 24 && sample_rate >= 88_200 {
            FlacQualityTier::HiResAudiophile
        } else if bit_depth >= 24 {
            FlacQualityTier::StudioMaster24
        } else {
            FlacQualityTier::CdQuality
        }
    }

    /// Prepares an automated FFmpeg transcode plan to compress any audio file to FLAC.
    #[must_use]
    pub fn plan_flac_compression(
        input_path: &str,
        output_flac_path: &str,
        input_format: &str,
        sample_rate: u32,
        bit_depth: u8,
        tags: Option<&FlacMetadataTags>,
        config: Option<FlacCompressionConfig>,
    ) -> FlacTranscodeJobPlan {
        let cfg = config.unwrap_or_default();
        let tier = Self::classify_tier(input_format, sample_rate, bit_depth);

        // Typical FLAC achieves ~55% of raw PCM size for music, ~40% for speech
        let estimated_compression_ratio = match tier {
            FlacQualityTier::TranscodedFromLossy => 0.85, // Lossy to lossless increases file size
            FlacQualityTier::HiResAudiophile => 0.50,
            FlacQualityTier::StudioMaster24 => 0.55,
            FlacQualityTier::CdQuality => 0.58,
        };

        // Build FFmpeg command arguments
        let mut args = vec![
            "-hide_banner".to_string(),
            "-y".to_string(),
            "-i".to_string(),
            format!("\"{input_path}\""),
            "-c:a".to_string(),
            "flac".to_string(),
            "-compression_level".to_string(),
            cfg.compression_level.clamp(0, 8).to_string(),
        ];

        // Sample rate override
        if let Some(target_sr) = cfg.target_sample_rate {
            args.push("-ar".to_string());
            args.push(target_sr.to_string());
        }

        // Bit depth precision override
        if let Some(target_bd) = cfg.target_bit_depth {
            let sample_fmt = if target_bd >= 24 { "s32" } else { "s16" };
            args.push("-sample_fmt".to_string());
            args.push(sample_fmt.to_string());
        }

        // Append VorbisComment tags
        if let Some(t) = tags {
            if let Some(ref title) = t.title {
                args.push("-metadata".to_string());
                args.push(format!("title=\"{title}\""));
            }
            if let Some(ref artist) = t.artist {
                args.push("-metadata".to_string());
                args.push(format!("artist=\"{artist}\""));
            }
            if let Some(ref album) = t.album {
                args.push("-metadata".to_string());
                args.push(format!("album=\"{album}\""));
            }
            if let Some(ref track) = t.track_number {
                args.push("-metadata".to_string());
                args.push(format!("track=\"{track}\""));
            }
            if let Some(ref year) = t.year {
                args.push("-metadata".to_string());
                args.push(format!("date=\"{year}\""));
            }
            if let Some(ref isrc) = t.isrc {
                args.push("-metadata".to_string());
                args.push(format!("isrc=\"{isrc}\""));
            }
            if let Some(ref lyrics) = t.lyrics {
                // Escape quotes in lyrics
                let safe_lyrics = lyrics.replace('"', "\\\"");
                args.push("-metadata".to_string());
                args.push(format!("lyrics=\"{safe_lyrics}\""));
            }
        }

        args.push(format!("\"{output_flac_path}\""));
        let ffmpeg_command = format!("ffmpeg {}", args.join(" "));

        FlacTranscodeJobPlan {
            input_path: input_path.to_string(),
            output_flac_path: output_flac_path.to_string(),
            input_format: input_format.to_string(),
            quality_tier: tier,
            config: cfg,
            ffmpeg_command,
            estimated_compression_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flac_quality_classification() {
        assert_eq!(
            FlacPipelineEngine::classify_tier("wav", 44100, 16),
            FlacQualityTier::CdQuality
        );
        assert_eq!(
            FlacPipelineEngine::classify_tier("aiff", 48000, 24),
            FlacQualityTier::StudioMaster24
        );
        assert_eq!(
            FlacPipelineEngine::classify_tier("wav", 96000, 24),
            FlacQualityTier::HiResAudiophile
        );
        assert_eq!(
            FlacPipelineEngine::classify_tier("mp3", 44100, 16),
            FlacQualityTier::TranscodedFromLossy
        );
    }

    #[test]
    fn test_plan_flac_compression_command() {
        let tags = FlacMetadataTags {
            title: Some("Resonance".into()),
            artist: Some("HOME".into()),
            album: Some("Odyssey".into()),
            track_number: Some(1),
            year: Some(2014),
            isrc: Some("US1234567890".into()),
            lyrics: Some("Synthetic dreams".into()),
            ..Default::default()
        };

        let plan = FlacPipelineEngine::plan_flac_compression(
            "/uploads/raw_master.wav",
            "/media/flac/resonance.flac",
            "wav",
            96000,
            24,
            Some(&tags),
            None,
        );

        assert_eq!(plan.quality_tier, FlacQualityTier::HiResAudiophile);
        assert_eq!(
            plan.quality_tier.badge(),
            "FLAC 24-bit/96kHz+ [Hi-Res Lossless]"
        );
        assert!(plan.ffmpeg_command.contains("-c:a flac"));
        assert!(plan.ffmpeg_command.contains("-compression_level 8"));
        assert!(plan.ffmpeg_command.contains("title=\"Resonance\""));
        assert!(plan.ffmpeg_command.contains("artist=\"HOME\""));
        assert!(plan.ffmpeg_command.contains("isrc=\"US1234567890\""));
    }
}
