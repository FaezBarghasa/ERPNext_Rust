#![no_std]

//! Zero-Allocation `no_std` Edge Scanner Engine (`erp-scanner-core`).
//!
//! Provides bare-metal / embedded runtime support:
//! - Static bounded memory (`heapless::Vec` and `heapless::String`)
//! - GS1-128 Application Identifier (AI) parsing for industrial supply chain tracking
//! - Fast barcode & 2D matrix symbology classification
//! - Inter-keystroke timing burst detection for hardware laser scanner wedges.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Supported barcode and 2D matrix symbologies in embedded core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SymbologyCore {
    QrCode,
    MicroQr,
    DataMatrix,
    Aztec,
    Gs1_128,
    Ean13,
    Ean8,
    Code128,
    Code39,
    Unknown,
}

/// Classifies a raw payload slice into its symbology without any dynamic heap allocation.
#[must_use]
pub fn classify_symbology(payload: &str) -> SymbologyCore {
    let trimmed = payload.trim_ascii();

    if trimmed.starts_with("GS1:")
        || trimmed.starts_with("(01)")
        || (trimmed.starts_with("01") && trimmed.len() >= 16)
    {
        return SymbologyCore::Gs1_128;
    }

    if trimmed.len() == 13 && trimmed.as_bytes().iter().all(u8::is_ascii_digit) {
        return SymbologyCore::Ean13;
    }

    if trimmed.len() == 8 && trimmed.as_bytes().iter().all(u8::is_ascii_digit) {
        return SymbologyCore::Ean8;
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return SymbologyCore::QrCode;
    }

    if trimmed.starts_with("ITEM:")
        || trimmed.starts_with("LOC:")
        || trimmed.starts_with("JOB:")
        || trimmed.starts_with("ASSET:")
        || trimmed.starts_with("BADGE:")
    {
        return SymbologyCore::QrCode;
    }

    if trimmed.starts_with("[)>") || trimmed.starts_with("\x1e06") {
        return SymbologyCore::DataMatrix;
    }

    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return SymbologyCore::Code128;
    }

    SymbologyCore::Unknown
}

/// Individual parsed GS1 Application Identifier (AI) element.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Gs1Element {
    pub ai: heapless::String<6>,
    pub raw_value: heapless::String<64>,
}

/// Structured outcome of zero-allocation GS1-128 parsing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Gs1ParsedData {
    pub elements: heapless::Vec<Gs1Element, 16>,
    pub gtin: Option<heapless::String<14>>,
    pub batch_lot: Option<heapless::String<32>>,
    pub serial_number: Option<heapless::String<32>>,
    pub expiry_yymmdd: Option<heapless::String<6>>,
    pub count: Option<u32>,
}

/// Errors during GS1 parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gs1ParseError {
    EmptyPayload,
    CapacityExceeded,
    MalformedAi,
    UnexpectedEof,
}

