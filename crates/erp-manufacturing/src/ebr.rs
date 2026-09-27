//! 21 CFR Part 11 Electronic Batch Records (EBR) & Dual-Witness Cryptographic Signatures.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WitnessSignature {
    pub witness_user_id: String,
    pub full_name: String,
    pub meaning_of_signature: String, // "Formulation Confirmed", "Quality Release", "Sterilization Validated"
    pub signed_at: chrono::DateTime<chrono::Utc>,
    pub cryptographic_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchStepRecord {
    pub step_number: u32,
    pub operation_name: String,
    pub parameter_values: serde_json::Value,
    pub primary_operator_signature: WitnessSignature,
    pub second_witness_signature: Option<WitnessSignature>,
    pub is_verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElectronicBatchRecord {
    pub batch_number: String,
    pub product_code: String,
    pub manufacturing_date: chrono::NaiveDate,
    pub expiry_date: chrono::NaiveDate,
    pub steps: Vec<BatchStepRecord>,
    pub is_released_for_distribution: bool,
}

impl ElectronicBatchRecord {
    #[must_use]
    pub fn new(batch_no: String, product: String, mfg_date: chrono::NaiveDate, exp_date: chrono::NaiveDate) -> Self {
        Self {
            batch_number: batch_no,
            product_code: product,
            manufacturing_date: mfg_date,
            expiry_date: exp_date,
            steps: Vec::new(),
            is_released_for_distribution: false,
        }
    }

    pub fn record_signed_step(
        &mut self,
        step_no: u32,
        op_name: String,
        params: serde_json::Value,
        operator_id: &str,
        operator_name: &str,
        operator_secret: &str,
        second_witness: Option<(&str, &str, &str)>, // (id, name, secret)
    ) {
        let op_sig = Self::sign_record(operator_id, operator_name, "Performed Step", &params, operator_secret);
        let wit_sig = second_witness.map(|(w_id, w_name, w_sec)| {
            Self::sign_record(w_id, w_name, "Verified Dual Witness", &params, w_sec)
        });

        let is_verified = wit_sig.is_some();
        self.steps.push(BatchStepRecord {
            step_number: step_no,
            operation_name: op_name,
            parameter_values: params,
            primary_operator_signature: op_sig,
            second_witness_signature: wit_sig,
            is_verified,
        });
    }

    fn sign_record(user_id: &str, name: &str, meaning: &str, data: &serde_json::Value, secret: &str) -> WitnessSignature {
        let now = chrono::Utc::now();
        let mut hasher = Sha256::new();
        hasher.update(user_id.as_bytes());
        hasher.update(meaning.as_bytes());
        hasher.update(data.to_string().as_bytes());
        hasher.update(now.to_rfc3339().as_bytes());
        hasher.update(secret.as_bytes());
        let hash: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();

        WitnessSignature {
            witness_user_id: user_id.to_string(),
            full_name: name.to_string(),
            meaning_of_signature: meaning.to_string(),
            signed_at: now,
            cryptographic_hash: hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebr_dual_witness_signing() {
        let mut ebr = ElectronicBatchRecord::new(
            "BATCH-2026-99A".into(),
            "VACCINE-LOT-01".into(),
            chrono::NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2028, 10, 1).unwrap(),
        );

        ebr.record_signed_step(
            1,
            "Sterile Active Ingredient Blending".into(),
            serde_json::json!({ "rpm": 1500, "temp_c": 4.2, "duration_mins": 60 }),
            "operator_alice",
            "Alice Smith",
            "alice_key_token",
            Some(("witness_bob", "Bob Jones", "bob_key_token")),
        );

        assert_eq!(ebr.steps.len(), 1);
        assert!(ebr.steps[0].is_verified);
        assert!(ebr.steps[0].second_witness_signature.is_some());
    }
}
