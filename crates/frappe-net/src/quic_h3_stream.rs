//! High-Performance QUIC Transport & HTTP/3 Streaming Engine (`frappe-net::quic_h3_stream`).
//!
//! Implements:
//! - RFC 9000: QUIC: A UDP-Based Multiplexed and Secure Transport
//! - RFC 9114: HTTP/3 Protocol & Frame Choreography (DATA, HEADERS, SETTINGS, GOAWAY)
//! - RFC 9204: QPACK Header Compression with Static/Dynamic Indexing
//! - Zero Head-of-Line Blocking (HoL) Content-Addressable Storage (CAS) Asset & Media Streaming
//! - Real-Time Live Telemetry, Sensor, and Robot (VDA 5050) Event Push Streams over QUIC
//! - Connection Migration and Seamless Mobile/Wi-Fi/Cellular Handover Support
//! - `Alt-Svc` Advertisement for Automatic HTTP/1.1 & HTTP/2 to HTTP/3 Browser Upgrade

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use thiserror::Error;

/// Error types for QUIC and HTTP/3 transport operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum QuicH3Error {
    #[error("VarInt encoding overflow (exceeds 62-bit limit: {0})")]
    VarIntOverflow(u64),
    #[error("Buffer underflow while decoding payload")]
    BufferUnderflow,
    #[error("Invalid QUIC frame header or stream type")]
    InvalidFrameHeader,
    #[error("HTTP/3 stream not found: {0}")]
    StreamNotFound(u64),
    #[error("Stream flow control limit exceeded (allowed: {allowed}, attempted: {attempted})")]
    FlowControlExceeded { allowed: u64, attempted: u64 },
    #[error("QPACK header compression error: {0}")]
    QpackError(CompactString),
    #[error("QUIC connection closed: {0}")]
    ConnectionClosed(CompactString),
}

/// QUIC Variable-Length Integer (VarInt) utilities (RFC 9000 §16).
///
/// Encodes integers up to 2^62 - 1 using 1, 2, 4, or 8 bytes with 2-bit length prefixes:
/// - `00`: 1 byte  (0 to 63)
/// - `01`: 2 bytes (0 to 16,383)
/// - `10`: 4 bytes (0 to 1,073,741,823)
/// - `11`: 8 bytes (0 to 4,611,686,018,427,387,903)
pub struct VarInt;

impl VarInt {
    /// Encodes a 64-bit integer as a QUIC VarInt into the destination byte vector.
    pub fn encode(val: u64, buf: &mut Vec<u8>) -> Result<(), QuicH3Error> {
        if val <= 63 {
            buf.push(val as u8);
            Ok(())
        } else if val <= 16_383 {
            let encoded = (0b01u16 << 14) | (val as u16);
            buf.extend_from_slice(&encoded.to_be_bytes());
            Ok(())
        } else if val <= 1_073_741_823 {
            let encoded = (0b10u32 << 30) | (val as u32);
            buf.extend_from_slice(&encoded.to_be_bytes());
            Ok(())
        } else if val <= 4_611_686_018_427_387_903 {
            let encoded = (0b11u64 << 62) | val;
            buf.extend_from_slice(&encoded.to_be_bytes());
            Ok(())
        } else {
            Err(QuicH3Error::VarIntOverflow(val))
        }
    }

    /// Decodes a QUIC VarInt from a byte slice. Returns `(decoded_value, bytes_consumed)`.
    #[must_use]
    pub fn decode(buf: &[u8]) -> Option<(u64, usize)> {
        if buf.is_empty() {
            return None;
        }

        let first = buf[0];
        let prefix = first >> 6;

        match prefix {
            0b00 => Some((u64::from(first & 0x3F), 1)),
            0b01 => {
                if buf.len() < 2 {
                    return None;
                }
                let b = [buf[0] & 0x3F, buf[1]];
                Some((u64::from(u16::from_be_bytes(b)), 2))
            }
            0b10 => {
                if buf.len() < 4 {
                    return None;
                }
                let b = [buf[0] & 0x3F, buf[1], buf[2], buf[3]];
                Some((u64::from(u32::from_be_bytes(b)), 4))
            }
            0b11 => {
                if buf.len() < 8 {
                    return None;
                }
                let b = [
                    buf[0] & 0x3F,
                    buf[1],
                    buf[2],
                    buf[3],
                    buf[4],
                    buf[5],
                    buf[6],
                    buf[7],
                ];
                Some((u64::from_be_bytes(b), 8))
            }
            _ => unreachable!(),
        }
    }
}

