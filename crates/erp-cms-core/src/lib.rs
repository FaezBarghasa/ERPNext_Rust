#![no_std]

//! Zero-Allocation `no_std` CMS / Visual Builder Core AST (`erp-cms-core`).
//!
//! Designed for constrained edge devices, industrial handhelds, warehouse kiosks,
//! digital signage displays, and microcontroller UART terminals.
//!
//! Features:
//! - Pure `#![no_std]` runtime with zero dynamic heap allocations.
//! - Bounded static buffers (`heapless::String` and `heapless::Vec`).
//! - Direct terminal/ANSI rendering for serial consoles and character LCDs.
//! - Compact binary wire format for low-bandwidth telemetry and radio streaming.

pub use heapless;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Strongly typed error enum for zero-allocation edge CMS operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeCmsError {
    CapacityExceeded,
    BufferTooSmall,
    InvalidMagic,
    InvalidUtf8,
    PayloadTruncated,
    ChecksumMismatch,
}

impl core::fmt::Display for EdgeCmsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CapacityExceeded => write!(f, "Block capacity exceeded in static buffer"),
            Self::BufferTooSmall => write!(f, "Provided serialization buffer is too small"),
            Self::InvalidMagic => write!(f, "Invalid magic packet header (expected ERPC)"),
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence in packet title"),
            Self::PayloadTruncated => write!(f, "Packet payload is truncated"),
            Self::ChecksumMismatch => write!(
                f,
                "Packet CRC16-CCITT checksum mismatch (corrupted wire frame)"
            ),
        }
    }
}

/// Computes CRC16-CCITT (poly 0x1021, init 0xFFFF) across byte slice with zero allocations.
#[inline]
#[must_use]
pub const fn crc16_ccitt(bytes: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    let mut i = 0;
    while i < bytes.len() {
        crc ^= (bytes[i] as u16) << 8;
        let mut bit = 0;
        while bit < 8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
            bit += 1;
        }
        i += 1;
    }
    crc
}

/// Supported responsive viewports for edge displays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EdgeBreakpoint {
    Desktop,
    Tablet,
    Mobile,
}

impl EdgeBreakpoint {
    #[must_use]
    pub const fn default_columns(&self) -> u16 {
        match self {
            Self::Desktop => 80,
            Self::Tablet => 40,
            Self::Mobile => 20,
        }
    }
}

/// Compact RGB color representation for edge displays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EdgeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl EdgeColor {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const ACCENT_GREEN: Self = Self::new(16, 185, 129);
    pub const CYAN: Self = Self::new(56, 189, 248);
}

/// Zero-allocation box model metrics (padding, margin, width, height).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EdgeBoxModel {
    pub padding_top: u8,
    pub padding_bottom: u8,
    pub margin_top: u8,
    pub margin_bottom: u8,
    pub border: bool,
}

/// Primitive visual and telemetry block types for edge screens.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EdgeBlockType {
    Heading {
        level: u8,
        text: heapless::String<128>,
    },
    Text {
        content: heapless::String<256>,
    },
    MetricCard {
        label: heapless::String<64>,
        value: heapless::String<32>,
    },
    Barcode {
        payload: heapless::String<128>,
        symbology: heapless::String<16>,
    },
    Callout {
        title: heapless::String<64>,
        text: heapless::String<128>,
    },
    Divider,
}

/// An individual positioned node within an edge layout tree.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EdgeBlockNode {
    pub id: heapless::String<32>,
    pub block: EdgeBlockType,
    pub box_model: EdgeBoxModel,
}

impl EdgeBlockNode {
    #[must_use]
    pub fn new(id: &str, block: EdgeBlockType) -> Self {
        let mut id_str = heapless::String::new();
        let _ = id_str.push_str(id);
        Self {
            id: id_str,
            block,
            box_model: EdgeBoxModel::default(),
        }
    }
}

/// Bounded edge canvas layout holding up to `CAP` blocks without dynamic heap allocations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EdgeCanvasLayout<const CAP: usize> {
    pub title: heapless::String<64>,
    pub blocks: heapless::Vec<EdgeBlockNode, CAP>,
    pub version: u32,
}

impl<const CAP: usize> EdgeCanvasLayout<CAP> {
    #[must_use]
    pub fn new(title: &str) -> Self {
        let mut title_str = heapless::String::new();
        let _ = title_str.push_str(title);
        Self {
            title: title_str,
            blocks: heapless::Vec::new(),
            version: 1,
        }
    }

    /// Appends a block node to the layout. Returns `Err(EdgeCmsError::CapacityExceeded)` if capacity exceeded.
    pub fn add_block(&mut self, node: EdgeBlockNode) -> Result<(), EdgeCmsError> {
        self.blocks
            .push(node)
            .map_err(|_| EdgeCmsError::CapacityExceeded)
    }

