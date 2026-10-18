//! RTMP Broadcast Ingest & Webhook Lifecycle Engine (`erp-cms::rtmp_ingest`).
//!
//! Synthesizes Open-Streaming-Platform (OSP) NGINX-RTMP webhook governance:
//! - Broadcaster authentication on `on_publish` via stream key.
//! - Active stream session state tracking, viewer counts, and live channel metadata.
//! - Broadcast teardown on `on_publish_done`.
//! - Live-to-VOD archival remuxing pipeline triggering on `on_record_done`.
//! - VOD asset catalog registration on `handle_archive_complete`.
//! - Zero-allocation `RtmpStreamTelemetryPacket` conversion for edge monitoring.

use chrono::{DateTime, Utc};
use erp_cms_core::RtmpStreamTelemetryPacket;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// RTMP Ingest Errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RtmpError {
    #[error("Invalid or unauthorized stream key: '{0}'")]
    UnauthorizedStreamKey(String),
    #[error("Stream session not found for key: '{0}'")]
    SessionNotFound(String),
    #[error("Channel '{0}' is already actively broadcasting")]
    ChannelAlreadyActive(String),
    #[error("Channel '{0}' not found in registry")]
    ChannelNotFound(String),
    #[error("Internal concurrency locking error")]
    LockError,
}

/// Broadcast stream lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RtmpStreamStatus {
    Idle,
    Live,
    Ended,
    Archiving,
    Archived,
}

/// NGINX-RTMP or SRS `on_publish` webhook form payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NginxRtmpPublishPayload {
    pub app: String,
    pub name: String,
    pub addr: String,
    #[serde(default)]
    pub clientid: Option<String>,
}

/// NGINX-RTMP or SRS `on_record_done` webhook form payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NginxRtmpRecordPayload {
    pub app: String,
    pub name: String,
    pub path: String,
}

/// Periodic status ping telemetry payload from RTMP ingest server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NginxRtmpUpdatePayload {
    pub name: String,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

/// Active or archived RTMP broadcast session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RtmpStreamSession {
    pub channel_id: String,
    pub stream_key: String,
    pub status: RtmpStreamStatus,
    pub client_ip: String,
    pub ingress_bitrate_kbps: u32,
    pub viewer_count: u32,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub recording_file: Option<String>,
    pub vod_asset_id: Option<String>,
    pub vod_url: Option<String>,
}

/// Manager coordinating RTMP stream keys, live session states, and webhook lifecycle hooks.
#[derive(Default, Clone)]
pub struct RtmpIngestManager {
    /// Stream key -> Channel ID mapping.
    registered_keys: Arc<RwLock<HashMap<String, String>>>,
    /// Stream key -> Active session metadata.
    active_sessions: Arc<RwLock<HashMap<String, RtmpStreamSession>>>,
}

impl RtmpIngestManager {
    /// Creates a new RTMP ingest manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an authorized stream key for a specific broadcast channel.
    pub fn register_channel(
        &self,
        channel_id: impl Into<String>,
        stream_key: impl Into<String>,
    ) -> Result<(), RtmpError> {
        let mut keys = self.registered_keys.write().map_err(|_| RtmpError::LockError)?;
        keys.insert(stream_key.into(), channel_id.into());
        Ok(())
    }

    /// Rotates the stream key for a given channel, revoking the previous one.
    pub fn rotate_stream_key(
        &self,
        channel_id: &str,
        new_stream_key: impl Into<String>,
    ) -> Result<String, RtmpError> {
        let mut keys = self.registered_keys.write().map_err(|_| RtmpError::LockError)?;
        let prev_key = keys
            .iter()
            .find(|(_, cid)| cid.as_str() == channel_id)
            .map(|(k, _)| k.clone());

        if let Some(old_k) = prev_key {
            keys.remove(&old_k);
        } else {
            return Err(RtmpError::ChannelNotFound(channel_id.to_string()));
        }

        let new_key_str = new_stream_key.into();
        keys.insert(new_key_str.clone(), channel_id.to_string());
        Ok(new_key_str)
    }

    /// Revokes an existing stream key immediately.
    pub fn revoke_stream_key(&self, stream_key: &str) -> Result<(), RtmpError> {
        let mut keys = self.registered_keys.write().map_err(|_| RtmpError::LockError)?;
        if keys.remove(stream_key).is_some() {
            Ok(())
        } else {
            Err(RtmpError::UnauthorizedStreamKey(stream_key.to_string()))
        }
    }

