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
}

impl core::fmt::Display for EdgeCmsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CapacityExceeded => write!(f, "Block capacity exceeded in static buffer"),
            Self::BufferTooSmall => write!(f, "Provided serialization buffer is too small"),
            Self::InvalidMagic => write!(f, "Invalid magic packet header (expected ERPC)"),
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence in packet title"),
            Self::PayloadTruncated => write!(f, "Packet payload is truncated"),
        }
    }
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
        self.blocks.push(node).map_err(|_| EdgeCmsError::CapacityExceeded)
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
                    writeln!(out, "[*BARCODE: {} ({})*]", payload.as_str(), symbology.as_str())?;
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
        writeln!(out, "\x1b[1;36m=== {} (v{}) ===\x1b[0m", self.title, self.version)?;
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
                    writeln!(out, "\x1b[7m {} \x1b[0m \x1b[1;32m{}\x1b[0m", label.as_str(), value.as_str())?;
                }
                EdgeBlockType::Barcode { payload, symbology } => {
                    writeln!(out, "\x1b[35m|||| {} [{}] ||||\x1b[0m", payload.as_str(), symbology.as_str())?;
                }
                EdgeBlockType::Callout { title, text } => {
                    writeln!(out, "\x1b[1;31m[!] {}\x1b[0m: {}", title.as_str(), text.as_str())?;
                }
                EdgeBlockType::Divider => {
                    writeln!(out, "\x1b[90m----------------------------------------\x1b[0m")?;
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
        let title = core::str::from_utf8(&buf[10..10 + t_len]).map_err(|_| EdgeCmsError::InvalidUtf8)?;
        Ok((version, block_count, title))
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
}