    /// Renders layout to standard plain text (e.g. for monochrome character LCD / UART).
    pub fn render_plain_text<W: core::fmt::Write>(&self, out: &mut W) -> core::fmt::Result {
        writeln!(out, "=== {} (v{}) ===", self.title, self.version)?;
        for node in &self.blocks {
            for _ in 0..node.box_model.margin_top {
                writeln!(out)?;
            }
            if node.box_model.border {
                writeln!(out, "+--------------------------------------+")?;
            }

            match &node.block {
                EdgeBlockType::Heading { level, text } => {
                    if *level <= 1 {
                        writeln!(out, ">>> {} <<<", text.as_str())?;
                    } else {
                        writeln!(out, "## {}", text.as_str())?;
                    }
                }
                EdgeBlockType::Text { content } => {
                    writeln!(out, "{}", content.as_str())?;
                }
                EdgeBlockType::MetricCard { label, value } => {
                    writeln!(out, "[{}: {}]", label.as_str(), value.as_str())?;
                }
                EdgeBlockType::Barcode { payload, symbology } => {
                    writeln!(
                        out,
                        "[*BARCODE: {} ({})*]",
                        payload.as_str(),
                        symbology.as_str()
                    )?;
                }
                EdgeBlockType::Callout { title, text } => {
                    writeln!(out, "! [{}] {}", title.as_str(), text.as_str())?;
                }
                EdgeBlockType::Divider => {
                    writeln!(out, "----------------------------------------")?;
                }
            }

            if node.box_model.border {
                writeln!(out, "+--------------------------------------+")?;
            }
            for _ in 0..node.box_model.padding_bottom {
                writeln!(out)?;
            }
        }
        Ok(())
    }

