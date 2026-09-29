//! HTTP/2 Pseudo-Header Ordering & Settings Frame Choreography (`erp_stealth_scraper::http2_frame`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// HTTP/2 Settings Parameter representation matching RFC 7540 / RFC 9113.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Http2SettingsFrame {
    pub header_table_size: u32,
    pub enable_push: bool,
    pub max_concurrent_streams: Option<u32>,
    pub initial_window_size: u32,
    pub max_frame_size: u32,
    pub max_header_list_size: Option<u32>,
}

impl Default for Http2SettingsFrame {
    /// Exact Chrome 130 HTTP/2 SETTINGS frame parameters.
    fn default() -> Self {
        Self {
            header_table_size: 65536,
            enable_push: false,
            max_concurrent_streams: None,
            initial_window_size: 6291456,
            max_frame_size: 16384,
            max_header_list_size: Some(262144),
        }
    }
}

/// HTTP/2 Pseudo-Header ordering and request choreography.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Http2Choreography {
    pub pseudo_header_order: Vec<CompactString>,
    pub window_update_increment: u32,
    pub priority_weight: u8,
    pub settings: Http2SettingsFrame,
}

impl Default for Http2Choreography {
    fn default() -> Self {
        Self {
            // Chrome enforces strict pseudo-header sequence: :method, :authority, :scheme, :path
            pseudo_header_order: vec![
                ":method".into(),
                ":authority".into(),
                ":scheme".into(),
                ":path".into(),
            ],
            window_update_increment: 15663105,
            priority_weight: 255,
            settings: Http2SettingsFrame::default(),
        }
    }
}

impl Http2Choreography {
    /// Verifies if a given sequence of pseudo-headers strictly matches the Chrome standard.
    #[must_use]
    pub fn is_valid_chrome_order(headers: &[&str]) -> bool {
        let expected = [":method", ":authority", ":scheme", ":path"];
        if headers.len() != expected.len() {
            return false;
        }
        headers.iter().zip(expected.iter()).all(|(a, b)| a == b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http2_choreography_default() {
        let choreo = Http2Choreography::default();
        assert_eq!(choreo.pseudo_header_order.len(), 4);
        assert_eq!(choreo.settings.initial_window_size, 6291456);
        assert!(Http2Choreography::is_valid_chrome_order(&[
            ":method",
            ":authority",
            ":scheme",
            ":path"
        ]));
        assert!(!Http2Choreography::is_valid_chrome_order(&[
            ":path",
            ":method",
            ":authority",
            ":scheme"
        ]));
    }
}
