//! TLS / JA3 / JA4 Fingerprint Camouflage & Browser Profile Emulation (`erp_stealth_scraper::fingerprint`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Target browser profile preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserPreset {
    Chrome130Windows,
    Chrome130MacArm,
    Safari18Mac,
    Firefox132Linux,
}

/// Structured JA4 TLS Fingerprint specification.
/// Format: `t13d1516h2_8daaf6152771_000000000000`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ja4Fingerprint {
    pub raw_ja4: CompactString,
    pub protocol: CompactString,
    pub sni: CompactString,
    pub cipher_count: usize,
    pub extension_count: usize,
    pub alpn: CompactString,
}

impl Ja4Fingerprint {
    /// Constructs a standardized JA4 fingerprint representation for the given preset.
    #[must_use]
    pub fn for_preset(preset: BrowserPreset) -> Self {
        match preset {
            BrowserPreset::Chrome130Windows | BrowserPreset::Chrome130MacArm => Self {
                raw_ja4: "t13d1516h2_8daaf6152771_e562703ab85b".into(),
                protocol: "TLS 1.3".into(),
                sni: "d".into(),
                cipher_count: 15,
                extension_count: 16,
                alpn: "h2".into(),
            },
            BrowserPreset::Safari18Mac => Self {
                raw_ja4: "t13d1911h2_b050c266cc7e_29d7c04118f6".into(),
                protocol: "TLS 1.3".into(),
                sni: "d".into(),
                cipher_count: 19,
                extension_count: 11,
                alpn: "h2".into(),
            },
            BrowserPreset::Firefox132Linux => Self {
                raw_ja4: "t13d1413h2_8daaf6152771_2c179c3f4a9b".into(),
                protocol: "TLS 1.3".into(),
                sni: "d".into(),
                cipher_count: 14,
                extension_count: 13,
                alpn: "h2".into(),
            },
        }
    }
}

/// TLS Camouflage profile enforcing exact cipher suites and extension orders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsCamouflageProfile {
    pub preset: BrowserPreset,
    pub ja4: Ja4Fingerprint,
    pub cipher_suites: Vec<CompactString>,
    pub supported_groups: Vec<CompactString>,
    pub signature_algorithms: Vec<CompactString>,
    pub alpn_protocols: Vec<CompactString>,
}

impl TlsCamouflageProfile {
    /// Generates a complete TLS profile configured for the specified browser preset.
    #[must_use]
    pub fn new(preset: BrowserPreset) -> Self {
        let ja4 = Ja4Fingerprint::for_preset(preset);

        let cipher_suites = vec![
            "TLS_AES_128_GCM_SHA256".into(),
            "TLS_AES_256_GCM_SHA384".into(),
            "TLS_CHACHA20_POLY1305_SHA256".into(),
            "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".into(),
            "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".into(),
            "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".into(),
            "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".into(),
            "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256".into(),
            "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".into(),
        ];

        let supported_groups = vec![
            "X25519".into(),
            "secp256r1".into(),
            "secp384r1".into(),
            "X25519MLKEM768".into(),
        ];

        let signature_algorithms = vec![
            "ecdsa_secp256r1_sha256".into(),
            "rsa_pss_rsae_sha256".into(),
            "rsa_pkcs1_sha256".into(),
            "ecdsa_secp384r1_sha384".into(),
            "rsa_pss_rsae_sha384".into(),
            "rsa_pkcs1_sha384".into(),
            "rsa_pss_rsae_sha512".into(),
            "rsa_pkcs1_sha512".into(),
        ];

        let alpn_protocols = vec!["h2".into(), "http/1.1".into()];

        Self {
            preset,
            ja4,
            cipher_suites,
            supported_groups,
            signature_algorithms,
            alpn_protocols,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ja4_fingerprint_generation() {
        let profile = TlsCamouflageProfile::new(BrowserPreset::Chrome130Windows);
        assert_eq!(profile.ja4.alpn, "h2");
        assert!(profile.ja4.raw_ja4.starts_with("t13d1516h2"));
        assert_eq!(profile.cipher_suites.len(), 9);
        assert!(profile.supported_groups.contains(&"X25519".into()));
    }
}
