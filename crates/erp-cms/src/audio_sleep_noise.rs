//! Procedural Sleep Noise, Soundscape Ambient Mixer & Sleep Timer Engine
//!
//! Generates zero-storage procedural acoustic textures (White, Pink via Voss-McCartney,
//! Brown via integrated random walk, and Binaural Alpha/Delta brainwave entrainment beats).
//! Provides a multi-stem ambient soundboard mixer and an exponential sleep fadeout timer.

use erp_cms_core::{AudioPlaybackMode, AudioPlaybackPacket, NoiseColor};
use serde::{Deserialize, Serialize};

/// PRNG state using Xorshift32 for deterministic, zero-allocation noise synthesis.
#[derive(Debug, Clone, Copy)]
pub struct FastRng {
    state: u32,
}

impl FastRng {
    /// Creates a new PRNG seeded with non-zero state.
    #[must_use]
    pub const fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x1234_5678 } else { seed },
        }
    }

    /// Next uniform `u32`.
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Next uniform float in range `[-1.0, 1.0]`.
    pub fn next_f32_signed(&mut self) -> f32 {
        let u = self.next_u32();
        // Convert to [0, 1) then scale to [-1, 1]
        let norm = (u as f32) / (u32::MAX as f32);
        (norm * 2.0) - 1.0
    }
}

/// Voss-McCartney 16-row pink noise generator (1/f spectral density).
#[derive(Debug, Clone)]
pub struct VossMcCartneyPinkNoise {
    rows: [f32; 16],
    running_sum: f32,
    index: u32,
    rng: FastRng,
}

impl VossMcCartneyPinkNoise {
    /// Creates a new Voss-McCartney generator.
    #[must_use]
    pub fn new(seed: u32) -> Self {
        let mut rng = FastRng::new(seed);
        let mut rows = [0.0f32; 16];
        let mut running_sum = 0.0f32;
        for r in &mut rows {
            *r = rng.next_f32_signed();
            running_sum += *r;
        }

        Self {
            rows,
            running_sum,
            index: 0,
            rng,
        }
    }

    /// Generates next 1/f pink noise sample normalized in range `[-1.0, 1.0]`.
    pub fn next_sample(&mut self) -> f32 {
        self.index = self.index.wrapping_add(1);
        let num_trailing_zeros = self.index.trailing_zeros().min(15) as usize;

        // Subtract old row value, add new random value
        self.running_sum -= self.rows[num_trailing_zeros];
        let new_val = self.rng.next_f32_signed();
        self.rows[num_trailing_zeros] = new_val;
        self.running_sum += new_val;

        // Add an extra unheld white noise sample to avoid stair-stepping
        let white = self.rng.next_f32_signed();
        let total = self.running_sum + white;

        // Scale by 1 / (16 + 1)
        (total / 17.0).clamp(-1.0, 1.0)
    }
}

/// Integrated Brownian noise generator (1/f^2 spectral density).
#[derive(Debug, Clone)]
pub struct BrownNoiseGenerator {
    current: f32,
    rng: FastRng,
}

impl BrownNoiseGenerator {
    /// Creates a new Brown noise generator.
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self {
            current: 0.0,
            rng: FastRng::new(seed),
        }
    }

    /// Generates next brown noise sample using a leaky integrator.
    pub fn next_sample(&mut self) -> f32 {
        let white = self.rng.next_f32_signed();
        // Leaky integrator to prevent long-term DC drift:
        // y[n] = 0.95 * y[n-1] + 0.1 * x[n]
        self.current = (0.95 * self.current) + (0.1 * white);
        self.current.clamp(-1.0, 1.0)
    }
}

/// Binaural beats entrainment synthesizer (e.g. 200 Hz carrier with 4 Hz Delta beat).
#[derive(Debug, Clone)]
pub struct BinauralBeatGenerator {
    carrier_hz: f32,
    beat_hz: f32,
    sample_rate: u32,
    phase_left: f32,
    phase_right: f32,
}

impl BinauralBeatGenerator {
    /// Creates a binaural beat generator.
    ///
    /// * `carrier_hz`: Frequency delivered to left ear (e.g. 216.0 Hz).
    /// * `beat_hz`: Frequency difference inducing brainwave state (e.g. 4.0 Hz Delta sleep).
    /// * `sample_rate`: Sample rate (e.g. 44100 or 48000 Hz).
    #[must_use]
    pub fn new(carrier_hz: f32, beat_hz: f32, sample_rate: u32) -> Self {
        Self {
            carrier_hz,
            beat_hz,
            sample_rate,
            phase_left: 0.0,
            phase_right: 0.0,
        }
    }

