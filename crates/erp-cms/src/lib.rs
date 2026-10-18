pub mod analytics;
pub mod block_canvas;
pub mod builder_core;
pub mod emdash_studio;
pub mod hls_engine;
pub mod luxury_storefront;
pub mod media_library;
pub mod print;
pub mod rtmp_ingest;
pub mod security;
pub mod seo_engine;
pub mod storefront;
pub mod taxonomy;
pub mod template_catalog;
pub mod theme_engine;
pub mod transcode_pipeline;
pub mod video_streaming;

pub use media_library::{MediaAsset, MediaError, MediaLibraryRegistry};
pub use taxonomy::{TaxonomyError, TaxonomyRegistry, TaxonomyTerm, TaxonomyType};

pub use analytics::{AnalyticsSummary, EdgeAnalyticsEngine, PageViewEvent};
pub use block_canvas::{CmsPage, FeatureItem, PageBlock, SsrEngine};
pub use builder_core::{
    BlockCssInspector, BlockValueResolver, BobAgentPromptRequest, CanvasBlock, CanvasBlockNode,
    CanvasBlockTree, CanvasBreakpoint, CanvasInteractionMode, CanvasReversionLedger,
    DeviceFrameSwitcher, DocTypeBindingFilter, DocTypeDataBinding, ResponsiveStyleCascade,
    ResponsiveStyleMap,
};
pub use emdash_studio::{
    CalloutStyle, CropBox, EditorialWorkflowError, EditorialWorkflowState, EmdashBlock,
    EmdashDocument, WebPVariantMetadata, calculate_focal_crop, generate_editorial_xml_sitemap,
    generate_responsive_webp_variants,
};
pub use luxury_storefront::{
    LUXURY_STOREFRONT_CSS, LUXURY_STOREFRONT_HTML, LUXURY_STOREFRONT_JS, LuxuryProduct,
    get_luxury_catalog, render_luxury_storefront_html,
};
pub use print::render_invoice_html;
pub use security::{HmacStreamingSigner, StreamingError, SvodPlaybackManager, parse_byte_range};
pub use seo_engine::{
    RssFeedItem, SeoMetadata, SitemapEntry, generate_dynamic_sitemap_xml, generate_robots_txt,
    generate_rss_2_xml, generate_sitemap_xml,
};
pub use storefront::{
    AtomicCheckoutEngine, CheckoutError, CheckoutItem, CheckoutResult, CustomerCheckoutRequest,
};
pub use template_catalog::{
    TemplateSuite, get_template_suite, list_template_suites, render_template_html,
    render_template_html_with_variant, render_template_index_html, render_theme_switcher_dock,
};
pub use theme_engine::{
    CompiledTheme, DesignTokens, RenderEngineKind, SlotDefinition, ThemeManifest, ThemeRegistry,
    ThemeVariant, WorkTypeClassification,
};
pub use hls_engine::{
    HlsAudioRendition, HlsMasterPlaylist, HlsMediaPlaylist, HlsMediaSegment, HlsSegmentCache,
    HlsSubtitleRendition, HlsVariant,
};
pub use rtmp_ingest::{
    NginxRtmpPublishPayload, NginxRtmpRecordPayload, NginxRtmpUpdatePayload, RtmpError,
    RtmpIngestManager, RtmpStreamSession, RtmpStreamStatus,
};
pub use transcode_pipeline::{
    HardwareAccelType, ResumableChunkEngine, ResumableUploadSession, TranscodeError, TranscodeJob,
};
pub use video_streaming::{
    AudioChannels, AudioCodec, ClientDeviceProfile, ColorRange, MediaContainer, MediaStreamInfo,
    PlayMethod, PlaybackDecisionEngine, SubtitleDelivery, SubtitleFormat, SubtitleTrackInfo,
    VideoCodec,
};
pub use erp_cms_core as core;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_to_edge_core_bridge() {
        let mut edge_layout: erp_cms_core::EdgeCanvasLayout<16> = erp_cms_core::EdgeCanvasLayout::new("Warehouse Display");
        let mut text = erp_cms_core::heapless::String::new();
        text.push_str("SKU-9923 Item Verified").unwrap();
        edge_layout.add_block(erp_cms_core::EdgeBlockNode::new("b1", erp_cms_core::EdgeBlockType::Text { content: text })).unwrap();
        assert_eq!(edge_layout.blocks.len(), 1);
    }

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
}
