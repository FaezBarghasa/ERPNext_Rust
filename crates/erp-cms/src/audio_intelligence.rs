//! Music Technical Credits (Liner Notes) & Transcription-Based Music Recommendation Engine
//!
//! Provides comprehensive musical metadata (Composers, Producers, Session Musicians,
//! Musical Key / Camelot Wheel, BPM, ISRC) and implements a transcription-driven
//! music recommendation algorithm combining lyric text semantic cosine similarity,
//! harmonic key transition compatibility, and tempo cadence matching.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Camelot Wheel harmonic key for DJ and harmonic playlist transitions.
///
/// Numbers 1..12, Letter 'A' for Minor, 'B' for Major.
/// (e.g., "8A" = A Minor, "8B" = C Major).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CamelotKey {
    /// Number 1 to 12.
    pub number: u8,
    /// True if Minor ('A'), false if Major ('B').
    pub is_minor: bool,
}

impl CamelotKey {
    /// Parses a Camelot key string (e.g., "8A", "11B", "4a").
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim().to_uppercase();
        if trimmed.len() < 2 || trimmed.len() > 3 {
            return None;
        }

        let is_minor = trimmed.ends_with('A');
        let is_major = trimmed.ends_with('B');
        if !is_minor && !is_major {
            return None;
        }

        let num_str = &trimmed[..trimmed.len() - 1];
        let num = num_str.parse::<u8>().ok()?;
        if (1..=12).contains(&num) {
            Some(Self {
                number: num,
                is_minor,
            })
        } else {
            None
        }
    }

    /// Returns standard Camelot notation string (e.g. "8A").
    #[must_use]
    pub fn code(&self) -> String {
        format!("{}{}", self.number, if self.is_minor { 'A' } else { 'B' })
    }

    /// Computes harmonic transition compatibility between two musical keys:
    /// - Identical key: 1.0 (perfect harmonic match)
    /// - Adjacent key (+1 or -1 on circle of fifths): 0.85 (smooth DJ energy shift)
    /// - Relative Major/Minor (same number, A <-> B): 0.80 (harmonic flip)
    /// - Diagonal transition (+/- 1 number AND A <-> B): 0.60
    /// - Other keys: 0.10 (harmonic clash)
    #[must_use]
    pub fn harmonic_compatibility(&self, other: &Self) -> f32 {
        if self.number == other.number && self.is_minor == other.is_minor {
            return 1.0;
        }

        if self.number == other.number && self.is_minor != other.is_minor {
            return 0.80; // Relative major/minor
        }

        // Circular distance on 12-hour wheel
        let diff = (self.number as i16 - other.number as i16).abs();
        let circular_dist = diff.min(12 - diff);

        if circular_dist == 1 && self.is_minor == other.is_minor {
            0.85 // Adjacent fifth
        } else if circular_dist == 1 && self.is_minor != other.is_minor {
            0.60 // Diagonal transition
        } else {
            0.10 // Distant harmonic relationship
        }
    }
}

/// Music personnel role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicContributor {
    pub name: String,
    pub role: String, // e.g. "Lead Vocals", "Producer", "Composer", "Mixing Engineer", "Synthesizer"
}

/// Comprehensive Technical Liner Notes & Music Information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackLinerNotes {
    pub track_id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub release_year: u32,
    pub genre: String,
    pub record_label: Option<String>,
    pub isrc: Option<String>,
    pub upc: Option<String>,
    pub tempo_bpm: f32,
    pub time_signature: String, // e.g. "4/4", "3/4", "6/8"
    pub musical_key: String,    // e.g. "A Minor"
    pub camelot_key: Option<CamelotKey>,
    pub contributors: Vec<MusicContributor>,
    pub transcription_text: String, // Full spoken or sung lyrics transcription
    pub acoustic_energy: f32,       // 0.0 (calm ambient) to 1.0 (energetic electronic/rock)
}

/// Recommendation result ranking candidate tracks with transparent matching rationales.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackRecommendation {
    pub candidate_track_id: String,
    pub title: String,
    pub artist: String,
    pub composite_score: f32, // 0.0 to 1.0
    pub lyric_semantic_score: f32,
    pub harmonic_compatibility_score: f32,
    pub tempo_compatibility_score: f32,
    pub explanation: String,
}

/// Semantic Transcription & Acoustic Music Intelligence Engine.
#[derive(Debug, Default, Clone)]
pub struct MusicIntelligenceEngine;