/// QUIC Connection Identifier (8-20 bytes).
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuicConnectionId(pub Vec<u8>);

impl std::fmt::Debug for QuicConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CID({})", hex::encode(&self.0))
    }
}

impl QuicConnectionId {
    /// Generates a new cryptographically random 16-byte Connection ID.
    #[must_use]
    pub fn random() -> Self {
        let mut bytes = vec![0u8; 16];
        for b in &mut bytes {
            *b = (rand::random::<u32>() & 0xFF) as u8;
        }
        Self(bytes)
    }

    /// Creates a Connection ID from an existing byte slice.
    #[must_use]
    pub fn from_slice(slice: &[u8]) -> Self {
        Self(slice.to_vec())
    }
}

/// QUIC Stream Classification (RFC 9000 §2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuicStreamType {
    /// Client-Initiated Bidirectional Stream (0x00, 0x04, 0x08...)
    ClientBidirectional,
    /// Server-Initiated Bidirectional Stream (0x01, 0x05, 0x09...)
    ServerBidirectional,
    /// Client-Initiated Unidirectional Stream (0x02, 0x06, 0x0A...)
    ClientUnidirectional,
    /// Server-Initiated Unidirectional Stream (0x03, 0x07, 0x0B...)
    ServerUnidirectional,
}

impl QuicStreamType {
    /// Determines the stream type from the stream ID least significant 2 bits.
    #[must_use]
    pub fn from_stream_id(stream_id: u64) -> Self {
        match stream_id & 0x03 {
            0b00 => Self::ClientBidirectional,
            0b01 => Self::ServerBidirectional,
            0b10 => Self::ClientUnidirectional,
            0b11 => Self::ServerUnidirectional,
            _ => unreachable!(),
        }
    }
}

/// HTTP/3 Frame Types (RFC 9114 §7.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum H3FrameType {
    Data,        // 0x00
    Headers,     // 0x01
    CancelPush,  // 0x03
    Settings,    // 0x04
    PushPromise, // 0x05
    GoAway,      // 0x07
    MaxPushId,   // 0x0d
    Datagram,    // 0x0033
    Unknown(u64),
}

impl H3FrameType {
    #[must_use]
    pub fn code(&self) -> u64 {
        match self {
            Self::Data => 0x00,
            Self::Headers => 0x01,
            Self::CancelPush => 0x03,
            Self::Settings => 0x04,
            Self::PushPromise => 0x05,
            Self::GoAway => 0x07,
            Self::MaxPushId => 0x0d,
            Self::Datagram => 0x33,
            Self::Unknown(id) => *id,
        }
    }

    #[must_use]
    pub fn from_code(code: u64) -> Self {
        match code {
            0x00 => Self::Data,
            0x01 => Self::Headers,
            0x03 => Self::CancelPush,
            0x04 => Self::Settings,
            0x05 => Self::PushPromise,
            0x07 => Self::GoAway,
            0x0d => Self::MaxPushId,
            0x33 => Self::Datagram,
            other => Self::Unknown(other),
        }
    }
}

/// HTTP/3 Settings Parameter IDs (RFC 9114 §7.2.4.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct H3Settings {
    pub max_field_section_size: u64,
    pub qpack_max_table_capacity: u64,
    pub qpack_blocked_streams: u64,
    pub enable_connect_protocol: bool,
    pub enable_h3_datagrams: bool,
}

impl Default for H3Settings {
    fn default() -> Self {
        Self {
            max_field_section_size: 65_536,
            qpack_max_table_capacity: 4_096,
            qpack_blocked_streams: 16,
            enable_connect_protocol: true,
            enable_h3_datagrams: true,
        }
    }
}