    /// Renders layout with ANSI escape sequences (colors, bold, inverse) for VT100 terminals.
    pub fn render_ansi<W: core::fmt::Write>(&self, out: &mut W) -> core::fmt::Result {
        // Clear screen & header
        writeln!(
            out,
            "\x1b[1;36m=== {} (v{}) ===\x1b[0m",
            self.title, self.version
        )?;
        for node in &self.blocks {
            match &node.block {
                EdgeBlockType::Heading { level, text } => {
                    if *level <= 1 {
                        writeln!(out, "\x1b[1;32m>>> {} <<<\x1b[0m", text.as_str())?;
                    } else {
                        writeln!(out, "\x1b[1;33m## {}\x1b[0m", text.as_str())?;
                    }
                }
                EdgeBlockType::Text { content } => {
                    writeln!(out, "{}", content.as_str())?;
                }
                EdgeBlockType::MetricCard { label, value } => {
                    writeln!(
                        out,
                        "\x1b[7m {} \x1b[0m \x1b[1;32m{}\x1b[0m",
                        label.as_str(),
                        value.as_str()
                    )?;
                }
                EdgeBlockType::Barcode { payload, symbology } => {
                    writeln!(
                        out,
                        "\x1b[35m|||| {} [{}] ||||\x1b[0m",
                        payload.as_str(),
                        symbology.as_str()
                    )?;
                }
                EdgeBlockType::Callout { title, text } => {
                    writeln!(
                        out,
                        "\x1b[1;31m[!] {}\x1b[0m: {}",
                        title.as_str(),
                        text.as_str()
                    )?;
                }
                EdgeBlockType::Divider => {
                    writeln!(
                        out,
                        "\x1b[90m----------------------------------------\x1b[0m"
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Serializes the canvas header into a fixed-length zero-allocation binary packet.
    ///
    /// Packet format:
    /// `[0..4]`   Magic: `0x45, 0x52, 0x50, 0x43` ("ERPC")
    /// `[4..6]`   Version: u16 LE
    /// `[6..8]`   Block count: u16 LE
    /// `[8..10]`  Title length: u16 LE
    /// `[10..74]` Title bytes
    pub fn serialize_packet_header<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], EdgeCmsError> {
        if buf.len() < 74 {
            return Err(EdgeCmsError::BufferTooSmall);
        }
        buf[0..4].copy_from_slice(b"ERPC");
        buf[4..6].copy_from_slice(&(self.version as u16).to_le_bytes());
        buf[6..8].copy_from_slice(&(self.blocks.len() as u16).to_le_bytes());

        let t_bytes = self.title.as_bytes();
        let t_len = t_bytes.len().min(64);
        buf[8..10].copy_from_slice(&(t_len as u16).to_le_bytes());
        buf[10..10 + t_len].copy_from_slice(&t_bytes[..t_len]);

        Ok(&buf[..10 + t_len])
    }

    /// Decodes a packet header and returns (version, block_count, title_slice).
    pub fn decode_packet_header(buf: &[u8]) -> Result<(u16, u16, &str), EdgeCmsError> {
        if buf.len() < 10 {
            return Err(EdgeCmsError::PayloadTruncated);
        }
        if &buf[0..4] != b"ERPC" {
            return Err(EdgeCmsError::InvalidMagic);
        }
        let version = u16::from_le_bytes([buf[4], buf[5]]);
        let block_count = u16::from_le_bytes([buf[6], buf[7]]);
        let t_len = u16::from_le_bytes([buf[8], buf[9]]) as usize;

        if buf.len() < 10 + t_len {
            return Err(EdgeCmsError::PayloadTruncated);
        }
        let title =
            core::str::from_utf8(&buf[10..10 + t_len]).map_err(|_| EdgeCmsError::InvalidUtf8)?;
        Ok((version, block_count, title))
    }
}

/// Zero-allocation video playback telemetry packet for edge signage and warehouse terminals.
///
/// Wire format:
/// `[0..4]`   Magic: `b"VODP"`
/// `[4..36]`  User ID (32 bytes UTF-8 null-padded)
/// `[36..68]` Media ID (32 bytes UTF-8 null-padded)
/// `[68..72]` Playhead position (u32 LE seconds)
/// `[72..76]` Total duration (u32 LE seconds)
/// `[76..78]` Buffered seconds (u16 LE)
/// `[78]`     Playback state: 0 = playing, 1 = paused
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoPlaybackPacket {
    pub user_id: heapless::String<32>,
    pub media_id: heapless::String<32>,
    pub playhead_sec: u32,
    pub duration_sec: u32,
    pub buffered_sec: u16,
    pub is_paused: bool,
}

impl VideoPlaybackPacket {
    /// Serializes playback telemetry into a fixed 81-byte slice (including 2-byte CRC16-CCITT).
    pub fn serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], EdgeCmsError> {
        if buf.len() < 81 {
            return Err(EdgeCmsError::BufferTooSmall);
        }
        buf[0..4].copy_from_slice(b"VODP");

        // Pad User ID to 32 bytes
        buf[4..36].fill(0);
        let u_bytes = self.user_id.as_bytes();
        let u_len = u_bytes.len().min(32);
        buf[4..4 + u_len].copy_from_slice(&u_bytes[..u_len]);

        // Pad Media ID to 32 bytes
        buf[36..68].fill(0);
        let m_bytes = self.media_id.as_bytes();
        let m_len = m_bytes.len().min(32);
        buf[36..36 + m_len].copy_from_slice(&m_bytes[..m_len]);

        buf[68..72].copy_from_slice(&self.playhead_sec.to_le_bytes());
        buf[72..76].copy_from_slice(&self.duration_sec.to_le_bytes());
        buf[76..78].copy_from_slice(&self.buffered_sec.to_le_bytes());
        buf[78] = if self.is_paused { 1 } else { 0 };

        let crc = crc16_ccitt(&buf[0..79]);
        buf[79..81].copy_from_slice(&crc.to_le_bytes());

        Ok(&buf[..81])
    }

    /// Decodes an 81-byte buffer into a `VideoPlaybackPacket` after validating CRC16-CCITT.
    pub fn decode(buf: &[u8]) -> Result<Self, EdgeCmsError> {
        if buf.len() < 81 {
            return Err(EdgeCmsError::PayloadTruncated);
        }
        if &buf[0..4] != b"VODP" {
            return Err(EdgeCmsError::InvalidMagic);
        }

        let computed_crc = crc16_ccitt(&buf[0..79]);
        let expected_crc = u16::from_le_bytes([buf[79], buf[80]]);
        if computed_crc != expected_crc {
            return Err(EdgeCmsError::ChecksumMismatch);
        }

        let u_slice = &buf[4..36];
        let u_end = u_slice.iter().position(|&b| b == 0).unwrap_or(32);
        let u_str =
            core::str::from_utf8(&u_slice[..u_end]).map_err(|_| EdgeCmsError::InvalidUtf8)?;
        let mut user_id = heapless::String::new();
        let _ = user_id.push_str(u_str);

        let m_slice = &buf[36..68];
        let m_end = m_slice.iter().position(|&b| b == 0).unwrap_or(32);
        let m_str =
            core::str::from_utf8(&m_slice[..m_end]).map_err(|_| EdgeCmsError::InvalidUtf8)?;
        let mut media_id = heapless::String::new();
        let _ = media_id.push_str(m_str);

        let playhead_sec = u32::from_le_bytes([buf[68], buf[69], buf[70], buf[71]]);
        let duration_sec = u32::from_le_bytes([buf[72], buf[73], buf[74], buf[75]]);
        let buffered_sec = u16::from_le_bytes([buf[76], buf[77]]);
        let is_paused = buf[78] == 1;

        Ok(Self {
            user_id,
            media_id,
            playhead_sec,
            duration_sec,
            buffered_sec,
            is_paused,
        })
    }

    /// Renders zero-allocation ANSI playback status bar for character LCD / serial terminals.
    pub fn render_ansi(&self, w: &mut impl core::fmt::Write) -> core::fmt::Result {
        let pct = (self.playhead_sec * 100)
            .checked_div(self.duration_sec)
            .unwrap_or(0);
        let status_icon = if self.is_paused { "||" } else { ">" };
        let p_h = self.playhead_sec / 3600;
        let p_m = (self.playhead_sec % 3600) / 60;
        let p_s = self.playhead_sec % 60;
        let d_h = self.duration_sec / 3600;
        let d_m = (self.duration_sec % 3600) / 60;
        let d_s = self.duration_sec % 60;

        write!(
            w,
            "\x1b[1;36m[{status_icon} {:02}:{:02}:{:02} / {:02}:{:02}:{:02}]\x1b[0m \x1b[32m[{}%]\x1b[0m (buf: {}s)",
            p_h, p_m, p_s, d_h, d_m, d_s, pct, self.buffered_sec
        )
    }
}

/// Zero-allocation RTMP broadcast telemetry packet for edge signage and warehouse terminals.
///
/// Wire format:
/// `[0..4]`   Magic: `b"RTMP"`
/// `[4..36]`  Channel ID (32 bytes UTF-8 null-padded)
/// `[36..40]` Ingress bitrate (u32 LE kbps)
/// `[40..44]` Viewer count (u32 LE)
/// `[44]`     Status: 1 = live, 0 = offline
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RtmpStreamTelemetryPacket {
    pub channel_id: heapless::String<32>,
    pub ingress_bitrate_kbps: u32,
    pub viewer_count: u32,
    pub is_live: bool,
}

impl RtmpStreamTelemetryPacket {
    /// Serializes RTMP broadcast telemetry into a fixed 47-byte slice (including 2-byte CRC16-CCITT).
    pub fn serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], EdgeCmsError> {
        if buf.len() < 47 {
            return Err(EdgeCmsError::BufferTooSmall);
        }
        buf[0..4].copy_from_slice(b"RTMP");

        // Pad Channel ID to 32 bytes
        buf[4..36].fill(0);
        let c_bytes = self.channel_id.as_bytes();
        let c_len = c_bytes.len().min(32);
        buf[4..4 + c_len].copy_from_slice(&c_bytes[..c_len]);

        buf[36..40].copy_from_slice(&self.ingress_bitrate_kbps.to_le_bytes());
        buf[40..44].copy_from_slice(&self.viewer_count.to_le_bytes());
        buf[44] = if self.is_live { 1 } else { 0 };

        let crc = crc16_ccitt(&buf[0..45]);
        buf[45..47].copy_from_slice(&crc.to_le_bytes());

        Ok(&buf[..47])
    }

    /// Decodes a 47-byte buffer into an `RtmpStreamTelemetryPacket` after validating CRC16-CCITT.
    pub fn decode(buf: &[u8]) -> Result<Self, EdgeCmsError> {
        if buf.len() < 47 {
            return Err(EdgeCmsError::PayloadTruncated);
        }
        if &buf[0..4] != b"RTMP" {
            return Err(EdgeCmsError::InvalidMagic);
        }

        let computed_crc = crc16_ccitt(&buf[0..45]);
        let expected_crc = u16::from_le_bytes([buf[45], buf[46]]);
        if computed_crc != expected_crc {
            return Err(EdgeCmsError::ChecksumMismatch);
        }

        let c_slice = &buf[4..36];
        let c_end = c_slice.iter().position(|&b| b == 0).unwrap_or(32);
        let c_str =
            core::str::from_utf8(&c_slice[..c_end]).map_err(|_| EdgeCmsError::InvalidUtf8)?;
        let mut channel_id = heapless::String::new();
        let _ = channel_id.push_str(c_str);

        let ingress_bitrate_kbps = u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]);
        let viewer_count = u32::from_le_bytes([buf[40], buf[41], buf[42], buf[43]]);
        let is_live = buf[44] == 1;

        Ok(Self {
            channel_id,
            ingress_bitrate_kbps,
            viewer_count,
            is_live,
        })
    }

