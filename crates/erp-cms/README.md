# erp-cms

Secure token-authenticated media streaming, timestamped subtitle parsing, and video transcoder management.

---

## 📦 Overview

`erp-cms` manages content delivery, HMAC-signed media access URLs, subtitles, and transcode workflows.

### Key Capabilities

- **Secure Media Delivery (`security.rs`)**: HMAC-SHA256 URL token signing with time-to-live (TTL) expiration.
- **Subtitle Processing (`subtitles.rs`)**: WebVTT and SRT subtitle parsing and timestamp synchronization.
- **Transcoding Management (`transcoder.rs`)**: Async media format conversions and rendition generation.
