# lib

## Classs

- [LiveDiff](LiveDiff.md) — Live-query mutation diff pushed over WebSocket.
- [QueueTask](QueueTask.md) — [derive(Debug, Clone)]
- [TaskState](TaskState.md) — [derive(Debug, Clone, Copy, PartialEq, Eq)]
- [TenantContext](TenantContext.md) — Contextual tenant information attached to requests.
- [TenantRegistry](TenantRegistry.md) — In-memory registry of active tenants.

## Functions

- [from_headers](from_headers.md) — Resolves tenant context from HTTP headers.
- [from_headers](from_headers_1.md) — Resolves tenant context from HTTP headers.
- [get](get.md) — [must_use]
- [get](get_1.md) — [must_use]
- [insert](insert.md)
- [insert](insert_1.md)
- [relate](relate.md) — Helper function to generate SurrealQL RELATE graph edge statement.
- [surreal_use](surreal_use.md) — Generates SurrealQL USE statement.
- [surreal_use](surreal_use_1.md) — Generates SurrealQL USE statement.
- [test_deduplication_storage](test_deduplication_storage.md) — [test]
- [to_frame](to_frame.md) — [must_use]
- [to_frame](to_frame_1.md) — [must_use]
- [transition](transition.md)
- [transition](transition_1.md)