    /// Renders zero-allocation ANSI broadcast status badge for serial terminal HUDs.
    pub fn render_ansi(&self, w: &mut impl core::fmt::Write) -> core::fmt::Result {
        if self.is_live {
            write!(
                w,
                "\x1b[1;31m[LIVE: {}]\x1b[0m \x1b[33m[{} kbps]\x1b[0m \x1b[32m[viewers: {}]\x1b[0m",
                self.channel_id.as_str(),
                self.ingress_bitrate_kbps,
                self.viewer_count
            )
        } else {
            write!(w, "\x1b[90m[OFFLINE: {}]\x1b[0m", self.channel_id.as_str())
        }
    }
}

/// Zero-allocation FAST linear channel playout status packet for edge signage.
///
/// Wire format:
/// `[0..4]`   Magic: `b"FAST"`
/// `[4..36]`  Channel ID (32 bytes UTF-8 null-padded)
/// `[36..84]` Program Title (48 bytes UTF-8 null-padded)
/// `[84..88]` Remaining seconds (u32 LE)
/// `[88..92]` Viewer count (u32 LE)
/// `[92]`     Is ad break active: 1 = yes, 0 = no
/// `[93..95]` CRC16-CCITT (u16 LE)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FastChannelStatusPacket {
    pub channel_id: heapless::String<32>,
    pub program_title: heapless::String<48>,
    pub remaining_sec: u32,
    pub viewer_count: u32,
    pub is_ad_break: bool,
}

