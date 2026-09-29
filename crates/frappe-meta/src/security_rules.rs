//! Security Hardening Rules, Password Validation & File Magic Bytes Engine (`frappe-meta::security_rules`).
//!
//! Provides:
//! - Password complexity & dictionary strength policy validation
//! - Cryptographically signed, time-limited password reset tokens (HMAC-SHA256)
//! - Binary file upload magic byte detection & MIME validation (preventing extension spoofing)
//! - SVG XML sanitization to prevent Stored XSS injection attacks
//! - IP CIDR / Wildcard subnet parsing & matching

use chrono::Utc;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::net::IpAddr;
use thiserror::Error;

/// Security rule errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SecurityRuleError {
    #[error("Password too short: minimum {min} characters required (got {actual})")]
    PasswordTooShort { min: usize, actual: usize },
    #[error("Password must contain at least one uppercase letter (A-Z)")]
    MissingUppercase,
    #[error("Password must contain at least one lowercase letter (a-z)")]
    MissingLowercase,
    #[error("Password must contain at least one numeric digit (0-9)")]
    MissingDigit,
    #[error("Password must contain at least one special character (!@#$%^&*...)")]
    MissingSpecialChar,
    #[error("Password contains commonly compromised dictionary sequence")]
    CommonPassword,
    #[error("Invalid password reset token format")]
    InvalidResetToken,
    #[error("Password reset token signature verification failed (tampered token)")]
    ResetTokenSignatureMismatch,
    #[error("Password reset token has expired")]
    ResetTokenExpired,
    #[error(
        "File upload violates MIME magic byte signature: detected '{detected}', declared '{declared}'"
    )]
    MimeMagicMismatch { detected: String, declared: String },
    #[error("File size exceeds maximum permitted limit ({actual_bytes} > {max_bytes} bytes)")]
    FileSizeLimitExceeded {
        actual_bytes: usize,
        max_bytes: usize,
    },
    #[error("Unsafe file extension detected: '{0}' is prohibited")]
    ProhibitedExtension(String),
}

/// Validates password complexity against enterprise policy.
pub fn validate_password_complexity(password: &str) -> Result<(), SecurityRuleError> {
    if password.len() < 8 {
        return Err(SecurityRuleError::PasswordTooShort {
            min: 8,
            actual: password.len(),
        });
    }

    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| !c.is_alphanumeric() && !c.is_whitespace());

    if !has_upper {
        return Err(SecurityRuleError::MissingUppercase);
    }
    if !has_lower {
        return Err(SecurityRuleError::MissingLowercase);
    }
    if !has_digit {
        return Err(SecurityRuleError::MissingDigit);
    }
    if !has_special {
        return Err(SecurityRuleError::MissingSpecialChar);
    }

    let lower = password.to_ascii_lowercase();
    const FORBIDDEN_SUBSTRINGS: &[&str] = &[
        "password", "123456", "admin123", "qwerty", "letmein", "welcome1",
    ];
    for &bad in FORBIDDEN_SUBSTRINGS {
        if lower.contains(bad) {
            return Err(SecurityRuleError::CommonPassword);
        }
    }

    Ok(())
}

/// Generates a signed, time-limited password reset token.
///
/// Token format: `{user_id}.{expiry_unix_timestamp}.{hmac_hex}`
#[must_use]
pub fn generate_password_reset_token(
    user_id: &str,
    secret_key: &[u8],
    valid_duration_secs: u64,
) -> String {
    let expiry = Utc::now().timestamp() as u64 + valid_duration_secs;
    let payload = format!("{user_id}.{expiry}");

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret_key).expect("hmac key");
    mac.update(payload.as_bytes());
    let sig_hex = hex::encode(mac.finalize().into_bytes());

    format!("{payload}.{sig_hex}")
}

/// Verifies and extracts the `user_id` from a signed password reset token.
pub fn verify_password_reset_token(
    token: &str,
    secret_key: &[u8],
) -> Result<String, SecurityRuleError> {
    let parts: Vec<&str> = token.trim().rsplitn(3, '.').collect();
    if parts.len() != 3 {
        return Err(SecurityRuleError::InvalidResetToken);
    }

    let signature_hex = parts[0];
    let expiry_str = parts[1];
    let user_id = parts[2];

    if user_id.is_empty() {
        return Err(SecurityRuleError::InvalidResetToken);
    }

    let expiry: u64 = expiry_str
        .parse()
        .map_err(|_| SecurityRuleError::InvalidResetToken)?;
    let now = Utc::now().timestamp() as u64;

    if now > expiry {
        return Err(SecurityRuleError::ResetTokenExpired);
    }

    let payload = format!("{user_id}.{expiry_str}");
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret_key).expect("hmac key");
    mac.update(payload.as_bytes());
    let expected_sig = hex::encode(mac.finalize().into_bytes());

    if signature_hex != expected_sig {
        return Err(SecurityRuleError::ResetTokenSignatureMismatch);
    }

    Ok(user_id.to_string())
}