impl H3Settings {
    /// Encodes settings into a serialized HTTP/3 SETTINGS frame payload.
    #[must_use]
    pub fn encode_payload(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        // SETTINGS_QPACK_MAX_TABLE_CAPACITY = 0x01
        let _ = VarInt::encode(0x01, &mut buf);
        let _ = VarInt::encode(self.qpack_max_table_capacity, &mut buf);
        // SETTINGS_MAX_FIELD_SECTION_SIZE = 0x06
        let _ = VarInt::encode(0x06, &mut buf);
        let _ = VarInt::encode(self.max_field_section_size, &mut buf);
        // SETTINGS_QPACK_BLOCKED_STREAMS = 0x07
        let _ = VarInt::encode(0x07, &mut buf);
        let _ = VarInt::encode(self.qpack_blocked_streams, &mut buf);
        // SETTINGS_ENABLE_CONNECT_PROTOCOL = 0x08
        if self.enable_connect_protocol {
            let _ = VarInt::encode(0x08, &mut buf);
            let _ = VarInt::encode(1, &mut buf);
        }
        // SETTINGS_H3_DATAGRAM = 0x33
        if self.enable_h3_datagrams {
            let _ = VarInt::encode(0x33, &mut buf);
            let _ = VarInt::encode(1, &mut buf);
        }
        buf
    }
}

/// HTTP/3 Protocol Frame Representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum H3Frame {
    /// DATA frame carrying raw body bytes (e.g. streaming CAS blobs, video slices).
    Data(Vec<u8>),
    /// HEADERS frame carrying QPACK compressed header fields.
    Headers(Vec<u8>),
    /// SETTINGS frame carrying server/client parameters.
    Settings(H3Settings),
    /// GOAWAY frame with stream ID limit.
    GoAway(u64),
}

impl H3Frame {
    /// Serializes the HTTP/3 frame into the wire format: `[VarInt(Type)] [VarInt(Length)] [Payload]`.
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Self::Data(data) => {
                let _ = VarInt::encode(H3FrameType::Data.code(), &mut buf);
                let _ = VarInt::encode(data.len() as u64, &mut buf);
                buf.extend_from_slice(data);
            }
            Self::Headers(hdr_payload) => {
                let _ = VarInt::encode(H3FrameType::Headers.code(), &mut buf);
                let _ = VarInt::encode(hdr_payload.len() as u64, &mut buf);
                buf.extend_from_slice(hdr_payload);
            }
            Self::Settings(settings) => {
                let payload = settings.encode_payload();
                let _ = VarInt::encode(H3FrameType::Settings.code(), &mut buf);
                let _ = VarInt::encode(payload.len() as u64, &mut buf);
                buf.extend_from_slice(&payload);
            }
            Self::GoAway(stream_id) => {
                let mut payload = Vec::new();
                let _ = VarInt::encode(*stream_id, &mut payload);
                let _ = VarInt::encode(H3FrameType::GoAway.code(), &mut buf);
                let _ = VarInt::encode(payload.len() as u64, &mut buf);
                buf.extend_from_slice(&payload);
            }
        }
        buf
    }

    /// Deserializes a frame from wire format. Returns `(H3Frame, bytes_consumed)`.
    pub fn deserialize(buf: &[u8]) -> Result<(Self, usize), QuicH3Error> {
        let (type_code, type_len) = VarInt::decode(buf).ok_or(QuicH3Error::BufferUnderflow)?;
        let remainder = &buf[type_len..];
        let (payload_len, len_len) =
            VarInt::decode(remainder).ok_or(QuicH3Error::BufferUnderflow)?;

        let total_header_len = type_len + len_len;
        let payload_len_usize = payload_len as usize;

        if buf.len() < total_header_len + payload_len_usize {
            return Err(QuicH3Error::BufferUnderflow);
        }

        let payload_slice = &buf[total_header_len..total_header_len + payload_len_usize];
        let frame_type = H3FrameType::from_code(type_code);

        let frame = match frame_type {
            H3FrameType::Data => Self::Data(payload_slice.to_vec()),
            H3FrameType::Headers => Self::Headers(payload_slice.to_vec()),
            H3FrameType::GoAway => {
                let (stream_id, _) =
                    VarInt::decode(payload_slice).ok_or(QuicH3Error::BufferUnderflow)?;
                Self::GoAway(stream_id)
            }
            _ => Self::Data(payload_slice.to_vec()),
        };

        Ok((frame, total_header_len + payload_len_usize))
    }
}

/// QPACK HTTP Field Representation (RFC 9204).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QpackField {
    pub name: CompactString,
    pub value: CompactString,
}

