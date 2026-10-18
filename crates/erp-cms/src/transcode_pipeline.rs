//! Resumable Chunked Ingest & Asynchronous Transcoder Pipeline (`erp-cms::transcode_pipeline`).
//!
//! Synthesizes MediaCMS ingestion and ladder transcoding:
//! - Resumable chunked upload staging with byte-offset verification and SHA-256 integrity checks.
//! - Tus protocol metadata parsing (`Upload-Metadata: filename dmlkZW8ubXA0...`).
//! - Hardware acceleration abstraction (`VAAPI`, `QSV`, `NVENC`) with automatic CPU fallback.
//! - Multi-bitrate Adaptive Bitrate (ABR) FFmpeg command generation for HLS ladders.
//! - Video thumbnail extraction and Netflix/YouTube scrubbing storyboard sprite sheets with WebVTT coordinates.
//! - Thread-safe asynchronous `TranscodeQueue` with lifecycle state transitions.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Transcoding and Ingestion Errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TranscodeError {
    #[error("Upload session '{0}' not found")]
    UploadNotFound(String),
    #[error("Invalid chunk offset: expected {expected}, provided {provided}")]
    InvalidChunkOffset { expected: u64, provided: u64 },
    #[error("Uploaded size {actual} exceeds declared total {declared}")]
    SizeOverflow { declared: u64, actual: u64 },
    #[error("Upload session is not yet complete")]
    UploadIncomplete,
    #[error("Transcode job '{0}' not found")]
    JobNotFound(String),
    #[error("Transcode job '{0}' is already finished or cancelled")]
    JobAlreadyFinished(String),
    #[error("Internal concurrency locking error")]
    LockError,
}

/// Supported hardware acceleration backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareAccelType {
    Vaapi,
    Qsv,
    Nvenc,
    Cpu,
}

impl HardwareAccelType {
    /// Detects the best available hardware acceleration engine based on environment/device paths.
    #[must_use]
    pub fn auto_detect() -> Self {
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/dev/dri/renderD128").exists() {
                return Self::Vaapi;
            }
        }
        Self::Cpu
    }
}

/// In-flight resumable upload session.
#[derive(Debug, Clone)]
pub struct ResumableUploadSession {
    pub upload_id: String,
    pub total_bytes: u64,
    pub received_bytes: u64,
    pub chunks: BTreeMap<u64, Vec<u8>>,
    pub metadata: HashMap<String, String>,
    pub is_complete: bool,
}

/// Resumable Chunked Ingest Engine supporting Tus protocol workflows.
#[derive(Default, Clone)]
pub struct ResumableChunkEngine {
    sessions: Arc<RwLock<HashMap<String, ResumableUploadSession>>>,
}

impl ResumableChunkEngine {
    /// Creates a new resumable chunk engine.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initiates a new chunked upload session.
    pub fn initiate_upload(
        &self,
        upload_id: impl Into<String>,
        total_bytes: u64,
    ) -> Result<(), TranscodeError> {
        self.initiate_upload_with_metadata(upload_id, total_bytes, HashMap::new())
    }

    /// Initiates a new chunked upload session with custom metadata (e.g. filename, filetype).
    pub fn initiate_upload_with_metadata(
        &self,
        upload_id: impl Into<String>,
        total_bytes: u64,
        metadata: HashMap<String, String>,
    ) -> Result<(), TranscodeError> {
        let mut lock = self
            .sessions
            .write()
            .map_err(|_| TranscodeError::LockError)?;
        let id = upload_id.into();
        lock.insert(
            id.clone(),
            ResumableUploadSession {
                upload_id: id,
                total_bytes,
                received_bytes: 0,
                chunks: BTreeMap::new(),
                metadata,
                is_complete: false,
            },
        );
        Ok(())
    }

    /// Parses Tus `Upload-Metadata` header string into key-value pairs.
    #[must_use]
    pub fn parse_tus_metadata(header_value: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for item in header_value.split(',') {
            let mut parts = item.split_whitespace();
            if let Some(key) = parts.next() {
                let val = parts.next().unwrap_or("");
                // Decode base64 payload if possible, otherwise keep raw
                let decoded = Self::decode_base64_str(val).unwrap_or_else(|| val.to_string());
                map.insert(key.to_string(), decoded);
            }
        }
        map
    }