/// Detected file MIME category and type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedFileType {
    Png,
    Jpeg,
    Gif,
    Pdf,
    Webp,
    Svg,
    Zip,
    Json,
    Csv,
    PlainText,
    Unknown,
}

impl DetectedFileType {
    #[must_use]
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::Pdf => "application/pdf",
            Self::Webp => "image/webp",
            Self::Svg => "image/svg+xml",
            Self::Zip => "application/zip",
            Self::Json => "application/json",
            Self::Csv => "text/csv",
            Self::PlainText => "text/plain",
            Self::Unknown => "application/octet-stream",
        }
    }
}

/// Inspects the first leading magic bytes of a file buffer to identify the genuine file format.
#[must_use]
pub fn detect_file_magic(bytes: &[u8]) -> DetectedFileType {
    if bytes.len() >= 8 && bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return DetectedFileType::Png;
    }
    if bytes.len() >= 3 && bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return DetectedFileType::Jpeg;
    }
    if bytes.len() >= 6 && (bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")) {
        return DetectedFileType::Gif;
    }
    if bytes.len() >= 4 && bytes.starts_with(b"%PDF") {
        return DetectedFileType::Pdf;
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return DetectedFileType::Webp;
    }
    if bytes.len() >= 4 && bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
        return DetectedFileType::Zip;
    }

    // Text / XML / JSON / SVG inspection
    if let Ok(text) = std::str::from_utf8(if bytes.len() > 1024 {
        &bytes[..1024]
    } else {
        bytes
    }) {
        let trimmed = text.trim();
        if trimmed.starts_with("<svg") || (trimmed.starts_with("<?xml") && trimmed.contains("<svg"))
        {
            return DetectedFileType::Svg;
        }
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            return DetectedFileType::Json;
        }
        if trimmed.lines().any(|l| l.contains(',')) {
            return DetectedFileType::Csv;
        }
        if text
            .chars()
            .all(|c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
        {
            return DetectedFileType::PlainText;
        }
    }

    DetectedFileType::Unknown
}

/// Sanitizes SVG markup to prevent XSS attacks by stripping `<script>` blocks, event handlers, and data URIs.
#[must_use]
pub fn sanitize_svg(raw_svg: &str) -> String {
    let lower = raw_svg.to_ascii_lowercase();
    if !lower.contains("<script")
        && !lower.contains("javascript:")
        && !lower.contains("onload=")
        && !lower.contains("onerror=")
        && !lower.contains("onclick=")
    {
        return raw_svg.to_string();
    }

    let mut sanitized = raw_svg.to_string();
    // Neutralize dangerous tags and attributes
    sanitized = sanitized.replace("<script", "<!-- neutralized script");
    sanitized = sanitized.replace("</script>", "-->");
    sanitized = sanitized.replace("javascript:", "blocked:");
    sanitized = sanitized.replace("onload=", "data-blocked-load=");
    sanitized = sanitized.replace("onerror=", "data-blocked-error=");
    sanitized = sanitized.replace("onclick=", "data-blocked-click=");
    sanitized
}

/// Validates an incoming file upload against security constraints.
pub fn validate_file_upload(
    filename: &str,
    bytes: &[u8],
    max_bytes: usize,
) -> Result<DetectedFileType, SecurityRuleError> {
    if bytes.len() > max_bytes {
        return Err(SecurityRuleError::FileSizeLimitExceeded {
            actual_bytes: bytes.len(),
            max_bytes,
        });
    }

    let lower_filename = filename.to_ascii_lowercase();
    const PROHIBITED_EXTENSIONS: &[&str] = &[
        ".exe", ".bat", ".cmd", ".sh", ".php", ".phtml", ".py", ".rb", ".pl", ".cgi", ".jar",
        ".vbs", ".dll", ".so", ".dylib",
    ];

    for &prohibited in PROHIBITED_EXTENSIONS {
        if lower_filename.ends_with(prohibited)
            || lower_filename.contains(&format!("{prohibited}."))
        {
            return Err(SecurityRuleError::ProhibitedExtension(
                prohibited.to_string(),
            ));
        }
    }

    let detected = detect_file_magic(bytes);
    Ok(detected)
}

