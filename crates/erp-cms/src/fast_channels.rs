//! 24/7 Virtual FAST Linear Channel Playout Engine (`erp-cms::fast_channels`).
//!
//! Synthesizes Paramount+ and Pluto TV virtual linear FAST (Free Ad-supported Streaming TV) architecture:
//! - Ordered 24/7 program grid scheduled along wall-clock epoch time.
//! - Virtual live playout converting on-demand VOD assets into continuous linear broadcasts without GPU/CPU re-encoding.
//! - Dynamic sliding-window HLS live manifest generation (`#EXT-X-MEDIA-SEQUENCE`) calculating active segments from epoch time.
//! - Interstitial ad break and inter-program splice markers (`#EXT-X-DISCONTINUITY`).
//! - Electronic Program Guide (EPG) generation conforming to standard OTT TV grid schedules.
//! - Real-time zero-allocation telemetry export into `erp_cms_core::FastChannelStatusPacket` (`"FAST"` wire packets).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Typed errors for FAST linear playout engine.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FastChannelError {
    #[error("Channel '{0}' not found in registry")]
    ChannelNotFound(String),

    #[error("Channel schedule contains no programs")]
    EmptySchedule,

    #[error("Epoch timestamp precedes schedule start epoch")]
    EpochBeforeStart,

    #[error("Title or string exceeds buffer capacity: {0}")]
    StringCapacityExceeded(String),
}

/// Individual video asset scheduled within a linear FAST channel grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearProgram {
    pub program_id: String,
    pub media_id: String,
    pub title: String,
    pub synopsis: String,
    pub duration_seconds: f64,
    pub segment_duration_seconds: f64,
    pub thumbnail_url: Option<String>,
    pub rating: Option<String>,
    pub has_ad_break: bool,
    pub ad_break_offset_sec: Option<f64>,
    pub ad_break_duration_sec: Option<f64>,
}

impl LinearProgram {
    /// Constructs a standard linear program with uniform segment duration.
    #[must_use]
    pub fn new(
        program_id: impl Into<String>,
        media_id: impl Into<String>,
        title: impl Into<String>,
        duration_seconds: f64,
        segment_duration_seconds: f64,
    ) -> Self {
        Self {
            program_id: program_id.into(),
            media_id: media_id.into(),
            title: title.into(),
            synopsis: String::new(),
            duration_seconds,
            segment_duration_seconds,
            thumbnail_url: None,
            rating: Some("TV-14".into()),
            has_ad_break: false,
            ad_break_offset_sec: None,
            ad_break_duration_sec: None,
        }
    }

    /// Attaches an interstitial mid-roll commercial break.
    #[must_use]
    pub fn with_ad_break(mut self, offset_seconds: f64, duration_seconds: f64) -> Self {
        self.has_ad_break = true;
        self.ad_break_offset_sec = Some(offset_seconds);
        self.ad_break_duration_sec = Some(duration_seconds);
        self
    }

    /// Total number of discrete media segments in this program.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        (self.duration_seconds / self.segment_duration_seconds).ceil() as usize
    }

    /// Resolves the URI for a given segment index and quality variant.
    #[must_use]
    pub fn segment_uri(&self, variant: &str, segment_index: usize) -> String {
        format!("{}/{}/seg_{}.ts", self.media_id, variant, segment_index)
    }
}

/// Point-in-time playout context resolved against continuous epoch time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveProgramContext {
    pub program_index: usize,
    pub program: LinearProgram,
    pub program_elapsed_sec: f64,
    pub program_remaining_sec: f64,
    pub active_segment_index: usize,
    pub is_ad_break_active: bool,
    pub global_media_sequence: u64,
}

/// Electronic Program Guide (EPG) entry for a single scheduled broadcast.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpgScheduleEntry {
    pub program_id: String,
    pub title: String,
    pub synopsis: String,
    pub start_time_iso: String,
    pub end_time_iso: String,
    pub duration_seconds: f64,
    pub is_live_now: bool,
}

/// Full Electronic Program Guide response for a linear channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpgChannelGuide {
    pub channel_id: String,
    pub channel_name: String,
    pub category: String,
    pub current_program: Option<EpgScheduleEntry>,
    pub upcoming_programs: Vec<EpgScheduleEntry>,
}

