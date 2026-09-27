# rbac

## Classs

- [HasPermissionEdge](HasPermissionEdge.md) — Graph edge definition: Role -> DocType with granular permissions.
- [HasRoleEdge](HasRoleEdge.md) — Graph edge definition: User -> Role
- [Permission](Permission.md) — Operations subject to RBAC evaluation.
- [Role](Role.md) — Organizational Role definition.
- [User](User.md) — User identity representation in the security graph.

## Functions

- [allows](allows.md) — Checks if this edge grants the specified permission.
- [allows](allows_1.md) — Checks if this edge grants the specified permission.
- [check_permission](check_permission.md) — Evaluates whether a user with given roles is permitted to perform an operation.
- [compile_rls_policy](compile_rls_policy.md) — Generates SurrealDB Row-Level Security (RLS) predicate for a table.
