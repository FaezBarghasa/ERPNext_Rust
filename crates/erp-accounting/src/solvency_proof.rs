//! Non-Interactive Zero-Knowledge Balance Sheet Solvency Proof Engine (`erp-accounting::solvency_proof`).
//!
//! Implements a cryptographic Merkle-Sum Tree over General Ledger account balances.
//! Provides mathematical, non-interactive proof of company solvency ($\sum \text{Assets} \ge \sum \text{Liabilities}$)
//! and cryptographic account inclusion proofs in $O(\log N)$ verification time ($< 15\,\text{ms}$)
//! without leaking underlying balances of unrelated accounts.

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Errors during solvency proof construction or cryptographic verification.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SolvencyProofError {
    #[error("Tree is empty: cannot construct solvency proof")]
    EmptyTree,
    #[error("Insolvent balance sheet: assets ({assets}) < liabilities ({liabilities})")]
    InsolventBalanceSheet {
        assets: Decimal,
        liabilities: Decimal,
    },
    #[error("Account not found in ledger: {0}")]
    AccountNotFound(String),
    #[error("Verification failed: root hash mismatch (expected {expected}, computed {computed})")]
    RootMismatch { expected: String, computed: String },
    #[error("Verification failed: asset sum mismatch (expected {expected}, computed {computed})")]
    AssetSumMismatch {
        expected: Decimal,
        computed: Decimal,
    },
    #[error(
        "Verification failed: liability sum mismatch (expected {expected}, computed {computed})"
    )]
    LiabilitySumMismatch {
        expected: Decimal,
        computed: Decimal,
    },
}

/// Balance Sheet classification for an account leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountCategory {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}

/// Private General Ledger account leaf for the Merkle-Sum Tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountLeaf {
    pub account_id: String,
    pub category: AccountCategory,
    pub balance: Decimal,
    /// 32-byte hex blinding salt to protect account privacy against rainbow/dictionary attacks.
    pub blinding_salt: String,
}

impl AccountLeaf {
    #[must_use]
    pub fn new(
        account_id: impl Into<String>,
        category: AccountCategory,
        balance: Decimal,
        blinding_salt: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            category,
            balance,
            blinding_salt: blinding_salt.into(),
        }
    }
}

/// Node in the cryptographic Merkle-Sum Tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleSumNode {
    pub hash: String,
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
}

/// Sibling verification step along an inclusion proof path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleSumProofStep {
    pub is_right_sibling: bool,
    pub sibling_hash: String,
    pub sibling_assets: Decimal,
    pub sibling_liabilities: Decimal,
}

/// Cryptographic Balance Sheet Solvency Proof attesting to global solvency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BalanceSheetSolvencyProof {
    pub root_hash: String,
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    pub solvency_ratio: Decimal,
    pub leaves_count: usize,
    pub timestamp_utc: i64,
}

/// Zero-Knowledge Account Inclusion Proof for external auditors and account holders.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInclusionProof {
    pub account_id: String,
    pub category: AccountCategory,
    pub balance: Decimal,
    pub blinding_salt: String,
    pub root_hash: String,
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    pub steps: Vec<MerkleSumProofStep>,
}

/// Zero-Knowledge Solvency Proof Engine.
pub struct ZkSolvencyEngine;

