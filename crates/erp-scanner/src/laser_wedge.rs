//! Hardware Laser Wedge HID Interceptor (`erp_scanner::laser_wedge`).

use std::time::Duration;

/// Keystroke burst analyzer distinguishing hardware laser scanners from manual typists.
pub struct LaserWedgeInterceptor {
    /// Maximum threshold between consecutive keystrokes to qualify as laser scan (e.g. 35ms).
    pub max_inter_keystroke_delta: Duration,
    buffer: String,
    last_keystroke_time_ms: u64,
    is_burst: bool,
}

impl LaserWedgeInterceptor {
    /// Constructs a new interceptor with threshold (default 35ms).
    #[must_use]
    pub fn new(max_delta_ms: u64) -> Self {
        Self {
            max_inter_keystroke_delta: Duration::from_millis(max_delta_ms),
            buffer: String::with_capacity(128),
            last_keystroke_time_ms: 0,
            is_burst: true,
        }
    }

    /// Feeds a single incoming key character with its precise millisecond timestamp.
    /// Returns `Some(decoded_barcode)` when `Enter` / `\n` terminator completes a valid high-speed burst.
    pub fn feed_char(&mut self, ch: char, timestamp_ms: u64) -> Option<String> {
        if ch == '\n' || ch == '\r' {
            if self.is_burst && !self.buffer.is_empty() {
                let result = self.buffer.clone();
                self.reset();
                return Some(result);
            }
            self.reset();
            return None;
        }

        if self.last_keystroke_time_ms > 0 {
            let delta = timestamp_ms.saturating_sub(self.last_keystroke_time_ms);
            if delta > self.max_inter_keystroke_delta.as_millis() as u64 {
                // Too slow -> human typing, not a hardware laser scanner
                self.is_burst = false;
            }
        }

        self.last_keystroke_time_ms = timestamp_ms;
        self.buffer.push(ch);

        None
    }

    /// Resets the internal state buffer.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.last_keystroke_time_ms = 0;
        self.is_burst = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_laser_scan_burst_detection() {
        let mut interceptor = LaserWedgeInterceptor::new(35);

        // Simulate high-speed barcode scan (10ms between chars)
        let chars = ['I', 'T', 'E', 'M', ':', 'X', '1', '\n'];
        let mut time = 1000u64;
        let mut result = None;

        for &c in &chars {
            time += 10;
            if let Some(barcode) = interceptor.feed_char(c, time) {
                result = Some(barcode);
            }
        }

        assert_eq!(result, Some("ITEM:X1".into()));
    }

    #[test]
    fn test_human_typing_rejection() {
        let mut interceptor = LaserWedgeInterceptor::new(35);

        // Simulate slow human typing (150ms between chars)
        let chars = ['I', 'T', 'E', 'M', ':', 'X', '1', '\n'];
        let mut time = 1000u64;
        let mut result = None;

        for &c in &chars {
            time += 150;
            if let Some(barcode) = interceptor.feed_char(c, time) {
                result = Some(barcode);
            }
        }

        assert_eq!(result, None);
    }
}