impl MusicIntelligenceEngine {
    /// Tokenizes and computes a term-frequency vector from lyric transcription text,
    /// removing common stopwords.
    #[must_use]
    pub fn extract_term_frequencies(text: &str) -> HashMap<String, f32> {
        let stopwords: HashSet<&'static str> = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "up", "about", "into", "over", "after", "is", "are", "was", "were", "be",
            "been", "being", "have", "has", "had", "do", "does", "did", "i", "you", "he", "she",
            "it", "we", "they", "my", "your", "his", "her", "its", "our", "their", "that", "this",
            "these", "those", "me", "him", "them", "us",
        ]
        .into_iter()
        .collect();

        let mut counts = HashMap::new();
        let mut total_words = 0.0f32;

        for word in text.split_whitespace() {
            let clean: String = word
                .chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(|c| c.to_lowercase())
                .collect();

            if clean.len() >= 3 && !stopwords.contains(clean.as_str()) {
                *counts.entry(clean).or_insert(0.0) += 1.0;
                total_words += 1.0;
            }
        }

        if total_words > 0.0 {
            for v in counts.values_mut() {
                *v /= total_words;
            }
        }

        counts
    }

    /// Computes cosine similarity between two term-frequency vectors.
    #[must_use]
    pub fn cosine_similarity(v1: &HashMap<String, f32>, v2: &HashMap<String, f32>) -> f32 {
        if v1.is_empty() || v2.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;

        for (term, freq_a) in v1 {
            norm_a += freq_a * freq_a;
            if let Some(freq_b) = v2.get(term) {
                dot_product += freq_a * freq_b;
            }
        }

        for freq_b in v2.values() {
            norm_b += freq_b * freq_b;
        }

        let denom = norm_a.sqrt() * norm_b.sqrt();
        if denom > 0.0 {
            (dot_product / denom).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Computes tempo compatibility score based on BPM ratio.
    ///
    /// Considers direct tempo matches, as well as half-time / double-time matches.
    #[must_use]
    pub fn tempo_compatibility(bpm_a: f32, bpm_b: f32) -> f32 {
        if bpm_a <= 0.0 || bpm_b <= 0.0 {
            return 0.5;
        }

        // Proximity ratio
        let ratio = bpm_a / bpm_b;

        // Check 1:1 match
        let diff_direct = (ratio - 1.0).abs();
        if diff_direct < 0.06 {
            return 1.0; // within 6% BPM is a perfect beatmatch
        }

        // Check half-time or double-time match (e.g. 70 BPM to 140 BPM)
        let diff_half = (ratio - 0.5).abs();
        let diff_double = (ratio - 2.0).abs();
        if diff_half < 0.06 || diff_double < 0.06 {
            return 0.90;
        }

        // Falloff exponential curve
        let log_dist = (ratio.ln()).abs();
        (-log_dist * 2.0).exp().clamp(0.0, 1.0)
    }

    /// Recommends and ranks candidate tracks based on:
    /// - Transcription semantic similarity (50% weight)
    /// - Camelot harmonic musical key compatibility (30% weight)
    /// - Tempo / BPM cadence match (20% weight)
    #[must_use]
    pub fn recommend_next_tracks(
        current_track: &TrackLinerNotes,
        candidates: &[TrackLinerNotes],
        limit: usize,
    ) -> Vec<TrackRecommendation> {
        let current_tf = Self::extract_term_frequencies(&current_track.transcription_text);

        let mut recommendations = Vec::new();

        for candidate in candidates {
            if candidate.track_id == current_track.track_id {
                continue;
            }

            let cand_tf = Self::extract_term_frequencies(&candidate.transcription_text);
            let lyric_sim = Self::cosine_similarity(&current_tf, &cand_tf);

            let harmonic_score = match (current_track.camelot_key, candidate.camelot_key) {
                (Some(k1), Some(k2)) => k1.harmonic_compatibility(&k2),
                _ => 0.5,
            };

            let tempo_score =
                Self::tempo_compatibility(current_track.tempo_bpm, candidate.tempo_bpm);

            let composite = (0.50 * lyric_sim) + (0.30 * harmonic_score) + (0.20 * tempo_score);

            let mut reasons = Vec::new();
            if lyric_sim > 0.3 {
                reasons.push(format!("Lyrical theme match ({:.0}%)", lyric_sim * 100.0));
            }
            if harmonic_score >= 0.8 {
                let k1_code = current_track
                    .camelot_key
                    .map(|k| k.code())
                    .unwrap_or_default();
                let k2_code = candidate.camelot_key.map(|k| k.code()).unwrap_or_default();
                reasons.push(format!("Harmonic key transition ({k1_code} -> {k2_code})"));
            }
            if tempo_score >= 0.85 {
                reasons.push(format!(
                    "Tempo match ({:.0} BPM ~ {:.0} BPM)",
                    current_track.tempo_bpm, candidate.tempo_bpm
                ));
            }
            if reasons.is_empty() {
                reasons.push("Acoustic energy alignment".to_string());
            }

            recommendations.push(TrackRecommendation {
                candidate_track_id: candidate.track_id.clone(),
                title: candidate.title.clone(),
                artist: candidate.artist.clone(),
                composite_score: composite,
                lyric_semantic_score: lyric_sim,
                harmonic_compatibility_score: harmonic_score,
                tempo_compatibility_score: tempo_score,
                explanation: reasons.join(" + "),
            });
        }

        // Sort descending by composite score
        recommendations.sort_by(|a, b| {
            b.composite_score
                .partial_cmp(&a.composite_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        recommendations.truncate(limit);
        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camelot_key_harmonic_compatibility() {
        let key_8a = CamelotKey::parse("8A").unwrap(); // A minor
        let key_8b = CamelotKey::parse("8b").unwrap(); // C major (relative)
        let key_9a = CamelotKey::parse("9A").unwrap(); // E minor (adjacent +1)
        let key_2a = CamelotKey::parse("2A").unwrap(); // E-flat minor (distant)

        assert_eq!(key_8a.harmonic_compatibility(&key_8a), 1.0);
        assert_eq!(key_8a.harmonic_compatibility(&key_9a), 0.85);
        assert_eq!(key_8a.harmonic_compatibility(&key_8b), 0.80);
        assert_eq!(key_8a.harmonic_compatibility(&key_2a), 0.10);
    }

    #[test]
    fn test_tempo_compatibility_direct_and_halftime() {
        // Direct match 128 BPM to 130 BPM
        assert_eq!(
            MusicIntelligenceEngine::tempo_compatibility(128.0, 130.0),
            1.0
        );

        // Half-time match: 70 BPM to 140 BPM
        assert!(MusicIntelligenceEngine::tempo_compatibility(70.0, 140.0) >= 0.90);

        // Clashing tempos: 85 BPM to 145 BPM
        assert!(MusicIntelligenceEngine::tempo_compatibility(85.0, 145.0) < 0.40);
    }

    #[test]
    fn test_transcription_recommendation_ranking() {
        let current = TrackLinerNotes {
            track_id: "track_1".into(),
            title: "Midnight Drive".into(),
            artist: "Synthwave Master".into(),
            album: "Neon Dreams".into(),
            release_year: 2024,
            genre: "Synthwave".into(),
            record_label: Some("Neon Records".into()),
            isrc: Some("USABC2400001".into()),
            upc: None,
            tempo_bpm: 120.0,
            time_signature: "4/4".into(),
            musical_key: "A Minor".into(),
            camelot_key: CamelotKey::parse("8A"),
            contributors: vec![MusicContributor {
                name: "Alex Vance".into(),
                role: "Synthesizer & Producer".into(),
            }],
            transcription_text:
                "Driving through the neon night city lights electric highway lonely skyline".into(),
            acoustic_energy: 0.75,
        };

        let candidate_match = TrackLinerNotes {
            track_id: "track_2".into(),
            title: "Electric Highway".into(),
            artist: "Cyber Runner".into(),
            album: "Overdrive".into(),
            release_year: 2024,
            genre: "Synthwave".into(),
            record_label: Some("Neon Records".into()),
            isrc: Some("USABC2400002".into()),
            upc: None,
            tempo_bpm: 122.0,
            time_signature: "4/4".into(),
            musical_key: "E Minor".into(),
            camelot_key: CamelotKey::parse("9A"),
            contributors: vec![],
            transcription_text: "Fast cars neon night shadows city skyline highway glow".into(),
            acoustic_energy: 0.80,
        };

        let candidate_mismatch = TrackLinerNotes {
            track_id: "track_3".into(),
            title: "Country Acoustic Morning".into(),
            artist: "Barn Duo".into(),
            album: "Hills".into(),
            release_year: 2021,
            genre: "Folk".into(),
            record_label: None,
            isrc: None,
            upc: None,
            tempo_bpm: 82.0,
            time_signature: "3/4".into(),
            musical_key: "E-flat Minor".into(),
            camelot_key: CamelotKey::parse("2A"),
            contributors: vec![],
            transcription_text: "Rooster morning coffee wooden porch sunrise tractor flowers"
                .into(),
            acoustic_energy: 0.30,
        };

        let recs = MusicIntelligenceEngine::recommend_next_tracks(
            &current,
            &[candidate_match.clone(), candidate_mismatch.clone()],
            5,
        );

        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].candidate_track_id, "track_2");
        assert!(recs[0].composite_score > recs[1].composite_score);
        assert!(recs[0].explanation.contains("Harmonic key transition"));
        assert!(recs[0].explanation.contains("Tempo match"));
    }
}