    /// Computes next stereo sample pair `(left, right)` in range `[-1.0, 1.0]`.
    pub fn next_stereo_sample(&mut self) -> (f32, f32) {
        let left = self.phase_left.sin();
        let right = self.phase_right.sin();

        let step_left = (2.0 * std::f32::consts::PI * self.carrier_hz) / (self.sample_rate as f32);
        let step_right =
            (2.0 * std::f32::consts::PI * (self.carrier_hz + self.beat_hz)) / (self.sample_rate as f32);

        self.phase_left = (self.phase_left + step_left) % (2.0 * std::f32::consts::PI);
        self.phase_right = (self.phase_right + step_right) % (2.0 * std::f32::consts::PI);

        (left, right)
    }
}

/// Procedural acoustic synthesizer combining colored noise and binaural tone generation.
#[derive(Debug, Clone)]
pub struct ProceduralNoiseEngine {
    color: NoiseColor,
    voss_pink: VossMcCartneyPinkNoise,
    brown: BrownNoiseGenerator,
    binaural: BinauralBeatGenerator,
    white_rng: FastRng,
}

impl ProceduralNoiseEngine {
    /// Creates a new procedural engine with the specified noise color and seed.
    #[must_use]
    pub fn new(color: NoiseColor, seed: u32, sample_rate: u32) -> Self {
        Self {
            color,
            voss_pink: VossMcCartneyPinkNoise::new(seed),
            brown: BrownNoiseGenerator::new(seed),
            binaural: BinauralBeatGenerator::new(200.0, 4.0, sample_rate),
            white_rng: FastRng::new(seed),
        }
    }

    /// Synthesizes `count` stereo PCM16 samples for direct streaming.
    #[must_use]
    pub fn generate_pcm16_stereo(&mut self, count: usize) -> Vec<i16> {
        let mut buffer = Vec::with_capacity(count * 2);

        for _ in 0..count {
            let (l, r) = match self.color {
                NoiseColor::None => (0.0, 0.0),
                NoiseColor::White => {
                    let w = self.white_rng.next_f32_signed();
                    (w, w)
                }
                NoiseColor::Pink => {
                    let p = self.voss_pink.next_sample();
                    (p, p)
                }
                NoiseColor::Brown => {
                    let b = self.brown.next_sample();
                    (b, b)
                }
                NoiseColor::Binaural => self.binaural.next_stereo_sample(),
            };

            let l_i16 = (l * (i16::MAX as f32)) as i16;
            let r_i16 = (r * (i16::MAX as f32)) as i16;
            buffer.push(l_i16);
            buffer.push(r_i16);
        }

        buffer
    }
}

/// Multi-stem ambient soundscape stem (e.g., Rain, Thunder, Wind, Fireplace).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmbientStem {
    /// Stem identifier (e.g. "heavy_rain", "fireplace").
    pub id: String,
    /// Human readable name.
    pub name: String,
    /// Stem volume multiplier [0.0, 1.0].
    pub volume: f32,
    /// Stereo panning [-1.0 (full left) to +1.0 (full right)].
    pub pan: f32,
    /// Whether this stem is muted.
    pub muted: bool,
}

/// Soundboard mixer combining multiple ambient background layers.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SoundboardMixer {
    /// Active stems.
    pub stems: Vec<AmbientStem>,
    /// Master volume [0.0, 1.0].
    pub master_volume: f32,
}

impl SoundboardMixer {
    /// Creates a soundboard mixer initialized with master volume 1.0.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stems: Vec::new(),
            master_volume: 1.0,
        }
    }

    /// Adds or updates a stem.
    pub fn set_stem(&mut self, stem: AmbientStem) {
        if let Some(existing) = self.stems.iter_mut().find(|s| s.id == stem.id) {
            *existing = stem;
        } else {
            self.stems.push(stem);
        }
    }

    /// Computes left and right gain multipliers for a given stem.
    #[must_use]
    pub fn compute_stem_gains(&self, stem: &AmbientStem) -> (f32, f32) {
        if stem.muted {
            return (0.0, 0.0);
        }

        let vol = stem.volume.clamp(0.0, 1.0) * self.master_volume.clamp(0.0, 1.0);
        let pan = stem.pan.clamp(-1.0, 1.0);

        // Constant power pan law
        let angle = (pan + 1.0) * (std::f32::consts::PI / 4.0);
        let gain_left = vol * angle.cos();
        let gain_right = vol * angle.sin();

        (gain_left, gain_right)
    }
}

/// Exponential sleep fadeout timer curve: V(t) = V0 * (1 - t / T)^2.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SleepTimerController {
    /// Initial volume at timer start [0.0, 1.0].
    pub initial_volume: f32,
    /// Total duration of timer in seconds.
    pub total_seconds: u32,
    /// Elapsed seconds since timer activation.
    pub elapsed_seconds: u32,
}

impl SleepTimerController {
    /// Creates a new sleep timer.
    #[must_use]
    pub fn new(initial_volume: f32, total_seconds: u32) -> Self {
        Self {
            initial_volume: initial_volume.clamp(0.0, 1.0),
            total_seconds: total_seconds.max(1),
            elapsed_seconds: 0,
        }
    }

