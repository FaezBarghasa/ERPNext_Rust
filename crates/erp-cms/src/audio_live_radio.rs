//! Live Radio Broadcasting, LL-HLS Audio & Owncast-Style Listener Presence Engine
//!
//! Provides live radio station registry, low-latency audio HLS playlist generation
//! (1-2s segments), real-time listener presence & heartbeat tracking, and live
//! listener chat engagement.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Live radio station metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadioStation {
    pub id: String,
    pub name: String,
    pub description: String,
    pub genre: String,
    pub current_track: Option<String>,
    pub current_artist: Option<String>,
    pub mount_point: String,
    pub sample_rate: u32,
    pub bitrate_kbps: u32,
    pub is_online: bool,
}

/// A single live chat message for live radio broadcasts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadioChatMessage {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub message: String,
    pub timestamp: i64,
}

/// Listener presence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListenerPresence {
    pub listener_id: String,
    pub user_agent: Option<String>,
    pub connected_at: i64,
    pub last_heartbeat: i64,
}

/// Live radio and presence registry.
#[derive(Debug, Default)]
pub struct LiveRadioRegistry {
    pub stations: RwLock<HashMap<String, RadioStation>>,
    pub listeners: RwLock<HashMap<String, HashMap<String, ListenerPresence>>>, // station_id -> (listener_id -> presence)
    pub chat_rooms: RwLock<HashMap<String, Vec<RadioChatMessage>>>, // station_id -> messages
}

impl LiveRadioRegistry {
    /// Creates a new registry with default demo stations.
    #[must_use]
    pub fn with_demo_stations() -> Self {
        let registry = Self::default();
        let synth_station = RadioStation {
            id: "synthwave_fm".into(),
            name: "Synthwave FM (24/7 Live)".into(),
            description: "Endless retro synth, outrun, and cyberpunk electronic beats".into(),
            genre: "Synthwave".into(),
            current_track: Some("Resonance Live".into()),
            current_artist: Some("HOME".into()),
            mount_point: "/live/synthwave_fm/live.m3u8".into(),
            sample_rate: 48000,
            bitrate_kbps: 320,
            is_online: true,
        };

        let ambient_station = RadioStation {
            id: "ambient_sleep".into(),
            name: "Ambient Sleep Sanctuary".into(),
            description: "Calm procedural soundscapes, binaural tones, and rain textures".into(),
            genre: "Ambient".into(),
            current_track: Some("Deep Delta Sleep 4Hz".into()),
            current_artist: Some("ERPNext Acoustics".into()),
            mount_point: "/live/ambient_sleep/live.m3u8".into(),
            sample_rate: 48000,
            bitrate_kbps: 192,
            is_online: true,
        };

        {
            let mut stations = registry.stations.write().unwrap();
            stations.insert(synth_station.id.clone(), synth_station);
            stations.insert(ambient_station.id.clone(), ambient_station);
        }

        registry
    }

    /// Registers a listener heartbeat, updating active listener list.
    pub fn record_heartbeat(
        &self,
        station_id: &str,
        listener_id: &str,
        user_agent: Option<String>,
    ) {
        let now = Utc::now().timestamp();
        let mut listeners_map = self.listeners.write().unwrap();
        let station_listeners = listeners_map.entry(station_id.to_string()).or_default();

        station_listeners
            .entry(listener_id.to_string())
            .and_modify(|p| p.last_heartbeat = now)
            .or_insert(ListenerPresence {
                listener_id: listener_id.to_string(),
                user_agent,
                connected_at: now,
                last_heartbeat: now,
            });
    }

    /// Removes a listener or purges stale listeners (>30s since last heartbeat).
    pub fn purge_stale_listeners(&self, station_id: &str, timeout_seconds: i64) -> usize {
        let now = Utc::now().timestamp();
        let mut listeners_map = self.listeners.write().unwrap();
        if let Some(station_listeners) = listeners_map.get_mut(station_id) {
            station_listeners.retain(|_, p| (now - p.last_heartbeat) < timeout_seconds);
            station_listeners.len()
        } else {
            0
        }
    }

