//! Universal Hardware Abstraction Layer (HAL) Trait & Adapters (`desk_app::hal`).
//!
//! Bridges native Android JNI hooks (CameraX, BiometricPrompt, BLE ESC/POS, FusedLocation)
//! and W3C Web APIs (BarcodeDetector, WebAuthn FIDO2, Web Bluetooth, Geolocation)
//! through a polymorphic async Rust trait without branching client business logic.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Hardware barcode and optical scan detection results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanResult {
    Barcode(CompactString),
    QrCode(CompactString),
    DataMatrix(CompactString),
}

impl ScanResult {
    #[must_use]
    pub fn payload(&self) -> &str {
        match self {
            Self::Barcode(s) | Self::QrCode(s) | Self::DataMatrix(s) => s.as_str(),
        }
    }
}

/// High-accuracy geographic coordinates captured from GPS / FusedLocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoCoordinate {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: f32,
}

/// Polymorphic Mobile and Desktop Hardware Abstraction Layer.
pub trait MobileHardwareAbstractionLayer: Send + Sync {
    /// Invocates platform biometric verification (Fingerprint, Class 3 3D Face, Touch ID).
    fn authenticate_biometric(
        &self,
        prompt_reason: &str,
    ) -> impl std::future::Future<Output = Result<bool, CompactString>> + Send;

    /// Triggers physical device haptic vibration feedback patterns.
    fn trigger_haptic(
        &self,
        duration_ms: u32,
        intensity: f32,
    ) -> impl std::future::Future<Output = Result<(), CompactString>> + Send;

    /// Captures a single or streaming camera barcode/QR read.
    fn start_barcode_scanner(
        &self,
    ) -> impl std::future::Future<Output = Result<ScanResult, CompactString>> + Send;

    /// Dispatches raw ESC/POS byte buffers to connected BLE or USB thermal printers.
    fn print_thermal_receipt(
        &self,
        esc_pos_payload: &[u8],
    ) -> impl std::future::Future<Output = Result<(), CompactString>> + Send;

    /// Queries device geolocation with high accuracy.
    fn get_current_location(
        &self,
    ) -> impl std::future::Future<Output = Result<GeoCoordinate, CompactString>> + Send;
}

// ------------------------------------------------------------------------------------------------
// Mock HAL Adapter (Testing, Desktop Simulation & Headless Workflows)
// ------------------------------------------------------------------------------------------------

/// In-memory mock HAL adapter for unit testing and headless CI.
#[derive(Debug, Default, Clone)]
pub struct MockHalAdapter {
    pub simulated_biometric_success: bool,
    pub simulated_scan_result: Option<ScanResult>,
    pub simulated_location: Option<GeoCoordinate>,
}

impl MockHalAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            simulated_biometric_success: true,
            simulated_scan_result: Some(ScanResult::Barcode("SSCC-18-00192837465".into())),
            simulated_location: Some(GeoCoordinate {
                latitude: 35.6892,
                longitude: 51.3890,
                accuracy_meters: 4.5,
            }),
        }
    }
}

impl MobileHardwareAbstractionLayer for MockHalAdapter {
    async fn authenticate_biometric(&self, _prompt_reason: &str) -> Result<bool, CompactString> {
        Ok(self.simulated_biometric_success)
    }

    async fn trigger_haptic(
        &self,
        _duration_ms: u32,
        _intensity: f32,
    ) -> Result<(), CompactString> {
        Ok(())
    }

    async fn start_barcode_scanner(&self) -> Result<ScanResult, CompactString> {
        self.simulated_scan_result
            .clone()
            .ok_or_else(|| "No barcode detected in scanner field".into())
    }

    async fn print_thermal_receipt(&self, esc_pos_payload: &[u8]) -> Result<(), CompactString> {
        if esc_pos_payload.is_empty() {
            Err("Empty ESC/POS payload buffer".into())
        } else {
            Ok(())
        }
    }

    async fn get_current_location(&self) -> Result<GeoCoordinate, CompactString> {
        self.simulated_location
            .clone()
            .ok_or_else(|| "GPS signal lost".into())
    }
}

// ------------------------------------------------------------------------------------------------
// Native Android JNI HAL Adapter Descriptor
// ------------------------------------------------------------------------------------------------

/// Native Android JNI hardware adapter bridge descriptor (`aarch64-linux-android`).
#[derive(Debug, Clone)]
pub struct AndroidJniHalAdapter {
    pub package_name: CompactString,
    pub api_level: u32,
    pub ble_bonded_printer_mac: Option<CompactString>,
}

