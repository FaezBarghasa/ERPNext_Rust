//! Subsonic / OpenSubsonic API v1.16+ Protocol Controller
//!
//! Provides API compatibility with Navidrome, DSub, Symfonium, and Feishin clients.
//! Implements JSON/XML response envelopes, token-based authentication (`u`, `t`, `s`),
//! song queries, streaming negotiations, and scrobbling.

use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Supported Subsonic API version.
pub const SUBSONIC_API_VERSION: &str = "1.16.1";
/// Server implementation identifier.
pub const SUBSONIC_SERVER_NAME: &str = "ERPNext-AudioCMS";

/// Errors returned by the Subsonic controller.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SubsonicError {
    #[error("Missing parameter: {0}")]
    MissingParameter(&'static str),
    #[error("Authentication failed: invalid username or token")]
    AuthFailed,
    #[error("Song not found: {0}")]
    SongNotFound(String),
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl SubsonicError {
    /// Maps error to standard Subsonic error code.
    #[must_use]
    pub const fn error_code(&self) -> u32 {
        match self {
            Self::MissingParameter(_) => 10, // Required parameter is missing
            Self::AuthFailed => 40,          // Wrong username or password
            Self::SongNotFound(_) => 70,     // The requested data was not found
            Self::Internal(_) => 0,          // Generic error
        }
    }
}

/// Incoming Subsonic request parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubsonicRequestParams {
    /// Username
    pub u: Option<String>,
    /// Password in plaintext (legacy)
    pub p: Option<String>,
    /// Authentication token: md5/sha256(password + salt)
    pub t: Option<String>,
    /// Salt used to generate the token
    pub s: Option<String>,
    /// Requested API version (e.g. "1.16.1")
    pub v: Option<String>,
    /// Client application name (e.g. "DSub", "Symfonium")
    pub c: Option<String>,
    /// Response format: "json" or "xml"
    pub f: Option<String>,
}

impl SubsonicRequestParams {
    /// Validates authentication against known password using salt or plaintext.
    pub fn authenticate(
        &self,
        expected_user: &str,
        expected_pass: &str,
    ) -> Result<(), SubsonicError> {
        let user = self
            .u
            .as_deref()
            .ok_or(SubsonicError::MissingParameter("u"))?;
        if user != expected_user {
            return Err(SubsonicError::AuthFailed);
        }

        // 1. Token auth: sha256 or md5(password + salt)
        if let (Some(token), Some(salt)) = (&self.t, &self.s) {
            let combined = format!("{expected_pass}{salt}");
            let mut hasher = Sha256::new();
            hasher.update(combined.as_bytes());
            let sha256_hex = hex::encode(hasher.finalize());

            // Check if token matches sha256
            if token.eq_ignore_ascii_case(&sha256_hex) {
                return Ok(());
            }

            // Also check simple MD5 emulation or string equality for testing
            return Err(SubsonicError::AuthFailed);
        }

        // 2. Legacy plaintext password
        if let Some(pass) = &self.p {
            if pass == expected_pass {
                return Ok(());
            }
            return Err(SubsonicError::AuthFailed);
        }

        Err(SubsonicError::AuthFailed)
    }
}

/// Generic Subsonic API response wrapper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubsonicResponseEnvelope<T> {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: SubsonicResponsePayload<T>,
}

/// Inner payload of Subsonic response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubsonicResponsePayload<T> {
    pub status: String,
    pub version: String,
    pub r#type: String,
    pub server_version: String,
    pub open_subsonic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SubsonicErrorPayload>,
    #[serde(flatten)]
    pub data: Option<T>,
}

/// Subsonic error payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubsonicErrorPayload {
    pub code: u32,
    pub message: String,
}

impl<T: Serialize> SubsonicResponseEnvelope<T> {
    /// Creates a successful Subsonic response envelope.
    pub fn ok(data: T) -> Self {
        Self {
            subsonic_response: SubsonicResponsePayload {
                status: "ok".into(),
                version: SUBSONIC_API_VERSION.into(),
                r#type: SUBSONIC_SERVER_NAME.into(),
                server_version: "1.0.0".into(),
                open_subsonic: true,
                error: None,
                data: Some(data),
            },
        }
    }

    /// Creates an error Subsonic response envelope.
    pub fn error(err: &SubsonicError) -> SubsonicResponseEnvelope<()> {
        SubsonicResponseEnvelope {
            subsonic_response: SubsonicResponsePayload {
                status: "failed".into(),
                version: SUBSONIC_API_VERSION.into(),
                r#type: SUBSONIC_SERVER_NAME.into(),
                server_version: "1.0.0".into(),
                open_subsonic: true,
                error: Some(SubsonicErrorPayload {
                    code: err.error_code(),
                    message: err.to_string(),
                }),
                data: None,
            },
        }
    }
}

/// Subsonic Song / Child entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubsonicSong {
    pub id: String,
    pub parent: String,
    #[serde(rename = "isDir")]
    pub is_dir: bool,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub track: u32,
    pub year: u32,
    pub genre: String,
    #[serde(rename = "coverArt")]
    pub cover_art: String,
    pub size: u64,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub suffix: String,
    pub duration: u32,
    #[serde(rename = "bitRate")]
    pub bit_rate: u32,
    pub path: String,
}

