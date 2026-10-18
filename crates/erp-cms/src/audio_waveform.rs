//! Audio Waveform Extraction and SoundCloud-Style Timed Comments Engine
//!
//! Provides zero-allocation downsampling of raw PCM audio into normalized
//! 100-point (thumbnail) and 1800-point (full scrub bar) peak arrays (0..255).
//! Generates vector SVG waveform paths for zero-JS immediate UI rendering and
//! manages millisecond-accurate timed comments with temporal cluster bucketing.

use serde::{Deserialize, Serialize};

/// Fixed size thumbnail waveform for compact UI previews and embeds.
pub const THUMBNAIL_PEAK_COUNT: usize = 100;
/// High resolution waveform for detailed seekbars and DAW-style timeline scrubbing.
pub const SCRUB_PEAK_COUNT: usize = 1800;

/// Normalized waveform peak summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioWaveformData {
    /// 100-point mini waveform for track listings and preview cards.
    pub thumbnail_peaks: Vec<u8>,
    /// 1800-point high-resolution waveform for main scrubbing player.
    pub scrub_peaks: Vec<u8>,
    /// Peak amplitude recorded across the original source (0..255).
    pub max_peak: u8,
    /// Total duration in milliseconds for timeline mapping.
    pub duration_ms: u64,
}

/// Waveform generation and normalization engine.
#[derive(Debug, Default, Clone)]
pub struct WaveformGenerator;

impl WaveformGenerator {
    /// Downsamples raw mono 16-bit signed PCM audio into normalized peak arrays.
    ///
    /// Values are mapped linearly between 0 and 255, with silent or empty buffers
    /// resulting in zeroes.
    #[must_use]
    pub fn from_pcm16_mono(pcm_samples: &[i16], duration_ms: u64) -> AudioWaveformData {
        if pcm_samples.is_empty() {
            return AudioWaveformData {
                thumbnail_peaks: vec![0u8; THUMBNAIL_PEAK_COUNT],
                scrub_peaks: vec![0u8; SCRUB_PEAK_COUNT],
                max_peak: 0,
                duration_ms,
            };
        }

        // Find absolute peak for dynamic normalization
        let max_abs = pcm_samples
            .iter()
            .map(|&s| {
                if s == i16::MIN {
                    i16::MAX as u32
                } else {
                    s.unsigned_abs() as u32
                }
            })
            .max()
            .unwrap_or(0);

        let normalize = |val: u32| -> u8 {
            (val * 255)
                .checked_div(max_abs)
                .map_or(0, |res| res.min(255) as u8)
        };

        let thumbnail_peaks = Self::extract_buckets(pcm_samples, THUMBNAIL_PEAK_COUNT, normalize);
        let scrub_peaks = Self::extract_buckets(pcm_samples, SCRUB_PEAK_COUNT, normalize);
        let max_peak = thumbnail_peaks.iter().copied().max().unwrap_or(0);

        AudioWaveformData {
            thumbnail_peaks,
            scrub_peaks,
            max_peak,
            duration_ms,
        }
    }

    /// Downsamples PCM samples into `target_count` buckets, taking the RMS or peak per bucket.
    fn extract_buckets<F>(pcm: &[i16], target_count: usize, normalize: F) -> Vec<u8>
    where
        F: Fn(u32) -> u8,
    {
        if pcm.is_empty() || target_count == 0 {
            return vec![0u8; target_count];
        }

        let mut result = Vec::with_capacity(target_count);
        let sample_len = pcm.len();

        for i in 0..target_count {
            let start = (i * sample_len) / target_count;
            let end = (((i + 1) * sample_len) / target_count)
                .max(start + 1)
                .min(sample_len);

            let mut peak = 0u32;
            for &sample in &pcm[start..end] {
                let abs_val = if sample == i16::MIN {
                    i16::MAX as u32
                } else {
                    sample.unsigned_abs() as u32
                };
                if abs_val > peak {
                    peak = abs_val;
                }
            }
            result.push(normalize(peak));
        }

        result
    }

