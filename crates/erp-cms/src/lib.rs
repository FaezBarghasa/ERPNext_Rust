pub mod analytics;
pub mod audio_book_sync;
pub mod audio_dsp_eq;
pub mod audio_flac_pipeline;
pub mod audio_intelligence;
pub mod audio_live_radio;
pub mod audio_lyrics;
pub mod audio_sleep_noise;
pub mod audio_streaming;
pub mod audio_subsonic;
pub mod audio_waveform;
pub mod block_canvas;
pub mod builder_core;
pub mod emdash_studio;
pub mod fast_channels;
pub mod hls_engine;
pub mod luxury_storefront;
pub mod media_library;
pub mod print;
pub mod resilient_streaming;
pub mod rtmp_ingest;
pub mod security;
pub mod seo_engine;
pub mod ssai_engine;
pub mod storefront;
pub mod stremio_addon;
pub mod taxonomy;
pub mod template_catalog;
pub mod theme_engine;
pub mod transcode_pipeline;
pub mod video_streaming;
pub mod xray_metadata;

pub use audio_book_sync::{AudioBookmarkRegistry, AudioChapter, AudioPlaybackBookmark};
pub use audio_dsp_eq::{AudioEqualizer, EQ_FREQUENCIES_HZ, EqBand, EqPreset};
pub use audio_flac_pipeline::{
    FlacCompressionConfig, FlacMetadataTags, FlacPipelineEngine, FlacQualityTier,
    FlacTranscodeJobPlan,
};
pub use audio_intelligence::{
    CamelotKey, MusicContributor, MusicIntelligenceEngine, TrackLinerNotes, TrackRecommendation,
};
pub use audio_live_radio::{ListenerPresence, LiveRadioRegistry, RadioChatMessage, RadioStation};
pub use audio_lyrics::{ActiveLyricSnapshot, LyricLine, LyricsDocument, SyllableTiming};
pub use audio_sleep_noise::{
    AmbientStem, BinauralBeatGenerator, BrownNoiseGenerator, FastRng, ProceduralNoiseEngine,
    SleepTimerController, SoundboardMixer, VossMcCartneyPinkNoise,
};
pub use audio_streaming::{
    AudioCodec as MusicCodec, AudioDeliveryAction, AudioDeliveryOptimizer, CrossfadeCurve,
    EbuR128Normalizer, LoudnessProfile,
};
pub use audio_subsonic::{
    GetLicenseResponse, GetMusicFoldersResponse, GetNowPlayingResponse, GetSongResponse,
    MusicFoldersContainer, NowPlayingContainer, NowPlayingEntry, PingResponse,
    SUBSONIC_API_VERSION, SUBSONIC_SERVER_NAME, ScrobbleRequest, ScrobbleResponse, Search3Response,
    SearchResult3Container, SubsonicController, SubsonicError, SubsonicErrorPayload,
    SubsonicLicense, SubsonicMusicFolder, SubsonicRequestParams, SubsonicResponseEnvelope,
    SubsonicResponsePayload, SubsonicSong,
};
pub use audio_waveform::{
    AudioWaveformData, CommentCluster, SCRUB_PEAK_COUNT, THUMBNAIL_PEAK_COUNT, TimedComment,
    TimedCommentTimeline, WaveformGenerator,
};

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
pub use erp_cms_core as core;
pub use fast_channels::{
    ActiveProgramContext, EpgChannelGuide, EpgScheduleEntry, FastChannelError, FastChannelRegistry,
    FastChannelSchedule, LinearManifestEngine, LinearProgram,
};
pub use hls_engine::{
    HlsAudioRendition, HlsMasterPlaylist, HlsMediaPlaylist, HlsMediaSegment, HlsSegmentCache,
    HlsSubtitleRendition, HlsVariant,
};
pub use luxury_storefront::{
    LUXURY_STOREFRONT_CSS, LUXURY_STOREFRONT_HTML, LUXURY_STOREFRONT_JS, LuxuryProduct,
    get_luxury_catalog, render_luxury_storefront_html,
};
pub use print::render_invoice_html;
pub use resilient_streaming::{
    NetworkCondition, QuicStreamOptimizer, ResilientPlaylistEngine, UserCacheConfig,
    UserCacheRegistry, UserCacheStats, UserCacheTier,
};
pub use rtmp_ingest::{
    NginxRtmpPublishPayload, NginxRtmpRecordPayload, NginxRtmpUpdatePayload, RtmpError,
    RtmpIngestManager, RtmpStreamSession, RtmpStreamStatus,
};
pub use security::{HmacStreamingSigner, StreamingError, SvodPlaybackManager, parse_byte_range};
pub use seo_engine::{
    RssFeedItem, SeoMetadata, SitemapEntry, generate_dynamic_sitemap_xml, generate_robots_txt,
    generate_rss_2_xml, generate_sitemap_xml,
};
pub use ssai_engine::{
    AdCreative, AdPod, AdTrackingBeacons, HlsDateRange, Scte35CommandType, Scte35Cue,
    Scte35SpliceInsert, SsaiError, SsaiManifestStitcher, SsaiStitchResult, mpeg_crc32,
};
pub use storefront::{
    AtomicCheckoutEngine, CheckoutError, CheckoutItem, CheckoutResult, CustomerCheckoutRequest,
};
pub use stremio_addon::{
    StremioAddonEngine, StremioCatalogDefinition, StremioCatalogItem, StremioCatalogResponse,
    StremioExtraProp, StremioManifest, StremioMetaDetail, StremioMetaResponse, StremioStream,
    StremioStreamBehaviorHints, StremioStreamsResponse, StremioSubtitle, StremioSubtitlesResponse,
    StremioVideo,
};
pub use template_catalog::{
    TemplateSuite, get_template_suite, list_template_suites, render_template_html,
    render_template_html_with_variant, render_template_index_html, render_theme_switcher_dock,
};
pub use theme_engine::{
    CompiledTheme, DesignTokens, RenderEngineKind, SlotDefinition, ThemeManifest, ThemeRegistry,
    ThemeVariant, WorkTypeClassification,
};
pub use transcode_pipeline::{
    HardwareAccelType, ResumableChunkEngine, ResumableUploadSession, StoryboardTile,
    TranscodeError, TranscodeJob, TranscodeJobStatus, TranscodeQueue,
};
pub use video_streaming::{
    AudioChannels, AudioCodec, ClientDeviceProfile, ColorRange, MediaContainer, MediaStreamInfo,
    PlayMethod, PlaybackDecisionEngine, SubtitleDelivery, SubtitleFormat, SubtitleTrackInfo,
    VideoCodec,
};
pub use xray_metadata::{
    ActorCredit, MusicTrackInfo, ProductCommerceTag, SceneTrivia, SceneTriviaCategory,
    VideoXRayCatalog, XRayRegistry, XRaySnapshot, XRayTimeRange,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_to_edge_core_bridge() {
        let mut edge_layout: erp_cms_core::EdgeCanvasLayout<16> =
            erp_cms_core::EdgeCanvasLayout::new("Warehouse Display");
        let mut text = erp_cms_core::heapless::String::new();
        text.push_str("SKU-9923 Item Verified").unwrap();
        edge_layout
            .add_block(erp_cms_core::EdgeBlockNode::new(
                "b1",
                erp_cms_core::EdgeBlockType::Text { content: text },
            ))
            .unwrap();
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

    #[test]
    fn test_ssai_manifest_stitching_end_to_end() {
        let mut playlist = hls_engine::HlsMediaPlaylist::new(6, false);
        for i in 0..6 {
            playlist.add_segment(hls_engine::HlsMediaSegment {
                sequence_number: i,
                duration_seconds: 6.0,
                uri: format!("vod/seg_{i}.ts"),
                has_discontinuity: false,
            });
        }

        let mut ad_pod = AdPod::new("pod-mid-99", 15.0);
        ad_pod.add_creative(AdCreative::preset_15s("ad-audi", "Audi"));

        let cue = Scte35Cue::new_splice_insert_cue_out(99, Scte35Cue::seconds_to_pts(18.0), 15.0);
        let (stitched, result) =
            SsaiManifestStitcher::stitch_midroll(&playlist, 18.0, &ad_pod, Some(&cue))
                .expect("stitch midroll failed");

        assert_eq!(result.inserted_ad_segments_count, 3);
        assert_eq!(result.discontinuities_injected, 2);

        let manifest = SsaiManifestStitcher::compile_manifest(&stitched, &result.daterange_tags);
        assert!(manifest.contains("#EXT-X-DATERANGE:ID=\"ad-pod-midroll-pod-mid-99\""));
        assert!(manifest.contains("ads/ad-audi/seg_0.ts"));
        assert!(manifest.contains("#EXT-X-DISCONTINUITY"));
    }
}
