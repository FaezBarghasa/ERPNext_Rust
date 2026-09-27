pub const VARIANTS: &[&str] = &["1080p","720p","480p"];
pub fn hls_playlist(variants: &[&str]) -> String { variants.iter().map(|v| format!("#EXT-X-STREAM-INF:BANDWIDTH=1000000\n{}.m3u8", v)).collect::<Vec<_>>().join("\n") }
#[cfg(test)] mod t { use super::*; #[test] fn hls(){ assert!(hls_playlist(VARIANTS).contains("720p")); } }