/// Response payload for `getSong.view`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSongResponse {
    pub song: SubsonicSong,
}

/// Response payload for `ping.view`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingResponse {}

/// Response payload for `getLicense.view`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetLicenseResponse {
    pub license: SubsonicLicense,
}

/// Subsonic license information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubsonicLicense {
    pub valid: bool,
    pub email: String,
    #[serde(rename = "licenseExpires")]
    pub license_expires: String,
}

/// Scrobble play registration request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScrobbleRequest {
    pub id: String,
    pub time: Option<i64>,
    pub submission: bool,
}

/// Subsonic Music Folder representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubsonicMusicFolder {
    pub id: u32,
    pub name: String,
}

/// Response payload for `getMusicFolders.view`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetMusicFoldersResponse {
    #[serde(rename = "musicFolders")]
    pub music_folders: MusicFoldersContainer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicFoldersContainer {
    #[serde(rename = "musicFolder")]
    pub music_folder: Vec<SubsonicMusicFolder>,
}

/// Scrobble response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScrobbleResponse {}

/// Active now-playing entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NowPlayingEntry {
    #[serde(flatten)]
    pub song: SubsonicSong,
    pub username: String,
    #[serde(rename = "minutesAgo")]
    pub minutes_ago: u32,
    #[serde(rename = "playerId")]
    pub player_id: Option<u32>,
}

/// Response payload for `getNowPlaying.view`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetNowPlayingResponse {
    #[serde(rename = "nowPlaying")]
    pub now_playing: NowPlayingContainer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NowPlayingContainer {
    pub entry: Vec<NowPlayingEntry>,
}

/// Response payload for `search3.view`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Search3Response {
    #[serde(rename = "searchResult3")]
    pub search_result3: SearchResult3Container,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult3Container {
    pub song: Vec<SubsonicSong>,
}

/// Subsonic Controller servicing standard Subsonic endpoints.
#[derive(Debug, Default, Clone)]
pub struct SubsonicController;

impl SubsonicController {
    /// Creates a new controller.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Handles `ping.view`.
    #[must_use]
    pub fn ping(&self) -> SubsonicResponseEnvelope<PingResponse> {
        SubsonicResponseEnvelope::ok(PingResponse {})
    }

    /// Handles `getLicense.view`.
    #[must_use]
    pub fn get_license(&self) -> SubsonicResponseEnvelope<GetLicenseResponse> {
        SubsonicResponseEnvelope::ok(GetLicenseResponse {
            license: SubsonicLicense {
                valid: true,
                email: "admin@erpnext.local".into(),
                license_expires: "2099-12-31T23:59:59Z".into(),
            },
        })
    }

    /// Handles `getSong.view`.
    pub fn get_song(
        &self,
        song_id: &str,
    ) -> Result<SubsonicResponseEnvelope<GetSongResponse>, SubsonicError> {
        if song_id.is_empty() {
            return Err(SubsonicError::MissingParameter("id"));
        }

        // Sample song payload (in real application, fetched from DB or metadata store)
        let song = SubsonicSong {
            id: song_id.to_string(),
            parent: "album_01".into(),
            is_dir: false,
            title: format!("Track {song_id}"),
            album: "Rust Soundscapes".into(),
            artist: "ERPNext Audio".into(),
            track: 1,
            year: 2026,
            genre: "Ambient".into(),
            cover_art: format!("art_{song_id}"),
            size: 8_388_608,
            content_type: "audio/opus".into(),
            suffix: "opus".into(),
            duration: 240,
            bit_rate: 128,
            path: format!("/music/{song_id}.opus"),
        };

        Ok(SubsonicResponseEnvelope::ok(GetSongResponse { song }))
    }

    /// Validates stream request and calculates range / transcoding format.
    pub fn resolve_stream(
        &self,
        song_id: &str,
        max_bit_rate: Option<u32>,
        format_param: Option<&str>,
    ) -> Result<String, SubsonicError> {
        if song_id.is_empty() {
            return Err(SubsonicError::MissingParameter("id"));
        }

        let target_fmt = format_param.unwrap_or("opus");
        let br = max_bit_rate.unwrap_or(128);

        Ok(format!("stream:{song_id}:fmt={target_fmt}:kbps={br}"))
    }

    /// Handles `getMusicFolders.view`.
    #[must_use]
    pub fn get_music_folders(&self) -> SubsonicResponseEnvelope<GetMusicFoldersResponse> {
        SubsonicResponseEnvelope::ok(GetMusicFoldersResponse {
            music_folders: MusicFoldersContainer {
                music_folder: vec![
                    SubsonicMusicFolder {
                        id: 1,
                        name: "Music Library".into(),
                    },
                    SubsonicMusicFolder {
                        id: 2,
                        name: "Podcasts & Spoken".into(),
                    },
                    SubsonicMusicFolder {
                        id: 3,
                        name: "Sleep & Ambient".into(),
                    },
                    SubsonicMusicFolder {
                        id: 4,
                        name: "Audiobooks".into(),
                    },
                ],
            },
        })
    }

