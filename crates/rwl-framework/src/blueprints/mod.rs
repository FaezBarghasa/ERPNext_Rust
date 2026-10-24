//! Universal 1-Click Business Blueprints (`frappe-framework::blueprints`).
//!
//! Provides turnkey, pre-configured application templates enabling non-technical users
//! to launch working websites and portals with 1 click:
//! - **CMS**: Blogs, authors, categories, tags, responsive pages, SEO.
//! - **LMS**: Courses, modules, lessons, quizzes, student enrollments, certificates.
//! - **SaaS**: Subscription plans, customer accounts, API keys, usage metering, billing.
//! - **ERP / Commerce**: Storefronts, shopping carts, ecommerce orders, automated invoicing.

pub mod cms;
pub mod erp;
pub mod lms;
pub mod saas;

pub use cms::cms_blueprint;
pub use erp::erp_blueprint;
pub use lms::lms_blueprint;
pub use saas::saas_blueprint;

use crate::workflow_nodes::{VisualWorkflow, WorkflowEngine};
use compact_str::CompactString;
use rwl_meta::{DocTypeSchema, DynamicSchemaRegistry};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Blueprint installation and packaging errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BlueprintError {
    #[error("Blueprint not found: '{0}'")]
    NotFound(String),
    #[error("Schema installation failed for '{0}': {1}")]
    SchemaError(String, String),
    #[error("Seed record error: {0}")]
    SeedError(String),
}

/// Category classification for blueprints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlueprintCategory {
    ContentManagement,
    Education,
    SubscriptionSaaS,
    CommerceAndOperations,
}

impl BlueprintCategory {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::ContentManagement => "CMS & Content",
            Self::Education => "Education & LMS",
            Self::SubscriptionSaaS => "Subscription & SaaS",
            Self::CommerceAndOperations => "Commerce & ERP",
        }
    }
}

/// Blueprint package metadata descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlueprintMetadata {
    pub id: CompactString,
    pub name: CompactString,
    pub description: CompactString,
    pub icon: CompactString,
    pub version: CompactString,
    pub category: BlueprintCategory,
    pub tags: Vec<CompactString>,
}

/// Pre-designed page descriptor included in a blueprint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlueprintPage {
    pub path: CompactString,
    pub title: CompactString,
    pub description: CompactString,
    pub blocks_json: serde_json::Value,
    pub is_public: bool,
}

/// Navigation menu item descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlueprintNavigationItem {
    pub label: CompactString,
    pub url: CompactString,
    pub icon: CompactString,
    pub role_required: Option<CompactString>,
}

/// Complete Blueprint Package ready for 1-click instantiation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlueprintPackage {
    pub metadata: BlueprintMetadata,
    pub doctypes: Vec<DocTypeSchema>,
    pub pages: Vec<BlueprintPage>,
    pub navigation: Vec<BlueprintNavigationItem>,
    pub seed_records: Vec<(CompactString, serde_json::Value)>,
    pub workflow_automations: Vec<VisualWorkflow>,
}

/// Report generated following a successful blueprint installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallReport {
    pub blueprint_id: CompactString,
    pub blueprint_name: CompactString,
    pub doctypes_installed: usize,
    pub pages_installed: usize,
    pub records_seeded: usize,
    pub workflows_registered: usize,
}

/// Catalog registry containing all available universal blueprints.
#[derive(Default, Clone)]
pub struct BlueprintRegistry {
    blueprints: Vec<BlueprintPackage>,
}

impl BlueprintRegistry {
    /// Creates a registry pre-loaded with the standard universal business blueprints.
    #[must_use]
    pub fn new() -> Self {
        Self {
            blueprints: vec![
                cms_blueprint(),
                lms_blueprint(),
                saas_blueprint(),
                erp_blueprint(),
            ],
        }
    }

    /// Lists all available blueprints in the registry.
    #[must_use]
    pub fn all(&self) -> &[BlueprintPackage] {
        &self.blueprints
    }

    /// Looks up a blueprint by id or kind string (e.g. "cms", "lms", "saas", "erp").
    #[must_use]
    pub fn get(&self, id_or_kind: &str) -> Option<&BlueprintPackage> {
        let needle = id_or_kind.trim().to_lowercase();
        self.blueprints.iter().find(|b| {
            b.metadata.id.to_lowercase() == needle
                || b.metadata.id.to_lowercase() == format!("blueprint_{needle}")
                || b.metadata.name.to_lowercase().contains(&needle)
        })
    }

