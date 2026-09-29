//! Android NDK Hardware Laser Broadcast Intent Decoder (`erp_scanner::android_intent`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Broadcast intent action types for industrial enterprise mobile terminals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScannerHardwareVendor {
    ZebraDataWedge,
    HoneywellMobilityEdge,
    NewlandEnterprise,
    CipherLab,
}

/// Parsed hardware broadcast intent data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedHardwareIntent {
    pub vendor: ScannerHardwareVendor,
    pub decoded_barcode: CompactString,
    pub symbology_type: CompactString,
    pub timestamp_utc: i64,
}

/// Android Broadcast Intent Parser.
pub struct AndroidIntentParser;

impl AndroidIntentParser {
    /// Extracts barcode string and symbology from standard Android Intent Extras.
    #[must_use]
    pub fn parse_zebra_intent(
        action: &str,
        data_string: Option<&str>,
        symbology_type: Option<&str>,
    ) -> Option<DecodedHardwareIntent> {
        // Zebra DataWedge intent action: com.symbol.datawedge.api.RESULT_ACTION or custom broadcast
        if !action.contains("datawedge") && !action.contains("SCAN_RESULT") {
            return None;
        }

        let barcode = data_string?;
        Some(DecodedHardwareIntent {
            vendor: ScannerHardwareVendor::ZebraDataWedge,
            decoded_barcode: barcode.into(),
            symbology_type: symbology_type.unwrap_or("LABEL-TYPE-UNKNOWN").into(),
            timestamp_utc: chrono::Utc::now().timestamp(),
        })
    }

    /// Extracts barcode from Honeywell Mobility Edge broadcast intents.
    #[must_use]
    pub fn parse_honeywell_intent(
        action: &str,
        data_string: Option<&str>,
    ) -> Option<DecodedHardwareIntent> {
        // Honeywell intent action: com.honeywell.decode.intent.action.SCAN_RESULT
        if !action.contains("honeywell") && !action.contains("decode") {
            return None;
        }

        let barcode = data_string?;
        Some(DecodedHardwareIntent {
            vendor: ScannerHardwareVendor::HoneywellMobilityEdge,
            decoded_barcode: barcode.into(),
            symbology_type: "HONEYWELL-AUTO".into(),
            timestamp_utc: chrono::Utc::now().timestamp(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_zebra_broadcast_intent() {
        let intent = AndroidIntentParser::parse_zebra_intent(
            "com.symbol.datawedge.api.RESULT_ACTION",
            Some("ITEM:PROD-8899"),
            Some("LABEL-TYPE-EAN128"),
        );
        assert!(intent.is_some());
        let res = intent.unwrap();
        assert_eq!(res.vendor, ScannerHardwareVendor::ZebraDataWedge);
        assert_eq!(res.decoded_barcode, "ITEM:PROD-8899");
    }

    #[test]
    fn test_parse_honeywell_intent() {
        let intent = AndroidIntentParser::parse_honeywell_intent(
            "com.honeywell.decode.intent.action.SCAN_RESULT",
            Some("LOC:Main-Rack:BIN-01"),
        );
        assert!(intent.is_some());
        let res = intent.unwrap();
        assert_eq!(res.vendor, ScannerHardwareVendor::HoneywellMobilityEdge);
        assert_eq!(res.decoded_barcode, "LOC:Main-Rack:BIN-01");
    }
}
