pub mod ai_schema;
pub mod dynamic_doc;
pub mod migration;
pub mod naming;
pub mod profiles;
pub mod rbac;
pub mod schema;
pub mod schema_compiler;

pub use ai_schema::{AiSchemaSynthesizer, SynthesisResult, SynthesizedEntity, SynthesizedField};
pub use dynamic_doc::{DocValue, DynamicDocument, SurrealDdlGenerator};
pub use migration::{
    diff_schema, generate_migration_ddl, ExistingDatabaseSchema, ExistingFieldDef,
    ExistingTableDef, MigrationDelta,
};
pub use naming::NamingSeriesParser;
pub use profiles::{ProfileRegistry, VerticalProfile};
pub use rbac::{
    check_permission, compile_rls_policy, HasPermissionEdge, HasRoleEdge, Permission, Role, User,
};
pub use schema::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType, SchemaError};
pub use schema_compiler::compile_to_surrealql;
