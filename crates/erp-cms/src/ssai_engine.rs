//! Server-Side Ad Insertion (SSAI) & SCTE-35 Manifest Manipulation Engine (`erp-cms::ssai_engine`).
//!
//! Implements Hulu and Netflix dynamic ad insertion architecture:
//! - ANSI/SCTE 35 2020 splice cue parsing (`splice_insert`, `time_signal`, `duration_ticks`, `pts_time`).
//! - Standard MPEG-2 CRC-32 checksum calculation for SCTE-35 sections.
//! - RFC 4648 Base64 & Hex wire encoding/decoding for `#EXT-SCTE35` and `#EXT-X-DATERANGE:SCTE35-OUT`.
//! - Targeted commercial `AdPod` and `AdCreative` representation with VAST/VMAP quartile tracking beacons.
//! - Dynamic HLS manifest manipulation inserting `#EXT-X-DISCONTINUITY` and `#EXT-X-DATERANGE` tags.
//! - Splicing precision at PTS boundaries for Pre-roll, Mid-roll, and Post-roll breaks.
//! - Elimination of client-side ad-blocker interception by streaming main content and ads from unified CDN origin.

use crate::hls_engine::{HlsMediaPlaylist, HlsMediaSegment};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// 90 kHz time base for MPEG/SCTE presentation timestamps.
pub const SCTE35_TIME_BASE_HZ: f64 = 90_000.0;

/// Standard table ID for SCTE-35 Splice Info Section.
pub const SCTE35_SPLICE_INFO_TABLE_ID: u8 = 0xFC;

/// Typed error conditions for SSAI and SCTE-35 operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SsaiError {
    #[error("Invalid SCTE-35 table ID: expected 0xFC, found 0x{0:02X}")]
    InvalidTableId(u8),

    #[error("Buffer underflow while reading SCTE-35 bitstream")]
    BufferUnderflow,

    #[error("Invalid section length: declared {declared}, buffer has {available}")]
    InvalidSectionLength { declared: usize, available: usize },

    #[error("CRC-32 checksum mismatch: expected 0x{expected:08X}, calculated 0x{calculated:08X}")]
    Crc32Mismatch { expected: u32, calculated: u32 },

    #[error("Unsupported SCTE-35 command type: 0x{0:02X}")]
    UnsupportedCommandType(u8),

    #[error("Hex decoding error: {0}")]
    HexDecodeError(String),

    #[error("Base64 decoding error: {0}")]
    Base64DecodeError(String),

    #[error("Ad pod contains no creatives or zero duration")]
    EmptyAdPod,

    #[error("Cue break duration missing or zero")]
    MissingBreakDuration,

    #[error("Stitching target time {target_time_sec}s exceeds playlist duration {playlist_duration_sec}s")]
    PlacementOutOfBounds {
        target_time_sec: u64,
        playlist_duration_sec: u64,
    },
}

/// SCTE-35 Splice Command Types according to ANSI/SCTE 35 Table 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scte35CommandType {
    SpliceNull = 0x00,
    SpliceSchedule = 0x04,
    SpliceInsert = 0x05,
    TimeSignal = 0x06,
    BandwidthReservation = 0x07,
    PrivateCommand = 0xFF,
}

impl Scte35CommandType {
    /// Resolves byte code to typed command.
    pub fn from_u8(val: u8) -> Result<Self, SsaiError> {
        match val {
            0x00 => Ok(Self::SpliceNull),
            0x04 => Ok(Self::SpliceSchedule),
            0x05 => Ok(Self::SpliceInsert),
            0x06 => Ok(Self::TimeSignal),
            0x07 => Ok(Self::BandwidthReservation),
            0xFF => Ok(Self::PrivateCommand),
            other => Err(SsaiError::UnsupportedCommandType(other)),
        }
    }
}

/// Structured payload of an ANSI/SCTE 35 `splice_insert` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scte35SpliceInsert {
    pub splice_event_id: u32,
    pub splice_event_cancel_indicator: bool,
    pub out_of_network_indicator: bool,
    pub program_splice_flag: bool,
    pub duration_flag: bool,
    pub splice_immediate_flag: bool,
    pub pts_time: Option<u64>,
    pub break_duration_ticks: Option<u64>,
    pub auto_return: bool,
    pub unique_program_id: u16,
    pub avail_num: u8,
    pub avails_expected: u8,
}

impl Scte35SpliceInsert {
    /// Break duration converted to seconds using 90 kHz time base.
    #[must_use]
    pub fn duration_seconds(&self) -> Option<f64> {
        self.break_duration_ticks
            .map(|ticks| ticks as f64 / SCTE35_TIME_BASE_HZ)
    }

    /// PTS time converted to seconds using 90 kHz time base.
    #[must_use]
    pub fn pts_seconds(&self) -> Option<f64> {
        self.pts_time
            .map(|ticks| ticks as f64 / SCTE35_TIME_BASE_HZ)
    }
}

/// Parsed ANSI/SCTE 35 Splice Cue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scte35Cue {
    pub protocol_version: u8,
    pub pts_adjustment: u64,
    pub cw_index: u8,
    pub tier: u16,
    pub command_type: Scte35CommandType,
    pub splice_insert: Option<Scte35SpliceInsert>,
    pub time_signal_pts: Option<u64>,
}

