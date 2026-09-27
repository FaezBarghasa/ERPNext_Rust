# migration

## Classs

- [ExistingDatabaseSchema](ExistingDatabaseSchema.md) — Snapshot of the live database dictionary.
- [ExistingFieldDef](ExistingFieldDef.md) — Representation of an existing field in SurrealDB live dictionary.
- [ExistingTableDef](ExistingTableDef.md) — Representation of an existing table definition in SurrealDB.
- [MigrationDelta](MigrationDelta.md) — Calculated structural difference between target schema and live database.

## Functions

- [diff_schema](diff_schema.md) — Calculates the structural diff between a target DocType and live database schema.
- [generate_migration_ddl](generate_migration_ddl.md) — Generates transactional forward migration DDL and corresponding reverse rollback DDL.