    /// Renders an SVG waveform path with mirror reflection for server-rendered UI.
    ///
    /// # Arguments
    /// * `peaks` - Slice of peaks (typically `thumbnail_peaks` or slice of `scrub_peaks`).
    /// * `width` - Output SVG viewBox width in pixels.
    /// * `height` - Output SVG viewBox height in pixels.
    /// * `played_pct` - Percentage (0..100) played to color progress bar.
    #[must_use]
    pub fn render_svg_waveform(peaks: &[u8], width: f32, height: f32, played_pct: f32) -> String {
        if peaks.is_empty() {
            return format!(
                r##"<svg viewBox="0 0 {width} {height}" width="100%" height="100%" xmlns="http://www.w3.org/2000/svg"><line x1="0" y1="{mid}" x2="{width}" y2="{mid}" stroke="#555" stroke-width="2"/></svg>"##,
                mid = height / 2.0
            );
        }

        let half_h = height / 2.0;
        let n = peaks.len();
        let step = width / (n as f32);
        let bar_width = (step * 0.75).max(1.0);
        let played_x = (width * (played_pct.clamp(0.0, 100.0) / 100.0)).max(0.0);

        let mut played_bars = String::with_capacity(peaks.len() * 64);
        let mut unplayed_bars = String::with_capacity(peaks.len() * 64);

        for (i, &peak) in peaks.iter().enumerate() {
            let x = i as f32 * step;
            let bar_h = ((peak as f32 / 255.0) * half_h).max(1.0);
            let y_top = half_h - bar_h;
            let total_bar_h = bar_h * 2.0;

            let rect = format!(
                r#"<rect x="{x:.1}" y="{y_top:.1}" width="{bar_width:.1}" height="{total_bar_h:.1}" rx="1" />"#
            );

            if x <= played_x {
                played_bars.push_str(&rect);
            } else {
                unplayed_bars.push_str(&rect);
            }
        }

        format!(
            r##"<svg viewBox="0 0 {width} {height}" width="100%" height="100%" xmlns="http://www.w3.org/2000/svg"><g fill="#f97316">{played_bars}</g><g fill="#64748b" opacity="0.6">{unplayed_bars}</g></svg>"##
        )
    }
}

/// SoundCloud-style timed comment tied to an exact millisecond along the playback timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedComment {
    /// Unique identifier for the comment.
    pub id: String,
    /// Author username.
    pub user: String,
    /// Avatar image URL.
    pub avatar_url: Option<String>,
    /// Millisecond offset within the track.
    pub timestamp_ms: u32,
    /// Comment body content.
    pub text: String,
    /// Upvotes / likes received.
    pub upvotes: u32,
    /// Unix timestamp when created.
    pub created_at: i64,
}

/// Aggregated comment density cluster for timeline heatmap bubbles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentCluster {
    /// Start of bucket range in ms.
    pub start_ms: u32,
    /// End of bucket range in ms.
    pub end_ms: u32,
    /// Count of comments falling into this temporal bucket.
    pub count: usize,
    /// Sample avatars to render in cluster bubble.
    pub sample_avatars: Vec<String>,
}

/// Timed comment collection with clustering capabilities.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TimedCommentTimeline {
    /// Ordered comments.
    pub comments: Vec<TimedComment>,
}

impl TimedCommentTimeline {
    /// Creates a new empty timeline.
    #[must_use]
    pub fn new() -> Self {
        Self {
            comments: Vec::new(),
        }
    }

    /// Adds a timed comment and maintains sorted order by `timestamp_ms`.
    pub fn add_comment(&mut self, comment: TimedComment) {
        let idx = self
            .comments
            .binary_search_by_key(&comment.timestamp_ms, |c| c.timestamp_ms)
            .unwrap_or_else(|e| e);
        self.comments.insert(idx, comment);
    }

    /// Finds comments within a given playback window `[start_ms, end_ms]`.
    #[must_use]
    pub fn get_window(&self, start_ms: u32, end_ms: u32) -> &[TimedComment] {
        if self.comments.is_empty() || start_ms > end_ms {
            return &[];
        }

        let start_idx = self
            .comments
            .binary_search_by_key(&start_ms, |c| c.timestamp_ms)
            .unwrap_or_else(|e| e);

        let end_idx = self
            .comments
            .binary_search_by_key(&(end_ms + 1), |c| c.timestamp_ms)
            .unwrap_or_else(|e| e);

        &self.comments[start_idx..end_idx.min(self.comments.len())]
    }

