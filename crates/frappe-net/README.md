# `frappe-net` — Multi-Tenant Transport, Real-Time Streaming & Gateway Router

Production Actix-Web, QUIC/HTTP3, and WebSocket gateway providing multi-tenant routing, dynamic security, video & audio streaming, and Subsonic API endpoints.

---

## 📦 Overview

`frappe-net` implements the primary transport layer for the `rustnext` platform. It features multi-tenant resolution across subdomains, strict bitemporal audit trails, dynamic Role-Based Access Control (RBAC), multi-factor authentication (MFA), real-time LiveSync event broadcasting, and high-performance streaming APIs.

---

## 🏛 Key Network & Architecture Capabilities

### 1. Transport Protocols & Routing
- **Actix-Web HTTP/1.1 & HTTP/2**: High-throughput REST and RPC endpoints handling document lifecycles, metadata reflection, printable documents, and payment processing.
- **QUIC & HTTP/3 Substrate (`quic_h3_stream.rs`)**: Low-latency transport built on `quinn` and `h3` with QPACK header compression, 0-RTT handshakes, and resilient streaming optimizations.
- **LiveSync WebSockets (`live.rs`)**: Scalable in-memory event broadcast hub streaming live document mutations and notifications to connected clients.
- **Automated ACME Gateway**: In-process TLS certificate negotiation (Let's Encrypt) and automatic HTTPS redirects.

### 2. Dynamic Enterprise Security & RBAC (`v2_routes.rs`)
- **Dynamic Role-Based Access Control**: In-memory role permission registry with granular permlevels (0..9) and per-DocType CRUD / Submit / Amend permissions.
- **Multi-Factor Authentication (MFA)**: TOTP enrollment, validation, and emergency backup codes.
- **LoginGuard & IP Filtering**: Rate-limiting token bucket algorithms, brute-force lockout defenses, and dynamic CIDR IP rule registry.
- **Audit Trail Ledger**: Immutable structured audit logs recording actor, IP, timestamp, and target document state.

### 3. Media & Audio Streaming Endpoints
- **OTT Video & HLS Delivery**:
  - `GET /api/v2/streaming/hls/{media_id}/master.m3u8`
  - `GET /api/v2/streaming/hls/{media_id}/variants/{variant}/playlist.m3u8`
  - `GET /api/v2/streaming/hls/{media_id}/segments/{segment}`
  - `GET /api/v2/streaming/fast/channels/{channel_id}/live.m3u8`
  - `GET /api/v2/streaming/stremio/manifest.json`
- **High-Level Audio CMS Endpoints**:
  - `POST /api/v2/audio/normalize` (EBU R128 loudness normalizer)
  - `POST /api/v2/audio/crossfade` (Gapless crossfading)
  - `POST /api/v2/audio/waveform` (Peak downsampling & SVG waveforms)
  - `POST /api/v2/audio/flac/compress` (Lossless FLAC transcode planner)
  - `POST /api/v2/audio/eq/filter` (10-Band EQ & presets filter generator)
  - `POST /api/v2/audio/lyrics/parse` & `POST /api/v2/audio/lyrics/current` (Line & Karaoke LRC)
  - `POST /api/v2/audio/music/info` & `POST /api/v2/audio/recommend` (Liner notes & transcription AI recommendations)
  - `POST /api/v2/audio/sleep/soundboard` & `POST /api/v2/audio/sleep/timer` (Procedural noise & decay timer)
  - `GET /api/v2/audio/radio/{station_id}/live.m3u8` (Live radio LL-HLS audio)
  - `POST /api/v2/audio/radio/{station_id}/heartbeat` & `GET /api/v2/audio/radio/{station_id}/listeners` (Presence)
  - `POST /api/v2/audio/radio/{station_id}/chat` & `GET /api/v2/audio/radio/{station_id}/chat` (Live chat)
  - `GET /api/v2/audio/podcasts/{media_id}/chapters` (Audiobook chapters)
  - `POST /api/v2/audio/playback/bookmark` & `GET /api/v2/audio/playback/bookmark/{user_id}/{media_id}` (LWW cross-device bookmark sync)
- **Subsonic / OpenSubsonic API v1.16+ Protocol (`/rest/*.view`)**:
  - `GET /rest/ping.view`
  - `GET /rest/getLicense.view`
  - `GET /rest/getSong.view`
  - `GET /rest/stream.view`
  - `GET /rest/getMusicFolders.view`
  - `GET /rest/scrobble.view`
  - `GET /rest/getNowPlaying.view`
  - `GET /rest/search3.view`

---

## 🛠 Verification

```bash
# Run all unit tests and integration suites
cargo test -p frappe-net

# Run dedicated API v2 integration tests
cargo test -p frappe-net --test v2_api_integration_tests

# Strict clippy check with zero warnings
cargo clippy -p frappe-net -- -D warnings
```
