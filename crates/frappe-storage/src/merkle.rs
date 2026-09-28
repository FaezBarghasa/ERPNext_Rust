//! Cryptographic Merkle and SHA-256 tamper-evident log tools for statutory audit trails (`frappe-storage::merkle`).
//!
//! Provides $O(\log N)$ Merkle proof generation and verification ensuring historical
//! transactions have not been tampered with.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Merkle inclusion proof node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleProofStep {
    pub is_right: bool,
    pub sibling_hash: String,
}

/// Cryptographic Merkle tree audit record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleProof {
    pub leaf_hash: String,
    pub root_hash: String,
    pub steps: Vec<MerkleProofStep>,
}

pub struct MerkleHasher {
    hasher: Sha256,
}

impl Default for MerkleHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl MerkleHasher {
    #[must_use]
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    #[must_use]
    pub fn finalize_hex(self) -> String {
        self.hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Computes hash of two concatenated child hashes.
    #[must_use]
    pub fn hash_pair(left: &str, right: &str) -> String {
        let mut h = Sha256::new();
        h.update(left.as_bytes());
        h.update(right.as_bytes());
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Computes root hash of a list of binary leaves.
    #[must_use]
    pub fn compute_merkle_root(leaves: &[String]) -> String {
        if leaves.is_empty() {
            return String::new();
        }
        let mut current_level: Vec<String> = leaves.to_vec();
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(Self::hash_pair(&chunk[0], &chunk[1]));
                } else {
                    next_level.push(chunk[0].clone());
                }
            }
            current_level = next_level;
        }
        current_level[0].clone()
    }

    /// Generates an $O(\log N)$ cryptographic inclusion proof for a leaf at `leaf_index`.
    #[must_use]
    pub fn generate_proof(leaves: &[String], leaf_index: usize) -> Option<MerkleProof> {
        if leaf_index >= leaves.len() {
            return None;
        }

        let root_hash = Self::compute_merkle_root(leaves);
        let leaf_hash = leaves[leaf_index].clone();
        let mut steps = Vec::new();

        let mut current_level = leaves.to_vec();
        let mut current_index = leaf_index;

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            let is_right_node = current_index % 2 == 1;
            let sibling_index = if is_right_node {
                current_index - 1
            } else if current_index + 1 < current_level.len() {
                current_index + 1
            } else {
                current_index
            };

            if sibling_index < current_level.len() && sibling_index != current_index {
                steps.push(MerkleProofStep {
                    is_right: !is_right_node,
                    sibling_hash: current_level[sibling_index].clone(),
                });
            }

            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(Self::hash_pair(&chunk[0], &chunk[1]));
                } else {
                    next_level.push(chunk[0].clone());
                }
            }

            current_index /= 2;
            current_level = next_level;
        }

        Some(MerkleProof {
            leaf_hash,
            root_hash,
            steps,
        })
    }

    /// Verifies a cryptographic Merkle inclusion proof in $O(\log N)$ time.
    #[must_use]
    pub fn verify_proof(proof: &MerkleProof) -> bool {
        let mut current_hash = proof.leaf_hash.clone();

        for step in &proof.steps {
            current_hash = if step.is_right {
                Self::hash_pair(&current_hash, &step.sibling_hash)
            } else {
                Self::hash_pair(&step.sibling_hash, &current_hash)
            };
        }

        current_hash == proof.root_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_computation() {
        let leaf1 = "leaf1".to_string();
        let leaf2 = "leaf2".to_string();
        let root = MerkleHasher::compute_merkle_root(&[leaf1, leaf2]);
        assert_eq!(root.len(), 64);
    }

    #[test]
    fn test_merkle_proof_generation_and_verification() {
        let leaves: Vec<String> = (0..8)
            .map(|i| {
                let mut h = Sha256::new();
                h.update(format!("transaction_{i}").as_bytes());
                h.finalize().iter().map(|b| format!("{b:02x}")).collect()
            })
            .collect();

        let root = MerkleHasher::compute_merkle_root(&leaves);

        for i in 0..8 {
            let proof = MerkleHasher::generate_proof(&leaves, i).unwrap();
            assert_eq!(proof.root_hash, root);
            assert!(MerkleHasher::verify_proof(&proof));
        }
    }
}