/// Ordered 24/7 program schedule grid for a linear FAST channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FastChannelSchedule {
    pub channel_id: String,
    pub channel_name: String,
    pub category: String,
    pub programs: Vec<LinearProgram>,
    pub start_epoch_seconds: u64,
    pub total_schedule_duration_seconds: f64,
}

impl FastChannelSchedule {
    /// Creates a new empty linear schedule anchored at a specific epoch timestamp.
    #[must_use]
    pub fn new(
        channel_id: impl Into<String>,
        channel_name: impl Into<String>,
        category: impl Into<String>,
        start_epoch_seconds: u64,
    ) -> Self {
        Self {
            channel_id: channel_id.into(),
            channel_name: channel_name.into(),
            category: category.into(),
            programs: Vec::new(),
            start_epoch_seconds,
            total_schedule_duration_seconds: 0.0,
        }
    }

    /// Appends a program to the linear schedule and recalculates the continuous cycle duration.
    pub fn add_program(&mut self, program: LinearProgram) {
        self.total_schedule_duration_seconds += program.duration_seconds;
        self.programs.push(program);
    }

    /// Resolves the active linear playout state for any given epoch timestamp.
    pub fn resolve_at_epoch(
        &self,
        current_epoch_sec: u64,
    ) -> Result<ActiveProgramContext, FastChannelError> {
        if self.programs.is_empty() {
            return Err(FastChannelError::EmptySchedule);
        }

        let elapsed_since_anchor = current_epoch_sec.saturating_sub(self.start_epoch_seconds);
        let total_dur = self.total_schedule_duration_seconds;
        if total_dur <= 0.0 {
            return Err(FastChannelError::EmptySchedule);
        }

        // Cycle position within 24/7 repeating loop
        let cycle_offset_sec = (elapsed_since_anchor as f64) % total_dur;

        let mut accumulated_sec = 0.0;
        let mut chosen_idx = 0;
        let mut program_offset = 0.0;

        for (idx, prog) in self.programs.iter().enumerate() {
            if cycle_offset_sec >= accumulated_sec
                && cycle_offset_sec < accumulated_sec + prog.duration_seconds
            {
                chosen_idx = idx;
                program_offset = cycle_offset_sec - accumulated_sec;
                break;
            }
            accumulated_sec += prog.duration_seconds;
        }

        let program = self.programs[chosen_idx].clone();
        let remaining_sec = (program.duration_seconds - program_offset).max(0.0);
        let active_segment_idx =
            (program_offset / program.segment_duration_seconds).floor() as usize;

        // Check if an ad break is active
        let mut is_ad_break = false;
        if program.has_ad_break
            && let (Some(ad_start), Some(ad_dur)) =
                (program.ad_break_offset_sec, program.ad_break_duration_sec)
            && program_offset >= ad_start
            && program_offset < ad_start + ad_dur
        {
            is_ad_break = true;
        }

        // Global monotonically increasing sequence based on anchor and segment duration
        let global_media_seq =
            (elapsed_since_anchor as f64 / program.segment_duration_seconds).floor() as u64;

        Ok(ActiveProgramContext {
            program_index: chosen_idx,
            program,
            program_elapsed_sec: program_offset,
            program_remaining_sec: remaining_sec,
            active_segment_index: active_segment_idx,
            is_ad_break_active: is_ad_break,
            global_media_sequence: global_media_seq,
        })
    }

