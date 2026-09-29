//! Typed Domain Intent Routing & Dispatch Matrix (`erp_scanner::intent_router`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Strongly-typed business intent extracted from decoded scan payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanIntent {
    /// Item Master lookup or Rapid Setup wizard.
    ItemSetup {
        item_code: CompactString,
    },
    /// Serial and Batch Bundle (SABB) parsing with GS1 AI segmentation.
    SerialAndBatchBundle {
        gtin: CompactString,
        batch_no: Option<CompactString>,
        serial_no: Option<CompactString>,
        expiry_date: Option<CompactString>,
    },
    /// Warehouse Bin location verification during Pick / Putaway.
    WarehouseBin {
        warehouse: CompactString,
        bin_code: CompactString,
    },
    /// Shop-Floor Manufacturing Job Card timer control.
    JobCard {
        job_card_id: CompactString,
    },
    /// Fixed Asset physical audit verification and geolocation update.
    FixedAsset {
        asset_id: CompactString,
    },
    /// Worker Badge authentication for floor kiosk / clock-in.
    WorkerBadge {
        worker_uuid: CompactString,
        auth_token: CompactString,
    },
    /// Generic / Unmapped barcode text.
    GenericText {
        raw_text: CompactString,
    },
}

/// Intent Router parsing formatted raw strings into actionable domain intents.
pub struct IntentRouter;

impl IntentRouter {
    /// Parses any raw scan string into a typed `ScanIntent`.
    #[must_use]
    pub fn parse_intent(raw: &str) -> ScanIntent {
        let trimmed = raw.trim();

        // 1. Explicit Prefixes
        if let Some(rest) = trimmed.strip_prefix("ITEM:") {
            return ScanIntent::ItemSetup {
                item_code: rest.into(),
            };
        }

        if let Some(rest) = trimmed.strip_prefix("LOC:") {
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() >= 2 {
                return ScanIntent::WarehouseBin {
                    warehouse: parts[0].into(),
                    bin_code: parts[1].into(),
                };
            }
            return ScanIntent::WarehouseBin {
                warehouse: "Default".into(),
                bin_code: rest.into(),
            };
        }

        if let Some(rest) = trimmed.strip_prefix("JOB:") {
            return ScanIntent::JobCard {
                job_card_id: rest.into(),
            };
        }

        if let Some(rest) = trimmed.strip_prefix("ASSET:") {
            return ScanIntent::FixedAsset {
                asset_id: rest.into(),
            };
        }

        if let Some(rest) = trimmed.strip_prefix("BADGE:") {
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() >= 2 {
                return ScanIntent::WorkerBadge {
                    worker_uuid: parts[0].into(),
                    auth_token: parts[1].into(),
                };
            }
            return ScanIntent::WorkerBadge {
                worker_uuid: rest.into(),
                auth_token: "pin_verified".into(),
            };
        }

        // 2. GS1 Application Identifier (AI) Parsing: (01)GTIN (10)BATCH (21)SERIAL (17)EXPIRY
        if trimmed.starts_with("GS1:") || trimmed.starts_with("(01)") {
            return Self::parse_gs1_string(trimmed);
        }

        // 3. Fallback: EAN-13 / EAN-8 / Code128 standard item code
        if (trimmed.len() == 13 || trimmed.len() == 8 || trimmed.len() == 12)
            && trimmed.chars().all(|c| c.is_ascii_digit())
        {
            return ScanIntent::ItemSetup {
                item_code: trimmed.into(),
            };
        }

        ScanIntent::GenericText {
            raw_text: trimmed.into(),
        }
    }

    /// Internal parser for GS1 AI formatted strings.
    fn parse_gs1_string(raw: &str) -> ScanIntent {
        let clean = raw.strip_prefix("GS1:").unwrap_or(raw);

        let mut gtin = CompactString::default();
        let mut batch_no = None;
        let mut serial_no = None;
        let mut expiry_date = None;

        // Parse (01) GTIN
        if let Some(start) = clean.find("(01)") {
            let slice = &clean[start + 4..];
            let end = slice.find('(').unwrap_or(slice.len());
            gtin = slice[..end].into();
        }

        // Parse (10) Batch
        if let Some(start) = clean.find("(10)") {
            let slice = &clean[start + 4..];
            let end = slice.find('(').unwrap_or(slice.len());
            batch_no = Some(slice[..end].into());
        }

        // Parse (21) Serial
        if let Some(start) = clean.find("(21)") {
            let slice = &clean[start + 4..];
            let end = slice.find('(').unwrap_or(slice.len());
            serial_no = Some(slice[..end].into());
        }

        // Parse (17) Expiry
        if let Some(start) = clean.find("(17)") {
            let slice = &clean[start + 4..];
            let end = slice.find('(').unwrap_or(slice.len());
            expiry_date = Some(slice[..end].into());
        }

        ScanIntent::SerialAndBatchBundle {
            gtin,
            batch_no,
            serial_no,
            expiry_date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_routing_prefixes() {
        assert_eq!(
            IntentRouter::parse_intent("ITEM:SKU-9900"),
            ScanIntent::ItemSetup {
                item_code: "SKU-9900".into()
            }
        );
        assert_eq!(
            IntentRouter::parse_intent("LOC:Stores - R:BIN-A01"),
            ScanIntent::WarehouseBin {
                warehouse: "Stores - R".into(),
                bin_code: "BIN-A01".into()
            }
        );
        assert_eq!(
            IntentRouter::parse_intent("JOB:JC-2026-8888"),
            ScanIntent::JobCard {
                job_card_id: "JC-2026-8888".into()
            }
        );
        assert_eq!(
            IntentRouter::parse_intent("ASSET:AST-00441"),
            ScanIntent::FixedAsset {
                asset_id: "AST-00441".into()
            }
        );
        assert_eq!(
            IntentRouter::parse_intent("BADGE:EMP-770:SECRET_TOKEN_XYZ"),
            ScanIntent::WorkerBadge {
                worker_uuid: "EMP-770".into(),
                auth_token: "SECRET_TOKEN_XYZ".into()
            }
        );
    }

    #[test]
    fn test_gs1_sabb_parsing() {
        let gs1_payload = "GS1:(01)05012345678900(10)BATCH-2026-X(21)SER-00049(17)261231";
        let intent = IntentRouter::parse_intent(gs1_payload);
        assert_eq!(
            intent,
            ScanIntent::SerialAndBatchBundle {
                gtin: "05012345678900".into(),
                batch_no: Some("BATCH-2026-X".into()),
                serial_no: Some("SER-00049".into()),
                expiry_date: Some("261231".into())
            }
        );
    }
}