    /// Validates `on_publish` webhook request from RTMP ingest server (e.g. NGINX-RTMP).
    ///
    /// If stream key is valid, transitions channel to `Live` state.
    pub fn handle_on_publish(
        &self,
        stream_key: &str,
        client_ip: &str,
    ) -> Result<RtmpStreamSession, RtmpError> {
        let keys = self.registered_keys.read().map_err(|_| RtmpError::LockError)?;
        let channel_id = keys
            .get(stream_key)
            .ok_or_else(|| RtmpError::UnauthorizedStreamKey(stream_key.to_string()))?;

        let mut sessions = self.active_sessions.write().map_err(|_| RtmpError::LockError)?;
        if let Some(existing) = sessions.get(stream_key)
            && existing.status == RtmpStreamStatus::Live
        {
            return Err(RtmpError::ChannelAlreadyActive(channel_id.clone()));
        }

        let session = RtmpStreamSession {
            channel_id: channel_id.clone(),
            stream_key: stream_key.to_string(),
            status: RtmpStreamStatus::Live,
            client_ip: client_ip.to_string(),
            ingress_bitrate_kbps: 4500, // Standard default ingest bitrate
            viewer_count: 0,
            started_at: Some(Utc::now()),
            ended_at: None,
            recording_file: None,
            vod_asset_id: None,
            vod_url: None,
        };

        sessions.insert(stream_key.to_string(), session.clone());
        Ok(session)
    }

    /// Convenience handler parsing strongly typed `NginxRtmpPublishPayload`.
    pub fn handle_on_publish_payload(
        &self,
        payload: &NginxRtmpPublishPayload,
    ) -> Result<RtmpStreamSession, RtmpError> {
        self.handle_on_publish(&payload.name, &payload.addr)
    }

    /// Updates live stream telemetry (viewer count, ingress bitrate).
    pub fn update_telemetry(
        &self,
        stream_key: &str,
        viewer_count: u32,
        ingress_bitrate_kbps: u32,
    ) -> Result<RtmpStreamSession, RtmpError> {
        let mut sessions = self.active_sessions.write().map_err(|_| RtmpError::LockError)?;
        let session = sessions
            .get_mut(stream_key)
            .ok_or_else(|| RtmpError::SessionNotFound(stream_key.to_string()))?;

        session.viewer_count = viewer_count;
        session.ingress_bitrate_kbps = ingress_bitrate_kbps;
        Ok(session.clone())
    }

    /// Handles `on_publish_done` webhook when broadcaster stops streaming.
    pub fn handle_on_publish_done(&self, stream_key: &str) -> Result<RtmpStreamSession, RtmpError> {
        let mut sessions = self.active_sessions.write().map_err(|_| RtmpError::LockError)?;
        let session = sessions
            .get_mut(stream_key)
            .ok_or_else(|| RtmpError::SessionNotFound(stream_key.to_string()))?;

        session.status = RtmpStreamStatus::Ended;
        session.ended_at = Some(Utc::now());
        Ok(session.clone())
    }

    /// Handles `on_record_done` webhook when NGINX completes writing the raw broadcast file.
    ///
    /// Generates FFmpeg command parameters to remux into web-optimized faststart MP4.
    pub fn handle_on_record_done(
        &self,
        stream_key: &str,
        raw_recording_path: &str,
    ) -> Result<(RtmpStreamSession, String), RtmpError> {
        let mut sessions = self.active_sessions.write().map_err(|_| RtmpError::LockError)?;
        let session = sessions
            .get_mut(stream_key)
            .ok_or_else(|| RtmpError::SessionNotFound(stream_key.to_string()))?;

        session.status = RtmpStreamStatus::Archiving;
        session.recording_file = Some(raw_recording_path.to_string());

        // Faststart remux command without re-encoding video/audio
        let output_mp4 = format!("{raw_recording_path}.mp4");
        let remux_cmd = format!(
            "ffmpeg -i \"{}\" -c copy -movflags +faststart \"{}\"",
            raw_recording_path, output_mp4
        );

        Ok((session.clone(), remux_cmd))
    }

