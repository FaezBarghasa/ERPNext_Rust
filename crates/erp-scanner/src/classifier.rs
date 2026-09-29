//! Scan Classification & Symbology Detection (`erp_scanner::classifier`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Supported barcode and 2D matrix symbologies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Symbology {
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

/// Classified raw scan signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawScanSignal {
    pub raw_data: CompactString,
    pub symbology: Symbology,
    pub source: ScanSource,
    pub timestamp_ms: u64,
}

/// Capture modality source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanSource {
    HardwareLaserWedge,
    CameraWasmStream,
    AndroidNdkBroadcast,
    ManualInput,
}

/// Classifier detecting symbologies from decoded scan patterns.
pub struct ScanClassifier;

impl ScanClassifier {
    /// Classifies a raw payload into its likely barcode symbology.
    #[must_use]
    pub fn classify_symbology(payload: &str) -> Symbology {
        let trimmed = payload.trim();

        if trimmed.starts_with("GS1:") || trimmed.starts_with("(01)") {
            return Symbology::Gs1_128;
        }

        if trimmed.len() == 13 && trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Symbology::Ean13;
        }

        if trimmed.len() == 8 && trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Symbology::Ean8;
        }

        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return Symbology::QrCode;
        }

        if trimmed.starts_with("ITEM:")
            || trimmed.starts_with("LOC:")
            || trimmed.starts_with("JOB:")
            || trimmed.starts_with("ASSET:")
            || trimmed.starts_with("BADGE:")
        {
            return Symbology::QrCode;
        }

        if trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.') {
            return Symbology::Code128;
        }

        Symbology::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_symbologies() {
        assert_eq!(
            ScanClassifier::classify_symbology("GS1:(01)01234567890128(10)BATCH123"),
            Symbology::Gs1_128
        );
        assert_eq!(
            ScanClassifier::classify_symbology("4006381333931"),
            Symbology::Ean13
        );
        assert_eq!(
            ScanClassifier::classify_symbology("12345678"),
            Symbology::Ean8
        );
        assert_eq!(
            ScanClassifier::classify_symbology("ITEM:WIDGET-001"),
            Symbology::QrCode
        );
        assert_eq!(
            ScanClassifier::classify_symbology("JOB:JC-2026-0001"),
            Symbology::QrCode
        );
    }
}