    /// Generates Electronic Program Guide (EPG) metadata around the active time window.
    #[must_use]
    pub fn generate_epg(&self, current_epoch_sec: u64) -> EpgChannelGuide {
        if self.programs.is_empty() || self.total_schedule_duration_seconds <= 0.0 {
            return EpgChannelGuide {
                channel_id: self.channel_id.clone(),
                channel_name: self.channel_name.clone(),
                category: self.category.clone(),
                current_program: None,
                upcoming_programs: Vec::new(),
            };
        }

        let context = match self.resolve_at_epoch(current_epoch_sec) {
            Ok(c) => c,
            Err(_) => {
                return EpgChannelGuide {
                    channel_id: self.channel_id.clone(),
                    channel_name: self.channel_name.clone(),
                    category: self.category.clone(),
                    current_program: None,
                    upcoming_programs: Vec::new(),
                };
            }
        };

        let current_prog = &context.program;
        let current_start_epoch = (current_epoch_sec as f64 - context.program_elapsed_sec) as i64;
        let current_end_epoch = current_start_epoch + (current_prog.duration_seconds as i64);

        let start_dt = DateTime::<Utc>::from_timestamp(current_start_epoch, 0)
            .unwrap_or_default()
            .to_rfc3339();
        let end_dt = DateTime::<Utc>::from_timestamp(current_end_epoch, 0)
            .unwrap_or_default()
            .to_rfc3339();

        let current_entry = EpgScheduleEntry {
            program_id: current_prog.program_id.clone(),
            title: current_prog.title.clone(),
            synopsis: current_prog.synopsis.clone(),
            start_time_iso: start_dt,
            end_time_iso: end_dt,
            duration_seconds: current_prog.duration_seconds,
            is_live_now: true,
        };

        // Project upcoming programs in continuous timeline
        let mut upcoming = Vec::new();
        let mut next_start = current_end_epoch;

        for offset in 1..=4 {
            let next_idx = (context.program_index + offset) % self.programs.len();
            let p = &self.programs[next_idx];
            let next_end = next_start + (p.duration_seconds as i64);

            upcoming.push(EpgScheduleEntry {
                program_id: p.program_id.clone(),
                title: p.title.clone(),
                synopsis: p.synopsis.clone(),
                start_time_iso: DateTime::<Utc>::from_timestamp(next_start, 0)
                    .unwrap_or_default()
                    .to_rfc3339(),
                end_time_iso: DateTime::<Utc>::from_timestamp(next_end, 0)
                    .unwrap_or_default()
                    .to_rfc3339(),
                duration_seconds: p.duration_seconds,
                is_live_now: false,
            });

            next_start = next_end;
        }

        EpgChannelGuide {
            channel_id: self.channel_id.clone(),
            channel_name: self.channel_name.clone(),
            category: self.category.clone(),
            current_program: Some(current_entry),
            upcoming_programs: upcoming,
        }
    }
}

/// Playout Manifest Generator for Virtual Linear Channels.
pub struct LinearManifestEngine;

impl LinearManifestEngine {
    /// Generates a live sliding-window HLS manifest (`#EXT-X-MEDIA-SEQUENCE`) from epoch time.
    pub fn generate_live_manifest(
        schedule: &FastChannelSchedule,
        current_epoch_sec: u64,
        window_size: usize,
        variant: &str,
    ) -> Result<String, FastChannelError> {
        let ctx = schedule.resolve_at_epoch(current_epoch_sec)?;
        let target_duration = ctx.program.segment_duration_seconds.ceil() as u32;

        let mut out = String::with_capacity(1024);
        out.push_str("#EXTM3U\n");
        out.push_str("#EXT-X-VERSION:4\n");
        out.push_str(&format!("#EXT-X-TARGETDURATION:{target_duration}\n"));
        out.push_str(&format!(
            "#EXT-X-MEDIA-SEQUENCE:{}\n",
            ctx.global_media_sequence
        ));
        out.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");

        let seg_dur = ctx.program.segment_duration_seconds;
        let active_seg = ctx.active_segment_index;
        let total_segs = ctx.program.segment_count();

        // Calculate window start segment index
        let start_seg = active_seg.saturating_sub(window_size.saturating_sub(1));

        for seg_idx in start_seg..=active_seg.min(total_segs.saturating_sub(1)) {
            // If segment falls right after ad break start, signal discontinuity
            if ctx.program.has_ad_break
                && let Some(ad_offset) = ctx.program.ad_break_offset_sec
            {
                let ad_seg_idx = (ad_offset / seg_dur).floor() as usize;
                if seg_idx == ad_seg_idx {
                    out.push_str("#EXT-X-DISCONTINUITY\n");
                }
            }

            let uri = ctx.program.segment_uri(variant, seg_idx);
            out.push_str(&format!("#EXTINF:{seg_dur:.3},\n{uri}\n"));
        }

        Ok(out)
    }