    /// Convenience handler parsing strongly typed `NginxRtmpRecordPayload`.
    pub fn handle_on_record_payload(
        &self,
        payload: &NginxRtmpRecordPayload,
    ) -> Result<(RtmpStreamSession, String), RtmpError> {
        self.handle_on_record_done(&payload.name, &payload.path)
    }

    /// Completes the archival lifecycle, transitioning to `Archived` and attaching VOD metadata.
    pub fn handle_archive_complete(
        &self,
        stream_key: &str,
        vod_asset_id: impl Into<String>,
        vod_url: impl Into<String>,
    ) -> Result<RtmpStreamSession, RtmpError> {
        let mut sessions = self.active_sessions.write().map_err(|_| RtmpError::LockError)?;
        let session = sessions
            .get_mut(stream_key)
            .ok_or_else(|| RtmpError::SessionNotFound(stream_key.to_string()))?;

        session.status = RtmpStreamStatus::Archived;
        session.vod_asset_id = Some(vod_asset_id.into());
        session.vod_url = Some(vod_url.into());
        Ok(session.clone())
    }

    /// Returns the live HLS playback manifest URL for an active stream.
    pub fn get_live_hls_manifest_url(
        &self,
        stream_key: &str,
        base_url: &str,
    ) -> Result<String, RtmpError> {
        let sessions = self.active_sessions.read().map_err(|_| RtmpError::LockError)?;
        let session = sessions
            .get(stream_key)
            .ok_or_else(|| RtmpError::SessionNotFound(stream_key.to_string()))?;

        let clean_base = base_url.trim_end_matches('/');
        Ok(format!("{clean_base}/live/{}/index.m3u8", session.channel_id))
    }

    /// Returns all currently active `Live` broadcast sessions.
    pub fn list_active_streams(&self) -> Result<Vec<RtmpStreamSession>, RtmpError> {
        let sessions = self.active_sessions.read().map_err(|_| RtmpError::LockError)?;
        Ok(sessions
            .values()
            .filter(|s| s.status == RtmpStreamStatus::Live)
            .cloned()
            .collect())
    }

    /// Returns the session status for a given stream key.
    pub fn get_session(&self, stream_key: &str) -> Result<Option<RtmpStreamSession>, RtmpError> {
        let sessions = self.active_sessions.read().map_err(|_| RtmpError::LockError)?;
        Ok(sessions.get(stream_key).cloned())
    }