    /// Ticks elapsed time by `seconds`.
    pub fn tick(&mut self, seconds: u32) {
        self.elapsed_seconds = (self.elapsed_seconds + seconds).min(self.total_seconds);
    }

    /// Calculates current volume based on quadratic decay:
    /// V(t) = V0 * (1 - t / T)^2
    #[must_use]
    pub fn current_volume(&self) -> f32 {
        if self.elapsed_seconds >= self.total_seconds {
            return 0.0;
        }

        let progress = (self.elapsed_seconds as f32) / (self.total_seconds as f32);
        let remaining = (1.0 - progress).clamp(0.0, 1.0);
        self.initial_volume * remaining * remaining
    }

    /// Returns remaining seconds until full silence.
    #[must_use]
    pub fn remaining_seconds(&self) -> u32 {
        self.total_seconds.saturating_sub(self.elapsed_seconds)
    }

    /// Converts current sleep state into a zero-allocation `AudioPlaybackPacket` for smart hardware.
    #[must_use]
    pub fn to_telemetry_packet(&self, color: NoiseColor, track_id: u64) -> AudioPlaybackPacket {
        let cur_vol_permille = (self.current_volume() * 1000.0).round() as u16;
        let rem_min = (self.remaining_seconds() / 60).min(255) as u8;

        let mut track_str = erp_cms_core::heapless::String::<32>::new();
        let _ = core::fmt::write(&mut track_str, format_args!("track_{track_id}"));

        AudioPlaybackPacket {
            track_id: track_str,
            position_ms: (self.elapsed_seconds as u32) * 1000,
            duration_ms: self.total_seconds * 1000,
            volume_permille: cur_vol_permille,
            mode: AudioPlaybackMode::SleepNoise,
            noise_color: color,
            sleep_timer_rem_min: rem_min,
            is_paused: false,
            is_gapless: false,
            is_normalizing: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pink_noise_generation() {
        let mut pink = VossMcCartneyPinkNoise::new(1337);
        let mut sum = 0.0f32;
        for _ in 0..1000 {
            let s = pink.next_sample();
            assert!(s >= -1.0 && s <= 1.0);
            sum += s;
        }
        let mean = sum / 1000.0;
        // Mean should hover near 0.0
        assert!(mean.abs() < 0.2);
    }

    #[test]
    fn test_brown_noise_leaky_bounds() {
        let mut brown = BrownNoiseGenerator::new(42);
        for _ in 0..2000 {
            let s = brown.next_sample();
            assert!(s >= -1.0 && s <= 1.0);
        }
    }

    #[test]
    fn test_binaural_beat_synthesis() {
        let mut binaural = BinauralBeatGenerator::new(200.0, 4.0, 44100);
        for _ in 0..100 {
            let (l, r) = binaural.next_stereo_sample();
            assert!(l >= -1.0 && l <= 1.0);
            assert!(r >= -1.0 && r <= 1.0);
        }
    }

    #[test]
    fn test_procedural_engine_pcm16() {
        let mut engine = ProceduralNoiseEngine::new(NoiseColor::Pink, 999, 44100);
        let pcm = engine.generate_pcm16_stereo(500);
        assert_eq!(pcm.len(), 1000); // 500 stereo pairs
    }

    #[test]
    fn test_soundboard_mixer_pan_gains() {
        let mut mixer = SoundboardMixer::new();
        mixer.set_stem(AmbientStem {
            id: "rain".into(),
            name: "Rain".into(),
            volume: 1.0,
            pan: 0.0, // Center
            muted: false,
        });

        let stem = &mixer.stems[0];
        let (gl, gr) = mixer.compute_stem_gains(stem);
        // Center pan should have roughly equal gains
        assert!((gl - gr).abs() < 0.01);
        assert!(gl > 0.6 && gl < 0.8); // cos(pi/4) approx 0.707
    }

    #[test]
    fn test_sleep_timer_exponential_decay() {
        let mut timer = SleepTimerController::new(1.0, 100);
        assert_eq!(timer.current_volume(), 1.0);

        // Halfway at 50s: V(50) = 1.0 * (1 - 0.5)^2 = 0.25
        timer.tick(50);
        assert!((timer.current_volume() - 0.25).abs() < 0.001);

        // At end: 100s -> 0.0
        timer.tick(50);
        assert_eq!(timer.current_volume(), 0.0);
        assert_eq!(timer.remaining_seconds(), 0);

        // Hardware packet check
        let packet = timer.to_telemetry_packet(NoiseColor::Brown, 777);
        assert_eq!(packet.track_id.as_str(), "track_777");
        assert_eq!(packet.volume_permille, 0);
        assert_eq!(packet.noise_color, NoiseColor::Brown);
    }
}