/// Zero-allocation parser for GS1-128 bar codes with Application Identifiers.
/// Supports parenthesized format `(01)...(10)...` and FNC1 delimiter `\x1d`.
pub fn parse_gs1_128(payload: &str) -> Result<Gs1ParsedData, Gs1ParseError> {
    let raw = payload
        .strip_prefix("GS1:")
        .unwrap_or(payload)
        .trim_ascii();

    if raw.is_empty() {
        return Err(Gs1ParseError::EmptyPayload);
    }

    let mut result = Gs1ParsedData::default();

    // Check if input uses parenthesized notation: (01)...(10)...
    if raw.contains('(') && raw.contains(')') {
        let mut rest = raw;
        while let Some(start_paren) = rest.find('(') {
            let after_open = &rest[start_paren + 1..];
            let close_paren = after_open.find(')').ok_or(Gs1ParseError::MalformedAi)?;
            let ai_str = &after_open[..close_paren];

            let after_close = &after_open[close_paren + 1..];
            // Value continues until next '(' or end of string
            let val_end = after_close.find('(').unwrap_or(after_close.len());
            let val_str = &after_close[..val_end];

            let mut ai = heapless::String::<6>::new();
            ai.push_str(ai_str)
                .map_err(|_| Gs1ParseError::CapacityExceeded)?;

            let mut val = heapless::String::<64>::new();
            val.push_str(val_str)
                .map_err(|_| Gs1ParseError::CapacityExceeded)?;

            // Store in structured convenience fields
            match ai.as_str() {
                "01" => {
                    let mut gtin = heapless::String::<14>::new();
                    let _ = gtin.push_str(val.as_str());
                    result.gtin = Some(gtin);
                }
                "10" => {
                    let mut lot = heapless::String::<32>::new();
                    let _ = lot.push_str(val.as_str());
                    result.batch_lot = Some(lot);
                }
                "21" => {
                    let mut serial = heapless::String::<32>::new();
                    let _ = serial.push_str(val.as_str());
                    result.serial_number = Some(serial);
                }
                "17" => {
                    let mut exp = heapless::String::<6>::new();
                    let _ = exp.push_str(val.as_str());
                    result.expiry_yymmdd = Some(exp);
                }
                "30" | "37" => {
                    if let Ok(c) = val.parse::<u32>() {
                        result.count = Some(c);
                    }
                }
                _ => {}
            }

            result
                .elements
                .push(Gs1Element { ai, raw_value: val })
                .map_err(|_| Gs1ParseError::CapacityExceeded)?;

            rest = &after_close[val_end..];
        }
    } else {
        // Stream format using FNC1 (\x1d) or fixed-length lookups
        let mut idx = 0;
        let bytes = raw.as_bytes();

        while idx < bytes.len() {
            if bytes[idx] == 0x1d {
                idx += 1;
                continue;
            }

            if idx + 2 > bytes.len() {
                break;
            }

            let ai_candidate = &raw[idx..idx + 2];
            match ai_candidate {
                "01" => {
                    // GTIN-14 is fixed 14 digits
                    idx += 2;
                    let end = (idx + 14).min(bytes.len());
                    let val_str = &raw[idx..end];
                    idx = end;

                    let mut ai = heapless::String::<6>::new();
                    let _ = ai.push_str("01");
                    let mut val = heapless::String::<64>::new();
                    let _ = val.push_str(val_str);

                    let mut gtin = heapless::String::<14>::new();
                    let _ = gtin.push_str(val_str);
                    result.gtin = Some(gtin);

                    let _ = result.elements.push(Gs1Element { ai, raw_value: val });
                }
                "17" => {
                    // Expiry is fixed 6 digits (YYMMDD)
                    idx += 2;
                    let end = (idx + 6).min(bytes.len());
                    let val_str = &raw[idx..end];
                    idx = end;

                    let mut ai = heapless::String::<6>::new();
                    let _ = ai.push_str("17");
                    let mut val = heapless::String::<64>::new();
                    let _ = val.push_str(val_str);

                    let mut exp = heapless::String::<6>::new();
                    let _ = exp.push_str(val_str);
                    result.expiry_yymmdd = Some(exp);

                    let _ = result.elements.push(Gs1Element { ai, raw_value: val });
                }
                "10" | "21" => {
                    // Variable length up to 20 chars, terminated by FNC1 (0x1d) or end
                    let ai_name = ai_candidate;
                    idx += 2;
                    let start = idx;
                    while idx < bytes.len() && bytes[idx] != 0x1d {
                        idx += 1;
                    }
                    let val_str = &raw[start..idx];

                    let mut ai = heapless::String::<6>::new();
                    let _ = ai.push_str(ai_name);
                    let mut val = heapless::String::<64>::new();
                    let _ = val.push_str(val_str);

                    if ai_name == "10" {
                        let mut lot = heapless::String::<32>::new();
                        let _ = lot.push_str(val_str);
                        result.batch_lot = Some(lot);
                    } else {
                        let mut serial = heapless::String::<32>::new();
                        let _ = serial.push_str(val_str);
                        result.serial_number = Some(serial);
                    }

                    let _ = result.elements.push(Gs1Element { ai, raw_value: val });
                }
                _ => {
                    // Unknown AI: consume rest
                    let val_str = &raw[idx..];
                    let mut ai = heapless::String::<6>::new();
                    let _ = ai.push_str("raw");
                    let mut val = heapless::String::<64>::new();
                    let _ = val.push_str(val_str);
                    let _ = result.elements.push(Gs1Element { ai, raw_value: val });
                    break;
                }
            }
        }
    }

    Ok(result)
}

