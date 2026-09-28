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
    ExistingDatabaseSchema, ExistingFieldDef, ExistingTableDef, MigrationDelta, diff_schema,
    generate_migration_ddl,
};
pub use naming::NamingSeriesParser;
pub use profiles::{ProfileRegistry, VerticalProfile};
pub use rbac::{
    HasPermissionEdge, HasRoleEdge, Permission, Role, User, check_permission, compile_rls_policy,
};
pub use schema::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType, SchemaError};
pub use schema_compiler::compile_to_surrealql;
