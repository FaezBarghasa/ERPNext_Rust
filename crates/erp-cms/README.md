# `erp-cms` — Enterprise Visual Block Canvas, OTT Video & Audio Streaming CMS

High-performance, local-first Content Management System, Visual Canvas Page Builder, OTT Video Delivery Network, and Multi-Service Audio Streaming Subsystem (Spotify, SoundCloud, Audioserve, Owncast, Navidrome & Procedural Soundscapes).

---

## 📦 Overview

`erp-cms` powers the content, visual assembly, and rich media streaming layers of the `rustnext` platform. It is engineered with zero-allocation compiled SSR, sub-10ms TTFB, hardware-accelerated video/audio transcoding pipelines, standards-compliant streaming protocols, and Subsonic API compatibility.

---

## 🏛 Major Architectural Subsystems

### 1. Visual Block Canvas AST & Emdash Studio
- **Declarative Tree AST (`builder_core.rs`)**: Reversible block graph supporting slot containers, responsive breakpoint cascade (`Mobile`, `Tablet`, `Desktop`, `Wide`), and dynamic two-way SurrealQL DocType data bindings (`DocTypeDataBinding`).
- **Compiled SSR Engine (`block_canvas.rs`)**: Pre-allocated string rendering engine compiling canvas trees into clean HTML with zero hydration slop in $<10\text{ms}$.
- **Emdash Editorial Workflow (`emdash_studio.rs`)**: High-volume publishing suite with multi-channel draft states (`Draft`, `Review`, `Scheduled`, `Published`, `Archived`), unified revisions diffing, focal-point smart cropping, and responsive WebP variant generation.

### 2. OTT Video Streaming, SSAI & FAST Channels
- **HLS v6 & CMAF Engine (`hls_engine.rs`)**: Master playlist compiler (`HlsMasterPlaylist`) with multi-bitrate video variants, separate multilingual audio tracks, WebVTT subtitle tracks, and byte-range segment slicing.
- **Server-Side Ad Insertion (`ssai_engine.rs`)**: SCTE-35 Cue-In / Cue-Out manifest stitcher with custom splice countdowns, date-range tags, and tracking beacons (`start`, `firstQuartile`, `midpoint`, `thirdQuartile`, `complete`).
- **FAST Linear Channels (`fast_channels.rs`)**: Programmatic linear 24/7 channels with EPG schedules, sliding-window live manifests, and cyclic program looping.
- **Stremio Addon Protocol (`stremio_addon.rs`)**: Full Stremio v1 addon compatibility delivering `manifest.json`, catalogs, metadata details, video streams, and external subtitle tracks.
- **Video X-Ray Metadata (`xray_metadata.rs`)**: Amazon Prime Video-style contextual overlays reporting active actors, music tracks, scene trivia, and commerce tags synchronized with current playback timestamps.
- **Resilient Low-Bandwidth Transport (`resilient_streaming.rs`)**: QUIC / HTTP/3 stream priority optimizations, predictive client prefetch caching (`UserCacheRegistry`), and multi-tier segment degradation.