/// Zero-allocation hardware laser wedge burst detector.
#[derive(Debug, Clone)]
pub struct NoStdLaserWedge<const CAP: usize> {
    buffer: heapless::String<CAP>,
    last_timestamp_ms: u64,
    max_delta_ms: u64,
}

impl<const CAP: usize> NoStdLaserWedge<CAP> {
    #[must_use]
    pub const fn new(max_delta_ms: u64) -> Self {
        Self {
            buffer: heapless::String::new(),
            last_timestamp_ms: 0,
            max_delta_ms,
        }
    }

    /// Feeds a character and timestamp. Returns the completed barcode string if a burst finishes.
    pub fn feed(&mut self, ch: char, timestamp_ms: u64) -> Option<heapless::String<CAP>> {
        let delta = timestamp_ms.saturating_sub(self.last_timestamp_ms);
        self.last_timestamp_ms = timestamp_ms;

        // Check for enter key termination
        if ch == '\r' || ch == '\n' {
            if !self.buffer.is_empty() {
                let out = self.buffer.clone();
                self.buffer.clear();
                return Some(out);
            }
            return None;
        }

        // If human typing delay occurred, reset buffer
        if delta > self.max_delta_ms && !self.buffer.is_empty() {
            self.buffer.clear();
        }

        // Push char if capacity permits
        let _ = self.buffer.push(ch);
        None
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.last_timestamp_ms = 0;
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn test_zero_alloc_symbology_classification() {
        assert_eq!(
            classify_symbology("GS1:(01)01234567890128(10)LOT42"),
            SymbologyCore::Gs1_128
        );
        assert_eq!(classify_symbology("4006381333931"), SymbologyCore::Ean13);
        assert_eq!(classify_symbology("12345678"), SymbologyCore::Ean8);
        assert_eq!(
            classify_symbology("ITEM:PUMP-001"),
            SymbologyCore::QrCode
        );
        assert_eq!(classify_symbology("INV-2026-99"), SymbologyCore::Code128);
    }

    #[test]
    fn test_gs1_128_parentheses_parsing() {
        let raw = "GS1:(01)01234567890128(10)BATCH99(17)261231(21)SER1001";
        let parsed = parse_gs1_128(raw).unwrap();

        assert_eq!(parsed.elements.len(), 4);
        assert_eq!(parsed.gtin.as_deref(), Some("01234567890128"));
        assert_eq!(parsed.batch_lot.as_deref(), Some("BATCH99"));
        assert_eq!(parsed.expiry_yymmdd.as_deref(), Some("261231"));
        assert_eq!(parsed.serial_number.as_deref(), Some("SER1001"));
    }

    #[test]
    fn test_gs1_128_stream_parsing() {
        let raw = "01012345678901281726123110BATCH77\x1d21SER555";
        let parsed = parse_gs1_128(raw).unwrap();

        assert_eq!(parsed.gtin.as_deref(), Some("01234567890128"));
        assert_eq!(parsed.expiry_yymmdd.as_deref(), Some("261231"));
        assert_eq!(parsed.batch_lot.as_deref(), Some("BATCH77"));
        assert_eq!(parsed.serial_number.as_deref(), Some("SER555"));
    }

    #[test]
    fn test_no_std_laser_wedge_interceptor() {
        let mut wedge = NoStdLaserWedge::<64>::new(35);

        // Burst typing (< 35ms delta)
        assert_eq!(wedge.feed('I', 100), None);
        assert_eq!(wedge.feed('T', 110), None);
        assert_eq!(wedge.feed('E', 120), None);
        assert_eq!(wedge.feed('M', 130), None);

        // Enter key finishes burst
        let result = wedge.feed('\n', 135);
        assert_eq!(result.as_deref(), Some("ITEM"));

        // Human delay resets buffer
        assert_eq!(wedge.feed('A', 500), None);
        assert_eq!(wedge.feed('B', 600), None); // > 35ms -> clears 'A' and buffers 'B'
        let result2 = wedge.feed('\r', 610);
        assert_eq!(result2.as_deref(), Some("B"));
    }
}