    /// Clusters comments into `bucket_count` segments along `duration_ms` for timeline badges.
    #[must_use]
    pub fn cluster_density(&self, duration_ms: u64, bucket_count: usize) -> Vec<CommentCluster> {
        if bucket_count == 0 || duration_ms == 0 {
            return Vec::new();
        }

        let bucket_duration = (duration_ms / (bucket_count as u64)).max(1);
        let mut clusters = Vec::with_capacity(bucket_count);

        for b in 0..bucket_count {
            let start = (b as u64 * bucket_duration) as u32;
            let end = (((b + 1) as u64 * bucket_duration).min(duration_ms)) as u32;

            let in_bucket = self.get_window(start, end);
            let mut avatars = Vec::new();

            for c in in_bucket {
                if let Some(ref av) = c.avatar_url
                    && avatars.len() < 3
                    && !avatars.contains(av)
                {
                    avatars.push(av.clone());
                }
            }

            clusters.push(CommentCluster {
                start_ms: start,
                end_ms: end,
                count: in_bucket.len(),
                sample_avatars: avatars,
            });
        }

        clusters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcm16_waveform_extraction() {
        // Synthesize 1000 samples of a simple alternating wave
        let mut samples = Vec::with_capacity(1000);
        for i in 0..1000 {
            let val = ((i % 100) as f32 / 100.0 * (i16::MAX as f32)) as i16;
            samples.push(val);
        }

        let waveform = WaveformGenerator::from_pcm16_mono(&samples, 5000);
        assert_eq!(waveform.thumbnail_peaks.len(), THUMBNAIL_PEAK_COUNT);
        assert_eq!(waveform.scrub_peaks.len(), SCRUB_PEAK_COUNT);
        assert_eq!(waveform.duration_ms, 5000);
        assert!(waveform.max_peak > 0);
    }

    #[test]
    fn test_empty_pcm_waveform() {
        let waveform = WaveformGenerator::from_pcm16_mono(&[], 0);
        assert_eq!(waveform.thumbnail_peaks, vec![0u8; THUMBNAIL_PEAK_COUNT]);
        assert_eq!(waveform.max_peak, 0);
    }

    #[test]
    fn test_svg_waveform_render() {
        let peaks = [50u8, 120u8, 255u8, 80u8];
        let svg = WaveformGenerator::render_svg_waveform(&peaks, 400.0, 80.0, 50.0);
        assert!(svg.contains(r#"<svg viewBox="0 0 400 80""#));
        assert!(svg.contains(r##"fill="#f97316""##));
        assert!(svg.contains(r##"fill="#64748b""##));
    }

    #[test]
    fn test_timed_comments_timeline() {
        let mut timeline = TimedCommentTimeline::new();
        timeline.add_comment(TimedComment {
            id: "c1".into(),
            user: "alice".into(),
            avatar_url: Some("alice.png".into()),
            timestamp_ms: 15_000,
            text: "Sick drop!".into(),
            upvotes: 42,
            created_at: 1700000000,
        });
        timeline.add_comment(TimedComment {
            id: "c2".into(),
            user: "bob".into(),
            avatar_url: None,
            timestamp_ms: 5_000,
            text: "Nice intro".into(),
            upvotes: 12,
            created_at: 1700000005,
        });
        timeline.add_comment(TimedComment {
            id: "c3".into(),
            user: "charlie".into(),
            avatar_url: Some("charlie.png".into()),
            timestamp_ms: 15_200,
            text: "Agreed!!".into(),
            upvotes: 8,
            created_at: 1700000010,
        });

        // Sorted check
        assert_eq!(timeline.comments[0].timestamp_ms, 5_000);
        assert_eq!(timeline.comments[1].timestamp_ms, 15_000);
        assert_eq!(timeline.comments[2].timestamp_ms, 15_200);

        // Window query
        let window = timeline.get_window(14_000, 16_000);
        assert_eq!(window.len(), 2);
        assert_eq!(window[0].id, "c1");
        assert_eq!(window[1].id, "c3");

        // Clustering
        let clusters = timeline.cluster_density(30_000, 3);
        assert_eq!(clusters.len(), 3);
        // Bucket 0: 0..10_000 -> has c2 (1)
        assert_eq!(clusters[0].count, 1);
        // Bucket 1: 10_000..20_000 -> has c1, c3 (2)
        assert_eq!(clusters[1].count, 2);
        assert_eq!(clusters[1].sample_avatars.len(), 2);
        // Bucket 2: 20_000..30_000 -> 0
        assert_eq!(clusters[2].count, 0);
    }
}