    /// Simple pure Rust Base64 string decoder for Tus metadata.
    fn decode_base64_str(input: &str) -> Option<String> {
        let mut bytes = Vec::new();
        let mut buffer: u32 = 0;
        let mut bits: u8 = 0;

        for &b in input.as_bytes() {
            let val = match b {
                b'A'..=b'Z' => b - b'A',
                b'a'..=b'z' => b - b'a' + 26,
                b'0'..=b'9' => b - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                b'=' => continue, // padding
                _ => return None,
            };

            buffer = (buffer << 6) | (val as u32);
            bits += 6;

            if bits >= 8 {
                bits -= 8;
                bytes.push((buffer >> bits) as u8);
                buffer &= (1 << bits) - 1;
            }
        }

        String::from_utf8(bytes).ok()
    }

    /// Appends a chunk at a designated byte offset. Returns `Ok(true)` if the upload is now complete.
    pub fn append_chunk(
        &self,
        upload_id: &str,
        offset: u64,
        data: Vec<u8>,
    ) -> Result<bool, TranscodeError> {
        let mut lock = self
            .sessions
            .write()
            .map_err(|_| TranscodeError::LockError)?;
        let session = lock
            .get_mut(upload_id)
            .ok_or_else(|| TranscodeError::UploadNotFound(upload_id.to_string()))?;

        if offset != session.received_bytes {
            return Err(TranscodeError::InvalidChunkOffset {
                expected: session.received_bytes,
                provided: offset,
            });
        }

        let chunk_len = data.len() as u64;
        if session.received_bytes + chunk_len > session.total_bytes {
            return Err(TranscodeError::SizeOverflow {
                declared: session.total_bytes,
                actual: session.received_bytes + chunk_len,
            });
        }

        session.received_bytes += chunk_len;
        session.chunks.insert(offset, data);

        if session.received_bytes == session.total_bytes {
            session.is_complete = true;
        }

        Ok(session.is_complete)
    }

    /// Returns the current upload progress (received_bytes, total_bytes).
    pub fn get_progress(&self, upload_id: &str) -> Result<(u64, u64), TranscodeError> {
        let lock = self
            .sessions
            .read()
            .map_err(|_| TranscodeError::LockError)?;
        let session = lock
            .get(upload_id)
            .ok_or_else(|| TranscodeError::UploadNotFound(upload_id.to_string()))?;
        Ok((session.received_bytes, session.total_bytes))
    }

    /// Reassembles the uploaded chunks into a single byte payload with SHA-256 hash.
    pub fn assemble_payload(&self, upload_id: &str) -> Result<(Vec<u8>, String), TranscodeError> {
        let lock = self
            .sessions
            .read()
            .map_err(|_| TranscodeError::LockError)?;
        let session = lock
            .get(upload_id)
            .ok_or_else(|| TranscodeError::UploadNotFound(upload_id.to_string()))?;

        if !session.is_complete {
            return Err(TranscodeError::UploadIncomplete);
        }

        let mut assembled = Vec::with_capacity(session.total_bytes as usize);
        for chunk in session.chunks.values() {
            assembled.extend_from_slice(chunk);
        }

        let mut hasher = Sha256::new();
        hasher.update(&assembled);
        let hash = hex::encode(hasher.finalize());

        Ok((assembled, hash))
    }
}

/// Transcoding task parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscodeJob {
    pub job_id: String,
    pub media_id: String,
    pub input_path: String,
    pub output_dir: String,
    pub hardware_accel: HardwareAccelType,
    pub enable_storyboard: bool,
}

impl TranscodeJob {
    /// Generates FFmpeg CLI invocation for multi-resolution HLS ladder.
    #[must_use]
    pub fn build_ffmpeg_ladder_command(&self) -> String {
        let hw_flags = match self.hardware_accel {
            HardwareAccelType::Vaapi => {
                "-init_hw_device vaapi=va:/dev/dri/renderD128 -filter_hw_device va -hwaccel vaapi "
            }
            HardwareAccelType::Nvenc => "-hwaccel cuda ",
            HardwareAccelType::Qsv => "-hwaccel qsv ",
            HardwareAccelType::Cpu => "",
        };

        let v_codec = match self.hardware_accel {
            HardwareAccelType::Vaapi => "h264_vaapi",
            HardwareAccelType::Nvenc => "h264_nvenc",
            HardwareAccelType::Qsv => "h264_qsv",
            HardwareAccelType::Cpu => "libx264",
        };

        format!(
            "ffmpeg {hw_flags}-i \"{input}\" \
            -filter_complex \"[0:v]split=3[v1],[v2],[v3]; \
            [v1]scale=w=1920:h=1080[v1out]; \
            [v2]scale=w=1280:h=720[v2out]; \
            [v3]scale=w=854:h=480[v3out]\" \
            -map \"[v1out]\" -c:v:0 {v_codec} -b:v:0 4500k \
            -map \"[v2out]\" -c:v:1 {v_codec} -b:v:1 2400k \
            -map \"[v3out]\" -c:v:2 {v_codec} -b:v:2 1200k \
            -map a:0 -c:a aac -b:a 128k \
            -f hls -hls_time 4 -hls_playlist_type vod \
            -master_pl_name master.m3u8 \
            -var_stream_map \"v:0,a:0,name:1080p v:1,a:0,name:720p v:2,a:0,name:480p\" \
            \"{output}/%v/index.m3u8\"",
            input = self.input_path,
            output = self.output_dir
        )
    }

