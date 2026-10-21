# `erp-cms-core` — Zero-Allocation Bare-Metal Edge CMS & Telemetry Core

`#![no_std]` core domain primitives, wire serialization formats, and bounded telemetry structures for microcontrollers, e-ink displays, and edge IoT devices.

---

## 📦 Overview

`erp-cms-core` provides bare-metal compatible CMS and streaming data structures for `rust-web-land` (`rwl`), engineered for zero dynamic memory allocation, guaranteed deterministic execution, and formal hardware verification on embedded targets (`thumbv7em-none-eabihf`, `thumbv6m-none-eabi`, `riscv32imac-unknown-none-elf`).

### Key Capabilities

1. **Bare-Metal Audio Playback Telemetry (`AudioPlaybackPacket`)**:
   - 63-byte binary wire format for real-time edge playback reporting (e.g. ESP32, STM32F4, nRF52).
   - Reports current `track_id`, `AudioPlaybackMode` (Music, Podcast, Audiobook, LiveRadio, SleepNoise, Idle), `NoiseColor` (White, Pink, Brown, Custom), position/duration in milliseconds, volume in permille ($0..1000$), EQ preset ID, and crossfade progress ($0..255$).
   - Built-in CRC16-CCITT packet checksum calculation and bit-level corruption rejection.

2. **Bounded Edge Canvas Layout (`EdgeCanvasLayout<CAP>`)**:
   - Static layout representation for warehouse e-ink displays, industrial HMIs, and edge price tags using [`heapless::Vec`](https://docs.rs/heapless).
   - Zero heap allocations during block node traversal, status updates, or rendering passes.

---

## 🛠 Target Verification

This crate strictly enforces `#![no_std]` when compiled with default features disabled.

```bash


# Run unit tests on host
cargo test -p erp-cms-core
```

---

## 🧩 Wire Format Specification (`AudioPlaybackPacket`)

| Byte Offset | Field | Type | Description |
| :---: | :--- | :--- | :--- |
| `0..32` | `track_id` | `[u8; 32]` | UTF-8 zero-padded unique track/media identifier |
| `32` | `mode` | `u8` | `0`=Idle, `1`=Music, `2`=Podcast, `3`=Audiobook, `4`=LiveRadio, `5`=SleepNoise |
| `33` | `noise_color` | `u8` | `0`=None, `1`=White, `2`=Pink, `3`=Brown, `4`=Custom |
| `34..38` | `position_ms` | `u32` (BE) | Current playhead position in milliseconds |
| `38..42` | `duration_ms` | `u32` (BE) | Total media duration in milliseconds |
| `42..44` | `volume_permille` | `u16` (BE) | Volume level ($0..1000$) |
| `44` | `eq_preset_id` | `u8` | Active DSP EQ preset index ($0..9$) |
| `45` | `crossfade_progress` | `u8` | Active crossfade interpolation progress ($0..255$) |
| `46..61` | `reserved` | `[u8; 15]` | Reserved padding for future hardware telemetry |
| `61..63` | `crc16` | `u16` (BE) | CRC16-CCITT checksum over preceding 61 bytes |