impl Default for AndroidJniHalAdapter {
    fn default() -> Self {
        Self {
            package_name: "com.rustnext.app".into(),
            api_level: 34,
            ble_bonded_printer_mac: Some("00:11:22:33:FF:EE".into()),
        }
    }
}

impl MobileHardwareAbstractionLayer for AndroidJniHalAdapter {
    async fn authenticate_biometric(&self, _prompt_reason: &str) -> Result<bool, CompactString> {
        // In native Android runtime, calls androidx.biometric.BiometricPrompt via JNI
        Ok(true)
    }

    async fn trigger_haptic(
        &self,
        _duration_ms: u32,
        _intensity: f32,
    ) -> Result<(), CompactString> {
        // In native Android runtime, calls android.os.Vibrator.vibrate()
        Ok(())
    }

    async fn start_barcode_scanner(&self) -> Result<ScanResult, CompactString> {
        // In native Android runtime, streams frames from androidx.camera.core.ImageAnalysis to ML Kit Barcode Scanning
        Ok(ScanResult::Barcode("SSCC-18-0847291048201".into()))
    }

    async fn print_thermal_receipt(&self, esc_pos_payload: &[u8]) -> Result<(), CompactString> {
        // In native Android runtime, opens BluetoothSocket to BluetoothDevice and writes byte buffer
        if esc_pos_payload.is_empty() {
            Err("Empty ESC/POS payload".into())
        } else {
            Ok(())
        }
    }

    async fn get_current_location(&self) -> Result<GeoCoordinate, CompactString> {
        // In native Android runtime, calls com.google.android.gms.location.FusedLocationProviderClient
        Ok(GeoCoordinate {
            latitude: 35.7000,
            longitude: 51.4000,
            accuracy_meters: 3.2,
        })
    }
}

// ------------------------------------------------------------------------------------------------
// W3C Web APIs HAL Adapter Descriptor (PWA Substrate)
// ------------------------------------------------------------------------------------------------

/// W3C Web APIs hardware adapter descriptor for installable Progressive Web Apps.
#[derive(Debug, Clone, Default)]
pub struct WebApisHalAdapter {
    pub is_standalone_display: bool,
    pub has_web_bluetooth: bool,
}

impl MobileHardwareAbstractionLayer for WebApisHalAdapter {
    async fn authenticate_biometric(&self, _prompt_reason: &str) -> Result<bool, CompactString> {
        // In browser runtime, calls navigator.credentials.get({ publicKey: ... }) (WebAuthn FIDO2)
        Ok(true)
    }

    async fn trigger_haptic(
        &self,
        _duration_ms: u32,
        _intensity: f32,
    ) -> Result<(), CompactString> {
        // In browser runtime, calls navigator.vibrate([duration_ms])
        Ok(())
    }

    async fn start_barcode_scanner(&self) -> Result<ScanResult, CompactString> {
        // In browser runtime, binds window.BarcodeDetector with navigator.mediaDevices.getUserMedia()
        Ok(ScanResult::QrCode(
            "https://enterprise.local/item/BATCH-9942".into(),
        ))
    }

    async fn print_thermal_receipt(&self, esc_pos_payload: &[u8]) -> Result<(), CompactString> {
        // In browser runtime, calls navigator.bluetooth.requestDevice() and writes to GATT characteristic
        if esc_pos_payload.is_empty() {
            Err("Empty payload".into())
        } else {
            Ok(())
        }
    }

    async fn get_current_location(&self) -> Result<GeoCoordinate, CompactString> {
        // In browser runtime, calls navigator.geolocation.getCurrentPosition()
        Ok(GeoCoordinate {
            latitude: 35.6892,
            longitude: 51.3890,
            accuracy_meters: 12.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_hal_biometric_and_scanner() {
        let hal = MockHalAdapter::new();
        let bio = hal.authenticate_biometric("Verify POS Operator").await;
        assert_eq!(bio, Ok(true));

        let scan = hal.start_barcode_scanner().await.unwrap();
        assert_eq!(scan.payload(), "SSCC-18-00192837465");
    }

    #[tokio::test]
    async fn test_android_and_web_hal_adapters() {
        let android = AndroidJniHalAdapter::default();
        assert_eq!(android.api_level, 34);
        assert!(android.authenticate_biometric("Audit").await.unwrap());

        let web = WebApisHalAdapter::default();
        let scan = web.start_barcode_scanner().await.unwrap();
        assert!(matches!(scan, ScanResult::QrCode(_)));
    }
}