    /// Generates FFmpeg CLI invocation for thumbnail frame extraction.
    #[must_use]
    pub fn build_thumbnail_extraction_command(
        input_path: &str,
        output_path: &str,
        timestamp_sec: f64,
    ) -> String {
        format!(
            "ffmpeg -ss {timestamp_sec:.2} -i \"{input_path}\" -vframes 1 -q:v 2 \"{output_path}\""
        )
    }

    /// Generates FFmpeg CLI invocation for thumbnail sprite sheet generation (e.g. 5x5 grid).
    #[must_use]
    pub fn build_storyboard_sprite_command(
        input_path: &str,
        output_sprite_path: &str,
        interval_sec: u32,
        cols: u32,
        rows: u32,
    ) -> String {
        format!(
            "ffmpeg -i \"{input_path}\" -vf \"fps=1/{interval_sec},scale=160:90,tile={cols}x{rows}\" -q:v 2 \"{output_sprite_path}\""
        )
    }
}

/// An individual thumbnail tile within a storyboard sprite sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryboardTile {
    pub start_sec: f64,
    pub end_sec: f64,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub sprite_uri: String,
}

impl StoryboardTile {
    /// Compiles a collection of storyboard tiles into an RFC-compliant WebVTT scrubbing manifest.
    #[must_use]
    pub fn compile_vtt(tiles: &[Self]) -> String {
        let mut out = String::from("WEBVTT\n\n");
        for tile in tiles {
            let start_h = (tile.start_sec / 3600.0) as u32;
            let start_m = ((tile.start_sec % 3600.0) / 60.0) as u32;
            let start_s = tile.start_sec % 60.0;

            let end_h = (tile.end_sec / 3600.0) as u32;
            let end_m = ((tile.end_sec % 3600.0) / 60.0) as u32;
            let end_s = tile.end_sec % 60.0;

            out.push_str(&format!(
                "{:02}:{:02}:{:06.3} --> {:02}:{:02}:{:06.3}\n{}#xywh={},{},{},{}\n\n",
                start_h,
                start_m,
                start_s,
                end_h,
                end_m,
                end_s,
                tile.sprite_uri,
                tile.x,
                tile.y,
                tile.width,
                tile.height
            ));
        }
        out
    }
}

/// Lifecycle states of a transcoding job in the worker queue.
#[derive(Debug, Clone, PartialEq)]
pub enum TranscodeJobStatus {
    Queued,
    Processing {
        progress_pct: f32,
        current_pass: u8,
    },
    Completed {
        duration_sec: f64,
        output_master_playlist: String,
        thumbnail_path: String,
        storyboard_vtt: Option<String>,
    },
    Failed {
        reason: String,
    },
    Cancelled,
}

/// In-memory asynchronous transcoding task queue.
#[derive(Default, Clone)]
pub struct TranscodeQueue {
    jobs: Arc<RwLock<HashMap<String, (TranscodeJob, TranscodeJobStatus)>>>,
}

impl TranscodeQueue {
    /// Creates a new transcode queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueues a new transcode job.
    pub fn enqueue_job(&self, job: TranscodeJob) -> Result<String, TranscodeError> {
        let mut lock = self.jobs.write().map_err(|_| TranscodeError::LockError)?;
        let id = job.job_id.clone();
        lock.insert(id.clone(), (job, TranscodeJobStatus::Queued));
        Ok(id)
    }

    /// Queries the current status of a transcode job.
    pub fn poll_job(&self, job_id: &str) -> Result<TranscodeJobStatus, TranscodeError> {
        let lock = self.jobs.read().map_err(|_| TranscodeError::LockError)?;
        let (_, status) = lock
            .get(job_id)
            .ok_or_else(|| TranscodeError::JobNotFound(job_id.to_string()))?;
        Ok(status.clone())
    }