impl Scte35Cue {
    /// Creates a synthetic SCTE-35 cue for an ad break entry (Cue Out).
    #[must_use]
    pub fn new_splice_insert_cue_out(
        splice_event_id: u32,
        pts_time: u64,
        duration_seconds: f64,
    ) -> Self {
        let break_duration_ticks = (duration_seconds * SCTE35_TIME_BASE_HZ) as u64;
        Self {
            protocol_version: 0,
            pts_adjustment: 0,
            cw_index: 0,
            tier: 0xFFF,
            command_type: Scte35CommandType::SpliceInsert,
            splice_insert: Some(Scte35SpliceInsert {
                splice_event_id,
                splice_event_cancel_indicator: false,
                out_of_network_indicator: true,
                program_splice_flag: true,
                duration_flag: true,
                splice_immediate_flag: false,
                pts_time: Some(pts_time),
                break_duration_ticks: Some(break_duration_ticks),
                auto_return: true,
                unique_program_id: 1,
                avail_num: 1,
                avails_expected: 1,
            }),
            time_signal_pts: None,
        }
    }

    /// Creates a synthetic SCTE-35 cue for returning to program content (Cue In).
    #[must_use]
    pub fn new_splice_insert_cue_in(splice_event_id: u32, pts_time: u64) -> Self {
        Self {
            protocol_version: 0,
            pts_adjustment: 0,
            cw_index: 0,
            tier: 0xFFF,
            command_type: Scte35CommandType::SpliceInsert,
            splice_insert: Some(Scte35SpliceInsert {
                splice_event_id,
                splice_event_cancel_indicator: false,
                out_of_network_indicator: false,
                program_splice_flag: true,
                duration_flag: false,
                splice_immediate_flag: false,
                pts_time: Some(pts_time),
                break_duration_ticks: None,
                auto_return: false,
                unique_program_id: 1,
                avail_num: 0,
                avails_expected: 1,
            }),
            time_signal_pts: None,
        }
    }

    /// Converts seconds to 90 kHz PTS ticks.
    #[must_use]
    pub fn seconds_to_pts(seconds: f64) -> u64 {
        (seconds * SCTE35_TIME_BASE_HZ).round() as u64
    }

    /// Converts 90 kHz PTS ticks to seconds.
    #[must_use]
    pub fn pts_to_seconds(pts: u64) -> f64 {
        pts as f64 / SCTE35_TIME_BASE_HZ
    }

    /// Parses an ANSI/SCTE 35 binary section buffer.
    pub fn parse_bytes(buffer: &[u8]) -> Result<Self, SsaiError> {
        if buffer.len() < 14 {
            return Err(SsaiError::BufferUnderflow);
        }

        let mut r = BitReader::new(buffer);

        let table_id = r.read_u8()?;
        if table_id != SCTE35_SPLICE_INFO_TABLE_ID {
            return Err(SsaiError::InvalidTableId(table_id));
        }

        let _section_syntax_indicator = r.read_bit()?;
        let _private_indicator = r.read_bit()?;
        let _reserved_sap = r.read_bits(2)?;
        let section_length = r.read_bits(12)? as usize;

        if buffer.len() < section_length + 3 {
            return Err(SsaiError::InvalidSectionLength {
                declared: section_length + 3,
                available: buffer.len(),
            });
        }

        // Verify CRC-32 over bytes [0 .. 3 + section_length - 4]
        let total_section_len = 3 + section_length;
        let data_to_check = &buffer[..total_section_len - 4];
        let calculated_crc = mpeg_crc32(data_to_check);

        let expected_crc = u32::from_be_bytes([
            buffer[total_section_len - 4],
            buffer[total_section_len - 3],
            buffer[total_section_len - 2],
            buffer[total_section_len - 1],
        ]);

        if calculated_crc != expected_crc {
            return Err(SsaiError::Crc32Mismatch {
                expected: expected_crc,
                calculated: calculated_crc,
            });
        }

        let protocol_version = r.read_u8()?;
        let _encrypted_packet = r.read_bit()?;
        let _encryption_algorithm = r.read_bits(6)?;
        let pts_adjustment = r.read_bits(33)?;
        let cw_index = r.read_u8()?;
        let tier = r.read_bits(12)? as u16;
        let splice_command_length = r.read_bits(12)? as usize;
        let command_type_byte = r.read_u8()?;
        let command_type = Scte35CommandType::from_u8(command_type_byte)?;

        let mut splice_insert = None;
        let mut time_signal_pts = None;

        match command_type {
            Scte35CommandType::SpliceInsert => {
                let splice_event_id = r.read_u32()?;
                let splice_event_cancel_indicator = r.read_bit()?;
                let _reserved = r.read_bits(7)?;

                if !splice_event_cancel_indicator {
                    let out_of_network_indicator = r.read_bit()?;
                    let program_splice_flag = r.read_bit()?;
                    let duration_flag = r.read_bit()?;
                    let splice_immediate_flag = r.read_bit()?;
                    let _reserved2 = r.read_bits(4)?;

                    let mut pts_time = None;
                    if program_splice_flag && !splice_immediate_flag {
                        let time_specified_flag = r.read_bit()?;
                        if time_specified_flag {
                            let _reserved_time = r.read_bits(6)?;
                            pts_time = Some(r.read_bits(33)?);
                        } else {
                            let _reserved_notime = r.read_bits(7)?;
                        }
                    }

                    let mut break_duration_ticks = None;
                    let mut auto_return = false;
                    if duration_flag {
                        auto_return = r.read_bit()?;
                        let _reserved_dur = r.read_bits(6)?;
                        break_duration_ticks = Some(r.read_bits(33)?);
                    }

                    let unique_program_id = r.read_u16()?;
                    let avail_num = r.read_u8()?;
                    let avails_expected = r.read_u8()?;

                    splice_insert = Some(Scte35SpliceInsert {
                        splice_event_id,
                        splice_event_cancel_indicator: false,
                        out_of_network_indicator,
                        program_splice_flag,
                        duration_flag,
                        splice_immediate_flag,
                        pts_time,
                        break_duration_ticks,
                        auto_return,
                        unique_program_id,
                        avail_num,
                        avails_expected,
                    });
                } else {
                    splice_insert = Some(Scte35SpliceInsert {
                        splice_event_id,
                        splice_event_cancel_indicator: true,
                        out_of_network_indicator: false,
                        program_splice_flag: false,
                        duration_flag: false,
                        splice_immediate_flag: false,
                        pts_time: None,
                        break_duration_ticks: None,
                        auto_return: false,
                        unique_program_id: 0,
                        avail_num: 0,
                        avails_expected: 0,
                    });
                }
            }
            Scte35CommandType::TimeSignal => {
                let time_specified_flag = r.read_bit()?;
                if time_specified_flag {
                    let _reserved = r.read_bits(6)?;
                    time_signal_pts = Some(r.read_bits(33)?);
                }
            }
            _ => {
                // Skip command length bytes
                let _ = splice_command_length;
            }
        }

        Ok(Self {
            protocol_version,
            pts_adjustment,
            cw_index,
            tier,
            command_type,
            splice_insert,
            time_signal_pts,
        })
    }