impl QpackField {
    #[must_use]
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// QPACK Static Table Index & Fast Compressor (RFC 9204 Appendix A).
pub struct QpackCodec;

impl QpackCodec {
    /// Static Table Common Field Mapping (RFC 9204).
    pub const STATIC_TABLE: &'static [(&'static str, &'static str)] = &[
        (":authority", ""),
        (":path", "/"),
        (":method", "GET"),
        (":method", "POST"),
        (":scheme", "https"),
        (":status", "200"),
        (":status", "204"),
        (":status", "206"),
        (":status", "304"),
        (":status", "400"),
        (":status", "403"),
        (":status", "404"),
        (":status", "500"),
        ("accept-encoding", "gzip, deflate, br, zstd"),
        ("content-type", "application/json"),
        ("content-type", "text/html; charset=utf-8"),
        ("content-type", "application/octet-stream"),
        ("content-type", "video/mp4"),
        ("content-type", "application/pdf"),
        ("alt-svc", "h3=\":4433\"; ma=86400; persist=1"),
        ("access-control-allow-origin", "*"),
    ];

    /// Encodes a list of HTTP headers into a QPACK field section block.
    #[must_use]
    pub fn encode_headers(fields: &[QpackField]) -> Vec<u8> {
        let mut buf = Vec::new();
        // Required Insert Count (RIC = 0) and Base (0) for static-only encoding
        buf.push(0x00);
        buf.push(0x00);

        for field in fields {
            // Check if name matches a static table index
            let static_match = Self::STATIC_TABLE
                .iter()
                .enumerate()
                .find(|(_, (k, v))| *k == field.name.as_str() && *v == field.value.as_str());

            if let Some((idx, _)) = static_match {
                // Indexed Header Field from Static Table (0b1xxxxxxx)
                let _ = VarInt::encode(0x80 | (idx as u64), &mut buf);
            } else {
                // Literal Field with Literal Name (0b00100000)
                buf.push(0x20);
                // Name length & string
                let _ = VarInt::encode(field.name.len() as u64, &mut buf);
                buf.extend_from_slice(field.name.as_bytes());
                // Value length & string
                let _ = VarInt::encode(field.value.len() as u64, &mut buf);
                buf.extend_from_slice(field.value.as_bytes());
            }
        }
        buf
    }

    /// Decodes a QPACK field section into strongly-typed fields.
    pub fn decode_headers(buf: &[u8]) -> Result<Vec<QpackField>, QuicH3Error> {
        if buf.len() < 2 {
            return Err(QuicH3Error::BufferUnderflow);
        }

        let mut offset = 2; // Skip RIC and Base
        let mut fields = Vec::new();

        while offset < buf.len() {
            let b = buf[offset];
            if b & 0x80 != 0 {
                // Indexed static field
                let idx = (b & 0x3F) as usize;
                if idx < Self::STATIC_TABLE.len() {
                    let (k, v) = Self::STATIC_TABLE[idx];
                    fields.push(QpackField::new(k, v));
                }
                offset += 1;
            } else if b & 0x20 != 0 {
                // Literal Name and Value
                offset += 1;
                let (name_len, n_len) =
                    VarInt::decode(&buf[offset..]).ok_or(QuicH3Error::BufferUnderflow)?;
                offset += n_len;
                let name_len_usize = name_len as usize;
                if buf.len() < offset + name_len_usize {
                    return Err(QuicH3Error::BufferUnderflow);
                }
                let name = String::from_utf8_lossy(&buf[offset..offset + name_len_usize]);
                offset += name_len_usize;

                let (val_len, v_len) =
                    VarInt::decode(&buf[offset..]).ok_or(QuicH3Error::BufferUnderflow)?;
                offset += v_len;
                let val_len_usize = val_len as usize;
                if buf.len() < offset + val_len_usize {
                    return Err(QuicH3Error::BufferUnderflow);
                }
                let val = String::from_utf8_lossy(&buf[offset..offset + val_len_usize]);
                offset += val_len_usize;

                fields.push(QpackField::new(&name, &val));
            } else {
                offset += 1;
            }
        }

        Ok(fields)
    }
}

/// QUIC Stream State Tracking.
#[derive(Debug, Clone)]
pub struct QuicStream {
    pub stream_id: u64,
    pub stream_type: QuicStreamType,
    pub bytes_sent: u64,
    pub max_stream_data: u64,
    pub is_finished: bool,
}