    /// Updates the progress of an in-flight job.
    pub fn update_progress(
        &self,
        job_id: &str,
        progress_pct: f32,
        current_pass: u8,
    ) -> Result<(), TranscodeError> {
        let mut lock = self.jobs.write().map_err(|_| TranscodeError::LockError)?;
        let (_, status) = lock
            .get_mut(job_id)
            .ok_or_else(|| TranscodeError::JobNotFound(job_id.to_string()))?;

        match status {
            TranscodeJobStatus::Completed { .. }
            | TranscodeJobStatus::Failed { .. }
            | TranscodeJobStatus::Cancelled => {
                Err(TranscodeError::JobAlreadyFinished(job_id.to_string()))
            }
            _ => {
                *status = TranscodeJobStatus::Processing {
                    progress_pct,
                    current_pass,
                };
                Ok(())
            }
        }
    }

    /// Marks a job as completed with output artifact URIs.
    pub fn complete_job(
        &self,
        job_id: &str,
        duration_sec: f64,
        output_master_playlist: impl Into<String>,
        thumbnail_path: impl Into<String>,
        storyboard_vtt: Option<String>,
    ) -> Result<(), TranscodeError> {
        let mut lock = self.jobs.write().map_err(|_| TranscodeError::LockError)?;
        let (_, status) = lock
            .get_mut(job_id)
            .ok_or_else(|| TranscodeError::JobNotFound(job_id.to_string()))?;

        *status = TranscodeJobStatus::Completed {
            duration_sec,
            output_master_playlist: output_master_playlist.into(),
            thumbnail_path: thumbnail_path.into(),
            storyboard_vtt,
        };
        Ok(())
    }

    /// Marks a job as failed with an error message.
    pub fn fail_job(&self, job_id: &str, reason: impl Into<String>) -> Result<(), TranscodeError> {
        let mut lock = self.jobs.write().map_err(|_| TranscodeError::LockError)?;
        let (_, status) = lock
            .get_mut(job_id)
            .ok_or_else(|| TranscodeError::JobNotFound(job_id.to_string()))?;

        *status = TranscodeJobStatus::Failed {
            reason: reason.into(),
        };
        Ok(())
    }

    /// Cancels a queued or processing transcode job.
    pub fn cancel_job(&self, job_id: &str) -> Result<(), TranscodeError> {
        let mut lock = self.jobs.write().map_err(|_| TranscodeError::LockError)?;
        let (_, status) = lock
            .get_mut(job_id)
            .ok_or_else(|| TranscodeError::JobNotFound(job_id.to_string()))?;

        *status = TranscodeJobStatus::Cancelled;
        Ok(())
    }