### 3. Master Audio Streaming CMS (Music, Podcasts, Audiobooks, Radio, Sleep)
- **Lossless Ingestion & Compression (`audio_flac_pipeline.rs`)**: Automatic lossless conversion to FLAC with compression level selection (0..8), 4-tier audiophile quality grading (`CdQuality`, `StudioMaster24`, `HiResAudiophile`, `TranscodedFromLossy`), and VorbisComment metadata preservation.
- **10-Band ISO Equalizer (`audio_dsp_eq.rs`)**: Biquad IIR filtering across 10 ISO standard center frequencies (31 Hz – 16 kHz), automatic preamp headroom attenuation to prevent digital clipping, and 10 acoustic presets (`Flat`, `BassBoost`, `VocalBoost`, `Electronic`, `Acoustic`, `Rock`, `Classical`, `PodcastSpeech`, `SleepRelax`).
- **Synchronized Lyrics & Karaoke (`audio_lyrics.rs`)**: Dual parser for line-synchronized and syllable/word-level karaoke LRC files (`<mm:ss.xx>`), with millisecond active playhead query (`get_active_snapshot`).
- **Music Intelligence & Semantic Recommendations (`audio_intelligence.rs`)**: Technical liner notes (ISRC, BPM, Camelot wheel, contributors), Camelot harmonic key distance scoring, lyric transcription TF-IDF vector cosine similarity, and harmonic transition ranking.
- **EBU R128 Loudness & Gapless Crossfade (`audio_streaming.rs`)**: Integrated -14 LUFS loudness normalization with -1 dBFS true-peak limiter clamp, and crossfade transition curves (`EqualPower`, `Linear`, `Logarithmic`).
- **SoundCloud-Grade Waveforms & Timed Comments (`audio_waveform.rs`)**: 100-point thumbnail and 1800-point scrub bar peak downsampling, mirror SVG vector rendering, and temporal comment clustering histogram.
- **Procedural Sleep Soundscapes & Ambient Mixer (`audio_sleep_noise.rs`)**: Algorithmic white noise, Voss-McCartney $1/f$ pink noise, Brownian $1/f^2$ leaky walk, binaural beats synthesizer, multi-stem soundboard mixer, and exponential sleep timer decay curve ($V(t) = V_0 \cdot (1 - t/T)^2$).
- **Owncast-Grade Live Radio & Presence (`audio_live_radio.rs`)**: Low-latency HLS audio playlist generator (`#EXT-X-TARGETDURATION:2`), real-time listener heartbeat tracker, and live listener chat room.
- **Audioserve-Grade Chapters & Bookmarks (`audio_book_sync.rs`)**: Audiobook chapter markers (M4B, ID3v2 CHAP) and Last-Write-Wins (LWW) cross-device playback position synchronization.
- **Subsonic / OpenSubsonic API v1.16+ Protocol (`audio_subsonic.rs`)**: Complete compatibility with Subsonic ecosystem clients (Symfonium, DSub, Amperfy, Feishin) covering token authentication (`u`, `t`, `s`), `ping.view`, `getLicense.view`, `getSong.view`, `stream.view`, `getMusicFolders.view`, `scrobble.view`, `getNowPlaying.view`, and `search3.view`.

### 4. 10 Enterprise Template Suites & 10 Aesthetic Archetypes (100 Permutations)
- **10 Vertical Template Suites (`template_catalog.rs`)**:
  1. `svod-streaming`: Cinema 4K streaming portal with season selectors and tokenized DRM playback.
  2. `lms-academy`: Course curriculum, lesson progress, interactive quizzes, and Merkle certificates.
  3. `digital-goods`: Digital creator storefront with signed software licenses and instant asset vaults.
  4. `b2b-industrial`: Corporate supply portal with RFQ matrices, Net-60 terms, and 3D CAD viewer.
  5. `b2c-retail`: Massive SKU catalog with facet filtering, flash sales, and cart sheet drawer.
  6. `trading-exchange`: Sub-millisecond financial desk with Level-2 order books and latency monitors.
  7. `real-estate-luxury`: Architectural property showcase with virtual tour maps and mortgage calculator.
  8. `healthcare-clinic`: Telehealth portal with doctor booking, triage calendar, and clinical services.
  9. `restaurant-culinary`: Fine dining bistro with interactive menus, table reservation, and wine list.
  10. `news-editorial`: High-density newsroom magazine with breaking tickers, author columns, and editions.
- **10 Aesthetic Themes (`theme_engine.rs`)**:
  `awwwards`, `cyberpunk`, `vaporwave`, `retrowave`, `neonwave`, `tasteful`, `brutalist`, `editorial`, `luxury`, `monochrome`.
- **Runtime Hot-Swapping (`?variant={theme}`)**: Instant switching via URL parameter or floating dock widget with $<1\text{ms}$ stylesheet recompilation and zero layout shift.

---

## 🛠 Verification & Quality Assurance

```bash
# Run all 114 unit and integration tests in erp-cms
cargo test -p erp-cms

# Strict clippy check with zero warnings
cargo clippy -p erp-cms -- -D warnings

# Format check
cargo fmt --check
```