    /// Serializes this SCTE-35 cue into a valid binary section with MPEG-2 CRC-32.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        // Build command payload first to determine splice_command_length
        let mut cmd_w = BitWriter::new();
        match self.command_type {
            Scte35CommandType::SpliceInsert => {
                if let Some(ref ins) = self.splice_insert {
                    cmd_w.write_u32(ins.splice_event_id);
                    cmd_w.write_bit(ins.splice_event_cancel_indicator);
                    cmd_w.write_bits(0x7F, 7); // reserved

                    if !ins.splice_event_cancel_indicator {
                        cmd_w.write_bit(ins.out_of_network_indicator);
                        cmd_w.write_bit(ins.program_splice_flag);
                        cmd_w.write_bit(ins.duration_flag);
                        cmd_w.write_bit(ins.splice_immediate_flag);
                        cmd_w.write_bits(0x0F, 4); // reserved

                        if ins.program_splice_flag && !ins.splice_immediate_flag {
                            if let Some(pts) = ins.pts_time {
                                cmd_w.write_bit(true); // time_specified_flag
                                cmd_w.write_bits(0x3F, 6); // reserved
                                cmd_w.write_bits(pts, 33);
                            } else {
                                cmd_w.write_bit(false);
                                cmd_w.write_bits(0x7F, 7);
                            }
                        }

                        if ins.duration_flag {
                            cmd_w.write_bit(ins.auto_return);
                            cmd_w.write_bits(0x3F, 6);
                            cmd_w.write_bits(ins.break_duration_ticks.unwrap_or(0), 33);
                        }

                        cmd_w.write_u16(ins.unique_program_id);
                        cmd_w.write_u8(ins.avail_num);
                        cmd_w.write_u8(ins.avails_expected);
                    }
                }
            }
            Scte35CommandType::TimeSignal => {
                if let Some(pts) = self.time_signal_pts {
                    cmd_w.write_bit(true);
                    cmd_w.write_bits(0x3F, 6);
                    cmd_w.write_bits(pts, 33);
                } else {
                    cmd_w.write_bit(false);
                    cmd_w.write_bits(0x7F, 7);
                }
            }
            _ => {}
        }

        let cmd_bytes = cmd_w.into_bytes();
        let splice_command_length = cmd_bytes.len();

        // Header + Fixed fields:
        // protocol_version (1B) + encrypted_packet/algo/pts_adj (5B) + cw_index (1B) + tier/cmd_len (3B) + cmd_type (1B) = 11 bytes
        // descriptor_loop_length (2B) = 2 bytes (0 descriptor)
        // CRC_32 (4B) = 4 bytes
        let section_length = 11 + splice_command_length + 2 + 4;

        let mut w = BitWriter::new();
        w.write_u8(SCTE35_SPLICE_INFO_TABLE_ID);
        w.write_bit(false); // section_syntax_indicator
        w.write_bit(false); // private_indicator
        w.write_bits(0x03, 2); // reserved
        w.write_bits(section_length as u64, 12);

        w.write_u8(self.protocol_version);
        w.write_bit(false); // encrypted_packet
        w.write_bits(0x00, 6); // encryption_algorithm
        w.write_bits(self.pts_adjustment, 33);
        w.write_u8(self.cw_index);
        w.write_bits(self.tier as u64, 12);
        w.write_bits(splice_command_length as u64, 12);
        w.write_u8(self.command_type as u8);

        let mut out = w.into_bytes();
        out.extend_from_slice(&cmd_bytes);
        // descriptor loop length = 0
        out.extend_from_slice(&[0x00, 0x00]);

        // Calculate and append CRC-32
        let crc = mpeg_crc32(&out);
        out.extend_from_slice(&crc.to_be_bytes());

