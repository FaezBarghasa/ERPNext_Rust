//! Role-Adaptive Multi-Persona Shell Architecture (`desk_app::persona_shell`).
//!
//! Projects specialized, role-tailored user interfaces from a single unified binary:
//! - Client / Customer (`/portal`): Editorial consumer self-service, streaming video HUD, LMS syllabus
//! - Warehouse Worker (`/worker`): Ruggedized high-contrast handheld, >= 56px touch targets, continuous scan
//! - Shopfloor MES Operator (`/factory`): Landscape tablet, digital traveler, dual-witness sign-offs, live SPC
//! - Manager / Lead Approver (`/approvals`): Swipe-driven triage deck, real-time KPI tickers
//! - System Administrator (`/admin`): High-density operational cockpit, Tokio/Surreal telemetry

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Universal Role / Persona Classifications across the enterprise ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonaRole {
    /// Consumer / Buyer: clean self-service, storefront, LMS syllabus, video HUD
    ClientCustomer,
    /// Picker / Packer / Courier: ruggedized high-contrast handheld, continuous fast-scan, haptics
    WarehouseWorker,
    /// Machine Operator / Technician: landscape tablet, digital traveler, dual-witness, SPC charts
    ShopfloorMes,
    /// Executive / Department Head: swipe approvals (Swipe-R approve, Swipe-L reject), KPI tickers
    ManagerApprover,
    /// Systems Engineer / DevOps: cluster telemetry, SurrealDB multiplexer, Wasm fuel monitor
    SystemAdmin,
}

impl PersonaRole {
    /// Maps URL route path to the respective persona role.
    #[must_use]
    pub fn from_route(route: &str) -> Self {
        match route {
            "/worker" | "/wms" | "/warehouse" => Self::WarehouseWorker,
            "/factory" | "/mes" | "/shopfloor" => Self::ShopfloorMes,
            "/approvals" | "/manager" | "/triage" => Self::ManagerApprover,
            "/admin" | "/cockpit" | "/ops" => Self::SystemAdmin,
            _ => Self::ClientCustomer,
        }
    }

    /// Primary route for the given persona.
    #[must_use]
    pub fn default_route(&self) -> &'static str {
        match self {
            Self::ClientCustomer => "/portal",
            Self::WarehouseWorker => "/worker",
            Self::ShopfloorMes => "/factory",
            Self::ManagerApprover => "/approvals",
            Self::SystemAdmin => "/admin",
        }
    }
}

/// UI & Hardware layout configuration tailored to each persona.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonaShellConfig {
    pub role: PersonaRole,
    pub title: CompactString,
    pub default_route: CompactString,
    pub touch_target_min_px: u32,
    pub continuous_scan_enabled: bool,
    pub orientation_lock: Option<CompactString>,
    pub high_contrast_mode: bool,
    pub dual_witness_required: bool,
    pub offline_persistence_strategy: CompactString,
    pub active_surfaces: Vec<CompactString>,
}

