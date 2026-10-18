//! Integration Tests for QUIC Streaming, Resilient Low-Speed / Lossy Profiles, and User Caching.

use actix_web::{App, test, web};
use frappe_net::v2_routes::{
    DynamicRbacState, streaming_cache_configure_handler, streaming_cache_prefetch_handler,
    streaming_cache_stats_handler, streaming_h3_segment_handler, streaming_master_playlist_handler,
    streaming_resilient_master_playlist_handler, streaming_segment_handler,
    streaming_variant_playlist_handler,
};

#[actix_web::test]
async fn test_quic_alt_svc_on_standard_streaming_endpoints() {
    let state = web::Data::new(DynamicRbacState::default());

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .route(
                "/api/v2/streaming/videos/{id}/master.m3u8",
                web::get().to(streaming_master_playlist_handler),
            )
            .route(
                "/api/v2/streaming/videos/{id}/{variant}/index.m3u8",
                web::get().to(streaming_variant_playlist_handler),
            )
            .route(
                "/api/v2/streaming/videos/{id}/{variant}/{segment}",
                web::get().to(streaming_segment_handler),
            ),
    )
    .await;

    // 1. Master playlist should advertise Alt-Svc: h3
    let req = test::TestRequest::get()
        .uri("/api/v2/streaming/videos/vid-test-01/master.m3u8")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let alt_svc = resp
        .headers()
        .get("Alt-Svc")
        .expect("Alt-Svc must be present")
        .to_str()
        .unwrap();
    assert!(alt_svc.contains("h3=\":4433\""));

    // 2. Variant playlist should advertise Alt-Svc: h3
    let req2 = test::TestRequest::get()
        .uri("/api/v2/streaming/videos/vid-test-01/1080p/index.m3u8")
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert!(resp2.status().is_success());
    assert!(
        resp2
            .headers()
            .get("Alt-Svc")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("h3=\":4433\"")
    );

    // 3. Segment should advertise Alt-Svc, Priority, Cache-Control: immutable, and X-Prefetch-Segments
    let req3 = test::TestRequest::get()
        .uri("/api/v2/streaming/videos/vid-test-01/1080p/seg_0.ts")
        .to_request();
    let resp3 = test::call_service(&app, req3).await;
    assert!(resp3.status().is_success());
    let headers = resp3.headers();
    assert!(
        headers
            .get("Alt-Svc")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("h3=\":4433\"")
    );
    assert!(headers.get("Priority").is_some());
    assert!(headers.get("X-Prefetch-Segments").is_some());
    assert_eq!(
        headers.get("Cache-Control").unwrap().to_str().unwrap(),
        "public, max-age=31536000, immutable"
    );
}

#[actix_web::test]
async fn test_resilient_master_playlist_for_lossy_and_low_speed_networks() {
    let state = web::Data::new(DynamicRbacState::default());

    let app = test::init_service(App::new().app_data(state.clone()).route(
        "/api/v2/streaming/videos/{id}/resilient.m3u8",
        web::get().to(streaming_resilient_master_playlist_handler),
    ))
    .await;

    // Test lossy network profile
    let req_lossy = test::TestRequest::get()
        .uri("/api/v2/streaming/videos/vid-lossy-01/resilient.m3u8?profile=lossy&loss=15.0")
        .to_request();
    let resp_lossy = test::call_service(&app, req_lossy).await;
    assert!(resp_lossy.status().is_success());

    let body_lossy = test::read_body(resp_lossy).await;
    let manifest_str = String::from_utf8(body_lossy.to_vec()).unwrap();
    assert!(manifest_str.contains("#EXT-X-INDEPENDENT-SEGMENTS"));
    // Under lossy profile, 180p-lossy (120kbps) must be prioritized
    assert!(manifest_str.contains("180p/index.m3u8"));
    assert!(manifest_str.contains("BANDWIDTH=120000"));

    // Test low-speed network profile
    let req_low_speed = test::TestRequest::get()
        .uri("/api/v2/streaming/videos/vid-low-01/resilient.m3u8?profile=low_speed&downlink_mbps=0.25")
        .to_request();
    let resp_low_speed = test::call_service(&app, req_low_speed).await;
    assert!(resp_low_speed.status().is_success());

    let body_low_speed = test::read_body(resp_low_speed).await;
    let manifest_str2 = String::from_utf8(body_low_speed.to_vec()).unwrap();
    // Under low speed profile, 240p-lowspeed (260kbps) must be at the top
    assert!(manifest_str2.contains("240p/index.m3u8"));
    assert!(manifest_str2.contains("BANDWIDTH=260000"));
}