    /// Converts active channel state into a zero-allocation 95-byte `FastChannelStatusPacket` (including CRC16-CCITT).
    pub fn to_status_packet(
        schedule: &FastChannelSchedule,
        current_epoch_sec: u64,
        viewer_count: u32,
    ) -> Result<erp_cms_core::FastChannelStatusPacket, FastChannelError> {
        let ctx = schedule.resolve_at_epoch(current_epoch_sec)?;

        let mut c_id = erp_cms_core::heapless::String::<32>::new();
        c_id.push_str(&schedule.channel_id)
            .map_err(|_| FastChannelError::StringCapacityExceeded(schedule.channel_id.clone()))?;

        let mut p_title = erp_cms_core::heapless::String::<48>::new();
        let title_slice = if ctx.program.title.len() > 48 {
            &ctx.program.title[..48]
        } else {
            &ctx.program.title
        };
        p_title
            .push_str(title_slice)
            .map_err(|_| FastChannelError::StringCapacityExceeded(ctx.program.title.clone()))?;

        Ok(erp_cms_core::FastChannelStatusPacket {
            channel_id: c_id,
            program_title: p_title,
            remaining_sec: ctx.program_remaining_sec.round() as u32,
            viewer_count,
            is_ad_break: ctx.is_ad_break_active,
        })
    }
}

/// In-memory thread-safe registry of 24/7 FAST linear channels.
#[derive(Clone)]
pub struct FastChannelRegistry {
    channels: Arc<RwLock<HashMap<String, FastChannelSchedule>>>,
    viewer_counts: Arc<RwLock<HashMap<String, u32>>>,
}

impl Default for FastChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FastChannelRegistry {
    /// Creates an empty FAST channel registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            viewer_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initializes a registry pre-loaded with standard 24/7 virtual linear channels (Paramount+/Pluto TV style).
    #[must_use]
    pub fn with_demo_channels() -> Self {
        let registry = Self::new();

        // 1. Cyberpunk Sci-Fi 24/7 Channel
        let anchor = 1_700_000_000; // Reference epoch anchor
        let mut sci_fi =
            FastChannelSchedule::new("fast_scifi", "Sci-Fi 24/7", "Sci-Fi & Cyberpunk", anchor);

        let mut ep1 = LinearProgram::new(
            "prog_cp_1",
            "cyberpunk_s01e01",
            "Neon Horizons: Pilot",
            1800.0,
            6.0,
        )
        .with_ad_break(600.0, 60.0);
        ep1.synopsis = "A rogue cyber-runner infiltrates an orbital server array.".into();
        sci_fi.add_program(ep1);

        let mut ep2 = LinearProgram::new(
            "prog_cp_2",
            "cyberpunk_s01e02",
            "Ghost in the Wire",
            1800.0,
            6.0,
        )
        .with_ad_break(900.0, 60.0);
        ep2.synopsis = "Neural malware begins corrupting high-orbit communications.".into();
        sci_fi.add_program(ep2);

        let mut ep3 =
            LinearProgram::new("prog_cp_3", "cyberpunk_s01e03", "Silicon Dawn", 1800.0, 6.0);
        ep3.synopsis = "The final battle in the sub-orbital databanks.".into();
        sci_fi.add_program(ep3);

        registry.register_channel(sci_fi);

        // 2. Tech News Live 24/7 Channel
        let mut tech_news = FastChannelSchedule::new(
            "fast_technews",
            "Tech Pulse Today",
            "Technology & Systems",
            anchor,
        );
        let mut seg1 = LinearProgram::new(
            "prog_tn_1",
            "technews_hourly",
            "Morning Systems Brief",
            3600.0,
            4.0,
        );
        seg1.synopsis =
            "Hourly breakdown of cloud architecture and semiconductor developments.".into();
        tech_news.add_program(seg1);

        registry.register_channel(tech_news);

        registry
    }

    /// Registers or updates a linear channel schedule.
    pub fn register_channel(&self, schedule: FastChannelSchedule) {
        if let Ok(mut lock) = self.channels.write() {
            lock.insert(schedule.channel_id.clone(), schedule);
        }
    }

    /// Retrieves the sliding-window live HLS manifest for a given channel.
    pub fn get_live_manifest(
        &self,
        channel_id: &str,
        current_epoch_sec: u64,
        variant: &str,
    ) -> Result<String, FastChannelError> {
        let lock = self
            .channels
            .read()
            .map_err(|_| FastChannelError::ChannelNotFound(channel_id.into()))?;
        let schedule = lock
            .get(channel_id)
            .ok_or_else(|| FastChannelError::ChannelNotFound(channel_id.into()))?;
        LinearManifestEngine::generate_live_manifest(schedule, current_epoch_sec, 5, variant)
    }

