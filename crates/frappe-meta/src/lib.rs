pub mod ai_schema;
pub mod audit;
pub mod auth;
pub mod class_inheritance;
pub mod dynamic_doc;
pub mod lazy_doc;
pub mod mfa;
pub mod migration;
pub mod naming;
pub mod profiles;
pub mod rbac;
pub mod role_tools;
pub mod schema;
pub mod schema_compiler;
pub mod security_rules;
pub mod tree;

pub use audit::{AuditAction, AuditEntry, AuditQueryFilter, AuditTrailRegistry, compute_json_diff};
pub use auth::{
    AuthError, DEFAULT_SESSION_EXPIRY_SECS, SessionClaims, hash_password, issue_token,
    verify_password, verify_token,
};
pub use mfa::{
    MfaError, MfaRecord, TotpConfig, base32_decode, base32_encode, compute_totp,
    generate_backup_codes, generate_otpauth_uri, generate_totp_secret,
};
pub use security_rules::{
    DetectedFileType, SecurityRuleError, detect_file_magic, generate_password_reset_token,
    matches_ip_rule, sanitize_svg, validate_file_upload, validate_password_complexity,
    verify_password_reset_token,
};

pub use ai_schema::{AiSchemaSynthesizer, SynthesisResult, SynthesizedEntity, SynthesizedField};
pub use class_inheritance::ClassInheritanceRegistry;
pub use dynamic_doc::{DocValue, DynamicDocument, SurrealDdlGenerator};
pub use lazy_doc::{LazyDocument, VirtualChildRow};
pub use migration::{
    ExistingDatabaseSchema, ExistingFieldDef, ExistingTableDef, MigrationDelta, diff_schema,
    generate_migration_ddl,
};
pub use naming::NamingSeriesParser;
pub use profiles::{ProfileRegistry, VerticalProfile};
pub use rbac::{
    DynamicRolePermissionRegistry, EffectiveDocTypePermissions, HasPermissionEdge, HasRoleEdge,
    Permission, ROLE_ACCOUNTANT_USER, ROLE_ADMINISTRATOR, ROLE_CONTENT_CREATOR, ROLE_HR_MANAGER,
    ROLE_MANUFACTURING_USER, ROLE_MARKETING_ADMIN, ROLE_PURCHASE_USER, ROLE_SALES_USER,
    ROLE_SYSTEM_MANAGER, ROLE_WAREHOUSE_MANAGER, ROLE_WEBSITE_UPDATER, ROLE_WORKER_USER, Role,
    STANDARD_ROLES, User, UserRecord, check_permission, compile_rls_policy,
};
pub use role_tools::{RoleEvaluator, RoleReplicationBundle};
pub use schema::{
    DataPreset, DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType, NamingRule, SchemaError,
};
pub use schema_compiler::compile_to_surrealql;
pub use tree::{NestedSetTree, TreeNode};
