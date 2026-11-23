//! Automated Side-by-Side Reconciliation Auditor (`erp-importer::reconciliation_auditor`).
//!
//! Validates:
//! 1. General Ledger Reconciliation: sum(Dr_MariaDB) == sum(Dr_SurrealDB) and sum(Cr_MariaDB) == sum(Cr_SurrealDB)
//! 2. Stock Ledger & Valuation Reconciliation: sum(Qty_MariaDB) == sum(Qty_SurrealDB) and sum(Val_MariaDB) == sum(Val_SurrealDB)
//! 3. Trial Balance Account-by-Account Balance Match ($0.00 exact decimal precision)
//! 4. Open Document Counts & SHA256 Hash Verification across standard DocTypes.

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Discrepancy observed between MariaDB source and SurrealDB target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditDiscrepancy {
    pub audit_domain: CompactString, // "General_Ledger", "Stock_Ledger", "Trial_Balance", "DocType_Parity"
    pub identifier: CompactString,
    pub source_metric: CompactString,
    pub target_metric: CompactString,
    pub delta: CompactString,
}

/// Complete Reconciliation Audit Suite Report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComprehensiveAuditReport {
    pub is_passed: bool,
    pub gl_dr_source: Decimal,
    pub gl_dr_target: Decimal,
    pub gl_cr_source: Decimal,
    pub gl_cr_target: Decimal,
    pub stock_qty_source: Decimal,
    pub stock_qty_target: Decimal,
    pub stock_val_source: Decimal,
    pub stock_val_target: Decimal,
    pub trial_balance_accounts_verified: usize,
    pub doctype_counts_source: HashMap<String, usize>,
    pub doctype_counts_target: HashMap<String, usize>,
    pub doctype_hashes_match: HashMap<String, bool>,
    pub discrepancies: Vec<AuditDiscrepancy>,
}

/// Automated Reconciliation Auditor Engine.
pub struct ReconciliationAuditor;

impl ReconciliationAuditor {
    /// Helper to extract exact `rust_decimal::Decimal` from various JSON formats.
    pub fn parse_decimal(val: Option<&Value>) -> Decimal {
        match val {
            Some(Value::String(s)) => s.parse::<Decimal>().unwrap_or(Decimal::ZERO),
            Some(Value::Number(n)) => {
                if let Some(i) = n.as_i64() {
                    Decimal::from(i)
                } else if let Some(f) = n.as_f64() {
                    Decimal::try_from(f).unwrap_or(Decimal::ZERO)
                } else {
                    Decimal::ZERO
                }
            }
            _ => Decimal::ZERO,
        }
    }