impl FastChannelStatusPacket {
    /// Serializes FAST channel telemetry into a fixed 95-byte slice (including 2-byte CRC16-CCITT).
    pub fn serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], EdgeCmsError> {
        if buf.len() < 95 {
            return Err(EdgeCmsError::BufferTooSmall);
        }
        buf[0..4].copy_from_slice(b"FAST");

        // Pad Channel ID to 32 bytes
        buf[4..36].fill(0);
        let c_bytes = self.channel_id.as_bytes();
        let c_len = c_bytes.len().min(32);
        buf[4..4 + c_len].copy_from_slice(&c_bytes[..c_len]);

        // Pad Program Title to 48 bytes
        buf[36..84].fill(0);
        let p_bytes = self.program_title.as_bytes();
        let p_len = p_bytes.len().min(48);
        buf[36..36 + p_len].copy_from_slice(&p_bytes[..p_len]);

        buf[84..88].copy_from_slice(&self.remaining_sec.to_le_bytes());
        buf[88..92].copy_from_slice(&self.viewer_count.to_le_bytes());
        buf[92] = if self.is_ad_break { 1 } else { 0 };

        let crc = crc16_ccitt(&buf[0..93]);
        buf[93..95].copy_from_slice(&crc.to_le_bytes());

        Ok(&buf[..95])
    }

    /// Decodes a 95-byte buffer into a `FastChannelStatusPacket` after validating CRC16-CCITT.
    pub fn decode(buf: &[u8]) -> Result<Self, EdgeCmsError> {
        if buf.len() < 95 {
            return Err(EdgeCmsError::PayloadTruncated);
        }
        if &buf[0..4] != b"FAST" {
            return Err(EdgeCmsError::InvalidMagic);
        }

        let computed_crc = crc16_ccitt(&buf[0..93]);
        let expected_crc = u16::from_le_bytes([buf[93], buf[94]]);
        if computed_crc != expected_crc {
            return Err(EdgeCmsError::ChecksumMismatch);
        }

        let c_slice = &buf[4..36];
        let c_end = c_slice.iter().position(|&b| b == 0).unwrap_or(32);
        let c_str =
            core::str::from_utf8(&c_slice[..c_end]).map_err(|_| EdgeCmsError::InvalidUtf8)?;
        let mut channel_id = heapless::String::new();
        let _ = channel_id.push_str(c_str);

        let p_slice = &buf[36..84];
        let p_end = p_slice.iter().position(|&b| b == 0).unwrap_or(48);
        let p_str =
            core::str::from_utf8(&p_slice[..p_end]).map_err(|_| EdgeCmsError::InvalidUtf8)?;
        let mut program_title = heapless::String::new();
        let _ = program_title.push_str(p_str);

        let remaining_sec = u32::from_le_bytes([buf[84], buf[85], buf[86], buf[87]]);
        let viewer_count = u32::from_le_bytes([buf[88], buf[89], buf[90], buf[91]]);
        let is_ad_break = buf[92] == 1;

        Ok(Self {
            channel_id,
            program_title,
            remaining_sec,
            viewer_count,
            is_ad_break,
        })
    }

    /// Renders zero-allocation ANSI linear channel status for edge signage and terminals.
    pub fn render_ansi(&self, w: &mut impl core::fmt::Write) -> core::fmt::Result {
        let ad_tag = if self.is_ad_break {
            "\x1b[1;33m[AD BREAK]\x1b[0m"
        } else {
            "\x1b[1;32m[PROGRAM]\x1b[0m"
        };
        let rem_m = self.remaining_sec / 60;
        let rem_s = self.remaining_sec % 60;

        write!(
            w,
            "\x1b[1;35m[FAST: {}]\x1b[0m {ad_tag} \x1b[36m[{}]\x1b[0m (rem: {:02}:{:02}) [viewers: {}]",
            self.channel_id.as_str(),
            self.program_title.as_str(),
            rem_m,
            rem_s,
            self.viewer_count
        )
    }
}

/// Playback mode for zero-allocation edge audio devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum AudioPlaybackMode {
    #[default]
    Music = 0,
    Podcast = 1,
    SleepNoise = 2,
    LiveRadio = 3,
}

impl AudioPlaybackMode {
    #[must_use]
    pub const fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::Podcast,
            2 => Self::SleepNoise,
            3 => Self::LiveRadio,
            _ => Self::Music,
        }
    }
}

/// Noise soundscape color for procedural sleep generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum NoiseColor {
    #[default]
    None = 0,
    White = 1,
    Pink = 2,
    Brown = 3,
    Binaural = 4,
}

impl NoiseColor {
    #[must_use]
    pub const fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::White,
            2 => Self::Pink,
            3 => Self::Brown,
            4 => Self::Binaural,
            _ => Self::None,
        }
    }
}

/// Zero-allocation audio telemetry packet for smart speakers, sleep devices, and edge displays.
///
/// Wire format (63 bytes with CRC16-CCITT):
/// `[0..4]`   Magic: `b"AUDP"`
/// `[4..36]`  Track / Sound ID (32 bytes UTF-8 null-padded)
/// `[36..40]` Position (u32 LE ms)
/// `[40..44]` Duration (u32 LE ms)
/// `[44..46]` Master Volume (u16 LE, 0..1000 = 0.0%..100.0%)
/// `[46]`     Playback Mode: 0=Music, 1=Podcast, 2=SleepNoise, 3=LiveRadio
/// `[47]`     Noise Type: 0=None, 1=White, 2=Pink, 3=Brown, 4=Binaural
/// `[48]`     Sleep timer remaining minutes (u8)
/// `[49]`     Playback flags: bit 0 = paused, bit 1 = gapless, bit 2 = normalizing
/// `[50..61]` Reserved (11 bytes)
/// `[61..63]` CRC16-CCITT checksum (u16 LE)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AudioPlaybackPacket {
    pub track_id: heapless::String<32>,
    pub position_ms: u32,
    pub duration_ms: u32,
    pub volume_permille: u16,
    pub mode: AudioPlaybackMode,
    pub noise_color: NoiseColor,
    pub sleep_timer_rem_min: u8,
    pub is_paused: bool,
    pub is_gapless: bool,
    pub is_normalizing: bool,
}

