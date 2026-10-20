//! HLS / RFC 8216 Video Streaming, RTMP Ingest, ABR Transcode, SSAI, and QUIC H3 Cache routes.

use super::*;
use actix_web::web;

/// Configures all Video Streaming, HLS, RTMP, and QUIC H3 Cache endpoints.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/api/v2/streaming/videos/{id}/master.m3u8",
        web::get().to(streaming_master_playlist_handler),
    )
    .route(
        "/api/v2/streaming/videos/{id}/variant_{quality}.m3u8",
        web::get().to(streaming_variant_playlist_handler),
    )
    .route(
        "/api/v2/streaming/videos/{id}/segment_{quality}_{idx}.ts",
        web::get().to(streaming_segment_handler),
    )
    .route(
        "/api/v2/streaming/rtmp/auth",
        web::post().to(streaming_rtmp_auth_handler),
    )
    .route(
        "/api/v2/streaming/rtmp/done",
        web::post().to(streaming_rtmp_done_handler),
    )
    .route(
        "/api/v2/streaming/playback/progress",
        web::post().to(streaming_playback_progress_handler),
    )
    .route(
        "/api/v2/streaming/videos/{id}/chunk",
        web::post().to(streaming_upload_chunk_handler),
    )
    .route(
        "/api/v2/streaming/resilient/videos/{id}/master.m3u8",
        web::get().to(streaming_resilient_master_playlist_handler),
    )
    .route(
        "/api/v2/streaming/cache/configure",
        web::post().to(streaming_cache_configure_handler),
    )
    .route(
        "/api/v2/streaming/cache/prefetch",
        web::post().to(streaming_cache_prefetch_handler),
    )
    .route(
        "/api/v2/streaming/cache/stats",
        web::get().to(streaming_cache_stats_handler),
    )
    .route(
        "/api/v2/streaming/fast/status",
        web::get().to(streaming_fast_status_handler),
    )
    .route(
        "/api/v2/streaming/fast/channel/{channel_id}/live.m3u8",
        web::get().to(streaming_fast_live_manifest_handler),
    )
    .route(
        "/api/v2/streaming/fast/channel/{channel_id}/epg",
        web::get().to(streaming_fast_epg_handler),
    )
    .route(
        "/api/v2/stream/h3/segment/{id}",
        web::get().to(streaming_h3_segment_handler),
    )
    .route(
        "/api/v2/stream/h3/file/{hash}",
        web::get().to(h3_stream_file_handler),
    )
    .route(
        "/api/v2/stream/h3/telemetry",
        web::post().to(h3_stream_telemetry_handler),
    )
    .route("/api/v2/quic/status", web::get().to(quic_status_handler));
}