    /// Canonicalizes a JSON Value so that numbers/decimals with different string/numeric representations compare identically.
    pub fn canonicalize_value(val: &Value) -> Value {
        match val {
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::String(Decimal::from(i).to_string())
                } else if let Some(f) = n.as_f64() {
                    if let Ok(d) = Decimal::try_from(f) {
                        Value::String(d.normalize().to_string())
                    } else {
                        Value::Number(n.clone())
                    }
                } else {
                    Value::Number(n.clone())
                }
            }
            Value::String(s) => {
                if let Ok(d) = s.parse::<Decimal>() {
                    Value::String(d.normalize().to_string())
                } else {
                    Value::String(s.clone())
                }
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(Self::canonicalize_value).collect())
            }
            Value::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (k, v) in obj {
                    map.insert(k.clone(), Self::canonicalize_value(v));
                }
                Value::Object(map)
            }
            _ => val.clone(),
        }
    }

    /// Computes deterministic SHA256 checksum for a list of document values.
    pub fn compute_documents_sha256(docs: &[Value]) -> String {
        let mut hasher = Sha256::new();
        // Canonicalize documents by stringifying sorted keys
        for doc in docs {
            let canon = Self::canonicalize_value(doc);
            if let Ok(canon_bytes) = serde_json::to_vec(&canon) {
                hasher.update(&canon_bytes);
            }
        }
        hex::encode(hasher.finalize())
    }

    /// Executes full side-by-side reconciliation between source MariaDB datasets and imported SurrealDB datasets.
    pub fn execute_audit(
        source_records_by_doctype: &HashMap<String, Vec<Value>>,
        target_records_by_doctype: &HashMap<String, Vec<Value>>,
    ) -> ComprehensiveAuditReport {
        let mut discrepancies = Vec::new();

        // 1. General Ledger Reconciliation
        let (mut gl_dr_src, mut gl_cr_src) = (Decimal::ZERO, Decimal::ZERO);
        let (mut gl_dr_tgt, mut gl_cr_tgt) = (Decimal::ZERO, Decimal::ZERO);

        let mut tb_src_map: HashMap<String, Decimal> = HashMap::new();
        let mut tb_tgt_map: HashMap<String, Decimal> = HashMap::new();

        if let Some(gl_entries) = source_records_by_doctype.get("GL Entry") {
            for row in gl_entries {
                let dr = Self::parse_decimal(row.get("debit"));
                let cr = Self::parse_decimal(row.get("credit"));
                let acc = row.get("account").and_then(|v| v.as_str()).unwrap_or("");
                gl_dr_src += dr;
                gl_cr_src += cr;
                *tb_src_map.entry(acc.to_string()).or_default() += dr - cr;
            }
        }

        if let Some(gl_entries) = target_records_by_doctype.get("GL Entry") {
            for row in gl_entries {
                let dr = Self::parse_decimal(row.get("debit"));
                let cr = Self::parse_decimal(row.get("credit"));
                let acc = row.get("account").and_then(|v| v.as_str()).unwrap_or("");
                gl_dr_tgt += dr;
                gl_cr_tgt += cr;
                *tb_tgt_map.entry(acc.to_string()).or_default() += dr - cr;
            }
        }

        if gl_dr_src != gl_dr_tgt {
            discrepancies.push(AuditDiscrepancy {
                audit_domain: "General_Ledger".into(),
                identifier: "Total_Debit".into(),
                source_metric: gl_dr_src.to_string().into(),
                target_metric: gl_dr_tgt.to_string().into(),
                delta: (gl_dr_tgt - gl_dr_src).to_string().into(),
            });
        }

        if gl_cr_src != gl_cr_tgt {
            discrepancies.push(AuditDiscrepancy {
                audit_domain: "General_Ledger".into(),
                identifier: "Total_Credit".into(),
                source_metric: gl_cr_src.to_string().into(),
                target_metric: gl_cr_tgt.to_string().into(),
                delta: (gl_cr_tgt - gl_cr_src).to_string().into(),
            });
        }

        // 2. Stock Ledger & Valuation Reconciliation
        let (mut stock_qty_src, mut stock_val_src) = (Decimal::ZERO, Decimal::ZERO);
        let (mut stock_qty_tgt, mut stock_val_tgt) = (Decimal::ZERO, Decimal::ZERO);

        if let Some(sle_entries) = source_records_by_doctype.get("Stock Ledger Entry") {
            for row in sle_entries {
                stock_qty_src += Self::parse_decimal(row.get("actual_qty"));
                stock_val_src += Self::parse_decimal(row.get("stock_value_difference"));
            }
        }

        if let Some(sle_entries) = target_records_by_doctype.get("Stock Ledger Entry") {
            for row in sle_entries {
                stock_qty_tgt += Self::parse_decimal(row.get("actual_qty"));
                stock_val_tgt += Self::parse_decimal(row.get("stock_value_difference"));
            }
        }

        if stock_qty_src != stock_qty_tgt {
            discrepancies.push(AuditDiscrepancy {
                audit_domain: "Stock_Ledger".into(),
                identifier: "Actual_Qty".into(),
                source_metric: stock_qty_src.to_string().into(),
                target_metric: stock_qty_tgt.to_string().into(),
                delta: (stock_qty_tgt - stock_qty_src).to_string().into(),
            });
        }

        if stock_val_src != stock_val_tgt {
            discrepancies.push(AuditDiscrepancy {
                audit_domain: "Stock_Ledger".into(),
                identifier: "Stock_Value_Difference".into(),
                source_metric: stock_val_src.to_string().into(),
                target_metric: stock_val_tgt.to_string().into(),
                delta: (stock_val_tgt - stock_val_src).to_string().into(),
            });
        }

        // 3. Trial Balance Account-by-Account Balance Match
        let mut all_accounts: Vec<String> = tb_src_map
            .keys()
            .chain(tb_tgt_map.keys())
            .cloned()
            .collect();
        all_accounts.sort();
        all_accounts.dedup();

        let mut trial_balance_accounts_verified = 0;
        for acc in &all_accounts {
            if acc.is_empty() {
                continue;
            }
            let src_bal = tb_src_map.get(acc).copied().unwrap_or(Decimal::ZERO);
            let tgt_bal = tb_tgt_map.get(acc).copied().unwrap_or(Decimal::ZERO);

            if src_bal != tgt_bal {
                discrepancies.push(AuditDiscrepancy {
                    audit_domain: "Trial_Balance".into(),
                    identifier: acc.clone().into(),
                    source_metric: src_bal.to_string().into(),
                    target_metric: tgt_bal.to_string().into(),
                    delta: (tgt_bal - src_bal).to_string().into(),
                });
            } else {
                trial_balance_accounts_verified += 1;
            }
        }

        // 4. Open Document Counts & SHA256 Hash Verification
        let mut doctype_counts_source = HashMap::new();
        let mut doctype_counts_target = HashMap::new();
        let mut doctype_hashes_match = HashMap::new();

        let mut all_doctypes: Vec<String> = source_records_by_doctype
            .keys()
            .chain(target_records_by_doctype.keys())
            .cloned()
            .collect();
        all_doctypes.sort();
        all_doctypes.dedup();

        for dt in all_doctypes {
            let src_docs = source_records_by_doctype.get(&dt).cloned().unwrap_or_default();
            let tgt_docs = target_records_by_doctype.get(&dt).cloned().unwrap_or_default();

            doctype_counts_source.insert(dt.clone(), src_docs.len());
            doctype_counts_target.insert(dt.clone(), tgt_docs.len());

            if src_docs.len() != tgt_docs.len() {
                discrepancies.push(AuditDiscrepancy {
                    audit_domain: "DocType_Parity".into(),
                    identifier: format!("Count_{dt}").into(),
                    source_metric: src_docs.len().to_string().into(),
                    target_metric: tgt_docs.len().to_string().into(),
                    delta: (tgt_docs.len() as i64 - src_docs.len() as i64).to_string().into(),
                });
            }

            let hash_src = Self::compute_documents_sha256(&src_docs);
            let hash_tgt = Self::compute_documents_sha256(&tgt_docs);
            let matched = hash_src == hash_tgt;
            doctype_hashes_match.insert(dt.clone(), matched);

            if !matched && src_docs.len() == tgt_docs.len() && !src_docs.is_empty() {
                discrepancies.push(AuditDiscrepancy {
                    audit_domain: "DocType_Parity".into(),
                    identifier: format!("Checksum_{dt}").into(),
                    source_metric: hash_src.into(),
                    target_metric: hash_tgt.into(),
                    delta: "HashMismatch".into(),
                });
            }
        }

        let is_passed = discrepancies.is_empty();

        ComprehensiveAuditReport {
            is_passed,
            gl_dr_source: gl_dr_src,
            gl_dr_target: gl_dr_tgt,
            gl_cr_source: gl_cr_src,
            gl_cr_target: gl_cr_tgt,
            stock_qty_source: stock_qty_src,
            stock_qty_target: stock_qty_tgt,
            stock_val_source: stock_val_src,
            stock_val_target: stock_val_tgt,
            trial_balance_accounts_verified,
            doctype_counts_source,
            doctype_counts_target,
            doctype_hashes_match,
            discrepancies,
        }
    }
}