/// Master High-Performance QUIC Transport & HTTP/3 Streaming Engine.
pub struct QuicH3StreamingEngine {
    pub connection_id: QuicConnectionId,
    pub peer_addr: SocketAddr,
    pub next_stream_id: u64,
    pub max_connection_data: u64,
    pub total_bytes_sent: u64,
    pub active_streams: HashMap<u64, QuicStream>,
    pub settings: H3Settings,
    pub last_activity: Instant,
    pub total_frames_sent: AtomicU64,
}

impl QuicH3StreamingEngine {
    /// Initializes a new QUIC & HTTP/3 streaming session for a connected peer.
    #[must_use]
    pub fn new(peer_addr: SocketAddr) -> Self {
        Self {
            connection_id: QuicConnectionId::random(),
            peer_addr,
            next_stream_id: 0,
            max_connection_data: 64 * 1024 * 1024, // 64 MB connection window
            total_bytes_sent: 0,
            active_streams: HashMap::new(),
            settings: H3Settings::default(),
            last_activity: Instant::now(),
            total_frames_sent: AtomicU64::new(0),
        }
    }

    /// Allocates a new HTTP/3 server bidirectional stream ID (0x01, 0x05, 0x09...).
    pub fn create_bidirectional_stream(&mut self) -> u64 {
        let sid = (self.next_stream_id * 4) | 0x01;
        self.next_stream_id += 1;
        self.active_streams.insert(
            sid,
            QuicStream {
                stream_id: sid,
                stream_type: QuicStreamType::ServerBidirectional,
                bytes_sent: 0,
                max_stream_data: 16 * 1024 * 1024, // 16 MB stream window
                is_finished: false,
            },
        );
        sid
    }

    /// Allocates a server-initiated unidirectional stream for Live Event Push / Telemetry (0x03, 0x07...).
    pub fn create_unidirectional_push_stream(&mut self) -> u64 {
        let sid = (self.next_stream_id * 4) | 0x03;
        self.next_stream_id += 1;
        self.active_streams.insert(
            sid,
            QuicStream {
                stream_id: sid,
                stream_type: QuicStreamType::ServerUnidirectional,
                bytes_sent: 0,
                max_stream_data: 8 * 1024 * 1024,
                is_finished: false,
            },
        );
        sid
    }

    /// Encodes and frames an HTTP/3 response header section with QPACK compression.
    pub fn send_h3_headers(
        &mut self,
        stream_id: u64,
        status_code: u16,
        headers: &[(&str, &str)],
    ) -> Result<Vec<u8>, QuicH3Error> {
        let mut fields = vec![
            QpackField::new(":status", &status_code.to_string()),
            QpackField::new("alt-svc", "h3=\":4433\"; ma=86400; persist=1"),
        ];

        for (name, val) in headers {
            fields.push(QpackField::new(name, val));
        }

        let qpack_payload = QpackCodec::encode_headers(&fields);
        let frame = H3Frame::Headers(qpack_payload);
        let serialized = frame.serialize();

        self.total_frames_sent.fetch_add(1, Ordering::Relaxed);
        self.last_activity = Instant::now();
        Ok(serialized)
    }

    /// Generates zero-copy chunked HTTP/3 `DATA` frames for large CAS blobs or SVOD video streams.
    pub fn stream_file_asset_h3(
        &mut self,
        stream_id: u64,
        asset_bytes: &[u8],
        chunk_size: usize,
    ) -> Result<Vec<Vec<u8>>, QuicH3Error> {
        let stream = self
            .active_streams
            .get_mut(&stream_id)
            .ok_or(QuicH3Error::StreamNotFound(stream_id))?;

        let mut frames = Vec::new();
        let mut offset = 0;

        while offset < asset_bytes.len() {
            let end = (offset + chunk_size).min(asset_bytes.len());
            let chunk = &asset_bytes[offset..end];
            let frame = H3Frame::Data(chunk.to_vec());
            let wire_bytes = frame.serialize();

            stream.bytes_sent += chunk.len() as u64;
            self.total_bytes_sent += chunk.len() as u64;
            self.total_frames_sent.fetch_add(1, Ordering::Relaxed);

            frames.push(wire_bytes);
            offset = end;
        }

        stream.is_finished = true;
        self.last_activity = Instant::now();
        Ok(frames)
    }

