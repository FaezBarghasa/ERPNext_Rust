//! Comprehensive Regional Tax & Sequential Audit Compliance Engine (`erp-accounting::regional_tax`).
//!
//! Provides enterprise cross-border taxation, VAT, Reverse Charge (EU B2B),
//! compound sales tax rules, and strict zero-gap sequential invoice numbering audits.

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Regional tax treatment categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxTreatment {
    /// Standard VAT or Sales Tax rate applied.
    Standard,
    /// Reduced VAT rate (e.g., food, books, medical essentials).
    Reduced,
    /// 0% VAT rate (exports, specific qualifying goods).
    ZeroRated,
    /// Tax exempt (financial services, healthcare).
    Exempt,
    /// Reverse charge for B2B cross-border transactions (buyer self-assesses VAT).
    ReverseCharge,
}

/// Regional jurisdiction classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxJurisdiction {
    EuropeanUnion { country_code: CompactString },
    UnitedKingdom,
    UnitedStates { state_code: CompactString },
    Canada { province_code: CompactString },
    Custom { name: CompactString },
}

/// Strongly typed errors for regional tax and invoice auditing.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegionalTaxError {
    #[error("Invalid line amount: {0}")]
    InvalidAmount(Decimal),
    #[error("Non-sequential invoice gap detected between {expected} and {actual}")]
    InvoiceSequenceGap { expected: u64, actual: u64 },
    #[error("Duplicate invoice number detected: {0}")]
    DuplicateInvoiceNumber(CompactString),
    #[error(
        "Unordered invoice timestamps: invoice {prev_num} at {prev_time} after {curr_num} at {curr_time}"
    )]
    UnorderedTimestamp {
        prev_num: CompactString,
        prev_time: DateTime<Utc>,
        curr_num: CompactString,
        curr_time: DateTime<Utc>,
    },
    #[error("Malformed invoice series format: {0}")]
    MalformedInvoiceSeries(CompactString),
}

/// Line item for regional tax assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxableLineItem {
    pub item_code: CompactString,
    pub description: CompactString,
    pub net_amount: Decimal,
    pub is_service: bool,
}

/// Party tax identity in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxParty {
    pub name: CompactString,
    pub country_code: CompactString,
    pub tax_id: Option<CompactString>,
    pub is_business: bool,
}

/// Evaluated tax breakdown line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionalTaxLine {
    pub tax_name: CompactString,
    pub treatment: TaxTreatment,
    pub rate: Decimal,
    pub taxable_basis: Decimal,
    pub tax_amount: Decimal,
    pub is_reverse_charge: bool,
}

/// Complete regional tax assessment result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionalTaxAssessment {
    pub net_subtotal: Decimal,
    pub total_tax_amount: Decimal,
    pub gross_total: Decimal,
    pub tax_lines: Vec<RegionalTaxLine>,
    pub compliance_notice: Option<CompactString>,
}

/// Enterprise regional tax determination engine.
pub struct RegionalTaxEngine;