impl AudioPlaybackPacket {
    /// Serializes audio telemetry into a fixed 63-byte slice (including 2-byte CRC16-CCITT).
    pub fn serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], EdgeCmsError> {
        if buf.len() < 63 {
            return Err(EdgeCmsError::BufferTooSmall);
        }
        buf[0..4].copy_from_slice(b"AUDP");

        // Pad Track ID to 32 bytes
        buf[4..36].fill(0);
        let t_bytes = self.track_id.as_bytes();
        let t_len = t_bytes.len().min(32);
        buf[4..4 + t_len].copy_from_slice(&t_bytes[..t_len]);

        buf[36..40].copy_from_slice(&self.position_ms.to_le_bytes());
        buf[40..44].copy_from_slice(&self.duration_ms.to_le_bytes());
        buf[44..46].copy_from_slice(&self.volume_permille.to_le_bytes());
        buf[46] = self.mode as u8;
        buf[47] = self.noise_color as u8;
        buf[48] = self.sleep_timer_rem_min;

        let mut flags = 0u8;
        if self.is_paused {
            flags |= 0b0000_0001;
        }
        if self.is_gapless {
            flags |= 0b0000_0010;
        }
        if self.is_normalizing {
            flags |= 0b0000_0100;
        }
        buf[49] = flags;

        // Reserved 11 bytes
        buf[50..61].fill(0);

        let crc = crc16_ccitt(&buf[0..61]);
        buf[61..63].copy_from_slice(&crc.to_le_bytes());

        Ok(&buf[..63])
    }

    /// Decodes a 63-byte buffer into an `AudioPlaybackPacket` after validating CRC16-CCITT.
    pub fn decode(buf: &[u8]) -> Result<Self, EdgeCmsError> {
        if buf.len() < 63 {
            return Err(EdgeCmsError::PayloadTruncated);
        }
        if &buf[0..4] != b"AUDP" {
            return Err(EdgeCmsError::InvalidMagic);
        }

        let computed_crc = crc16_ccitt(&buf[0..61]);
        let expected_crc = u16::from_le_bytes([buf[61], buf[62]]);
        if computed_crc != expected_crc {
            return Err(EdgeCmsError::ChecksumMismatch);
        }

        let t_slice = &buf[4..36];
        let t_end = t_slice.iter().position(|&b| b == 0).unwrap_or(32);
        let t_str =
            core::str::from_utf8(&t_slice[..t_end]).map_err(|_| EdgeCmsError::InvalidUtf8)?;
        let mut track_id = heapless::String::new();
        let _ = track_id.push_str(t_str);

        let position_ms = u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]);
        let duration_ms = u32::from_le_bytes([buf[40], buf[41], buf[42], buf[43]]);
        let volume_permille = u16::from_le_bytes([buf[44], buf[45]]);
        let mode = AudioPlaybackMode::from_u8(buf[46]);
        let noise_color = NoiseColor::from_u8(buf[47]);
        let sleep_timer_rem_min = buf[48];
        let flags = buf[49];
        let is_paused = (flags & 0b0000_0001) != 0;
        let is_gapless = (flags & 0b0000_0010) != 0;
        let is_normalizing = (flags & 0b0000_0100) != 0;

        Ok(Self {
            track_id,
            position_ms,
            duration_ms,
            volume_permille,
            mode,
            noise_color,
            sleep_timer_rem_min,
            is_paused,
            is_gapless,
            is_normalizing,
        })
    }

    /// Renders zero-allocation ANSI playback badge for serial consoles, character LCDs, and smart speaker HUDs.
    pub fn render_ansi(&self, w: &mut impl core::fmt::Write) -> core::fmt::Result {
        let status_icon = if self.is_paused { "||" } else { ">" };
        let mode_tag = match self.mode {
            AudioPlaybackMode::Music => "\x1b[1;32m[MUSIC]\x1b[0m",
            AudioPlaybackMode::Podcast => "\x1b[1;34m[PODCAST]\x1b[0m",
            AudioPlaybackMode::SleepNoise => "\x1b[1;35m[SLEEP NOISE]\x1b[0m",
            AudioPlaybackMode::LiveRadio => "\x1b[1;31m[LIVE RADIO]\x1b[0m",
        };
        let p_m = (self.position_ms / 1000) / 60;
        let p_s = (self.position_ms / 1000) % 60;
        let d_m = (self.duration_ms / 1000) / 60;
        let d_s = (self.duration_ms / 1000) % 60;
        let vol_pct = self.volume_permille / 10;

        write!(
            w,
            "{mode_tag} \x1b[1;36m[{status_icon} {:02}:{:02} / {:02}:{:02}]\x1b[0m \x1b[33m[Vol: {}%]\x1b[0m (ID: {})",
            p_m,
            p_s,
            d_m,
            d_s,
            vol_pct,
            self.track_id.as_str()
        )?;

        if self.sleep_timer_rem_min > 0 {
            write!(w, " \x1b[95m[Zzz: {}m]\x1b[0m", self.sleep_timer_rem_min)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StringWriter<const N: usize> {
        buf: heapless::String<N>,
    }

    impl<const N: usize> StringWriter<N> {
        fn new() -> Self {
            Self {
                buf: heapless::String::new(),
            }
        }
    }

    impl<const N: usize> core::fmt::Write for StringWriter<N> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            self.buf.push_str(s).map_err(|_| core::fmt::Error)
        }
    }

    #[test]
    fn test_edge_canvas_zero_allocation_rendering() {
        let mut layout: EdgeCanvasLayout<16> = EdgeCanvasLayout::new("Scanner Warehouse HUD");

        let mut h1_text = heapless::String::new();
        h1_text.push_str("Pallet Verification").unwrap();
        layout
            .add_block(EdgeBlockNode::new(
                "h1",
                EdgeBlockType::Heading {
                    level: 1,
                    text: h1_text,
                },
            ))
            .unwrap();

        let mut val_text = heapless::String::new();
        val_text.push_str("PLT-98234-A").unwrap();
        let mut lbl_text = heapless::String::new();
        lbl_text.push_str("Active Tag").unwrap();
        layout
            .add_block(EdgeBlockNode::new(
                "metric",
                EdgeBlockType::MetricCard {
                    label: lbl_text,
                    value: val_text,
                },
            ))
            .unwrap();

        let mut bc_payload = heapless::String::new();
        bc_payload.push_str("0101234567890128").unwrap();
        let mut bc_sym = heapless::String::new();
        bc_sym.push_str("GS1-128").unwrap();
        layout
            .add_block(EdgeBlockNode::new(
                "bc",
                EdgeBlockType::Barcode {
                    payload: bc_payload,
                    symbology: bc_sym,
                },
            ))
            .unwrap();

        layout
            .add_block(EdgeBlockNode::new("div", EdgeBlockType::Divider))
            .unwrap();

        // 1. Plain text rendering
        let mut plain_out = StringWriter::<1024>::new();
        layout.render_plain_text(&mut plain_out).unwrap();
        let plain_str = plain_out.buf.as_str();

        assert!(plain_str.contains("=== Scanner Warehouse HUD (v1) ==="));
        assert!(plain_str.contains(">>> Pallet Verification <<<"));
        assert!(plain_str.contains("[Active Tag: PLT-98234-A]"));
        assert!(plain_str.contains("[*BARCODE: 0101234567890128 (GS1-128)*]"));

        // 2. ANSI terminal rendering
        let mut ansi_out = StringWriter::<1024>::new();
        layout.render_ansi(&mut ansi_out).unwrap();
        let ansi_str = ansi_out.buf.as_str();

        assert!(ansi_str.contains("\x1b[1;36m"));
        assert!(ansi_str.contains("\x1b[1;32m"));
    }

    #[test]
    fn test_edge_packet_header_roundtrip() {
        let mut layout: EdgeCanvasLayout<8> = EdgeCanvasLayout::new("Kiosk Display");
        layout.version = 42;
        layout
            .add_block(EdgeBlockNode::new("d", EdgeBlockType::Divider))
            .unwrap();

        let mut buf = [0u8; 128];
        let packet = layout.serialize_packet_header(&mut buf).unwrap();
        assert_eq!(&packet[0..4], b"ERPC");

        let (ver, count, title) = EdgeCanvasLayout::<8>::decode_packet_header(packet).unwrap();
        assert_eq!(ver, 42);
        assert_eq!(count, 1);
        assert_eq!(title, "Kiosk Display");
    }

    #[test]
    fn test_edge_breakpoint_columns() {
        assert_eq!(EdgeBreakpoint::Desktop.default_columns(), 80);
        assert_eq!(EdgeBreakpoint::Tablet.default_columns(), 40);
        assert_eq!(EdgeBreakpoint::Mobile.default_columns(), 20);
    }

    #[test]
    fn test_video_playback_packet_roundtrip() {
        let mut user_id = heapless::String::new();
        user_id.push_str("usr_8829").unwrap();
        let mut media_id = heapless::String::new();
        media_id.push_str("vid_4412").unwrap();

        let packet = VideoPlaybackPacket {
            user_id,
            media_id,
            playhead_sec: 1420,
            duration_sec: 7200,
            buffered_sec: 60,
            is_paused: false,
        };

        let mut buf = [0u8; 128];
        let bytes = packet.serialize(&mut buf).unwrap();
        assert_eq!(bytes.len(), 81);
        assert_eq!(&bytes[0..4], b"VODP");

        let decoded = VideoPlaybackPacket::decode(bytes).unwrap();
        assert_eq!(decoded.user_id.as_str(), "usr_8829");
        assert_eq!(decoded.media_id.as_str(), "vid_4412");
        assert_eq!(decoded.playhead_sec, 1420);
        assert_eq!(decoded.duration_sec, 7200);
        assert_eq!(decoded.buffered_sec, 60);
        assert!(!decoded.is_paused);

        // Verification of CRC16 corruption rejection
        let mut corrupted = [0u8; 81];
        corrupted.copy_from_slice(bytes);
        corrupted[10] ^= 0xFF;
        assert_eq!(
            VideoPlaybackPacket::decode(&corrupted),
            Err(EdgeCmsError::ChecksumMismatch)
        );
    }

    #[test]
    fn test_rtmp_telemetry_packet_roundtrip() {
        let mut channel_id = heapless::String::new();
        channel_id.push_str("live_stream_prime").unwrap();

        let packet = RtmpStreamTelemetryPacket {
            channel_id,
            ingress_bitrate_kbps: 4500,
            viewer_count: 1240,
            is_live: true,
        };

        let mut buf = [0u8; 64];
        let bytes = packet.serialize(&mut buf).unwrap();
        assert_eq!(bytes.len(), 47);
        assert_eq!(&bytes[0..4], b"RTMP");

        let decoded = RtmpStreamTelemetryPacket::decode(bytes).unwrap();
        assert_eq!(decoded.channel_id.as_str(), "live_stream_prime");
        assert_eq!(decoded.ingress_bitrate_kbps, 4500);
        assert_eq!(decoded.viewer_count, 1240);
        assert!(decoded.is_live);

        // Verification of CRC16 corruption rejection
        let mut corrupted = [0u8; 47];
        corrupted.copy_from_slice(bytes);
        corrupted[15] ^= 0xAA;
        assert_eq!(
            RtmpStreamTelemetryPacket::decode(&corrupted),
            Err(EdgeCmsError::ChecksumMismatch)
        );
    }

    #[test]
    fn test_fast_channel_status_packet_roundtrip() {
        let mut channel_id = heapless::String::new();
        channel_id.push_str("ch_action_movies").unwrap();
        let mut program_title = heapless::String::new();
        program_title
            .push_str("Die Hard 2 - Midnight Feature")
            .unwrap();

        let packet = FastChannelStatusPacket {
            channel_id,
            program_title,
            remaining_sec: 1845,
            viewer_count: 5210,
            is_ad_break: false,
        };

        let mut buf = [0u8; 128];
        let bytes = packet.serialize(&mut buf).unwrap();
        assert_eq!(bytes.len(), 95);
        assert_eq!(&bytes[0..4], b"FAST");

        let decoded = FastChannelStatusPacket::decode(bytes).unwrap();
        assert_eq!(decoded.channel_id.as_str(), "ch_action_movies");
        assert_eq!(
            decoded.program_title.as_str(),
            "Die Hard 2 - Midnight Feature"
        );
        assert_eq!(decoded.remaining_sec, 1845);
        assert_eq!(decoded.viewer_count, 5210);
        assert!(!decoded.is_ad_break);

        // Verification of CRC16 corruption rejection
        let mut corrupted = [0u8; 95];
        corrupted.copy_from_slice(bytes);
        corrupted[20] ^= 0x55;
        assert_eq!(
            FastChannelStatusPacket::decode(&corrupted),
            Err(EdgeCmsError::ChecksumMismatch)
        );
    }

    #[test]
    fn test_ansi_rendering_edge_dashboards() {
        // 1. Video playback ANSI
        let mut u_id = heapless::String::new();
        u_id.push_str("u1").unwrap();
        let mut m_id = heapless::String::new();
        m_id.push_str("m1").unwrap();
        let v_pkt = VideoPlaybackPacket {
            user_id: u_id,
            media_id: m_id,
            playhead_sec: 3661, // 01:01:01
            duration_sec: 7200, // 02:00:00
            buffered_sec: 45,
            is_paused: false,
        };
        let mut v_out = heapless::String::<128>::new();
        v_pkt.render_ansi(&mut v_out).unwrap();
        assert!(v_out.contains("01:01:01 / 02:00:00"));
        assert!(v_out.contains("50%"));

        // 2. RTMP telemetry ANSI
        let mut c_id = heapless::String::new();
        c_id.push_str("gaming").unwrap();
        let r_pkt = RtmpStreamTelemetryPacket {
            channel_id: c_id,
            ingress_bitrate_kbps: 6000,
            viewer_count: 950,
            is_live: true,
        };
        let mut r_out = heapless::String::<128>::new();
        r_pkt.render_ansi(&mut r_out).unwrap();
        assert!(r_out.contains("LIVE: gaming"));
        assert!(r_out.contains("6000 kbps"));

        // 3. FAST channel ANSI
        let mut f_cid = heapless::String::new();
        f_cid.push_str("retro").unwrap();
        let mut f_title = heapless::String::new();
        f_title.push_str("Classic Cartoons").unwrap();
        let f_pkt = FastChannelStatusPacket {
            channel_id: f_cid,
            program_title: f_title,
            remaining_sec: 300,
            viewer_count: 420,
            is_ad_break: true,
        };
        let mut f_out = heapless::String::<128>::new();
        f_pkt.render_ansi(&mut f_out).unwrap();
        assert!(f_out.contains("FAST: retro"));
        assert!(f_out.contains("AD BREAK"));
        assert!(f_out.contains("Classic Cartoons"));
    }

    #[test]
    fn test_audio_playback_packet_roundtrip() {
        let mut track_id = heapless::String::new();
        track_id.push_str("trk_neon_dreams_99").unwrap();

        let packet = AudioPlaybackPacket {
            track_id,
            position_ms: 125000,
            duration_ms: 240000,
            volume_permille: 850,
            mode: AudioPlaybackMode::Music,
            noise_color: NoiseColor::None,
            sleep_timer_rem_min: 30,
            is_paused: false,
            is_gapless: true,
            is_normalizing: true,
        };

        let mut buf = [0u8; 128];
        let bytes = packet.serialize(&mut buf).unwrap();
        assert_eq!(bytes.len(), 63);
        assert_eq!(&bytes[0..4], b"AUDP");

        let decoded = AudioPlaybackPacket::decode(bytes).unwrap();
        assert_eq!(decoded.track_id.as_str(), "trk_neon_dreams_99");
        assert_eq!(decoded.position_ms, 125000);
        assert_eq!(decoded.duration_ms, 240000);
        assert_eq!(decoded.volume_permille, 850);
        assert_eq!(decoded.mode, AudioPlaybackMode::Music);
        assert_eq!(decoded.noise_color, NoiseColor::None);
        assert_eq!(decoded.sleep_timer_rem_min, 30);
        assert!(!decoded.is_paused);
        assert!(decoded.is_gapless);
        assert!(decoded.is_normalizing);

        // Verify ANSI rendering
        let mut ansi_out = heapless::String::<128>::new();
        decoded.render_ansi(&mut ansi_out).unwrap();
        assert!(ansi_out.contains("MUSIC"));
        assert!(ansi_out.contains("02:05 / 04:00"));
        assert!(ansi_out.contains("Vol: 85%"));
        assert!(ansi_out.contains("Zzz: 30m"));

        // Verify CRC16 corruption rejection
        let mut corrupted = [0u8; 63];
        corrupted.copy_from_slice(bytes);
        corrupted[10] ^= 0xEE;
        assert_eq!(
            AudioPlaybackPacket::decode(&corrupted),
            Err(EdgeCmsError::ChecksumMismatch)
        );
    }
}
