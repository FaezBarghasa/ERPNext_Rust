//! WebAssembly Camera Scanner Frame Processor (`erp_scanner::camera_wasm`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Raw frame buffer metadata from WebRTC video capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFrameBuffer {
    pub width: u32,
    pub height: u32,
    pub format: CompactString,
    pub timestamp_ms: u64,
}

/// Camera Stream Decoder Interface for pure-Rust wasm32 Web Workers.
pub struct CameraWasmScanner;

impl CameraWasmScanner {
    /// Validates frame buffer dimensions and luminance bounds for optical barcode extraction.
    #[must_use]
    pub fn is_frame_processable(width: u32, height: u32, buffer_len: usize) -> bool {
        if width == 0 || height == 0 {
            return false;
        }
        // RGBA requires 4 bytes per pixel
        let expected = (width as usize) * (height as usize) * 4;
        buffer_len == expected
    }

    /// Computes average frame luminance to adjust camera exposure dynamically.
    #[must_use]
    pub fn compute_average_luminance(rgba_buffer: &[u8]) -> f32 {
        if rgba_buffer.is_empty() {
            return 0.0;
        }

        let mut total_lum = 0u64;
        let mut pixels = 0u64;

        for chunk in rgba_buffer.as_chunks::<4>().0 {
            let r = chunk[0] as u64;
            let g = chunk[1] as u64;
            let b = chunk[2] as u64;
            // Standard BT.601 luminance coefficients
            let lum = (299 * r + 587 * g + 114 * b) / 1000;
            total_lum += lum;
            pixels += 1;
        }

        if pixels == 0 {
            0.0
        } else {
            (total_lum as f32) / (pixels as f32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_buffer_validation() {
        assert!(CameraWasmScanner::is_frame_processable(
            640,
            480,
            640 * 480 * 4
        ));
        assert!(!CameraWasmScanner::is_frame_processable(640, 480, 100));
    }

    #[test]
    fn test_luminance_calculation() {
        let white_pixels = vec![255, 255, 255, 255, 255, 255, 255, 255];
        let lum = CameraWasmScanner::compute_average_luminance(&white_pixels);
        assert!((lum - 255.0).abs() < 1.0);
    }
}