    /// Retrieves Electronic Program Guide (EPG) metadata for a channel.
    pub fn get_epg(
        &self,
        channel_id: &str,
        current_epoch_sec: u64,
    ) -> Result<EpgChannelGuide, FastChannelError> {
        let lock = self
            .channels
            .read()
            .map_err(|_| FastChannelError::ChannelNotFound(channel_id.into()))?;
        let schedule = lock
            .get(channel_id)
            .ok_or_else(|| FastChannelError::ChannelNotFound(channel_id.into()))?;
        Ok(schedule.generate_epg(current_epoch_sec))
    }

    /// Returns a zero-allocation status wire packet for edge telemetry displays.
    pub fn get_status_packet(
        &self,
        channel_id: &str,
        current_epoch_sec: u64,
    ) -> Result<erp_cms_core::FastChannelStatusPacket, FastChannelError> {
        let lock = self
            .channels
            .read()
            .map_err(|_| FastChannelError::ChannelNotFound(channel_id.into()))?;
        let schedule = lock
            .get(channel_id)
            .ok_or_else(|| FastChannelError::ChannelNotFound(channel_id.into()))?;
        let viewers = self.get_viewer_count(channel_id);
        LinearManifestEngine::to_status_packet(schedule, current_epoch_sec, viewers)
    }

    /// Increments active viewer count.
    pub fn increment_viewers(&self, channel_id: &str) {
        if let Ok(mut lock) = self.viewer_counts.write() {
            let entry = lock.entry(channel_id.to_string()).or_insert(0);
            *entry = entry.saturating_add(1);
        }
    }

    /// Decrements active viewer count.
    pub fn decrement_viewers(&self, channel_id: &str) {
        if let Ok(mut lock) = self.viewer_counts.write()
            && let Some(entry) = lock.get_mut(channel_id)
        {
            *entry = entry.saturating_sub(1);
        }
    }

    /// Returns active viewer count for a channel.
    #[must_use]
    pub fn get_viewer_count(&self, channel_id: &str) -> u32 {
        self.viewer_counts
            .read()
            .ok()
            .and_then(|lock| lock.get(channel_id).copied())
            .unwrap_or(0)
    }
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_epoch_resolution_and_cycle() {
        let anchor = 1_000_000;
        let mut schedule = FastChannelSchedule::new("ch_test", "Test Channel", "General", anchor);

        // Prog 1: 1000s, Prog 2: 2000s -> Total 3000s cycle
        schedule.add_program(
            LinearProgram::new("p1", "media_p1", "Program 1", 1000.0, 5.0)
                .with_ad_break(300.0, 30.0),
        );
        schedule.add_program(LinearProgram::new(
            "p2",
            "media_p2",
            "Program 2",
            2000.0,
            5.0,
        ));

        assert_eq!(schedule.total_schedule_duration_seconds, 3000.0);

        // At epoch 1_000_500: In Prog 1, 500s elapsed, 500s remaining, segment 100
        let ctx = schedule
            .resolve_at_epoch(1_000_500)
            .expect("resolve failed");
        assert_eq!(ctx.program_index, 0);
        assert_eq!(ctx.program.title, "Program 1");
        assert_eq!(ctx.program_elapsed_sec, 500.0);
        assert_eq!(ctx.program_remaining_sec, 500.0);
        assert_eq!(ctx.active_segment_index, 100);
        assert!(!ctx.is_ad_break_active);

        // At epoch 1_000_310: In Prog 1 ad break (300s - 330s)
        let ctx_ad = schedule
            .resolve_at_epoch(1_000_310)
            .expect("resolve failed");
        assert!(ctx_ad.is_ad_break_active);

        // At epoch 1_001_500: In Prog 2 (1000s - 3000s), 500s elapsed in Prog 2
        let ctx2 = schedule
            .resolve_at_epoch(1_001_500)
            .expect("resolve failed");
        assert_eq!(ctx2.program_index, 1);
        assert_eq!(ctx2.program.title, "Program 2");
        assert_eq!(ctx2.program_elapsed_sec, 500.0);
        assert_eq!(ctx2.program_remaining_sec, 1500.0);

        // At epoch 1_003_500: Cycled over 3000s -> In Prog 1, 500s elapsed
        let ctx_cycle = schedule
            .resolve_at_epoch(1_003_500)
            .expect("resolve failed");
        assert_eq!(ctx_cycle.program_index, 0);
        assert_eq!(ctx_cycle.program_elapsed_sec, 500.0);
    }