    /// Converts active stream session telemetry into zero-allocation wire packet `RtmpStreamTelemetryPacket`.
    pub fn to_telemetry_packet(
        &self,
        stream_key: &str,
    ) -> Result<RtmpStreamTelemetryPacket, RtmpError> {
        let session = self
            .get_session(stream_key)?
            .ok_or_else(|| RtmpError::SessionNotFound(stream_key.to_string()))?;

        let mut channel_id = erp_cms_core::heapless::String::<32>::new();
        let c_bytes = session.channel_id.as_bytes();
        let c_len = c_bytes.len().min(32);
        if let Ok(c_str) = core::str::from_utf8(&c_bytes[..c_len]) {
            let _ = channel_id.push_str(c_str);
        }

        Ok(RtmpStreamTelemetryPacket {
            channel_id,
            ingress_bitrate_kbps: session.ingress_bitrate_kbps,
            viewer_count: session.viewer_count,
            is_live: session.status == RtmpStreamStatus::Live,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtmp_publish_lifecycle() {
        let manager = RtmpIngestManager::new();
        manager
            .register_channel("channel-main", "live_secret_key_8842")
            .unwrap();

        // 1. Rejection on unauthorized key
        let rejected = manager.handle_on_publish("invalid_key", "192.168.1.50");
        assert!(matches!(rejected, Err(RtmpError::UnauthorizedStreamKey(_))));

        // 2. Successful on_publish
        let live_session = manager
            .handle_on_publish("live_secret_key_8842", "192.168.1.50")
            .unwrap();
        assert_eq!(live_session.channel_id, "channel-main");
        assert_eq!(live_session.status, RtmpStreamStatus::Live);
        assert!(live_session.started_at.is_some());

        // 3. Prevent duplicate publish on already active channel
        let dup = manager.handle_on_publish("live_secret_key_8842", "192.168.1.51");
        assert!(matches!(dup, Err(RtmpError::ChannelAlreadyActive(_))));

        // 4. Successful on_publish_done teardown
        let ended_session = manager.handle_on_publish_done("live_secret_key_8842").unwrap();
        assert_eq!(ended_session.status, RtmpStreamStatus::Ended);
        assert!(ended_session.ended_at.is_some());

        // 5. Successful on_record_done post-processing
        let (archiving_session, remux_cmd) = manager
            .handle_on_record_done("live_secret_key_8842", "/var/recordings/stream_8842.flv")
            .unwrap();
        assert_eq!(archiving_session.status, RtmpStreamStatus::Archiving);
        assert!(remux_cmd.contains("-movflags +faststart"));
        assert!(remux_cmd.contains("/var/recordings/stream_8842.flv.mp4"));

        // 6. Complete archival into VOD asset
        let archived = manager
            .handle_archive_complete(
                "live_secret_key_8842",
                "vod_asset_77",
                "https://cdn.example.com/vod/vod_asset_77/master.m3u8",
            )
            .unwrap();
        assert_eq!(archived.status, RtmpStreamStatus::Archived);
        assert_eq!(archived.vod_asset_id.as_deref(), Some("vod_asset_77"));
    }

    #[test]
    fn test_stream_key_rotation_and_revocation() {
        let manager = RtmpIngestManager::new();
        manager
            .register_channel("channel-esports", "key_v1_old")
            .unwrap();

        // Rotate key
        let new_key = manager
            .rotate_stream_key("channel-esports", "key_v2_fresh")
            .unwrap();
        assert_eq!(new_key, "key_v2_fresh");

        // Old key rejected
        let res_old = manager.handle_on_publish("key_v1_old", "10.0.0.1");
        assert!(matches!(res_old, Err(RtmpError::UnauthorizedStreamKey(_))));

        // New key accepted
        let res_new = manager.handle_on_publish("key_v2_fresh", "10.0.0.1");
        assert!(res_new.is_ok());

        // Revoke key
        manager.revoke_stream_key("key_v2_fresh").unwrap();
        let res_revoked = manager.handle_on_publish("key_v2_fresh", "10.0.0.1");
        assert!(matches!(res_revoked, Err(RtmpError::UnauthorizedStreamKey(_))));
    }

    #[test]
    fn test_payload_handlers_and_telemetry_packet() {
        let manager = RtmpIngestManager::new();
        manager.register_channel("chan-gaming", "gamers_pass").unwrap();

        // 1. Publish with Nginx payload
        let pub_payload = NginxRtmpPublishPayload {
            app: "live".into(),
            name: "gamers_pass".into(),
            addr: "172.16.0.4".into(),
            clientid: Some("conn_901".into()),
        };
        let session = manager.handle_on_publish_payload(&pub_payload).unwrap();
        assert_eq!(session.client_ip, "172.16.0.4");

        // 2. Telemetry update
        manager.update_telemetry("gamers_pass", 1542, 6000).unwrap();

        // 3. Export to no_std telemetry packet
        let packet = manager.to_telemetry_packet("gamers_pass").unwrap();
        assert_eq!(packet.channel_id.as_str(), "chan-gaming");
        assert_eq!(packet.viewer_count, 1542);
        assert_eq!(packet.ingress_bitrate_kbps, 6000);
        assert!(packet.is_live);

        // 4. Wire serialize packet
        let mut buf = [0u8; 45];
        let bytes = packet.serialize(&mut buf).unwrap();
        assert_eq!(&bytes[0..4], b"RTMP");

        // 5. Query live HLS URL
        let hls_url = manager
            .get_live_hls_manifest_url("gamers_pass", "https://stream.erp.local/api/v2")
            .unwrap();
        assert_eq!(hls_url, "https://stream.erp.local/api/v2/live/chan-gaming/index.m3u8");

        // 6. Record payload
        let rec_payload = NginxRtmpRecordPayload {
            app: "live".into(),
            name: "gamers_pass".into(),
            path: "/tmp/rec_gaming.flv".into(),
        };
        let (archiving_session, cmd) = manager.handle_on_record_payload(&rec_payload).unwrap();
        assert_eq!(archiving_session.status, RtmpStreamStatus::Archiving);
        assert!(cmd.contains("/tmp/rec_gaming.flv.mp4"));
    }
}