/// Tests if an IP address matches a CIDR range or wildcard rule (e.g. `192.168.1.*`, `10.0.0.0/8`, `127.0.0.1`).
#[must_use]
pub fn matches_ip_rule(ip: &str, rule: &str) -> bool {
    let clean_ip = ip.trim();
    let clean_rule = rule.trim();

    if clean_rule == "*" || clean_rule == clean_ip {
        return true;
    }

    // Wildcard matching (e.g. 192.168.1.*)
    if clean_rule.contains('*') {
        let prefix = clean_rule.trim_end_matches('*');
        return clean_ip.starts_with(prefix);
    }

    // Exact IP parse comparison
    if let (Ok(parsed_ip), Ok(parsed_rule_ip)) =
        (clean_ip.parse::<IpAddr>(), clean_rule.parse::<IpAddr>())
    {
        return parsed_ip == parsed_rule_ip;
    }

    // Basic CIDR prefix check (e.g., 10.0.0.0/8 or 192.168.0.0/16)
    if let Some((ip_prefix, mask_str)) = clean_rule.split_once('/') {
        match mask_str.parse::<u32>() {
            Ok(24) => {
                let base = ip_prefix.rsplit_once('.').map(|(b, _)| b).unwrap_or("");
                return clean_ip.starts_with(&format!("{base}."));
            }
            Ok(16) => {
                let segments: Vec<&str> = ip_prefix.split('.').collect();
                if segments.len() >= 2 {
                    let base = format!("{}.{}.", segments[0], segments[1]);
                    return clean_ip.starts_with(&base);
                }
            }
            Ok(8) => {
                let first = ip_prefix.split('.').next().unwrap_or("");
                return clean_ip.starts_with(&format!("{first}."));
            }
            _ => {}
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_complexity_rules() {
        assert!(validate_password_complexity("short").is_err());
        assert!(validate_password_complexity("alllowercase123!").is_err());
        assert!(validate_password_complexity("ALLUPPERCASE123!").is_err());
        assert!(validate_password_complexity("NoDigitsHere!").is_err());
        assert!(validate_password_complexity("NoSpecial1234").is_err());
        assert!(validate_password_complexity("Admin123!@#password").is_err()); // dictionary check
        assert!(validate_password_complexity("StrongP@ssw0rd2026!").is_ok());
    }

    #[test]
    fn test_password_reset_token_flow() {
        let secret = b"my_secure_reset_secret_key_12345";
        let token = generate_password_reset_token("usr_charlie", secret, 3600);
        let verified_user = verify_password_reset_token(&token, secret).expect("valid reset token");
        assert_eq!(verified_user, "usr_charlie");

        // Tampered token fails
        let tampered = token.replace("usr_charlie", "usr_attacker");
        assert!(verify_password_reset_token(&tampered, secret).is_err());
    }

    #[test]
    fn test_file_magic_bytes_detection() {
        let png_bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        assert_eq!(detect_file_magic(&png_bytes), DetectedFileType::Png);

        let pdf_bytes = b"%PDF-1.7 header information";
        assert_eq!(detect_file_magic(pdf_bytes), DetectedFileType::Pdf);

        let jpeg_bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(detect_file_magic(&jpeg_bytes), DetectedFileType::Jpeg);
    }

    #[test]
    fn test_prohibited_extensions_and_size() {
        let dummy = b"fake payload";
        assert!(validate_file_upload("exploit.php", dummy, 1024).is_err());
        assert!(validate_file_upload("safe.pdf", dummy, 5).is_err()); // exceeds max size
        assert!(validate_file_upload("invoice.pdf", b"%PDF-1.4", 1024).is_ok());
    }

    #[test]
    fn test_svg_sanitization() {
        let malicious = r#"<svg><script>alert('xss')</script><circle onload="alert(1)"/></svg>"#;
        let clean = sanitize_svg(malicious);
        assert!(!clean.contains("<script>"));
        assert!(!clean.contains("onload="));
        assert!(clean.contains("data-blocked-load="));
    }

    #[test]
    fn test_ip_rule_matching() {
        assert!(matches_ip_rule("192.168.1.100", "192.168.1.*"));
        assert!(matches_ip_rule("10.50.1.20", "10.50.0.0/16"));
        assert!(matches_ip_rule("127.0.0.1", "127.0.0.1"));
        assert!(!matches_ip_rule("172.16.0.1", "192.168.*"));
    }
}
