//! Synchronized Lyrics & Karaoke Engine (LRC / TTML / Syllable Timings)
//!
//! Parses and generates standard Line-Synchronized and Enhanced Word/Syllable-Synchronized
//! LRC formats (Spotify / Apple Music style karaoke). Provides millisecond-accurate active
//! line and word resolution for live UI scrolling and syllable highlighting.

use serde::{Deserialize, Serialize};

/// A single timed syllable or word within a line for karaoke mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyllableTiming {
    /// Text of the syllable or word.
    pub text: String,
    /// Start timestamp in milliseconds.
    pub start_ms: u32,
    /// Duration in milliseconds.
    pub duration_ms: u32,
}

/// A timed lyric line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LyricLine {
    /// Start timestamp in milliseconds from track beginning.
    pub start_ms: u32,
    /// End timestamp in milliseconds (inferred from next line or explicit duration).
    pub end_ms: u32,
    /// Full line text.
    pub text: String,
    /// Optional word/syllable level timing breakdowns for karaoke mode.
    pub syllables: Vec<SyllableTiming>,
}

/// Full lyrics document with metadata headers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LyricsDocument {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub author: Option<String>,
    /// Global time offset in milliseconds (+/-).
    pub offset_ms: i32,
    /// Whether this document contains word/syllable level timings.
    pub is_karaoke: bool,
    /// Ordered list of lyric lines.
    pub lines: Vec<LyricLine>,
}

/// Active lyric highlight snapshot for UI renderers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveLyricSnapshot {
    /// 0-indexed line index, or None if before first line.
    pub current_line_idx: Option<usize>,
    /// Current line text.
    pub current_line_text: Option<String>,
    /// Progress ratio through current line `[0.0, 1.0]`.
    pub line_progress: f32,
    /// Active syllable index if karaoke timings are available.
    pub active_syllable_idx: Option<usize>,
    /// Upcoming next line text for preview.
    pub next_line_text: Option<String>,
}

