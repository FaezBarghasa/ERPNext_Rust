//! `desk-app` — The Pure-Rust Reactive Enterprise Desk Application.
//!
//! Provides:
//! - Hardware Abstraction Layer (`hal`): Android JNI & W3C Web APIs bridge.
//! - Role-Adaptive Multi-Persona Shell (`persona_shell`): 5 specialized persona projections.
//! - Local-First Edge Synchronization (`edge_sync`): Vector clocks, local mutation buffers, and deterministic join semilattice.
//! - Dual Mobile Delivery Engine (`mobile_pwa`): PWA Web Manifest, offline Service Worker, and Android NDK descriptors.

pub mod edge_sync;
pub mod hal;
pub mod mobile_pwa;
pub mod persona_shell;

pub use edge_sync::{
    CloudSyncArbiter, EdgeMutationEnvelope, EdgeMutationKind, LocalMutationBuffer, VectorClock,
};
pub use hal::{
    AndroidJniHalAdapter, GeoCoordinate, MobileHardwareAbstractionLayer, MockHalAdapter,
    ScanResult, WebApisHalAdapter,
};
pub use mobile_pwa::{AndroidNdkBuildConfig, ManifestIcon, PwaWebManifest, ServiceWorkerGenerator};
pub use persona_shell::{
    render_worker_kiosk_html, PersonaRole, PersonaSessionState, PersonaShellConfig,
};