impl RegionalTaxEngine {
    /// Evaluates European Union VAT rules:
    /// - Domestic (same country): standard rate (or reduced) applies.
    /// - Intra-EU B2B with valid buyer VAT ID: Reverse Charge (0% applied, buyer self-assesses).
    /// - Intra-EU B2C (no VAT ID): Seller domestic rate applies (under OSS threshold).
    /// - Export outside EU: 0% Zero-Rated export.
    pub fn evaluate_eu_vat(
        seller: &TaxParty,
        buyer: &TaxParty,
        lines: &[TaxableLineItem],
        standard_rate: Decimal,
        is_intra_eu: bool,
    ) -> Result<RegionalTaxAssessment, RegionalTaxError> {
        let mut net_subtotal = Decimal::ZERO;
        for line in lines {
            if line.net_amount < Decimal::ZERO {
                return Err(RegionalTaxError::InvalidAmount(line.net_amount));
            }
            net_subtotal += line.net_amount;
        }

        let is_domestic = seller
            .country_code
            .eq_ignore_ascii_case(&buyer.country_code);

        if is_domestic {
            // Domestic transaction: apply standard rate
            let tax_amount = (net_subtotal * standard_rate / dec!(100)).round_dp(2);
            let tax_line = RegionalTaxLine {
                tax_name: format!("{} VAT ({}%)", seller.country_code, standard_rate).into(),
                treatment: TaxTreatment::Standard,
                rate: standard_rate,
                taxable_basis: net_subtotal,
                tax_amount,
                is_reverse_charge: false,
            };

            Ok(RegionalTaxAssessment {
                net_subtotal,
                total_tax_amount: tax_amount,
                gross_total: net_subtotal + tax_amount,
                tax_lines: vec![tax_line],
                compliance_notice: None,
            })
        } else if is_intra_eu && buyer.is_business && buyer.tax_id.is_some() {
            // Intra-EU B2B: Reverse Charge applies
            let tax_line = RegionalTaxLine {
                tax_name: "Intra-EU B2B Supply (Article 138 EU VAT Directive)".into(),
                treatment: TaxTreatment::ReverseCharge,
                rate: Decimal::ZERO,
                taxable_basis: net_subtotal,
                tax_amount: Decimal::ZERO,
                is_reverse_charge: true,
            };

            Ok(RegionalTaxAssessment {
                net_subtotal,
                total_tax_amount: Decimal::ZERO,
                gross_total: net_subtotal,
                tax_lines: vec![tax_line],
                compliance_notice: Some(
                    "Reverse charge: VAT to be accounted for by the recipient as per Article 196 EU VAT Directive."
                        .into(),
                ),
            })
        } else if !is_intra_eu {
            // Extra-EU Export: 0% Zero-Rated
            let tax_line = RegionalTaxLine {
                tax_name: "Export outside EU (Zero-Rated)".into(),
                treatment: TaxTreatment::ZeroRated,
                rate: Decimal::ZERO,
                taxable_basis: net_subtotal,
                tax_amount: Decimal::ZERO,
                is_reverse_charge: false,
            };

            Ok(RegionalTaxAssessment {
                net_subtotal,
                total_tax_amount: Decimal::ZERO,
                gross_total: net_subtotal,
                tax_lines: vec![tax_line],
                compliance_notice: Some("Zero-rated export outside EU jurisdiction.".into()),
            })
        } else {
            // Intra-EU B2C: Apply domestic rate
            let tax_amount = (net_subtotal * standard_rate / dec!(100)).round_dp(2);
            let tax_line = RegionalTaxLine {
                tax_name: format!("EU B2C VAT ({}%)", standard_rate).into(),
                treatment: TaxTreatment::Standard,
                rate: standard_rate,
                taxable_basis: net_subtotal,
                tax_amount,
                is_reverse_charge: false,
            };

            Ok(RegionalTaxAssessment {
                net_subtotal,
                total_tax_amount: tax_amount,
                gross_total: net_subtotal + tax_amount,
                tax_lines: vec![tax_line],
                compliance_notice: None,
            })
        }
    }

    /// Evaluates compound Canadian taxes (e.g. GST 5% + QST 9.975% compound or Provincial PST).
    pub fn evaluate_canadian_compound_tax(
        net_subtotal: Decimal,
        gst_rate: Decimal,
        provincial_rate: Decimal,
        compound_provincial: bool,
    ) -> Result<RegionalTaxAssessment, RegionalTaxError> {
        if net_subtotal < Decimal::ZERO {
            return Err(RegionalTaxError::InvalidAmount(net_subtotal));
        }

        let gst_amount = (net_subtotal * gst_rate / dec!(100)).round_dp(2);
        let pst_basis = if compound_provincial {
            net_subtotal + gst_amount
        } else {
            net_subtotal
        };
        let pst_amount = (pst_basis * provincial_rate / dec!(100)).round_dp(2);

        let lines = vec![
            RegionalTaxLine {
                tax_name: "GST (Federal)".into(),
                treatment: TaxTreatment::Standard,
                rate: gst_rate,
                taxable_basis: net_subtotal,
                tax_amount: gst_amount,
                is_reverse_charge: false,
            },
            RegionalTaxLine {
                tax_name: "PST / QST (Provincial)".into(),
                treatment: TaxTreatment::Standard,
                rate: provincial_rate,
                taxable_basis: pst_basis,
                tax_amount: pst_amount,
                is_reverse_charge: false,
            },
        ];

        let total_tax = gst_amount + pst_amount;
        Ok(RegionalTaxAssessment {
            net_subtotal,
            total_tax_amount: total_tax,
            gross_total: net_subtotal + total_tax,
            tax_lines: lines,
            compliance_notice: None,
        })
    }
}

