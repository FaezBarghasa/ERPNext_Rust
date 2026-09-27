//! Cryptographic Merkle and SHA-256 tamper-evident log tools for statutory audit trails.

use sha2::{Digest, Sha256};

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
                    let mut h = Sha256::new();
                    h.update(chunk[0].as_bytes());
                    h.update(chunk[1].as_bytes());
                    let hex_out: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
                    next_level.push(hex_out);
                } else {
                    next_level.push(chunk[0].clone());
                }
            }
            current_level = next_level;
        }
        current_level[0].clone()
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
}
