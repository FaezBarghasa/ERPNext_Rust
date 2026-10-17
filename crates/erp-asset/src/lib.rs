//! Ultra-Advanced Enterprise Asset Management (EAM & APM) Core Engine.

pub mod depreciation;
pub mod lrs;
pub mod mro;
#[cfg(feature = "aerospace-lrs")]
pub mod reliability;
pub mod safety;

pub use depreciation::{
    Asset, AssetCapitalization, AssetError, AssetMaintenance, AssetParams, AssetStatus,
    CapitalizationItem, CapitalizationService, DepreciationMethod, DepreciationScheduleEntry,
    MaintenancePeriodicity, MaintenanceTask,
};
pub use lrs::{LinearMarker, LinearSegment, LrsEngine};
pub use mro::{MroPartRequirement, PoissonMroOptimizer};
#[cfg(feature = "aerospace-lrs")]
pub use reliability::{FmecaRecord, WeibullParameters};
pub use safety::{IsolationPointType, LotoTag, PermitToWork};