        out
    }

    /// Formats this cue as a hexadecimal string prefixed with `0x`.
    #[must_use]
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.to_bytes()))
    }

    /// Parses a cue from a hexadecimal string (`0x...` or plain hex).
    pub fn from_hex(hex_str: &str) -> Result<Self, SsaiError> {
        let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(clean).map_err(|e| SsaiError::HexDecodeError(e.to_string()))?;
        Self::parse_bytes(&bytes)
    }

    /// Encodes this cue to an RFC 4648 Base64 string.
    #[must_use]
    pub fn to_base64(&self) -> String {
        base64_encode(&self.to_bytes())
    }

    /// Parses a cue from an RFC 4648 Base64 string.
    pub fn from_base64(b64_str: &str) -> Result<Self, SsaiError> {
        let bytes = base64_decode(b64_str)?;
        Self::parse_bytes(&bytes)
    }
}

/// Tracking beacon event URLs (VAST/VMAP telemetry callbacks).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdTrackingBeacons {
    pub impression_urls: Vec<String>,
    pub start_urls: Vec<String>,
    pub first_quartile_urls: Vec<String>,
    pub midpoint_urls: Vec<String>,
    pub third_quartile_urls: Vec<String>,
    pub complete_urls: Vec<String>,
}

/// Single targeted commercial asset within an ad pod.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdCreative {
    pub ad_id: String,
    pub title: String,
    pub advertiser: String,
    pub duration_seconds: f64,
    pub segments: Vec<HlsMediaSegment>,
    pub tracking: AdTrackingBeacons,
}

impl AdCreative {
    /// Constructs a standard synthetic 15-second ad creative.
    #[must_use]
    pub fn preset_15s(ad_id: impl Into<String>, advertiser: impl Into<String>) -> Self {
        let id = ad_id.into();
        let adv = advertiser.into();
        let segments = vec![
            HlsMediaSegment {
                sequence_number: 0,
                duration_seconds: 5.0,
                uri: format!("ads/{id}/seg_0.ts"),
                has_discontinuity: false,
            },
            HlsMediaSegment {
                sequence_number: 1,
                duration_seconds: 5.0,
                uri: format!("ads/{id}/seg_1.ts"),
                has_discontinuity: false,
            },
            HlsMediaSegment {
                sequence_number: 2,
                duration_seconds: 5.0,
                uri: format!("ads/{id}/seg_2.ts"),
                has_discontinuity: false,
            },
        ];

        Self {
            ad_id: id.clone(),
            title: format!("Commercial - {adv}"),
            advertiser: adv,
            duration_seconds: 15.0,
            segments,
            tracking: AdTrackingBeacons::default(),
        }
    }
}

/// Commercial ad break pod bundling one or more targeted ad creatives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdPod {
    pub pod_id: String,
    pub target_duration_seconds: f64,
    pub creatives: Vec<AdCreative>,
}

impl AdPod {
    /// Creates a new empty ad pod.
    #[must_use]
    pub fn new(pod_id: impl Into<String>, target_duration_seconds: f64) -> Self {
        Self {
            pod_id: pod_id.into(),
            target_duration_seconds,
            creatives: Vec::new(),
        }
    }

    /// Appends an ad creative to this pod.
    pub fn add_creative(&mut self, creative: AdCreative) {
        self.creatives.push(creative);
    }

    /// Total duration of all contained ad creatives.
    #[must_use]
    pub fn actual_duration_seconds(&self) -> f64 {
        self.creatives.iter().map(|c| c.duration_seconds).sum()
    }

    /// Flattens all media segments across all creatives into a continuous sequence.
    #[must_use]
    pub fn all_segments(&self) -> Vec<HlsMediaSegment> {
        let mut out = Vec::new();
        for creative in &self.creatives {
            out.extend(creative.segments.clone());
        }
        out
    }
}

/// Represents an RFC 8216 `#EXT-X-DATERANGE` metadata entry for SSAI client tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HlsDateRange {
    pub id: String,
    pub class_name: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub duration_seconds: Option<f64>,
    pub planned_duration_seconds: Option<f64>,
    pub scte35_out_hex: Option<String>,
    pub scte35_in_hex: Option<String>,
    pub custom_attributes: HashMap<String, String>,
}

impl HlsDateRange {
    /// Compiles into an RFC 8216 compliant `#EXT-X-DATERANGE` tag string.
    #[must_use]
    pub fn format_tag(&self) -> String {
        let mut out = format!("#EXT-X-DATERANGE:ID=\"{}\",START-DATE=\"{}\"", self.id, self.start_date);

        if let Some(ref cls) = self.class_name {
            out.push_str(&format!(",CLASS=\"{cls}\""));
        }
        if let Some(ref end) = self.end_date {
            out.push_str(&format!(",END-DATE=\"{end}\""));
        }
        if let Some(dur) = self.duration_seconds {
            out.push_str(&format!(",DURATION={dur:.3}"));
        }
        if let Some(pdur) = self.planned_duration_seconds {
            out.push_str(&format!(",PLANNED-DURATION={pdur:.3}"));
        }
        if let Some(ref scte_out) = self.scte35_out_hex {
            out.push_str(&format!(",SCTE35-OUT={scte_out}"));
        }
        if let Some(ref scte_in) = self.scte35_in_hex {
            out.push_str(&format!(",SCTE35-IN={scte_in}"));
        }
        for (k, v) in &self.custom_attributes {
            out.push_str(&format!(",{k}=\"{v}\""));
        }

        out
    }
}