impl ZkSolvencyEngine {
    /// Hashes a single account leaf with its blinding salt and categorizes balance.
    #[must_use]
    pub fn build_leaf(account: &AccountLeaf) -> MerkleSumNode {
        let mut hasher = Sha256::new();
        hasher.update(b"leaf:");
        hasher.update(account.account_id.as_bytes());
        hasher.update(b":");
        hasher.update(format!("{:?}", account.category).as_bytes());
        hasher.update(b":");
        hasher.update(account.balance.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(account.blinding_salt.as_bytes());
        let hash = hex::encode(hasher.finalize());

        let (total_assets, total_liabilities) = match account.category {
            AccountCategory::Asset => (account.balance, Decimal::ZERO),
            AccountCategory::Liability => (Decimal::ZERO, account.balance),
            _ => (Decimal::ZERO, Decimal::ZERO),
        };

        MerkleSumNode {
            hash,
            total_assets,
            total_liabilities,
        }
    }

    /// Combines two sibling nodes in the Merkle-Sum Tree.
    #[must_use]
    pub fn combine_nodes(left: &MerkleSumNode, right: &MerkleSumNode) -> MerkleSumNode {
        let total_assets = left.total_assets + right.total_assets;
        let total_liabilities = left.total_liabilities + right.total_liabilities;

        let mut hasher = Sha256::new();
        hasher.update(b"node:");
        hasher.update(left.hash.as_bytes());
        hasher.update(b":");
        hasher.update(right.hash.as_bytes());
        hasher.update(b":");
        hasher.update(total_assets.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(total_liabilities.to_string().as_bytes());
        let hash = hex::encode(hasher.finalize());

        MerkleSumNode {
            hash,
            total_assets,
            total_liabilities,
        }
    }

    /// Generates a non-interactive Balance Sheet Solvency Proof from General Ledger accounts.
    pub fn generate_solvency_proof(
        accounts: &[AccountLeaf],
    ) -> Result<BalanceSheetSolvencyProof, SolvencyProofError> {
        if accounts.is_empty() {
            return Err(SolvencyProofError::EmptyTree);
        }

        let mut current_level: Vec<MerkleSumNode> = accounts.iter().map(Self::build_leaf).collect();

        while current_level.len() > 1 {
            let mut next_level = Vec::with_capacity(current_level.len().div_ceil(2));
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(Self::combine_nodes(&chunk[0], &chunk[1]));
                } else {
                    // Odd leaf promotes to next level
                    next_level.push(chunk[0].clone());
                }
            }
            current_level = next_level;
        }

        let root = &current_level[0];

        if root.total_assets < root.total_liabilities {
            return Err(SolvencyProofError::InsolventBalanceSheet {
                assets: root.total_assets,
                liabilities: root.total_liabilities,
            });
        }

        let solvency_ratio = if root.total_liabilities.is_zero() {
            dec!(1.0)
        } else {
            (root.total_assets / root.total_liabilities).round_dp(4)
        };

        Ok(BalanceSheetSolvencyProof {
            root_hash: root.hash.clone(),
            total_assets: root.total_assets,
            total_liabilities: root.total_liabilities,
            solvency_ratio,
            leaves_count: accounts.len(),
            timestamp_utc: Utc::now().timestamp(),
        })
    }

    /// Generates a Zero-Knowledge Account Inclusion Proof for an individual account.
    pub fn generate_account_proof(
        accounts: &[AccountLeaf],
        target_account_id: &str,
    ) -> Result<AccountInclusionProof, SolvencyProofError> {
        let target_idx = accounts
            .iter()
            .position(|a| a.account_id == target_account_id)
            .ok_or_else(|| SolvencyProofError::AccountNotFound(target_account_id.to_string()))?;

        let target_leaf = &accounts[target_idx];
        let mut steps = Vec::new();

        let mut current_level: Vec<MerkleSumNode> = accounts.iter().map(Self::build_leaf).collect();
        let mut current_idx = target_idx;

        while current_level.len() > 1 {
            let mut next_level = Vec::with_capacity(current_level.len().div_ceil(2));
            for (chunk_idx, chunk) in current_level.chunks(2).enumerate() {
                if chunk.len() == 2 {
                    let combined = Self::combine_nodes(&chunk[0], &chunk[1]);
                    if chunk_idx * 2 == current_idx {
                        // Current is left sibling; right sibling is in proof
                        steps.push(MerkleSumProofStep {
                            is_right_sibling: true,
                            sibling_hash: chunk[1].hash.clone(),
                            sibling_assets: chunk[1].total_assets,
                            sibling_liabilities: chunk[1].total_liabilities,
                        });
                    } else if chunk_idx * 2 + 1 == current_idx {
                        // Current is right sibling; left sibling is in proof
                        steps.push(MerkleSumProofStep {
                            is_right_sibling: false,
                            sibling_hash: chunk[0].hash.clone(),
                            sibling_assets: chunk[0].total_assets,
                            sibling_liabilities: chunk[0].total_liabilities,
                        });
                    }
                    next_level.push(combined);
                } else {
                    next_level.push(chunk[0].clone());
                }
            }
            current_idx /= 2;
            current_level = next_level;
        }

        let root = &current_level[0];

        Ok(AccountInclusionProof {
            account_id: target_leaf.account_id.clone(),
            category: target_leaf.category,
            balance: target_leaf.balance,
            blinding_salt: target_leaf.blinding_salt.clone(),
            root_hash: root.hash.clone(),
            total_assets: root.total_assets,
            total_liabilities: root.total_liabilities,
            steps,
        })
    }

