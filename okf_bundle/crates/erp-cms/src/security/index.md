# security

## Classs

- [HmacStreamingSigner](HmacStreamingSigner.md) — Signed HMAC-SHA256 Token Generator & Validator (Milestone 5.4).
- [StreamingError](StreamingError.md) — Streaming and CMS security errors.
- [SvodPlaybackManager](SvodPlaybackManager.md) — SVoD Subscription Gating & Real-Time Playback Concurrency Limiter (Milestone 5.5).

## Functions

- [encode](encode.md)
- [end_playback](end_playback.md) — Ends an active playback session.
- [end_playback](end_playback_1.md) — Ends an active playback session.
- [generate_token](generate_token.md) — Generates a signed token string: `hex(hmac(media_id || expiry))`.
- [generate_token](generate_token_1.md) — Generates a signed token string: `hex(hmac(media_id || expiry))`.
- [new](new.md) — Creates a new signer with the specified secret key.
- [new](new_1.md) — Creates a new signer with the specified secret key.
- [new](new_2.md) — Creates a new playback manager.
- [new](new_3.md) — Creates a new playback manager.
- [parse_byte_range](parse_byte_range.md) — HTTP 206 Byte Range Parser (Milestone 5.4).
- [start_playback](start_playback.md) — Starts a playback session if subscription is active and concurrency limit is not breached.
- [start_playback](start_playback_1.md) — Starts a playback session if subscription is active and concurrency limit is not breached.
- [verify_token](verify_token.md) — Validates an incoming signed token against media ID and current timestamp.
- [verify_token](verify_token_1.md) — Validates an incoming signed token against media ID and current timestamp.