/// Result summary of an SSAI manifest stitching operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SsaiStitchResult {
    pub original_segment_count: usize,
    pub stitched_segment_count: usize,
    pub inserted_ad_segments_count: usize,
    pub inserted_ad_pods_count: usize,
    pub discontinuities_injected: usize,
    pub daterange_tags: Vec<HlsDateRange>,
}

/// Server-Side Ad Insertion (SSAI) Dynamic Manifest Stitcher.
pub struct SsaiManifestStitcher;

impl SsaiManifestStitcher {
    /// Injects an ad pod as a Pre-Roll break (at index 0 before main content starts).
    pub fn stitch_preroll(
        base_playlist: &HlsMediaPlaylist,
        ad_pod: &AdPod,
        scte35_cue: Option<&Scte35Cue>,
    ) -> Result<(HlsMediaPlaylist, SsaiStitchResult), SsaiError> {
        let ad_segs = ad_pod.all_segments();
        if ad_segs.is_empty() {
            return Err(SsaiError::EmptyAdPod);
        }

        let mut stitched = base_playlist.clone();
        stitched.segments.clear();

        // 1. First ad segment has discontinuity
        for (idx, mut seg) in ad_segs.into_iter().enumerate() {
            if idx == 0 {
                seg.has_discontinuity = true;
            }
            stitched.segments.push(seg);
        }

        // 2. Main content starts with discontinuity to reset PTS
        let mut original_segs = base_playlist.segments.clone();
        if let Some(first_main) = original_segs.first_mut() {
            first_main.has_discontinuity = true;
        }
        stitched.segments.extend(original_segs);

        // DateRange tag for client quartile beacon tracking
        let daterange = HlsDateRange {
            id: format!("ad-pod-preroll-{}", ad_pod.pod_id),
            class_name: Some("com.apple.hls.interstitial".into()),
            start_date: chrono::Utc::now().to_rfc3339(),
            end_date: None,
            duration_seconds: Some(ad_pod.actual_duration_seconds()),
            planned_duration_seconds: Some(ad_pod.target_duration_seconds),
            scte35_out_hex: scte35_cue.map(|c| c.to_hex()),
            scte35_in_hex: None,
            custom_attributes: HashMap::new(),
        };

        let result = SsaiStitchResult {
            original_segment_count: base_playlist.segments.len(),
            stitched_segment_count: stitched.segments.len(),
            inserted_ad_segments_count: ad_pod.all_segments().len(),
            inserted_ad_pods_count: 1,
            discontinuities_injected: 2,
            daterange_tags: vec![daterange],
        };

        Ok((stitched, result))
    }

    /// Injects an ad pod as a Mid-Roll break at a target playhead offset in seconds.
    pub fn stitch_midroll(
        base_playlist: &HlsMediaPlaylist,
        target_break_time_sec: f64,
        ad_pod: &AdPod,
        scte35_cue: Option<&Scte35Cue>,
    ) -> Result<(HlsMediaPlaylist, SsaiStitchResult), SsaiError> {
        let ad_segs = ad_pod.all_segments();
        if ad_segs.is_empty() {
            return Err(SsaiError::EmptyAdPod);
        }

        // Find boundary segment index matching break time
        let mut accumulated_time = 0.0;
        let mut splice_index = base_playlist.segments.len();

        for (idx, seg) in base_playlist.segments.iter().enumerate() {
            if accumulated_time >= target_break_time_sec {
                splice_index = idx;
                break;
            }
            accumulated_time += seg.duration_seconds;
        }

        if splice_index > base_playlist.segments.len() {
            return Err(SsaiError::PlacementOutOfBounds {
                target_time_sec: target_break_time_sec as u64,
                playlist_duration_sec: accumulated_time as u64,
            });
        }

        let mut stitched = base_playlist.clone();
        stitched.segments.clear();

        // 1. Content prior to ad break
        stitched
            .segments
            .extend(base_playlist.segments[..splice_index].iter().cloned());

        // 2. Ad pod with leading discontinuity
        for (idx, mut seg) in ad_segs.into_iter().enumerate() {
            if idx == 0 {
                seg.has_discontinuity = true;
            }
            stitched.segments.push(seg);
        }

        // 3. Resuming main content with discontinuity
        let mut remaining = base_playlist.segments[splice_index..].to_vec();
        if let Some(first_resume) = remaining.first_mut() {
            first_resume.has_discontinuity = true;
        }
        stitched.segments.extend(remaining);

        let daterange = HlsDateRange {
            id: format!("ad-pod-midroll-{}", ad_pod.pod_id),
            class_name: Some("com.apple.hls.interstitial".into()),
            start_date: chrono::Utc::now().to_rfc3339(),
            end_date: None,
            duration_seconds: Some(ad_pod.actual_duration_seconds()),
            planned_duration_seconds: Some(ad_pod.target_duration_seconds),
            scte35_out_hex: scte35_cue.map(|c| c.to_hex()),
            scte35_in_hex: None,
            custom_attributes: HashMap::new(),
        };

        let result = SsaiStitchResult {
            original_segment_count: base_playlist.segments.len(),
            stitched_segment_count: stitched.segments.len(),
            inserted_ad_segments_count: ad_pod.all_segments().len(),
            inserted_ad_pods_count: 1,
            discontinuities_injected: 2,
            daterange_tags: vec![daterange],
        };

        Ok((stitched, result))
    }

