//! Parametric & Graphic Audio Equalizer (EQ) DSP Engine
//!
//! Provides a 10-band graphic / parametric biquad equalizer (31Hz to 16kHz),
//! industry-standard mastering and playback presets (Bass Boost, Vocal, Acoustic,
//! Electronic, Classical, Podcast, Sleep), and generates optimized FFmpeg
//! audio filter graphs for realtime and offline rendering.

use serde::{Deserialize, Serialize};

/// 10 standard ISO frequency centers in Hertz.
pub const EQ_FREQUENCIES_HZ: [u32; 10] = [31, 63, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

/// Standard EQ presets for consumer, audiophile, and spoken audio listening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EqPreset {
    #[default]
    Flat,
    BassBoost,
    BassReducer,
    VocalBoost,
    Acoustic,
    Electronic,
    Rock,
    Classical,
    PodcastSpeech,
    SleepRelax,
}

impl EqPreset {
    /// Returns 10-band gain offsets in decibels `[-12.0, +12.0]` for the preset.
    #[must_use]
    pub const fn band_gains_db(&self) -> [f32; 10] {
        match self {
            Self::Flat => [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            // Heavy low-end emphasis for sub-bass/bass
            Self::BassBoost => [6.0, 5.5, 4.0, 2.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
            // Low-end attenuation for boominess reduction
            Self::BassReducer => [-5.0, -4.5, -3.5, -2.0, -0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
            // Mid-range presence boost (500Hz - 4kHz) to bring forward human vocal clarity
            Self::VocalBoost => [-2.0, -1.0, 0.0, 1.5, 3.5, 4.0, 3.0, 1.5, 0.0, -1.0],
            // Warm lows, subtle mid dip, crisp acoustic instrument presence
            Self::Acoustic => [3.0, 2.5, 1.5, 0.5, 1.0, 2.0, 2.5, 3.0, 2.5, 1.5],
            // Punchy sub-bass, scoop in low-mids, vibrant airy treble
            Self::Electronic => [5.5, 5.0, 2.0, 0.0, -1.5, 1.5, 0.0, 2.0, 4.0, 4.5],
            // Classic rock V-curve
            Self::Rock => [4.5, 3.5, 2.0, -0.5, -1.5, 0.5, 2.0, 3.5, 4.0, 4.5],
            // Smooth warm classical response with open air
            Self::Classical => [3.5, 3.0, 2.0, 1.0, -0.5, -0.5, 0.0, 1.5, 2.5, 3.0],
            // High-pass vocal clarity, suppresses low rumble and sibilant hiss
            Self::PodcastSpeech => [-6.0, -4.0, 0.0, 2.0, 4.0, 3.5, 2.5, 1.0, -2.0, -4.0],
            // Warm, soft acoustic roll-off suppressing harsh highs for sleep and meditation
            Self::SleepRelax => [2.0, 2.0, 1.5, 0.5, 0.0, -1.0, -2.5, -4.5, -6.5, -9.0],
        }
    }
}

/// A single parametric EQ band specification.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EqBand {
    /// Center frequency in Hz (e.g. 1000).
    pub frequency_hz: u32,
    /// Gain in decibels, clamped to `[-15.0, +15.0]`.
    pub gain_db: f32,
    /// Quality factor Q (width of peak/dip). Standard graphic EQ uses ~1.414.
    pub q_factor: f32,
}

impl EqBand {
    /// Creates a new band with validation.
    #[must_use]
    pub fn new(frequency_hz: u32, gain_db: f32, q_factor: f32) -> Self {
        Self {
            frequency_hz: frequency_hz.clamp(20, 20_000),
            gain_db: gain_db.clamp(-15.0, 15.0),
            q_factor: q_factor.clamp(0.1, 10.0),
        }
    }
}

/// 10-band Graphic / Parametric Equalizer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioEqualizer {
    /// Active 10 bands.
    pub bands: [EqBand; 10],
    /// Optional master preamp gain in dB to prevent digital clipping when boosting bands.
    pub preamp_gain_db: f32,
}

impl Default for AudioEqualizer {
    fn default() -> Self {
        Self::from_preset(EqPreset::Flat)
    }
}

impl AudioEqualizer {
    /// Creates a new equalizer initialized from an industry preset.
    #[must_use]
    pub fn from_preset(preset: EqPreset) -> Self {
        let gains = preset.band_gains_db();
        let mut bands = [EqBand::new(1000, 0.0, 1.414); 10];

        let mut max_boost = 0.0f32;
        for (i, &freq) in EQ_FREQUENCIES_HZ.iter().enumerate() {
            let g = gains[i];
            if g > max_boost {
                max_boost = g;
            }
            bands[i] = EqBand::new(freq, g, 1.414);
        }

        // Automatic preamp attenuation: if highest boost is +6dB, set preamp to -6dB to avoid clipping
        let preamp_gain_db = if max_boost > 0.0 { -max_boost } else { 0.0 };

        Self {
            bands,
            preamp_gain_db,
        }
    }