impl LyricsDocument {
    /// Parses an LRC formatted string (supporting both standard and enhanced syllable formats).
    ///
    /// Example standard:
    /// `[00:12.50] Hello darkness my old friend`
    ///
    /// Example enhanced:
    /// `[00:12.50] <00:12.50> Hello <00:13.00> darkness <00:13.80> my <00:14.20> old <00:14.60> friend`
    #[must_use]
    pub fn parse_lrc(input: &str) -> Self {
        let mut doc = Self::default();
        let mut raw_lines = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check metadata tags: [ar: Artist], [ti: Title], etc.
            if let Some(tag_content) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']'))
                && let Some((key, val)) = tag_content.split_once(':')
            {
                match key.trim() {
                    "ti" => doc.title = Some(val.trim().to_string()),
                    "ar" => doc.artist = Some(val.trim().to_string()),
                    "al" => doc.album = Some(val.trim().to_string()),
                    "by" => doc.author = Some(val.trim().to_string()),
                    "offset" => {
                        if let Ok(offset) = val.trim().parse::<i32>() {
                            doc.offset_ms = offset;
                        }
                    }
                    _ => {}
                }
                continue;
            }

            // Check timed lyric line: [mm:ss.xx] Line content
            if trimmed.starts_with('[')
                && let Some(close_idx) = trimmed.find(']')
            {
                let time_part = &trimmed[1..close_idx];
                let text_part = trimmed[close_idx + 1..].trim();

                if let Some(ms) = Self::parse_timestamp_ms(time_part) {
                    let (cleaned_text, syllables) = Self::parse_syllables(text_part, ms);
                    if !syllables.is_empty() {
                        doc.is_karaoke = true;
                    }
                    raw_lines.push((ms, cleaned_text, syllables));
                }
            }
        }

        // Sort lines by start time
        raw_lines.sort_by_key(|(ms, _, _)| *ms);

        // Build LyricLine structs and calculate end_ms
        let mut lines = Vec::with_capacity(raw_lines.len());
        for i in 0..raw_lines.len() {
            let (start_ms, text, syllables) = raw_lines[i].clone();
            // Default duration is until next line start, or +4000ms for the last line
            let end_ms = if i + 1 < raw_lines.len() {
                raw_lines[i + 1].0
            } else {
                start_ms + 4000
            };

            lines.push(LyricLine {
                start_ms,
                end_ms,
                text,
                syllables,
            });
        }

        doc.lines = lines;
        doc
    }

    /// Parses `mm:ss.xx` or `mm:ss.xxx` into milliseconds.
    fn parse_timestamp_ms(timestamp: &str) -> Option<u32> {
        let (min_str, sec_str) = timestamp.split_once(':')?;
        let min = min_str.parse::<u32>().ok()?;

        let (sec, ms) = if let Some((s_str, frac_str)) = sec_str.split_once('.') {
            let s = s_str.parse::<u32>().ok()?;
            let ms = if frac_str.len() == 2 {
                frac_str.parse::<u32>().ok()? * 10
            } else if frac_str.len() >= 3 {
                frac_str[..3].parse::<u32>().ok()?
            } else {
                frac_str.parse::<u32>().ok()? * 100
            };
            (s, ms)
        } else {
            (sec_str.parse::<u32>().ok()?, 0)
        };

        Some((min * 60 * 1000) + (sec * 1000) + ms)
    }

    /// Parses enhanced syllable/word markers `<mm:ss.xx> word`.
    fn parse_syllables(text: &str, line_start_ms: u32) -> (String, Vec<SyllableTiming>) {
        if !text.contains('<') || !text.contains('>') {
            return (text.to_string(), Vec::new());
        }

        let mut cleaned_words = Vec::new();
        let mut syllables = Vec::new();
        let mut chars = text.chars().peekable();

        let mut current_word = String::new();
        let mut current_time: Option<u32> = None;

        while let Some(c) = chars.next() {
            if c == '<' {
                // If we had a previous word buffered
                if !current_word.trim().is_empty() {
                    let w = current_word.trim().to_string();
                    cleaned_words.push(w.clone());
                    let start = current_time.unwrap_or(line_start_ms);
                    syllables.push(SyllableTiming {
                        text: w,
                        start_ms: start,
                        duration_ms: 300, // placeholder, updated later
                    });
                    current_word.clear();
                }

                // Read timestamp until '>'
                let mut tag = String::new();
                for inner in chars.by_ref() {
                    if inner == '>' {
                        break;
                    }
                    tag.push(inner);
                }
                current_time = Self::parse_timestamp_ms(&tag);
            } else {
                current_word.push(c);
            }
        }

        if !current_word.trim().is_empty() {
            let w = current_word.trim().to_string();
            cleaned_words.push(w.clone());
            let start = current_time.unwrap_or(line_start_ms);
            syllables.push(SyllableTiming {
                text: w,
                start_ms: start,
                duration_ms: 400,
            });
        }

        // Adjust durations between consecutive syllables
        for i in 0..syllables.len() {
            if i + 1 < syllables.len() {
                let next_start = syllables[i + 1].start_ms;
                if next_start > syllables[i].start_ms {
                    syllables[i].duration_ms = next_start - syllables[i].start_ms;
                }
            }
        }

        (cleaned_words.join(" "), syllables)
    }

    /// Resolves active lyric line and active syllable for a given playhead millisecond offset.
    #[must_use]
    pub fn get_active_snapshot(&self, position_ms: u32) -> ActiveLyricSnapshot {
        if self.lines.is_empty() {
            return ActiveLyricSnapshot {
                current_line_idx: None,
                current_line_text: None,
                line_progress: 0.0,
                active_syllable_idx: None,
                next_line_text: None,
            };
        }

        // Apply global offset
        let adjusted_pos = ((position_ms as i64) - (self.offset_ms as i64)).max(0) as u32;

        let mut current_idx = None;
        for (i, line) in self.lines.iter().enumerate() {
            if adjusted_pos >= line.start_ms && adjusted_pos < line.end_ms {
                current_idx = Some(i);
                break;
            }
        }

        // If between lines or past last line, find closest previous line
        if current_idx.is_none() {
            for (i, line) in self.lines.iter().enumerate().rev() {
                if adjusted_pos >= line.start_ms {
                    current_idx = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = current_idx {
            let line = &self.lines[idx];
            let line_duration = line.end_ms.saturating_sub(line.start_ms).max(1) as f32;
            let elapsed_in_line = adjusted_pos.saturating_sub(line.start_ms) as f32;
            let progress = (elapsed_in_line / line_duration).clamp(0.0, 1.0);

            // Find active syllable if available
            let mut active_syl = None;
            for (s_idx, syl) in line.syllables.iter().enumerate() {
                if adjusted_pos >= syl.start_ms && adjusted_pos < (syl.start_ms + syl.duration_ms) {
                    active_syl = Some(s_idx);
                    break;
                }
            }

            let next_text = self.lines.get(idx + 1).map(|l| l.text.clone());

            ActiveLyricSnapshot {
                current_line_idx: Some(idx),
                current_line_text: Some(line.text.clone()),
                line_progress: progress,
                active_syllable_idx: active_syl,
                next_line_text: next_text,
            }
        } else {
            // Before first lyric
            ActiveLyricSnapshot {
                current_line_idx: None,
                current_line_text: None,
                line_progress: 0.0,
                active_syllable_idx: None,
                next_line_text: self.lines.first().map(|l| l.text.clone()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_lrc() {
        let lrc = r#"
[ti:Midnight City]
[ar:M83]
[al:Hurry Up, We're Dreaming]
[00:10.00]Waiting in a car
[00:15.50]Waiting for a ride in the dark
[00:22.00]The city is my church
"#;
        let doc = LyricsDocument::parse_lrc(lrc);
        assert_eq!(doc.title.as_deref(), Some("Midnight City"));
        assert_eq!(doc.artist.as_deref(), Some("M83"));
        assert_eq!(doc.lines.len(), 3);
        assert_eq!(doc.lines[0].start_ms, 10_000);
        assert_eq!(doc.lines[0].end_ms, 15_500);
        assert_eq!(doc.lines[0].text, "Waiting in a car");

        // Active snapshot at 12s (middle of line 0)
        let snap = doc.get_active_snapshot(12_000);
        assert_eq!(snap.current_line_idx, Some(0));
        assert_eq!(snap.current_line_text.as_deref(), Some("Waiting in a car"));
        assert!(snap.line_progress > 0.3 && snap.line_progress < 0.5);
        assert_eq!(
            snap.next_line_text.as_deref(),
            Some("Waiting for a ride in the dark")
        );
    }

    #[test]
    fn test_parse_enhanced_karaoke_lrc() {
        let lrc = r#"
[00:05.00]<00:05.00>Never <00:05.80>gonna <00:06.40>give <00:07.00>you <00:07.50>up
"#;
        let doc = LyricsDocument::parse_lrc(lrc);
        assert!(doc.is_karaoke);
        assert_eq!(doc.lines.len(), 1);
        let line = &doc.lines[0];
        assert_eq!(line.text, "Never gonna give you up");
        assert_eq!(line.syllables.len(), 5);
        assert_eq!(line.syllables[0].text, "Never");
        assert_eq!(line.syllables[0].start_ms, 5_000);
        assert_eq!(line.syllables[0].duration_ms, 800); // 5800 - 5000

        // Snapshot inside "gonna" at 6000ms
        let snap = doc.get_active_snapshot(6_000);
        assert_eq!(snap.active_syllable_idx, Some(1));
    }
}