    /// Injects an ad pod as a Post-Roll break (at the very end of main content).
    pub fn stitch_postroll(
        base_playlist: &HlsMediaPlaylist,
        ad_pod: &AdPod,
        scte35_cue: Option<&Scte35Cue>,
    ) -> Result<(HlsMediaPlaylist, SsaiStitchResult), SsaiError> {
        let ad_segs = ad_pod.all_segments();
        if ad_segs.is_empty() {
            return Err(SsaiError::EmptyAdPod);
        }

        let mut stitched = base_playlist.clone();

        for (idx, mut seg) in ad_segs.into_iter().enumerate() {
            if idx == 0 {
                seg.has_discontinuity = true;
            }
            stitched.segments.push(seg);
        }

        let daterange = HlsDateRange {
            id: format!("ad-pod-postroll-{}", ad_pod.pod_id),
            class_name: Some("com.apple.hls.interstitial".into()),
            start_date: chrono::Utc::now().to_rfc3339(),
            end_date: None,
            duration_seconds: Some(ad_pod.actual_duration_seconds()),
            planned_duration_seconds: Some(ad_pod.target_duration_seconds),
            scte35_out_hex: scte35_cue.map(|c| c.to_hex()),
            scte35_in_hex: None,
            custom_attributes: HashMap::new(),
        };

        let result = SsaiStitchResult {
            original_segment_count: base_playlist.segments.len(),
            stitched_segment_count: stitched.segments.len(),
            inserted_ad_segments_count: ad_pod.all_segments().len(),
            inserted_ad_pods_count: 1,
            discontinuities_injected: 1,
            daterange_tags: vec![daterange],
        };

        Ok((stitched, result))
    }

    /// Compiles a fully stitched HLS Media Playlist with `#EXT-X-DATERANGE` tag injections.
    #[must_use]
    pub fn compile_manifest(
        playlist: &HlsMediaPlaylist,
        daterange_tags: &[HlsDateRange],
    ) -> String {
        let mut out = String::with_capacity(2048);
        out.push_str("#EXTM3U\n");
        out.push_str("#EXT-X-VERSION:4\n");
        out.push_str(&format!("#EXT-X-TARGETDURATION:{}\n", playlist.target_duration));
        out.push_str(&format!("#EXT-X-MEDIA-SEQUENCE:{}\n", playlist.media_sequence));

        if !playlist.is_live {
            out.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
        }

        // Instate DateRange tags at the playlist header
        for dr in daterange_tags {
            out.push_str(&dr.format_tag());
            out.push('\n');
        }

        if let Some(ref init) = playlist.init_segment_uri {
            out.push_str(&format!("#EXT-X-MAP:URI=\"{init}\"\n"));
        }

        for seg in &playlist.segments {
            if seg.has_discontinuity {
                out.push_str("#EXT-X-DISCONTINUITY\n");
            }
            out.push_str(&format!("#EXTINF:{:.3},\n{}\n", seg.duration_seconds, seg.uri));
        }

        if !playlist.is_live {
            out.push_str("#EXT-X-ENDLIST\n");
        }

        out
    }
}

// -----------------------------------------------------------------------------
// BitReader & BitWriter Helpers
// -----------------------------------------------------------------------------

struct BitReader<'a> {
    buffer: &'a [u8],
    bit_offset: usize,
}

impl<'a> BitReader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            bit_offset: 0,
        }
    }

    fn read_bit(&mut self) -> Result<bool, SsaiError> {
        let byte_idx = self.bit_offset / 8;
        let bit_in_byte = 7 - (self.bit_offset % 8);
        if byte_idx >= self.buffer.len() {
            return Err(SsaiError::BufferUnderflow);
        }
        let bit = (self.buffer[byte_idx] >> bit_in_byte) & 1 == 1;
        self.bit_offset += 1;
        Ok(bit)
    }

    fn read_bits(&mut self, count: usize) -> Result<u64, SsaiError> {
        let mut val = 0u64;
        for _ in 0..count {
            val = (val << 1) | (u64::from(self.read_bit()?));
        }
        Ok(val)
    }

    fn read_u8(&mut self) -> Result<u8, SsaiError> {
        self.read_bits(8).map(|v| v as u8)
    }

    fn read_u16(&mut self) -> Result<u16, SsaiError> {
        self.read_bits(16).map(|v| v as u16)
    }

    fn read_u32(&mut self) -> Result<u32, SsaiError> {
        self.read_bits(32).map(|v| v as u32)
    }
}

struct BitWriter {
    bytes: Vec<u8>,
    current_byte: u8,
    bits_in_byte: usize,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            bytes: Vec::with_capacity(64),
            current_byte: 0,
            bits_in_byte: 0,
        }
    }

    fn write_bit(&mut self, bit: bool) {
        if bit {
            self.current_byte |= 1 << (7 - self.bits_in_byte);
        }
        self.bits_in_byte += 1;
        if self.bits_in_byte == 8 {
            self.bytes.push(self.current_byte);
            self.current_byte = 0;
            self.bits_in_byte = 0;
        }
    }

    fn write_bits(&mut self, val: u64, count: usize) {
        for i in (0..count).rev() {
            let bit = ((val >> i) & 1) == 1;
            self.write_bit(bit);
        }
    }

    fn write_u8(&mut self, val: u8) {
        self.write_bits(val as u64, 8);
    }

    fn write_u16(&mut self, val: u16) {
        self.write_bits(val as u64, 16);
    }

    fn write_u32(&mut self, val: u32) {
        self.write_bits(val as u64, 32);
    }

    fn into_bytes(mut self) -> Vec<u8> {
        if self.bits_in_byte > 0 {
            self.bytes.push(self.current_byte);
        }
        self.bytes
    }
}