    /// Returns active listener count for a given station.
    #[must_use]
    pub fn get_listener_count(&self, station_id: &str) -> usize {
        let listeners_map = self.listeners.read().unwrap();
        listeners_map.get(station_id).map(|m| m.len()).unwrap_or(0)
    }

    /// Returns all active listener presence records for a given station.
    #[must_use]
    pub fn get_listeners(&self, station_id: &str) -> Vec<ListenerPresence> {
        let listeners_map = self.listeners.read().unwrap();
        listeners_map
            .get(station_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Adds a live chat message for the station.
    pub fn post_chat_message(
        &self,
        station_id: &str,
        user_id: &str,
        username: &str,
        message: &str,
    ) -> RadioChatMessage {
        let now = Utc::now().timestamp();
        let chat = RadioChatMessage {
            id: format!("msg_{}_{}", now, user_id),
            user_id: user_id.to_string(),
            username: username.to_string(),
            message: message.to_string(),
            timestamp: now,
        };

        let mut chat_rooms = self.chat_rooms.write().unwrap();
        let room = chat_rooms.entry(station_id.to_string()).or_default();
        room.push(chat.clone());
        if room.len() > 100 {
            room.drain(0..room.len() - 100);
        }

        chat
    }

    /// Returns recent chat messages for a station.
    #[must_use]
    pub fn get_recent_chat(&self, station_id: &str, limit: usize) -> Vec<RadioChatMessage> {
        let chat_rooms = self.chat_rooms.read().unwrap();
        if let Some(room) = chat_rooms.get(station_id) {
            let start = room.len().saturating_sub(limit);
            room[start..].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Generates live audio HLS media playlist (`.m3u8`) with low target duration (2.0s).
    #[must_use]
    pub fn generate_live_hls_playlist(station_id: &str, sequence: u64) -> String {
        let mut playlist = String::with_capacity(512);
        playlist.push_str("#EXTM3U\n");
        playlist.push_str("#EXT-X-VERSION:6\n");
        playlist.push_str("#EXT-X-TARGETDURATION:2\n");
        playlist.push_str(&format!("#EXT-X-MEDIA-SEQUENCE:{sequence}\n"));

        // Sliding window of 5 live audio chunks (10 seconds total buffer)
        for i in 0..5 {
            let seg_seq = sequence + i;
            playlist.push_str("#EXTINF:2.000000,\n");
            playlist.push_str(&format!("chunks/{station_id}_seg_{seg_seq}.aac\n"));
        }

        playlist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_radio_registry_and_listeners() {
        let registry = LiveRadioRegistry::with_demo_stations();
        assert_eq!(registry.get_listener_count("synthwave_fm"), 0);

        registry.record_heartbeat("synthwave_fm", "listener_alice", Some("Firefox".into()));
        registry.record_heartbeat("synthwave_fm", "listener_bob", Some("VLC".into()));
        assert_eq!(registry.get_listener_count("synthwave_fm"), 2);

        // Purge with high timeout -> keeps both
        let active = registry.purge_stale_listeners("synthwave_fm", 60);
        assert_eq!(active, 2);

        // Chat
        let msg = registry.post_chat_message(
            "synthwave_fm",
            "usr_1",
            "Alice",
            "This synth bass is insane!",
        );
        assert_eq!(msg.username, "Alice");
        let history = registry.get_recent_chat("synthwave_fm", 10);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].message, "This synth bass is insane!");
    }

    #[test]
    fn test_live_hls_audio_playlist() {
        let m3u8 = LiveRadioRegistry::generate_live_hls_playlist("synthwave_fm", 100);
        assert!(m3u8.contains("#EXT-X-TARGETDURATION:2"));
        assert!(m3u8.contains("#EXT-X-MEDIA-SEQUENCE:100"));
        assert!(m3u8.contains("chunks/synthwave_fm_seg_100.aac"));
        assert!(m3u8.contains("chunks/synthwave_fm_seg_104.aac"));
    }
}