/// Metadata record for sequential invoice auditing.
#[derive(Debug, Clone)]
pub struct InvoiceSequenceRecord {
    pub invoice_number: CompactString,
    pub issued_at: DateTime<Utc>,
    pub gross_amount: Decimal,
}

/// Result of a tax compliance sequential audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditReport {
    pub is_compliant: bool,
    pub total_audited: usize,
    pub detected_gaps: Vec<(u64, u64)>,
    pub detected_duplicates: Vec<CompactString>,
    pub timestamp_anomalies: usize,
}

/// Sequential non-gap invoice numbering auditor (GoBD, Italian SDI, French FEC compliance).
pub struct InvoiceSequenceAuditor;

impl InvoiceSequenceAuditor {
    /// Extracts the integer suffix from a standardized invoice format (e.g. `INV-2026-00042` -> 42).
    pub fn parse_sequence_number(invoice_number: &str) -> Result<u64, RegionalTaxError> {
        let last_segment = invoice_number
            .rsplit('-')
            .next()
            .ok_or_else(|| RegionalTaxError::MalformedInvoiceSeries(invoice_number.into()))?;

        last_segment
            .parse::<u64>()
            .map_err(|_| RegionalTaxError::MalformedInvoiceSeries(invoice_number.into()))
    }

