//! Dual Mobile Delivery Engine: Android NDK Binary & Tier-1 PWA Substrate (`desk_app::mobile_pwa`).
//!
//! Generates W3C Web App Manifests, offline Service Worker scripts (`sw.js`),
//! and Android NDK packaging descriptors (`cargo-apk` / `cargo-mobile2`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// W3C Web App Manifest Icon descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestIcon {
    pub src: CompactString,
    pub sizes: CompactString,
    pub r#type: CompactString,
    pub purpose: CompactString,
}

/// Dynamic, Theme-Aware W3C Web App Manifest Schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PwaWebManifest {
    pub name: CompactString,
    pub short_name: CompactString,
    pub start_url: CompactString,
    pub display: CompactString,
    pub background_color: CompactString,
    pub theme_color: CompactString,
    pub orientation: CompactString,
    pub icons: Vec<ManifestIcon>,
}

impl PwaWebManifest {
    /// Generates a production PWA Web App Manifest tailored to the active tenant theme.
    #[must_use]
    pub fn new(app_name: &str, primary_color: &str, bg_color: &str) -> Self {
        Self {
            name: app_name.into(),
            short_name: "rustnext".into(),
            start_url: "/".into(),
            display: "standalone".into(),
            background_color: bg_color.into(),
            theme_color: primary_color.into(),
            orientation: "any".into(),
            icons: vec![
                ManifestIcon {
                    src: "/assets/icons/icon-192.png".into(),
                    sizes: "192x192".into(),
                    r#type: "image/png".into(),
                    purpose: "any maskable".into(),
                },
                ManifestIcon {
                    src: "/assets/icons/icon-512.png".into(),
                    sizes: "512x512".into(),
                    r#type: "image/png".into(),
                    purpose: "any maskable".into(),
                },
                ManifestIcon {
                    src: "/assets/icons/icon-vector.svg".into(),
                    sizes: "any".into(),
                    r#type: "image/svg+xml".into(),
                    purpose: "monochrome".into(),
                },
            ],
        }
    }

    /// Serializes manifest to formatted JSON.
    #[must_use]
    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// Offline Service Worker Script Generator (`sw.js`).
pub struct ServiceWorkerGenerator;

impl ServiceWorkerGenerator {
    /// Compiles an optimized, caching Service Worker implementing Cache-First for assets and
    /// Stale-While-Revalidate for dynamic SurrealQL data streams.
    #[must_use]
    pub fn generate_service_worker_js(cache_version: &str) -> String {
        format!(
            r#"// rustnext Enterprise Offline Service Worker
const CACHE_NAME = 'rustnext-core-{cache_version}';
const STATIC_ASSETS = [
    '/',
    '/desk_app_bg.wasm',
    '/assets/css/app.css',
    '/manifest.webmanifest',
    '/assets/icons/icon-192.png',
    '/assets/icons/icon-512.png'
];

self.addEventListener('install', (event) => {{
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => cache.addAll(STATIC_ASSETS))
    );
    self.skipWaiting();
}});

self.addEventListener('activate', (event) => {{
    event.waitUntil(
        caches.keys().then((keys) => Promise.all(
            keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k))
        ))
    );
    self.clients.claim();
}});

self.addEventListener('fetch', (event) => {{
    const url = new URL(event.request.url);
    if (url.pathname.startsWith('/api/')) {{
        // Stale-While-Revalidate for dynamic API endpoints
        event.respondWith(
            caches.open('rustnext-data').then((cache) =>
                cache.match(event.request).then((cachedResponse) => {{
                    const fetchPromise = fetch(event.request).then((networkResponse) => {{
                        if (networkResponse.status === 200) {{
                            cache.put(event.request, networkResponse.clone());
                        }}
                        return networkResponse;
                    }}).catch(() => cachedResponse);
                    return cachedResponse || fetchPromise;
                }})
            )
        );
    }} else {{
        // Cache-First for static WASM, HTML, and CSS assets
        event.respondWith(
            caches.match(event.request).then((response) => response || fetch(event.request))
        );
    }}
}});
"#
        )
    }
}

/// Native Android NDK Packaging Descriptor (`cargo-apk`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AndroidNdkBuildConfig {
    pub package_id: CompactString,
    pub app_name: CompactString,
    pub min_sdk_version: u32,
    pub target_sdk_version: u32,
    pub target_architectures: Vec<CompactString>,
    pub required_permissions: Vec<CompactString>,
}

impl Default for AndroidNdkBuildConfig {
    fn default() -> Self {
        Self {
            package_id: "com.rustnext.app".into(),
            app_name: "rustnext Enterprise".into(),
            min_sdk_version: 26,
            target_sdk_version: 34,
            target_architectures: vec![
                "aarch64-linux-android".into(),
                "armv7-linux-androideabi".into(),
                "x86_64-linux-android".into(),
            ],
            required_permissions: vec![
                "android.permission.CAMERA".into(),
                "android.permission.USE_BIOMETRIC".into(),
                "android.permission.BLUETOOTH_SCAN".into(),
                "android.permission.BLUETOOTH_CONNECT".into(),
                "android.permission.ACCESS_FINE_LOCATION".into(),
                "android.permission.VIBRATE".into(),
                "android.permission.INTERNET".into(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwa_manifest_generation() {
        let manifest = PwaWebManifest::new("rustnext Mobile", "#e6c887", "#070709");
        assert_eq!(manifest.display, "standalone");
        assert_eq!(manifest.icons.len(), 3);
        let json = manifest.to_json_string();
        assert!(json.contains("standalone"));
        assert!(json.contains("#e6c887"));
    }

    #[test]
    fn test_service_worker_script_generation() {
        let sw_js = ServiceWorkerGenerator::generate_service_worker_js("v0.2.0");
        assert!(sw_js.contains("rustnext-core-v0.2.0"));
        assert!(sw_js.contains("desk_app_bg.wasm"));
        assert!(sw_js.contains("Stale-While-Revalidate"));
    }

    #[test]
    fn test_android_ndk_build_config_defaults() {
        let cfg = AndroidNdkBuildConfig::default();
        assert_eq!(cfg.min_sdk_version, 26);
        assert!(
            cfg.target_architectures
                .contains(&"aarch64-linux-android".into())
        );
        assert!(
            cfg.required_permissions
                .contains(&"android.permission.CAMERA".into())
        );
    }
}
