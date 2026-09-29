pub mod ai_schema;
pub mod auth;
pub mod class_inheritance;
pub mod dynamic_doc;
pub mod lazy_doc;
pub mod migration;
pub mod naming;
pub mod profiles;
pub mod rbac;
pub mod role_tools;
pub mod schema;
pub mod schema_compiler;
pub mod tree;

pub use auth::{
    AuthError, DEFAULT_SESSION_EXPIRY_SECS, SessionClaims, hash_password, issue_token,
    verify_password, verify_token,
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