    #[test]
    fn test_live_manifest_sliding_window_generation() {
        let anchor = 1_000_000;
        let mut schedule = FastChannelSchedule::new("ch_live", "Live News", "News", anchor);
        schedule.add_program(LinearProgram::new(
            "prog_news",
            "news_vod_1",
            "Evening News",
            600.0,
            6.0,
        ));

        // At epoch 1_000_060: elapsed 60s, active segment = 10
        let manifest =
            LinearManifestEngine::generate_live_manifest(&schedule, 1_000_060, 4, "720p")
                .expect("manifest generation failed");

        assert!(manifest.contains("#EXTM3U"));
        assert!(manifest.contains("#EXT-X-VERSION:4"));
        assert!(manifest.contains("#EXT-X-TARGETDURATION:6"));
        assert!(manifest.contains("#EXT-X-MEDIA-SEQUENCE:10"));
        assert!(!manifest.contains("#EXT-X-ENDLIST")); // Live stream invariant

        // Window size 4 includes segments 7, 8, 9, 10
        assert!(manifest.contains("news_vod_1/720p/seg_7.ts"));
        assert!(manifest.contains("news_vod_1/720p/seg_8.ts"));
        assert!(manifest.contains("news_vod_1/720p/seg_9.ts"));
        assert!(manifest.contains("news_vod_1/720p/seg_10.ts"));
    }

    #[test]
    fn test_epg_generation() {
        let anchor = 1_700_000_000;
        let mut schedule = FastChannelSchedule::new("fast_pluto", "Pluto Retro", "Classic", anchor);
        schedule.add_program(LinearProgram::new(
            "retro_1",
            "m_r1",
            "Retro Show 1",
            1800.0,
            6.0,
        ));
        schedule.add_program(LinearProgram::new(
            "retro_2",
            "m_r2",
            "Retro Show 2",
            1800.0,
            6.0,
        ));

        let epg = schedule.generate_epg(1_700_000_900); // 900s in -> inside Retro Show 1
        assert_eq!(epg.channel_id, "fast_pluto");
        let cur = epg.current_program.expect("Expected current program");
        assert_eq!(cur.title, "Retro Show 1");
        assert!(cur.is_live_now);
        assert!(!epg.upcoming_programs.is_empty());
        assert_eq!(epg.upcoming_programs[0].title, "Retro Show 2");
    }

    #[test]
    fn test_status_packet_wire_export() {
        let anchor = 1_700_000_000;
        let mut schedule = FastChannelSchedule::new("ch_fast_wire", "Wire Channel", "Tech", anchor);
        schedule.add_program(LinearProgram::new(
            "p_w1",
            "m_w1",
            "High Concurrency Rust",
            1200.0,
            4.0,
        ));

        let packet = LinearManifestEngine::to_status_packet(&schedule, 1_700_000_200, 1542)
            .expect("failed to build status packet");

        assert_eq!(packet.channel_id.as_str(), "ch_fast_wire");
        assert_eq!(packet.program_title.as_str(), "High Concurrency Rust");
        assert_eq!(packet.remaining_sec, 1000);
        assert_eq!(packet.viewer_count, 1542);
        assert!(!packet.is_ad_break);

        // Serialize into 95-byte wire buffer (including CRC16-CCITT)
        let mut buf = [0u8; 95];
        let serialized = packet
            .serialize(&mut buf)
            .expect("serialize wire packet failed");
        assert_eq!(serialized.len(), 95);
        assert_eq!(&serialized[0..4], b"FAST");

        // Decode roundtrip
        let decoded = erp_cms_core::FastChannelStatusPacket::decode(serialized)
            .expect("decode wire packet failed");
        assert_eq!(decoded.channel_id.as_str(), "ch_fast_wire");
        assert_eq!(decoded.viewer_count, 1542);
    }
}