#[actix_web::test]
async fn test_user_cache_configuration_and_stats() {
    let state = web::Data::new(DynamicRbacState::default());

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .route(
                "/api/v2/streaming/cache/configure",
                web::post().to(streaming_cache_configure_handler),
            )
            .route(
                "/api/v2/streaming/cache/stats",
                web::get().to(streaming_cache_stats_handler),
            )
            .route(
                "/api/v2/streaming/cache/prefetch",
                web::post().to(streaming_cache_prefetch_handler),
            ),
    )
    .await;

    // Configure client for Increased cache tier
    let config_payload = serde_json::json!({
        "client_id": "user_flaky_cell_99",
        "tier": "increased",
        "max_cache_mb": 350,
        "prefetch_depth": 12
    });

    let req = test::TestRequest::post()
        .uri("/api/v2/streaming/cache/configure")
        .set_json(&config_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "success");
    assert_eq!(json["client_id"], "user_flaky_cell_99");
    assert_eq!(json["tier"], "Increased");
    assert_eq!(json["max_cache_mb"], 350);
    assert_eq!(json["prefetch_depth"], 12);
    assert_eq!(json["burst_prefetch_enabled"], true);

    // Calculate forward prefetch segments for this client
    let prefetch_payload = serde_json::json!({
        "client_id": "user_flaky_cell_99",
        "video_id": "vid-action-movie",
        "variant": "720p",
        "current_sequence": 5,
        "max_sequence": 50
    });

    let req_prefetch = test::TestRequest::post()
        .uri("/api/v2/streaming/cache/prefetch")
        .set_json(&prefetch_payload)
        .to_request();
    let resp_prefetch = test::call_service(&app, req_prefetch).await;
    assert!(resp_prefetch.status().is_success());

    let prefetch_body = test::read_body(resp_prefetch).await;
    let prefetch_json: serde_json::Value = serde_json::from_slice(&prefetch_body).unwrap();
    assert_eq!(prefetch_json["status"], "success");
    assert_eq!(prefetch_json["prefetch_depth"], 12);
    assert_eq!(prefetch_json["quic_stream_priority"], "u=4, i");
    let segments = prefetch_json["segments"].as_array().unwrap();
    assert_eq!(segments.len(), 12);
    assert_eq!(segments[0], "seg_6.ts");
    assert_eq!(segments[11], "seg_17.ts");

    // Check stats
    let req_stats = test::TestRequest::get()
        .uri("/api/v2/streaming/cache/stats")
        .to_request();
    let resp_stats = test::call_service(&app, req_stats).await;
    assert!(resp_stats.status().is_success());

    let stats_body = test::read_body(resp_stats).await;
    let stats_json: serde_json::Value = serde_json::from_slice(&stats_body).unwrap();
    assert_eq!(stats_json["status"], "success");
    assert_eq!(stats_json["total_configured_users"], 1);
    assert!(stats_json["total_prefetch_batches"].as_u64().unwrap() >= 1);
}

#[actix_web::test]
async fn test_quic_h3_video_segment_streaming() {
    let state = web::Data::new(DynamicRbacState::default());

    let app = test::init_service(App::new().app_data(state.clone()).route(
        "/api/v2/streaming/h3/segment/{id}/{variant}/{segment}",
        web::get().to(streaming_h3_segment_handler),
    ))
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v2/streaming/h3/segment/vid-h3-01/360p/seg_0.ts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let headers = resp.headers();
    assert_eq!(
        headers.get("Content-Type").unwrap().to_str().unwrap(),
        "application/octet-stream"
    );
    assert!(headers.get("X-QUIC-Stream-ID").is_some());
    assert_eq!(headers.get("Priority").unwrap().to_str().unwrap(), "u=1, i");
    assert!(
        headers
            .get("Alt-Svc")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("h3=\":4433\"")
    );

    let body = test::read_body(resp).await;
    // Body contains serialized HTTP/3 HEADERS and DATA frames
    assert!(!body.is_empty());
}