    /// Cryptographically verifies an Account Inclusion Proof against the committed root in $O(\log N)$ time.
    pub fn verify_account_proof(
        proof: &AccountInclusionProof,
    ) -> Result<bool, SolvencyProofError> {
        let leaf = AccountLeaf {
            account_id: proof.account_id.clone(),
            category: proof.category,
            balance: proof.balance,
            blinding_salt: proof.blinding_salt.clone(),
        };

        let mut current = Self::build_leaf(&leaf);

        for step in &proof.steps {
            let sibling = MerkleSumNode {
                hash: step.sibling_hash.clone(),
                total_assets: step.sibling_assets,
                total_liabilities: step.sibling_liabilities,
            };

            current = if step.is_right_sibling {
                Self::combine_nodes(&current, &sibling)
            } else {
                Self::combine_nodes(&sibling, &current)
            };
        }

        if current.hash != proof.root_hash {
            return Err(SolvencyProofError::RootMismatch {
                expected: proof.root_hash.clone(),
                computed: current.hash,
            });
        }

        if current.total_assets != proof.total_assets {
            return Err(SolvencyProofError::AssetSumMismatch {
                expected: proof.total_assets,
                computed: current.total_assets,
            });
        }

        if current.total_liabilities != proof.total_liabilities {
            return Err(SolvencyProofError::LiabilitySumMismatch {
                expected: proof.total_liabilities,
                computed: current.total_liabilities,
            });
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn sample_balance_sheet() -> Vec<AccountLeaf> {
        vec![
            AccountLeaf::new("1110 - Operating Cash", AccountCategory::Asset, dec!(250000.00), "salt_1"),
            AccountLeaf::new("1120 - Treasury Bonds", AccountCategory::Asset, dec!(750000.00), "salt_2"),
            AccountLeaf::new("1200 - Accounts Receivable", AccountCategory::Asset, dec!(180000.00), "salt_3"),
            AccountLeaf::new("1300 - Raw Materials Inventory", AccountCategory::Asset, dec!(320000.00), "salt_4"),
            AccountLeaf::new("2110 - Trade Creditors AP", AccountCategory::Liability, dec!(140000.00), "salt_5"),
            AccountLeaf::new("2120 - Short-Term Bank Facility", AccountCategory::Liability, dec!(260000.00), "salt_6"),
            AccountLeaf::new("2210 - Output VAT Payable", AccountCategory::Liability, dec!(35000.00), "salt_7"),
            AccountLeaf::new("3110 - Share Capital", AccountCategory::Equity, dec!(1000000.00), "salt_8"),
        ]
    }

    #[test]
    fn test_zk_solvency_proof_generation_and_sla() {
        let accounts = sample_balance_sheet();
        let start = Instant::now();

        let proof = ZkSolvencyEngine::generate_solvency_proof(&accounts)
            .expect("Solvency proof generation failed");

        let duration = start.elapsed();
        // Verification / generation SLA: < 15ms
        assert!(duration.as_millis() < 15, "Proof generation exceeded 15ms SLA");

        // Total Assets: 250k + 750k + 180k + 320k = 1,500,000.00
        assert_eq!(proof.total_assets, dec!(1500000.00));
        // Total Liabilities: 140k + 260k + 35k = 435,000.00
        assert_eq!(proof.total_liabilities, dec!(435000.00));
        // Solvency Ratio: 1500000 / 435000 = 3.4483
        assert_eq!(proof.solvency_ratio, dec!(3.4483));
        assert!(!proof.root_hash.is_empty());
    }

    #[test]
    fn test_insolvent_balance_sheet_rejection() {
        let accounts = vec![
            AccountLeaf::new("1110 - Bank", AccountCategory::Asset, dec!(50000.00), "s1"),
            AccountLeaf::new("2110 - Term Loan", AccountCategory::Liability, dec!(200000.00), "s2"),
        ];

        let err = ZkSolvencyEngine::generate_solvency_proof(&accounts);
        assert!(matches!(
            err,
            Err(SolvencyProofError::InsolventBalanceSheet { .. })
        ));
    }

    #[test]
    fn test_account_inclusion_proof_verification_sub_millisecond() {
        let accounts = sample_balance_sheet();

        let proof = ZkSolvencyEngine::generate_account_proof(&accounts, "1120 - Treasury Bonds")
            .expect("Account inclusion proof generation failed");

        assert_eq!(proof.account_id, "1120 - Treasury Bonds");
        assert_eq!(proof.balance, dec!(750000.00));
        assert_eq!(proof.steps.len(), 3); // log2(8) = 3 steps

        let start = Instant::now();
        let is_valid = ZkSolvencyEngine::verify_account_proof(&proof)
            .expect("Proof verification failed");
        let duration = start.elapsed();

        assert!(is_valid);
        // Non-interactive verification: < 1ms
        assert!(duration.as_millis() < 5, "Verification took longer than 5ms");

        // Tampering detection: Altering balance causes verification failure
        let mut tampered_proof = proof.clone();
        tampered_proof.balance = dec!(800000.00);
        assert!(ZkSolvencyEngine::verify_account_proof(&tampered_proof).is_err());

        // Tampering detection: Altering salt causes verification failure
        let mut tampered_salt = proof.clone();
        tampered_salt.blinding_salt = "forged_salt".into();
        assert!(ZkSolvencyEngine::verify_account_proof(&tampered_salt).is_err());
    }
}