impl PersonaShellConfig {
    /// Synthesizes the exact shell projection for a specified persona role.
    #[must_use]
    pub fn for_role(role: PersonaRole) -> Self {
        match role {
            PersonaRole::ClientCustomer => Self {
                role,
                title: "Customer Self-Service & Digital Portal".into(),
                default_route: "/portal".into(),
                touch_target_min_px: 44,
                continuous_scan_enabled: false,
                orientation_lock: None,
                high_contrast_mode: false,
                dual_witness_required: false,
                offline_persistence_strategy: "StaleWhileRevalidate-IndexedDB".into(),
                active_surfaces: vec![
                    "StorefrontCatalog".into(),
                    "OrderTrackingTimeline".into(),
                    "LmsSyllabusTree".into(),
                    "VideoStreamingHud".into(),
                    "TypstDiplomaViewer".into(),
                ],
            },
            PersonaRole::WarehouseWorker => Self {
                role,
                title: "Ruggedized Warehouse & Field Scanner".into(),
                default_route: "/worker".into(),
                touch_target_min_px: 56, // Hard invariant: >= 56px touch target for industrial gloves
                continuous_scan_enabled: true,
                orientation_lock: Some("portrait".into()),
                high_contrast_mode: true,
                dual_witness_required: false,
                offline_persistence_strategy: "FullOfflineBatch-SurrealKV".into(),
                active_surfaces: vec![
                    "ContinuousBarcodeCamera".into(),
                    "DirectedPickPathHud".into(),
                    "OfflineSyncCounterQueue".into(),
                    "HapticAffirmationFeedback".into(),
                    "BleThermalBeltPrinter".into(),
                ],
            },
            PersonaRole::ShopfloorMes => Self {
                role,
                title: "Shopfloor Manufacturing Execution System (MES)".into(),
                default_route: "/factory".into(),
                touch_target_min_px: 48,
                continuous_scan_enabled: true,
                orientation_lock: Some("landscape".into()),
                high_contrast_mode: false,
                dual_witness_required: true,
                offline_persistence_strategy: "LocalShiftBuffer-SurrealKV".into(),
                active_surfaces: vec![
                    "JobCardDigitalTraveler".into(),
                    "StepByStepEBR".into(),
                    "DualWitnessCryptographicSignoff".into(),
                    "MachineScrapCounter".into(),
                    "RealTimeNelsonSpcChart".into(),
                ],
            },
            PersonaRole::ManagerApprover => Self {
                role,
                title: "Executive Triage & Approval Deck".into(),
                default_route: "/approvals".into(),
                touch_target_min_px: 48,
                continuous_scan_enabled: false,
                orientation_lock: Some("portrait".into()),
                high_contrast_mode: false,
                dual_witness_required: false,
                offline_persistence_strategy: "CachedApprovalBatch".into(),
                active_surfaces: vec![
                    "SwipeApprovalCardStack".into(),
                    "LiveGrossMarginTicker".into(),
                    "CashFlowWebsocketStream".into(),
                    "AmrFleetStatusWidget".into(),
                    "VoiceDictationAmendment".into(),
                ],
            },
            PersonaRole::SystemAdmin => Self {
                role,
                title: "Planetary Cluster Operational Cockpit".into(),
                default_route: "/admin".into(),
                touch_target_min_px: 40,
                continuous_scan_enabled: false,
                orientation_lock: None,
                high_contrast_mode: true,
                dual_witness_required: false,
                offline_persistence_strategy: "EmergencyDiagnosticShell".into(),
                active_surfaces: vec![
                    "ActixWorkerThreadGauges".into(),
                    "SurrealDbLiveMultiplexer".into(),
                    "DynamicSchemaDesigner".into(),
                    "WasmFuelFaultAuditor".into(),
                    "MultiTenantDatabaseRouter".into(),
                ],
            },
        }
    }
}

/// Dynamic State Projector for Active Persona Session.
#[derive(Debug, Clone)]
pub struct PersonaSessionState {
    pub user_id: CompactString,
    pub tenant_id: CompactString,
    pub active_role: PersonaRole,
    pub available_roles: Vec<PersonaRole>,
    pub active_config: PersonaShellConfig,
}

impl PersonaSessionState {
    #[must_use]
    pub fn new(
        user_id: CompactString,
        tenant_id: CompactString,
        roles: Vec<PersonaRole>,
    ) -> Self {
        let initial_role = roles.first().copied().unwrap_or(PersonaRole::ClientCustomer);
        let config = PersonaShellConfig::for_role(initial_role);
        Self {
            user_id,
            tenant_id,
            active_role: initial_role,
            available_roles: roles,
            active_config: config,
        }
    }

    /// Switches the active persona projection at runtime.
    pub fn switch_role(&mut self, new_role: PersonaRole) -> Result<(), CompactString> {
        if self.available_roles.contains(&new_role) {
            self.active_role = new_role;
            self.active_config = PersonaShellConfig::for_role(new_role);
            Ok(())
        } else {
            Err(format!("Role {:?} is not authorized for user {}", new_role, self.user_id).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warehouse_worker_touch_target_invariant() {
        let worker_cfg = PersonaShellConfig::for_role(PersonaRole::WarehouseWorker);
        // Hard invariant: Touch target >= 56px for warehouse gloves
        assert!(worker_cfg.touch_target_min_px >= 56);
        assert!(worker_cfg.continuous_scan_enabled);
        assert_eq!(worker_cfg.default_route, "/worker");
    }

    #[test]
    fn test_shopfloor_mes_dual_witness_invariant() {
        let mes_cfg = PersonaShellConfig::for_role(PersonaRole::ShopfloorMes);
        assert!(mes_cfg.dual_witness_required);
        assert_eq!(mes_cfg.orientation_lock.as_deref(), Some("landscape"));
    }

    #[test]
    fn test_persona_session_switching() {
        let mut session = PersonaSessionState::new(
            "USR-LEAD-001".into(),
            "tenant_mfg".into(),
            vec![
                PersonaRole::ManagerApprover,
                PersonaRole::WarehouseWorker,
            ],
        );
        assert_eq!(session.active_role, PersonaRole::ManagerApprover);

        session.switch_role(PersonaRole::WarehouseWorker).unwrap();
        assert_eq!(session.active_role, PersonaRole::WarehouseWorker);
        assert_eq!(session.active_config.touch_target_min_px, 56);

        // Unauthorized switch rejection
        let err = session.switch_role(PersonaRole::SystemAdmin);
        assert!(err.is_err());
    }
}