    /// Audits an ordered list of invoice records to verify:
    /// 1. Strict monotonically increasing sequence numbers with zero missing gaps.
    /// 2. Zero duplicate invoice numbers.
    /// 3. Monotonically non-decreasing timestamps (an invoice with higher seq cannot have an earlier timestamp).
    pub fn audit_invoice_series(
        invoices: &[InvoiceSequenceRecord],
    ) -> Result<AuditReport, RegionalTaxError> {
        if invoices.is_empty() {
            return Ok(AuditReport {
                is_compliant: true,
                total_audited: 0,
                detected_gaps: Vec::new(),
                detected_duplicates: Vec::new(),
                timestamp_anomalies: 0,
            });
        }

        let mut detected_gaps = Vec::new();
        let mut detected_duplicates = Vec::new();
        let mut timestamp_anomalies = 0;

        let mut seen_numbers = std::collections::HashSet::new();

        let first_num = Self::parse_sequence_number(&invoices[0].invoice_number)?;
        seen_numbers.insert(invoices[0].invoice_number.clone());

        let mut expected_seq = first_num + 1;
        let mut prev_time = invoices[0].issued_at;
        let mut prev_num = invoices[0].invoice_number.clone();

        for inv in invoices.iter().skip(1) {
            if seen_numbers.contains(&inv.invoice_number) {
                detected_duplicates.push(inv.invoice_number.clone());
            } else {
                seen_numbers.insert(inv.invoice_number.clone());
            }

            let curr_seq = Self::parse_sequence_number(&inv.invoice_number)?;

            if curr_seq > expected_seq {
                detected_gaps.push((expected_seq, curr_seq - 1));
                expected_seq = curr_seq + 1;
            } else if curr_seq == expected_seq {
                expected_seq += 1;
            }

            if inv.issued_at < prev_time {
                timestamp_anomalies += 1;
            }

            prev_time = inv.issued_at;
            prev_num = inv.invoice_number.clone();
        }

        let _ = prev_num; // silence unused warning

        let is_compliant =
            detected_gaps.is_empty() && detected_duplicates.is_empty() && timestamp_anomalies == 0;

        Ok(AuditReport {
            is_compliant,
            total_audited: invoices.len(),
            detected_gaps,
            detected_duplicates,
            timestamp_anomalies,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rust_decimal_macros::dec;

    #[test]
    fn test_eu_vat_domestic_vs_intra_eu_b2b_reverse_charge() {
        let seller = TaxParty {
            name: "Berlin Supply GmbH".into(),
            country_code: "DE".into(),
            tax_id: Some("DE123456789".into()),
            is_business: true,
        };

        let domestic_buyer = TaxParty {
            name: "Munich Retail UG".into(),
            country_code: "DE".into(),
            tax_id: Some("DE987654321".into()),
            is_business: true,
        };

        let french_b2b_buyer = TaxParty {
            name: "Paris Tech SAS".into(),
            country_code: "FR".into(),
            tax_id: Some("FR99887766554".into()),
            is_business: true,
        };

        let lines = vec![TaxableLineItem {
            item_code: "SRV-CLOUD-01".into(),
            description: "Dedicated Cloud Server Node".into(),
            net_amount: dec!(1000.00),
            is_service: true,
        }];

        // 1. Domestic Germany -> 19% Standard MwSt
        let de_res =
            RegionalTaxEngine::evaluate_eu_vat(&seller, &domestic_buyer, &lines, dec!(19.0), true)
                .unwrap();
        assert_eq!(de_res.net_subtotal, dec!(1000.00));
        assert_eq!(de_res.total_tax_amount, dec!(190.00));
        assert_eq!(de_res.gross_total, dec!(1190.00));
        assert!(!de_res.tax_lines[0].is_reverse_charge);
        assert!(de_res.compliance_notice.is_none());

        // 2. Intra-EU B2B Germany -> France -> Reverse Charge
        let fr_res = RegionalTaxEngine::evaluate_eu_vat(
            &seller,
            &french_b2b_buyer,
            &lines,
            dec!(19.0),
            true,
        )
        .unwrap();
        assert_eq!(fr_res.net_subtotal, dec!(1000.00));
        assert_eq!(fr_res.total_tax_amount, dec!(0.00));
        assert_eq!(fr_res.gross_total, dec!(1000.00));
        assert!(fr_res.tax_lines[0].is_reverse_charge);
        assert!(fr_res.compliance_notice.unwrap().contains("Reverse charge"));
    }

    #[test]
    fn test_canadian_compound_tax() {
        let net = dec!(500.00);
        let res =
            RegionalTaxEngine::evaluate_canadian_compound_tax(net, dec!(5.0), dec!(9.975), true)
                .unwrap();

        assert_eq!(res.net_subtotal, dec!(500.00));
        assert_eq!(res.tax_lines.len(), 2);
        assert_eq!(res.tax_lines[0].tax_amount, dec!(25.00)); // GST: 5% of 500 = 25
        // Compound basis: 500 + 25 = 525. QST 9.975% of 525 = 52.36875 -> 52.37
        assert_eq!(res.tax_lines[1].taxable_basis, dec!(525.00));
        assert_eq!(res.tax_lines[1].tax_amount, dec!(52.37));
        assert_eq!(res.total_tax_amount, dec!(77.37));
        assert_eq!(res.gross_total, dec!(577.37));
    }

    #[test]
    fn test_invoice_sequence_audit_clean_and_gap_detection() {
        let t0 = Utc.with_ymd_and_hms(2026, 10, 1, 10, 0, 0).unwrap();
        let t1 = Utc.with_ymd_and_hms(2026, 10, 1, 11, 0, 0).unwrap();
        let t2 = Utc.with_ymd_and_hms(2026, 10, 1, 12, 0, 0).unwrap();

        let clean_invoices = vec![
            InvoiceSequenceRecord {
                invoice_number: "SINV-2026-00001".into(),
                issued_at: t0,
                gross_amount: dec!(100.00),
            },
            InvoiceSequenceRecord {
                invoice_number: "SINV-2026-00002".into(),
                issued_at: t1,
                gross_amount: dec!(200.00),
            },
            InvoiceSequenceRecord {
                invoice_number: "SINV-2026-00003".into(),
                issued_at: t2,
                gross_amount: dec!(300.00),
            },
        ];

        let clean_report = InvoiceSequenceAuditor::audit_invoice_series(&clean_invoices).unwrap();
        assert!(clean_report.is_compliant);
        assert_eq!(clean_report.total_audited, 3);
        assert!(clean_report.detected_gaps.is_empty());

        // Introduce a gap (00001 -> 00003, missing 00002)
        let gapped_invoices = vec![
            InvoiceSequenceRecord {
                invoice_number: "SINV-2026-00001".into(),
                issued_at: t0,
                gross_amount: dec!(100.00),
            },
            InvoiceSequenceRecord {
                invoice_number: "SINV-2026-00003".into(),
                issued_at: t2,
                gross_amount: dec!(300.00),
            },
        ];

        let gap_report = InvoiceSequenceAuditor::audit_invoice_series(&gapped_invoices).unwrap();
        assert!(!gap_report.is_compliant);
        assert_eq!(gap_report.detected_gaps, vec![(2, 2)]);
    }
}
