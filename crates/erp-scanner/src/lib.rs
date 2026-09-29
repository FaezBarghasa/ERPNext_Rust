//! Native High-Velocity QR & Barcode Intelligence Engine (`erp-scanner`).
//!
//! Implements Pillar XX: Decoupled multi-symbology detection, hardware laser wedge
//! inter-keystroke temporal interception (<35ms), WASM camera video frame ingestion,
//! and strongly-typed domain intent routing.

pub mod android_intent;
pub mod camera_wasm;
pub mod classifier;
pub mod intent_router;
pub mod laser_wedge;

pub use android_intent::{AndroidIntentParser, DecodedHardwareIntent, ScannerHardwareVendor};
pub use camera_wasm::{CameraFrameBuffer, CameraWasmScanner};
pub use classifier::{RawScanSignal, ScanClassifier, ScanSource, Symbology};
pub use intent_router::{IntentRouter, ScanIntent};
pub use laser_wedge::LaserWedgeInterceptor;

use compact_str::CompactString;

/// High-level Unified Scanner Facade.
pub struct ErpScannerFacade {
    interceptor: laser_wedge::LaserWedgeInterceptor,
}

impl Default for ErpScannerFacade {
    fn default() -> Self {
        Self::new(35)
    }
}

impl ErpScannerFacade {
    /// Creates a new scanner engine with laser timing threshold in milliseconds.
    #[must_use]
    pub fn new(max_inter_keystroke_delta_ms: u64) -> Self {
        Self {
            interceptor: laser_wedge::LaserWedgeInterceptor::new(max_inter_keystroke_delta_ms),
        }
    }

    /// Processes an incoming raw string payload from any capture modality into a typed `ScanIntent`.
    #[must_use]
    pub fn process_raw_scan(&self, raw_payload: &str, source: ScanSource) -> (RawScanSignal, ScanIntent) {
        let symbology = ScanClassifier::classify_symbology(raw_payload);
        let intent = IntentRouter::parse_intent(raw_payload);

        let signal = RawScanSignal {
            raw_data: CompactString::from(raw_payload),
            symbology,
            source,
            timestamp_ms: chrono::Utc::now().timestamp_millis() as u64,
        };

        (signal, intent)
    }

    /// Feeds hardware keyboard keystrokes to detect laser scan bursts.
    pub fn feed_keystroke(&mut self, ch: char, timestamp_ms: u64) -> Option<(RawScanSignal, ScanIntent)> {
        self.interceptor
            .feed_char(ch, timestamp_ms)
            .map(|decoded| self.process_raw_scan(&decoded, ScanSource::HardwareLaserWedge))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_facade_end_to_end() {
        let facade = ErpScannerFacade::default();
        let (signal, intent) = facade.process_raw_scan("ITEM:WIDGET-2026", ScanSource::CameraWasmStream);

        assert_eq!(signal.symbology, Symbology::QrCode);
        assert_eq!(signal.source, ScanSource::CameraWasmStream);
        assert_eq!(
            intent,
            ScanIntent::ItemSetup {
                item_code: "WIDGET-2026".into()
            }
        );
    }
}