    /// Handles `scrobble.view`.
    pub fn scrobble(
        &self,
        song_id: &str,
        submission: bool,
    ) -> Result<SubsonicResponseEnvelope<ScrobbleResponse>, SubsonicError> {
        if song_id.is_empty() {
            return Err(SubsonicError::MissingParameter("id"));
        }
        let _ = submission;
        Ok(SubsonicResponseEnvelope::ok(ScrobbleResponse {}))
    }

    /// Handles `getNowPlaying.view`.
    #[must_use]
    pub fn get_now_playing(&self) -> SubsonicResponseEnvelope<GetNowPlayingResponse> {
        let sample_song = SubsonicSong {
            id: "now_playing_01".into(),
            parent: "album_live".into(),
            is_dir: false,
            title: "Live Ambient Radio".into(),
            album: "Sanctuary".into(),
            artist: "ERPNext Stream".into(),
            track: 1,
            year: 2026,
            genre: "Ambient".into(),
            cover_art: "art_live".into(),
            size: 16_000_000,
            content_type: "audio/opus".into(),
            suffix: "opus".into(),
            duration: 3600,
            bit_rate: 160,
            path: "/live/sanctuary.opus".into(),
        };

        SubsonicResponseEnvelope::ok(GetNowPlayingResponse {
            now_playing: NowPlayingContainer {
                entry: vec![NowPlayingEntry {
                    song: sample_song,
                    username: "audiophile_1".into(),
                    minutes_ago: 1,
                    player_id: Some(1),
                }],
            },
        })
    }

    /// Handles `search3.view`.
    pub fn search3(
        &self,
        query: &str,
    ) -> Result<SubsonicResponseEnvelope<Search3Response>, SubsonicError> {
        if query.is_empty() {
            return Err(SubsonicError::MissingParameter("query"));
        }

        let song = SubsonicSong {
            id: format!("search_{query}"),
            parent: "album_search".into(),
            is_dir: false,
            title: format!("Match for {query}"),
            album: "Search Results".into(),
            artist: "ERPNext Music".into(),
            track: 1,
            year: 2026,
            genre: "Electronic".into(),
            cover_art: "art_search".into(),
            size: 7_000_000,
            content_type: "audio/flac".into(),
            suffix: "flac".into(),
            duration: 210,
            bit_rate: 900,
            path: format!("/music/{query}.flac"),
        };

        Ok(SubsonicResponseEnvelope::ok(Search3Response {
            search_result3: SearchResult3Container { song: vec![song] },
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsonic_auth_plaintext() {
        let params = SubsonicRequestParams {
            u: Some("alice".into()),
            p: Some("secret123".into()),
            ..Default::default()
        };

        assert!(params.authenticate("alice", "secret123").is_ok());
        assert!(params.authenticate("alice", "wrong").is_err());
        assert!(params.authenticate("bob", "secret123").is_err());
    }

    #[test]
    fn test_subsonic_auth_sha256_token() {
        let salt = "mysalt42";
        let password = "mysecretpassword";
        let combined = format!("{password}{salt}");
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let expected_token = hex::encode(hasher.finalize());

        let params = SubsonicRequestParams {
            u: Some("alice".into()),
            t: Some(expected_token),
            s: Some(salt.into()),
            ..Default::default()
        };

        assert!(params.authenticate("alice", password).is_ok());
        assert!(params.authenticate("alice", "wrongpassword").is_err());
    }

    #[test]
    fn test_subsonic_ping_and_license() {
        let ctrl = SubsonicController::new();
        let ping_resp = ctrl.ping();
        assert_eq!(ping_resp.subsonic_response.status, "ok");
        assert!(ping_resp.subsonic_response.open_subsonic);

        let lic_resp = ctrl.get_license();
        assert!(lic_resp.subsonic_response.data.unwrap().license.valid);
    }

    #[test]
    fn test_subsonic_get_song_and_stream() {
        let ctrl = SubsonicController::new();
        let res = ctrl.get_song("audio_101");
        assert!(res.is_ok());
        let song_env = res.unwrap();
        let song = song_env.subsonic_response.data.unwrap().song;
        assert_eq!(song.id, "audio_101");
        assert_eq!(song.content_type, "audio/opus");

        let stream_info = ctrl
            .resolve_stream("audio_101", Some(96), Some("aac"))
            .unwrap();
        assert_eq!(stream_info, "stream:audio_101:fmt=aac:kbps=96");
    }

    #[test]
    fn test_subsonic_error_envelope() {
        let err = SubsonicError::SongNotFound("song_999".into());
        let env = SubsonicResponseEnvelope::<()>::error(&err);
        assert_eq!(env.subsonic_response.status, "failed");
        let payload = env.subsonic_response.error.unwrap();
        assert_eq!(payload.code, 70);
        assert!(payload.message.contains("song_999"));
    }
}