    /// Sets custom gain for a specific frequency band index (0..9).
    pub fn set_band_gain(&mut self, band_idx: usize, gain_db: f32) {
        if band_idx < 10 {
            self.bands[band_idx].gain_db = gain_db.clamp(-15.0, 15.0);
            self.recalculate_preamp();
        }
    }

    /// Automatically recalculates safety preamp attenuation based on maximum band boost.
    pub fn recalculate_preamp(&mut self) {
        let max_boost = self
            .bands
            .iter()
            .map(|b| b.gain_db)
            .fold(0.0f32, |acc, g| if g > acc { g } else { acc });
        self.preamp_gain_db = if max_boost > 0.0 { -max_boost } else { 0.0 };
    }

    /// Builds an FFmpeg equalizer filter string chaining biquad filters or volume preamp.
    ///
    /// Example output:
    /// `volume=volume=-6.0dB,equalizer=f=31:width_type=q:w=1.414:g=6.0,equalizer=f=63:width_type=q:w=1.414:g=5.5...`
    #[must_use]
    pub fn build_ffmpeg_filter(&self) -> String {
        let mut filters = Vec::with_capacity(11);

        if self.preamp_gain_db.abs() > 0.01 {
            filters.push(format!("volume=volume={:.1}dB", self.preamp_gain_db));
        }

        for band in &self.bands {
            if band.gain_db.abs() > 0.01 {
                filters.push(format!(
                    "equalizer=f={}:width_type=q:w={:.3}:g={:.1}",
                    band.frequency_hz, band.q_factor, band.gain_db
                ));
            }
        }

        if filters.is_empty() {
            "anull".to_string()
        } else {
            filters.join(",")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_presets_and_preamp() {
        let bass = AudioEqualizer::from_preset(EqPreset::BassBoost);
        assert_eq!(bass.bands[0].frequency_hz, 31);
        assert_eq!(bass.bands[0].gain_db, 6.0);
        // Preamp should be -6.0 to prevent clipping
        assert_eq!(bass.preamp_gain_db, -6.0);

        let filter = bass.build_ffmpeg_filter();
        assert!(filter.contains("volume=volume=-6.0dB"));
        assert!(filter.contains("equalizer=f=31:width_type=q:w=1.414:g=6.0"));
    }

    #[test]
    fn test_flat_eq_filter() {
        let flat = AudioEqualizer::from_preset(EqPreset::Flat);
        assert_eq!(flat.build_ffmpeg_filter(), "anull");
    }

    #[test]
    fn test_custom_band_adjustment() {
        let mut eq = AudioEqualizer::default();
        eq.set_band_gain(5, 4.5); // 1000Hz band
        assert_eq!(eq.bands[5].gain_db, 4.5);
        assert_eq!(eq.preamp_gain_db, -4.5);

        let filter = eq.build_ffmpeg_filter();
        assert!(filter.contains("equalizer=f=1000:width_type=q:w=1.414:g=4.5"));
    }
}
