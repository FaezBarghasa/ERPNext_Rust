//! Integration Tests for QUIC Transport & HTTP/3 Streaming Engine (`frappe-net::quic_h3_stream`).

use actix_web::{App, test, web};
use frappe_net::quic_h3_stream::{
    H3Frame, H3Settings, QpackCodec, QpackField, QuicConnectionId, QuicH3StreamingEngine,
    QuicStreamType, VarInt,
};
use frappe_net::tenant::{ConnectionPoolManager, MicroTopologyConfig};
use frappe_net::{configure_app, quic_status_handler};
use std::net::SocketAddr;

#[actix_rt::test]
async fn test_quic_status_http_endpoint() {
    let app = test::init_service(
        App::new().route("/api/v2/quic/status", web::get().to(quic_status_handler)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v2/quic/status")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");
    assert_eq!(json["status"], "ACTIVE");
    assert_eq!(json["transport"], "QUIC (RFC 9000) / HTTP/3 (RFC 9114)");
    assert_eq!(json["zero_hol_blocking"], true);
    assert_eq!(json["alt_svc_advertised_port"], 4433);
}

#[test]
fn test_quic_varint_boundary_encodings() {
    // RFC 9000 1-byte, 2-byte, 4-byte, 8-byte boundaries
    let boundaries = [
        (0u64, 1),
        (63u64, 1),
        (64u64, 2),
        (16383u64, 2),
        (16384u64, 4),
        (1073741823u64, 4),
        (1073741824u64, 8),
        (4611686018427387903u64, 8),
    ];

    for (val, expected_len) in boundaries {
        let mut buf = Vec::new();
        VarInt::encode(val, &mut buf).expect("Encode succeeds");
        assert_eq!(
            buf.len(),
            expected_len,
            "Expected len {expected_len} for {val}"
        );

        let (decoded, consumed) = VarInt::decode(&buf).expect("Decode succeeds");
        assert_eq!(decoded, val);
        assert_eq!(consumed, expected_len);
    }
}

#[test]
fn test_h3_settings_and_qpack_compression_roundtrip() {
    let settings = H3Settings {
        max_field_section_size: 131072,
        qpack_max_table_capacity: 8192,
        qpack_blocked_streams: 32,
        enable_connect_protocol: true,
        enable_h3_datagrams: true,
    };

    let settings_frame = H3Frame::Settings(settings);
    let serialized = settings_frame.serialize();

    let (deserialized, consumed) =
        H3Frame::deserialize(&serialized).expect("Deserialization succeeds");
    assert_eq!(consumed, serialized.len());
    match deserialized {
        H3Frame::Data(_) => panic!("Expected H3Frame::Settings"),
        _ => {}
    }

    // QPACK Header Section with custom & static fields
    let headers = vec![
        QpackField::new(":status", "200"),
        QpackField::new(":method", "GET"),
        QpackField::new(":path", "/files/sha256-48a201f"),
        QpackField::new("content-type", "video/mp4"),
        QpackField::new("x-stream-priority", "high"),
    ];

    let qpack_bytes = QpackCodec::encode_headers(&headers);
    let decoded = QpackCodec::decode_headers(&qpack_bytes).expect("QPACK decoding succeeds");

    assert_eq!(decoded.len(), 5);
    assert_eq!(decoded[0].name, ":status");
    assert_eq!(decoded[0].value, "200");
    assert_eq!(decoded[3].name, "content-type");
    assert_eq!(decoded[3].value, "video/mp4");
    assert_eq!(decoded[4].name, "x-stream-priority");
    assert_eq!(decoded[4].value, "high");
}

#[test]
fn test_quic_h3_streaming_chunked_video_and_event_push() {
    let addr: SocketAddr = "192.168.1.150:4433".parse().unwrap();
    let mut engine = QuicH3StreamingEngine::new(addr);

    // 1. Client Bidirectional Video Streaming Session
    let stream_id = engine.create_bidirectional_stream();
    assert_eq!(
        QuicStreamType::from_stream_id(stream_id),
        QuicStreamType::ServerBidirectional
    );

    // Simulated 256 KB video stream sliced into 64 KB HTTP/3 DATA frames
    let video_bytes = vec![0x7E; 256 * 1024];
    let frames = engine
        .stream_file_asset_h3(stream_id, &video_bytes, 64 * 1024)
        .expect("Streaming succeeds");

    assert_eq!(frames.len(), 4);
    assert_eq!(engine.total_bytes_sent, 256 * 1024);

    // 2. Unidirectional Real-Time Live Push Event
    let push_id = engine.create_unidirectional_push_stream();
    assert_eq!(
        QuicStreamType::from_stream_id(push_id),
        QuicStreamType::ServerUnidirectional
    );

    let event_payload =
        br#"{"doctype":"SalesOrder","docname":"SO-2026-0901","status":"Submitted"}"#;
    let push_frame = engine
        .push_live_event_h3(push_id, "doc_mutation", event_payload)
        .expect("Push succeeds");

    let (frame, _) = H3Frame::deserialize(&push_frame).expect("Deserialization succeeds");
    match frame {
        H3Frame::Data(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            assert!(text.starts_with("doc_mutation:"));
            assert!(text.contains("SO-2026-0901"));
        }
        _ => panic!("Expected H3Frame::Data"),
    }

    // 3. Connection Migration Handover Verification
    let mobile_wifi_addr: SocketAddr = "10.0.1.200:58100".parse().unwrap();
    engine.handle_connection_migration(mobile_wifi_addr);
    assert_eq!(engine.peer_addr, mobile_wifi_addr);
}

#[actix_rt::test]
async fn test_full_app_alt_svc_and_h3_stream_routes() {
    let pool_mgr = ConnectionPoolManager::new();
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr, topology, None)),
    )
    .await;

    // Verify /api/v2/quic/status
    let req = test::TestRequest::get()
        .uri("/api/v2/quic/status")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify /api/v2/stream/h3/telemetry
    let req = test::TestRequest::post()
        .uri("/api/v2/stream/h3/telemetry")
        .set_json(serde_json::json!({
            "topic": "amr_robot_telemetry",
            "vehicle_id": "AMR-04",
            "battery_pct": 94.5,
            "status": "InTransit"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert!(resp.headers().contains_key("alt-svc"));
    assert!(resp.headers().contains_key("x-h3-push-id"));
}