// -----------------------------------------------------------------------------
// MPEG-2 CRC-32 (ISO/IEC 13818-1)
// -----------------------------------------------------------------------------

/// Computes standard ISO/IEC 13818-1 / ANSI SCTE-35 CRC-32 (poly 0x04C11DB7).
#[must_use]
pub fn mpeg_crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        for bit in 0..8 {
            let bit_val = ((byte >> (7 - bit)) & 1) as u32;
            let c31 = (crc >> 31) & 1;
            crc <<= 1;
            if c31 ^ bit_val != 0 {
                crc ^= 0x04C1_1DB7;
            }
        }
    }
    crc
}

// -----------------------------------------------------------------------------
// RFC 4648 Base64 Safe Codec
// -----------------------------------------------------------------------------

const B64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i] as usize;
        let b1 = if i + 1 < input.len() { input[i + 1] as usize } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] as usize } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(B64_CHARS[(triple >> 18) & 0x3F] as char);
        out.push(B64_CHARS[(triple >> 12) & 0x3F] as char);

        if i + 1 < input.len() {
            out.push(B64_CHARS[(triple >> 6) & 0x3F] as char);
        } else {
            out.push('=');
        }

        if i + 2 < input.len() {
            out.push(B64_CHARS[triple & 0x3F] as char);
        } else {
            out.push('=');
        }

        i += 3;
    }
    out
}

fn base64_decode(input: &str) -> Result<Vec<u8>, SsaiError> {
    let clean = input.trim();
    let mut out = Vec::with_capacity((clean.len() * 3) / 4);

    let decode_char = |c: char| -> Result<u8, SsaiError> {
        match c {
            'A'..='Z' => Ok(c as u8 - b'A'),
            'a'..='z' => Ok(c as u8 - b'a' + 26),
            '0'..='9' => Ok(c as u8 - b'0' + 52),
            '+' => Ok(62),
            '/' => Ok(63),
            '=' => Ok(0),
            other => Err(SsaiError::Base64DecodeError(format!(
                "Invalid Base64 character: {other}"
            ))),
        }
    };

    let chars: Vec<char> = clean.chars().filter(|c| !c.is_whitespace()).collect();
    if !chars.len().is_multiple_of(4) {
        return Err(SsaiError::Base64DecodeError(
            "Base64 string length must be a multiple of 4".into(),
        ));
    }

    let mut i = 0;
    while i < chars.len() {
        let c0 = decode_char(chars[i])? as usize;
        let c1 = decode_char(chars[i + 1])? as usize;
        let c2 = decode_char(chars[i + 2])? as usize;
        let c3 = decode_char(chars[i + 3])? as usize;

        let triple = (c0 << 18) | (c1 << 12) | (c2 << 6) | c3;

        out.push(((triple >> 16) & 0xFF) as u8);
        if chars[i + 2] != '=' {
            out.push(((triple >> 8) & 0xFF) as u8);
        }
        if chars[i + 3] != '=' {
            out.push((triple & 0xFF) as u8);
        }

        i += 4;
    }

    Ok(out)
}