    /// Returns the total count of registered jobs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.jobs.read().map(|l| l.len()).unwrap_or(0)
    }

    /// Returns true if no jobs are in the queue.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resumable_chunk_upload_assembly() {
        let engine = ResumableChunkEngine::new();
        engine.initiate_upload("session_abc", 15).unwrap();

        // 1. Chunk 1: [0..5]
        let done1 = engine
            .append_chunk("session_abc", 0, b"HELLO".to_vec())
            .unwrap();
        assert!(!done1);

        // 2. Reject out-of-order chunk (expected offset 5, given 10)
        let err = engine.append_chunk("session_abc", 10, b"WORLD".to_vec());
        assert!(matches!(
            err,
            Err(TranscodeError::InvalidChunkOffset {
                expected: 5,
                provided: 10
            })
        ));

        // 3. Chunk 2: [5..10]
        let done2 = engine
            .append_chunk("session_abc", 5, b"_RUST".to_vec())
            .unwrap();
        assert!(!done2);

        // 4. Chunk 3: [10..15]
        let done3 = engine
            .append_chunk("session_abc", 10, b"_FLOW".to_vec())
            .unwrap();
        assert!(done3); // Upload completed!

        // 5. Assemble payload
        let (data, hash) = engine.assemble_payload("session_abc").unwrap();
        assert_eq!(data, b"HELLO_RUST_FLOW");
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_tus_metadata_parsing() {
        // "filename dmlkZW8ubXA0,filetype dmlkZW8vbXA0" -> "video.mp4", "video/mp4"
        let header = "filename dmlkZW8ubXA0,filetype dmlkZW8vbXA0";
        let parsed = ResumableChunkEngine::parse_tus_metadata(header);
        assert_eq!(
            parsed.get("filename").map(|s| s.as_str()),
            Some("video.mp4")
        );
        assert_eq!(
            parsed.get("filetype").map(|s| s.as_str()),
            Some("video/mp4")
        );
    }

    #[test]
    fn test_ffmpeg_ladder_and_storyboard_commands() {
        let job = TranscodeJob {
            job_id: "job-1".into(),
            media_id: "media-99".into(),
            input_path: "/spool/master.mp4".into(),
            output_dir: "/media/hls/media-99".into(),
            hardware_accel: HardwareAccelType::Cpu,
            enable_storyboard: true,
        };

        let cmd = job.build_ffmpeg_ladder_command();
        assert!(cmd.contains("libx264"));
        assert!(cmd.contains("-master_pl_name master.m3u8"));
        assert!(cmd.contains("/media/hls/media-99/%v/index.m3u8"));

        let thumb_cmd = TranscodeJob::build_thumbnail_extraction_command(
            "/spool/master.mp4",
            "/media/hls/media-99/thumb.jpg",
            12.5,
        );
        assert_eq!(
            thumb_cmd,
            "ffmpeg -ss 12.50 -i \"/spool/master.mp4\" -vframes 1 -q:v 2 \"/media/hls/media-99/thumb.jpg\""
        );

        let sprite_cmd = TranscodeJob::build_storyboard_sprite_command(
            "/spool/master.mp4",
            "/media/hls/media-99/storyboard.jpg",
            10,
            5,
            5,
        );
        assert!(sprite_cmd.contains("fps=1/10,scale=160:90,tile=5x5"));
    }

    #[test]
    fn test_storyboard_vtt_compilation() {
        let tiles = vec![
            StoryboardTile {
                start_sec: 0.0,
                end_sec: 10.0,
                x: 0,
                y: 0,
                width: 160,
                height: 90,
                sprite_uri: "storyboard.jpg".into(),
            },
            StoryboardTile {
                start_sec: 10.0,
                end_sec: 20.0,
                x: 160,
                y: 0,
                width: 160,
                height: 90,
                sprite_uri: "storyboard.jpg".into(),
            },
        ];

        let vtt = StoryboardTile::compile_vtt(&tiles);
        assert!(vtt.starts_with("WEBVTT\n\n"));
        assert!(vtt.contains("00:00:00.000 --> 00:00:10.000\nstoryboard.jpg#xywh=0,0,160,90"));
        assert!(vtt.contains("00:00:10.000 --> 00:00:20.000\nstoryboard.jpg#xywh=160,0,160,90"));
    }

    #[test]
    fn test_transcode_queue_lifecycle() {
        let queue = TranscodeQueue::new();
        assert!(queue.is_empty());

        let job = TranscodeJob {
            job_id: "tx-404".into(),
            media_id: "movie-1".into(),
            input_path: "/uploads/raw.mov".into(),
            output_dir: "/hls/movie-1".into(),
            hardware_accel: HardwareAccelType::Cpu,
            enable_storyboard: true,
        };

        queue.enqueue_job(job).unwrap();
        assert_eq!(queue.len(), 1);
        assert_eq!(
            queue.poll_job("tx-404").unwrap(),
            TranscodeJobStatus::Queued
        );

        // Update progress
        queue.update_progress("tx-404", 45.0, 1).unwrap();
        assert_eq!(
            queue.poll_job("tx-404").unwrap(),
            TranscodeJobStatus::Processing {
                progress_pct: 45.0,
                current_pass: 1
            }
        );

        // Complete job
        queue
            .complete_job(
                "tx-404",
                124.5,
                "/hls/movie-1/master.m3u8",
                "/hls/movie-1/thumb.jpg",
                Some("/hls/movie-1/storyboard.vtt".into()),
            )
            .unwrap();

        match queue.poll_job("tx-404").unwrap() {
            TranscodeJobStatus::Completed {
                duration_sec,
                output_master_playlist,
                thumbnail_path,
                storyboard_vtt,
            } => {
                assert_eq!(duration_sec, 124.5);
                assert_eq!(output_master_playlist, "/hls/movie-1/master.m3u8");
                assert_eq!(thumbnail_path, "/hls/movie-1/thumb.jpg");
                assert_eq!(
                    storyboard_vtt.as_deref(),
                    Some("/hls/movie-1/storyboard.vtt")
                );
            }
            other => panic!("Unexpected status: {:?}", other),
        }

        // Updating a completed job returns error
        assert!(matches!(
            queue.update_progress("tx-404", 99.0, 2),
            Err(TranscodeError::JobAlreadyFinished(_))
        ));
    }
}
