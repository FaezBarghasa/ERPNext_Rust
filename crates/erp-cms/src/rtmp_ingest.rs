//! RTMP Broadcast Ingest & Webhook Lifecycle Engine (`erp-cms::rtmp_ingest`).
//!
//! Synthesizes Open-Streaming-Platform (OSP) NGINX-RTMP webhook governance:
//! - Broadcaster authentication on `on_publish` via stream key.
//! - Active stream session state tracking and live channel metadata.
//! - Broadcast teardown on `on_publish_done`.
//! - Live-to-VOD archival remuxing pipeline triggering on `on_record_done`.

use chrono::{DateTime, Utc};
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

/// Active or archived RTMP broadcast session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RtmpStreamSession {
    pub channel_id: String,
    pub stream_key: String,
    pub status: RtmpStreamStatus,
    pub client_ip: String,
    pub ingress_bitrate_kbps: u32,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub recording_file: Option<String>,
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
            started_at: Some(Utc::now()),
            ended_at: None,
            recording_file: None,
        };

        sessions.insert(stream_key.to_string(), session.clone());
        Ok(session)
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

    /// Returns the session status for a given stream key.
    pub fn get_session(&self, stream_key: &str) -> Result<Option<RtmpStreamSession>, RtmpError> {
        let sessions = self.active_sessions.read().map_err(|_| RtmpError::LockError)?;
        Ok(sessions.get(stream_key).cloned())
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
    }
}
