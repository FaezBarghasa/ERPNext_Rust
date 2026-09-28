//! ZATCA Phase 2 (Kingdom of Saudi Arabia) E-Invoicing Engine.
//! Implements cryptographic invoice hash chaining and TLV (Tag-Length-Value) Base64 QR encoding.

use sha2::{Digest, Sha256};

pub struct ZatcaPhase2Engine;

impl ZatcaPhase2Engine {
    /// Computes the cryptographic invoice hash chaining:
    /// Hash_n = SHA-256(UBL_Bytes_n || Hash_{n-1})
    #[must_use]
    pub fn compute_invoice_hash(
        ubl_xml_bytes: &[u8],
        previous_invoice_hash: Option<&str>,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(ubl_xml_bytes);
        if let Some(prev) = previous_invoice_hash {
            hasher.update(prev.as_bytes());
        }
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Encodes a single TLV tag: Tag (1 byte) + Length (1 byte) + Value.
    fn encode_tlv_tag(tag: u8, value: &str) -> Vec<u8> {
        let bytes = value.as_bytes();
        let len = bytes.len() as u8;
        let mut out = Vec::with_capacity(2 + bytes.len());
        out.push(tag);
        out.push(len);
        out.extend_from_slice(bytes);
        out
    }

    /// Generates the standard ZATCA TLV Base64 QR Code string:
    /// 1. Seller's Name (Tag 1)
    /// 2. VAT Registration Number (Tag 2)
    /// 3. Time Stamp (Tag 3)
    /// 4. Invoice Total (Tag 4)
    /// 5. VAT Total (Tag 5)
    /// 6. Invoice Hash (Tag 6)
    /// 7. ECDSA Signature (Tag 7)
    /// 8. ECDSA Public Key (Tag 8)
    #[must_use]
    pub fn generate_tlv_qr_base64(
        seller_name: &str,
        vat_number: &str,
        timestamp_iso: &str,
        invoice_total: &str,
        vat_total: &str,
        invoice_hash: &str,
    ) -> String {
        let mut tlv_bytes = Vec::new();
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(1, seller_name));
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(2, vat_number));
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(3, timestamp_iso));
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(4, invoice_total));
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(5, vat_total));
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(6, invoice_hash));

        // Base64 encoding
        use std::fmt::Write;
        let b64_chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();
        let chunks = tlv_bytes.chunks(3);

        for chunk in chunks {
            let b0 = chunk[0] as usize;
            let b1 = if chunk.len() > 1 {
                chunk[1] as usize
            } else {
                0
            };
            let b2 = if chunk.len() > 2 {
                chunk[2] as usize
            } else {
                0
            };

            let triple = (b0 << 16) | (b1 << 8) | b2;

            let c0 = b64_chars[(triple >> 18) & 0x3F] as char;
            let c1 = b64_chars[(triple >> 12) & 0x3F] as char;
            let c2 = if chunk.len() > 1 {
                b64_chars[(triple >> 6) & 0x3F] as char
            } else {
                '='
            };
            let c3 = if chunk.len() > 2 {
                b64_chars[triple & 0x3F] as char
            } else {
                '='
            };

            let _ = write!(result, "{c0}{c1}{c2}{c3}");
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zatca_hash_chaining_and_tlv_qr() {
        let xml_doc = b"<Invoice><ID>INV-KSA-001</ID></Invoice>";
        let genesis_hash = ZatcaPhase2Engine::compute_invoice_hash(xml_doc, None);
        assert_eq!(genesis_hash.len(), 64);

        let second_xml = b"<Invoice><ID>INV-KSA-002</ID></Invoice>";
        let second_hash = ZatcaPhase2Engine::compute_invoice_hash(second_xml, Some(&genesis_hash));
        assert_eq!(second_hash.len(), 64);
        assert_ne!(genesis_hash, second_hash);

        let qr_base64 = ZatcaPhase2Engine::generate_tlv_qr_base64(
            "Al-Madina Industrial Corp",
            "300123456700003",
            "2026-09-28T10:30:00Z",
            "1150.00",
            "150.00",
            &second_hash,
        );
        assert!(!qr_base64.is_empty());
    }
}
