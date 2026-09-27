/// Signed token check stub: token = "sig:payload" (Stage 4.6.4).
pub fn verify_token(token: &str, secret: &str) -> bool { token.starts_with(secret) }
#[cfg(test)] mod t { use super::*; #[test] fn tok(){ assert!(verify_token("s3cr3t:abc","s3cr3t")); } }
