pub mod migration;
pub mod naming;
pub mod rbac;
pub mod schema;
pub mod schema_compiler;

pub use migration::{
    diff_schema, generate_migration_ddl, ExistingDatabaseSchema, ExistingFieldDef,
    ExistingTableDef, MigrationDelta,
};
pub use naming::NamingSeriesParser;
pub use rbac::{
    check_permission, compile_rls_policy, HasPermissionEdge, HasRoleEdge, Permission, Role, User,
};
pub use schema::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType, SchemaError};
pub use schema_compiler::compile_to_surrealql;