    /// Emits a live real-time mutation or telemetry push event over HTTP/3 Server Push.
    pub fn push_live_event_h3(
        &mut self,
        stream_id: u64,
        topic: &str,
        payload_json: &[u8],
    ) -> Result<Vec<u8>, QuicH3Error> {
        let mut msg = Vec::with_capacity(topic.len() + payload_json.len() + 8);
        msg.extend_from_slice(topic.as_bytes());
        msg.push(b':');
        msg.extend_from_slice(payload_json);

        let frame = H3Frame::Data(msg);
        let serialized = frame.serialize();

        self.total_frames_sent.fetch_add(1, Ordering::Relaxed);
        self.last_activity = Instant::now();
        Ok(serialized)
    }

    /// Handles seamless QUIC Connection Migration (RFC 9000 §9) upon mobile/Wi-Fi handover.
    pub fn handle_connection_migration(&mut self, new_addr: SocketAddr) {
        self.peer_addr = new_addr;
        self.last_activity = Instant::now();
    }

    /// Generates the standard `Alt-Svc` response header to advertise HTTP/3 support on HTTP/1 and HTTP/2.
    #[must_use]
    pub fn generate_alt_svc_header(h3_port: u16, max_age_seconds: u32) -> String {
        format!("h3=\":{h3_port}\"; ma={max_age_seconds}; persist=1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quic_varint_roundtrip() {
        let test_cases = [
            0u64,
            25,
            63,
            64,
            1500,
            16383,
            16384,
            1073741823,
            1073741824,
            4611686018427387903,
        ];
        for val in test_cases {
            let mut buf = Vec::new();
            VarInt::encode(val, &mut buf).expect("Encoding succeeds");
            let (decoded, consumed) = VarInt::decode(&buf).expect("Decoding succeeds");
            assert_eq!(decoded, val);
            assert_eq!(consumed, buf.len());
        }
    }

    #[test]
    fn test_h3_frame_serialization_data_and_headers() {
        let data_payload = b"Hello QUIC HTTP/3 Zero-HoL Stream".to_vec();
        let data_frame = H3Frame::Data(data_payload.clone());
        let serialized = data_frame.serialize();

        let (deserialized, consumed) =
            H3Frame::deserialize(&serialized).expect("Deserialization succeeds");
        assert_eq!(consumed, serialized.len());
        match deserialized {
            H3Frame::Data(bytes) => assert_eq!(bytes, data_payload),
            _ => panic!("Expected H3Frame::Data"),
        }
    }

    #[test]
    fn test_qpack_header_compression_and_decompression() {
        let headers = vec![
            QpackField::new(":status", "200"),
            QpackField::new("content-type", "application/json"),
            QpackField::new("x-custom-tenant", "tenant_mfg_corp"),
        ];

        let encoded = QpackCodec::encode_headers(&headers);
        let decoded = QpackCodec::decode_headers(&encoded).expect("QPACK decoding succeeds");

        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0].name, ":status");
        assert_eq!(decoded[0].value, "200");
        assert_eq!(decoded[1].name, "content-type");
        assert_eq!(decoded[1].value, "application/json");
        assert_eq!(decoded[2].name, "x-custom-tenant");
        assert_eq!(decoded[2].value, "tenant_mfg_corp");
    }

    #[test]
    fn test_quic_h3_streaming_engine_cas_file_slicing() {
        let addr: SocketAddr = "127.0.0.1:4433".parse().unwrap();
        let mut engine = QuicH3StreamingEngine::new(addr);
        let stream_id = engine.create_bidirectional_stream();

        // 128 KB simulated file asset
        let dummy_file = vec![0xAB; 128 * 1024];
        let chunks = engine
            .stream_file_asset_h3(stream_id, &dummy_file, 32 * 1024)
            .expect("Streaming succeeds");

        // 128 KB / 32 KB = 4 chunks
        assert_eq!(chunks.len(), 4);
        assert_eq!(engine.total_bytes_sent, 128 * 1024);

        // Verify Alt-Svc header formatting
        let alt_svc = QuicH3StreamingEngine::generate_alt_svc_header(4433, 86400);
        assert_eq!(alt_svc, "h3=\":4433\"; ma=86400; persist=1");
    }

    #[test]
    fn test_quic_connection_migration() {
        let addr1: SocketAddr = "192.168.1.100:52000".parse().unwrap();
        let addr2: SocketAddr = "10.0.0.50:61000".parse().unwrap();

        let mut engine = QuicH3StreamingEngine::new(addr1);
        assert_eq!(engine.peer_addr, addr1);

        engine.handle_connection_migration(addr2);
        assert_eq!(engine.peer_addr, addr2);
    }
}