impl fmt::Display for Scte35Cue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SCTE-35 Cue (type={:?}, pts_adj={})",
            self.command_type, self.pts_adjustment
        )
    }
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scte35_cue_out_roundtrip_bytes_hex_base64() {
        let pts = Scte35Cue::seconds_to_pts(120.0);
        let cue = Scte35Cue::new_splice_insert_cue_out(1001, pts, 30.0);

        let bytes = cue.to_bytes();
        assert_eq!(bytes[0], SCTE35_SPLICE_INFO_TABLE_ID);

        let parsed = Scte35Cue::parse_bytes(&bytes).expect("Failed to parse SCTE-35 bytes");
        assert_eq!(parsed.command_type, Scte35CommandType::SpliceInsert);

        let insert = parsed.splice_insert.expect("Expected splice_insert payload");
        assert_eq!(insert.splice_event_id, 1001);
        assert!(insert.out_of_network_indicator);
        assert_eq!(insert.pts_time, Some(pts));
        assert_eq!(insert.duration_seconds(), Some(30.0));

        // Hex roundtrip
        let hex_str = cue.to_hex();
        assert!(hex_str.starts_with("0x"));
        let from_hex = Scte35Cue::from_hex(&hex_str).expect("Failed to parse hex cue");
        assert_eq!(from_hex.splice_insert.as_ref().unwrap().splice_event_id, 1001);

        // Base64 roundtrip
        let b64_str = cue.to_base64();
        let from_b64 = Scte35Cue::from_base64(&b64_str).expect("Failed to parse base64 cue");
        assert_eq!(from_b64.splice_insert.as_ref().unwrap().splice_event_id, 1001);
    }

    #[test]
    fn test_scte35_cue_in_roundtrip() {
        let pts = Scte35Cue::seconds_to_pts(150.0);
        let cue = Scte35Cue::new_splice_insert_cue_in(1001, pts);

        let bytes = cue.to_bytes();
        let parsed = Scte35Cue::parse_bytes(&bytes).expect("Failed to parse SCTE-35 cue in");
        let insert = parsed.splice_insert.expect("Expected splice insert");

        assert!(!insert.out_of_network_indicator);
        assert_eq!(insert.pts_time, Some(pts));
        assert_eq!(insert.break_duration_ticks, None);
    }

    #[test]
    fn test_ad_pod_stitching_preroll() {
        let mut playlist = HlsMediaPlaylist::new(6, false);
        for i in 0..5 {
            playlist.add_segment(HlsMediaSegment {
                sequence_number: i,
                duration_seconds: 6.0,
                uri: format!("content/seg_{i}.ts"),
                has_discontinuity: false,
            });
        }

        let mut ad_pod = AdPod::new("pod-pre-1", 15.0);
        ad_pod.add_creative(AdCreative::preset_15s("ad-nike-1", "Nike"));

        let cue = Scte35Cue::new_splice_insert_cue_out(2001, 0, 15.0);
        let (stitched, result) = SsaiManifestStitcher::stitch_preroll(&playlist, &ad_pod, Some(&cue))
            .expect("Failed to stitch preroll");

        assert_eq!(result.original_segment_count, 5);
        assert_eq!(result.inserted_ad_segments_count, 3);
        assert_eq!(result.stitched_segment_count, 8);
        assert_eq!(result.discontinuities_injected, 2);

        // Segment 0 (first ad) has discontinuity
        assert!(stitched.segments[0].has_discontinuity);
        assert_eq!(stitched.segments[0].uri, "ads/ad-nike-1/seg_0.ts");

        // Segment 3 (first content segment) has discontinuity
        assert!(stitched.segments[3].has_discontinuity);
        assert_eq!(stitched.segments[3].uri, "content/seg_0.ts");

        // Compile manifest check
        let m3u8 = SsaiManifestStitcher::compile_manifest(&stitched, &result.daterange_tags);
        assert!(m3u8.contains("#EXT-X-DATERANGE:ID=\"ad-pod-preroll-pod-pre-1\""));
        assert!(m3u8.contains("SCTE35-OUT=0x"));
        assert!(m3u8.contains("#EXT-X-DISCONTINUITY"));
        assert!(m3u8.contains("ads/ad-nike-1/seg_0.ts"));
    }

    #[test]
    fn test_ad_pod_stitching_midroll() {
        let mut playlist = HlsMediaPlaylist::new(6, false);
        for i in 0..10 {
            playlist.add_segment(HlsMediaSegment {
                sequence_number: i,
                duration_seconds: 6.0,
                uri: format!("content/seg_{i}.ts"),
                has_discontinuity: false,
            });
        }

        // Break at 24.0s (after segment 3, which ends at 24.0s)
        let mut ad_pod = AdPod::new("pod-mid-1", 15.0);
        ad_pod.add_creative(AdCreative::preset_15s("ad-coca-cola", "Coca-Cola"));

        let cue = Scte35Cue::new_splice_insert_cue_out(3001, Scte35Cue::seconds_to_pts(24.0), 15.0);
        let (stitched, result) = SsaiManifestStitcher::stitch_midroll(&playlist, 24.0, &ad_pod, Some(&cue))
            .expect("Failed to stitch midroll");

        assert_eq!(result.original_segment_count, 10);
        assert_eq!(result.inserted_ad_segments_count, 3);
        assert_eq!(result.stitched_segment_count, 13);
        assert_eq!(result.discontinuities_injected, 2);

        // Content seg_0, seg_1, seg_2, seg_3 come first (4 segments)
        assert_eq!(stitched.segments[3].uri, "content/seg_3.ts");
        assert!(!stitched.segments[3].has_discontinuity);

        // Next is first ad segment, which has discontinuity
        assert!(stitched.segments[4].has_discontinuity);
        assert_eq!(stitched.segments[4].uri, "ads/ad-coca-cola/seg_0.ts");

        // Next is seg 7 (first resume segment: content/seg_4.ts), which has discontinuity
        assert!(stitched.segments[7].has_discontinuity);
        assert_eq!(stitched.segments[7].uri, "content/seg_4.ts");

        let m3u8 = SsaiManifestStitcher::compile_manifest(&stitched, &result.daterange_tags);
        assert!(m3u8.contains("#EXT-X-DATERANGE:ID=\"ad-pod-midroll-pod-mid-1\""));
        assert!(m3u8.contains("DURATION=15.000"));
    }

    #[test]
    fn test_ad_pod_stitching_postroll() {
        let mut playlist = HlsMediaPlaylist::new(6, false);
        for i in 0..3 {
            playlist.add_segment(HlsMediaSegment {
                sequence_number: i,
                duration_seconds: 6.0,
                uri: format!("content/seg_{i}.ts"),
                has_discontinuity: false,
            });
        }

        let mut ad_pod = AdPod::new("pod-post-1", 15.0);
        ad_pod.add_creative(AdCreative::preset_15s("ad-apple", "Apple"));

        let (stitched, result) = SsaiManifestStitcher::stitch_postroll(&playlist, &ad_pod, None)
            .expect("Failed to stitch postroll");

        assert_eq!(result.stitched_segment_count, 6);
        assert_eq!(result.discontinuities_injected, 1);
        assert!(stitched.segments[3].has_discontinuity);
        assert_eq!(stitched.segments[3].uri, "ads/ad-apple/seg_0.ts");
    }
}
