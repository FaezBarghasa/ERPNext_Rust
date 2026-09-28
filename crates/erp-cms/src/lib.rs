pub mod block_canvas;
pub mod luxury_storefront;
pub mod print;
pub mod security;
pub mod storefront;
pub mod subtitles;
pub mod transcoder;

pub use block_canvas::{CmsPage, FeatureItem, PageBlock, SsrEngine};
pub use luxury_storefront::{
    LUXURY_STOREFRONT_CSS, LUXURY_STOREFRONT_HTML, LUXURY_STOREFRONT_JS, LuxuryProduct,
    get_luxury_catalog, render_luxury_storefront_html,
};
pub use print::render_invoice_html;
pub use security::{HmacStreamingSigner, StreamingError, SvodPlaybackManager, parse_byte_range};
pub use storefront::{
    AtomicCheckoutEngine, CheckoutError, CheckoutItem, CheckoutResult, CustomerCheckoutRequest,
};
pub use subtitles::{SubtitleSearchEngine, SubtitleSegment};
pub use transcoder::{HlsPlaylistGenerator, STANDARD_VARIANTS, VideoVariant};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_range_parser() {
        let total = 10_000u64;

        // bytes=1000-2000 -> (1000, 2000)
        let (start, end) = parse_byte_range("bytes=1000-2000", total).unwrap();
        assert_eq!(start, 1000);
        assert_eq!(end, 2000);

        // bytes=5000- -> (5000, 9999)
        let (start, end) = parse_byte_range("bytes=5000-", total).unwrap();
        assert_eq!(start, 5000);
        assert_eq!(end, 9999);

        // bytes=-500 -> (9500, 9999)
        let (start, end) = parse_byte_range("bytes=-500", total).unwrap();
        assert_eq!(start, 9500);
        assert_eq!(end, 9999);

        // Unsatisfiable range (start >= total)
        assert!(parse_byte_range("bytes=15000-20000", total).is_err());
    }

    #[test]
    fn test_hmac_signed_token_verification_and_expiry() {
        let secret = b"super-secret-key-123456";
        let signer = HmacStreamingSigner::new(secret);
        let media_id = "video-lesson-42";
        let expiry = 2_000_000_000u64;

        let token = signer.generate_token(media_id, expiry);

        // Valid token
        assert!(
            signer
                .verify_token(media_id, expiry, 1_900_000_000, &token)
                .is_ok()
        );

        // Forged token
        assert!(
            signer
                .verify_token(media_id, expiry, 1_900_000_000, "forged_hex_token")
                .is_err()
        );

        // Expired token (current > expiry)
        let exp_err = signer.verify_token(media_id, expiry, 2_000_000_001, &token);
        assert!(matches!(exp_err, Err(StreamingError::TokenExpired { .. })));
    }

    #[test]
    fn test_svod_concurrency_limiter() {
        let manager = SvodPlaybackManager::new();

        // 1-stream allowance
        assert!(
            manager
                .start_playback("user_1", "session_1".into(), true, 1)
                .is_ok()
        );

        // Attempt second concurrent stream -> Breaches limit
        let second_stream = manager.start_playback("user_1", "session_2".into(), true, 1);
        assert!(matches!(
            second_stream,
            Err(StreamingError::StreamLimitExceeded { .. })
        ));

        // End first stream -> Second stream now succeeds
        manager.end_playback("user_1", "session_1");
        assert!(
            manager
                .start_playback("user_1", "session_2".into(), true, 1)
                .is_ok()
        );
    }

    #[test]
    fn test_hls_master_playlist_and_cosine_similarity() {
        let master = HlsPlaylistGenerator::generate_master_playlist(STANDARD_VARIANTS);
        assert!(master.contains("#EXTM3U"));
        assert!(master.contains("1080p.m3u8"));
        assert!(master.contains("720p.m3u8"));
        assert!(master.contains("480p.m3u8"));

        // Cosine similarity between identical vectors = 1.0
        let vec_a = vec![1.0, 2.0, 3.0];
        let sim_identical = SubtitleSearchEngine::cosine_similarity(&vec_a, &vec_a);
        assert!((sim_identical - 1.0).abs() < 1e-5);
    }
}