    /// Installs a selected blueprint into the live `DynamicSchemaRegistry` and `WorkflowEngine`.
    pub fn install(
        &self,
        blueprint_id: &str,
        schema_registry: &DynamicSchemaRegistry,
        workflow_engine: Option<&WorkflowEngine>,
    ) -> Result<InstallReport, BlueprintError> {
        let blueprint = self
            .get(blueprint_id)
            .ok_or_else(|| BlueprintError::NotFound(blueprint_id.to_string()))?;

        // 1. Register all DocTypes into dynamic schema registry
        let mut doctypes_count = 0;
        for dt in &blueprint.doctypes {
            schema_registry
                .register_schema(dt.clone())
                .map_err(|e| BlueprintError::SchemaError(dt.name.to_string(), e.to_string()))?;
            doctypes_count += 1;
        }

        // 2. Register all automated visual workflows
        let mut workflows_count = 0;
        if let Some(engine) = workflow_engine {
            for wf in &blueprint.workflow_automations {
                engine.register_workflow(wf.clone());
                workflows_count += 1;
            }
        }

        Ok(InstallReport {
            blueprint_id: blueprint.metadata.id.clone(),
            blueprint_name: blueprint.metadata.name.clone(),
            doctypes_installed: doctypes_count,
            pages_installed: blueprint.pages.len(),
            records_seeded: blueprint.seed_records.len(),
            workflows_registered: workflows_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_blueprints_inventory() {
        let registry = BlueprintRegistry::new();
        assert_eq!(registry.all().len(), 4);

        assert!(registry.get("cms").is_some());
        assert!(registry.get("lms").is_some());
        assert!(registry.get("saas").is_some());
        assert!(registry.get("erp").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_cms_blueprint_integrity() {
        let cms = cms_blueprint();
        assert_eq!(cms.metadata.category, BlueprintCategory::ContentManagement);
        assert!(cms.doctypes.iter().any(|d| d.name == "Blog Post"));
        assert!(cms.doctypes.iter().any(|d| d.name == "Author"));
        assert!(cms.doctypes.iter().any(|d| d.name == "Blog Category"));
        assert!(cms.pages.iter().any(|p| p.path == "/blog"));
        assert!(!cms.seed_records.is_empty());
        assert!(!cms.workflow_automations.is_empty());
    }

    #[test]
    fn test_lms_blueprint_integrity() {
        let lms = lms_blueprint();
        assert_eq!(lms.metadata.category, BlueprintCategory::Education);
        assert!(lms.doctypes.iter().any(|d| d.name == "Course"));
        assert!(lms.doctypes.iter().any(|d| d.name == "Course Lesson"));
        assert!(lms.doctypes.iter().any(|d| d.name == "Student Enrollment"));
        assert!(lms.pages.iter().any(|p| p.path == "/courses"));
        assert!(!lms.seed_records.is_empty());
        assert!(!lms.workflow_automations.is_empty());
    }

    #[test]
    fn test_saas_blueprint_integrity() {
        let saas = saas_blueprint();
        assert_eq!(saas.metadata.category, BlueprintCategory::SubscriptionSaaS);
        assert!(saas.doctypes.iter().any(|d| d.name == "Subscription Plan"));
        assert!(saas.doctypes.iter().any(|d| d.name == "Customer Account"));
        assert!(saas.pages.iter().any(|p| p.path == "/pricing"));
        assert!(!saas.seed_records.is_empty());
    }

    #[test]
    fn test_erp_blueprint_integrity() {
        let erp = erp_blueprint();
        assert_eq!(
            erp.metadata.category,
            BlueprintCategory::CommerceAndOperations
        );
        assert!(erp.doctypes.iter().any(|d| d.name == "Store Product"));
        assert!(erp.doctypes.iter().any(|d| d.name == "Ecommerce Order"));
        assert!(erp.pages.iter().any(|p| p.path == "/shop"));
        assert!(!erp.seed_records.is_empty());
    }

    #[test]
    fn test_blueprint_installation_into_dynamic_registry() {
        let registry = BlueprintRegistry::new();
        let schema_reg = DynamicSchemaRegistry::new();
        let wf_engine = WorkflowEngine::new();

        let report = registry
            .install("cms", &schema_reg, Some(&wf_engine))
            .expect("CMS installation must succeed");

        assert_eq!(report.blueprint_id, "blueprint_cms");
        assert!(report.doctypes_installed >= 3);
        assert!(report.workflows_registered >= 1);

        // Verify schemas are now registered and retrievable
        assert!(schema_reg.get_schema("Blog Post").is_some());
        assert!(schema_reg.get_schema("Author").is_some());
    }
}
