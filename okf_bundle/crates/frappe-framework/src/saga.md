---
okf_version: "0.2"
type: Module
title: saga
description: Distributed Saga Orchestration Coordinator with forward execution and compensating backward rollbacks.
resource: crates/frappe-framework/src/saga.rs
tags:
  - "lang:rust"
  - "type:Module"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/saga
language: rust
---

# saga

Distributed Saga Orchestration Coordinator with forward execution and compensating backward rollbacks.

## Docstring

Distributed Saga Orchestration Coordinator with forward execution and compensating backward rollbacks.

## Relationships

| Type | Target |
|------|--------|
| related | [SagaStep](/crates/frappe-framework/src/saga/SagaStep.md) |
| related | [SagaStatus](/crates/frappe-framework/src/saga/SagaStatus.md) |
| related | [SagaAction](/crates/frappe-framework/src/saga/SagaAction.md) |
| related | [SagaTransaction](/crates/frappe-framework/src/saga/SagaTransaction.md) |
| related | [SagaCoordinator](/crates/frappe-framework/src/saga/SagaCoordinator.md) |
| related | [default](/crates/frappe-framework/src/saga/default.md) |
| related | [default](/crates/frappe-framework/src/saga/default.md) |
| related | [new](/crates/frappe-framework/src/saga/new.md) |
| related | [execute_saga](/crates/frappe-framework/src/saga/execute_saga.md) |
| related | [new](/crates/frappe-framework/src/saga/new.md) |
| related | [execute_saga](/crates/frappe-framework/src/saga/execute_saga.md) |
| related | [MockStep](/crates/frappe-framework/src/saga/MockStep.md) |
| related | [execute](/crates/frappe-framework/src/saga/execute.md) |
| related | [compensate](/crates/frappe-framework/src/saga/compensate.md) |
| related | [execute](/crates/frappe-framework/src/saga/execute.md) |
| related | [compensate](/crates/frappe-framework/src/saga/compensate.md) |
| related | [test_saga_forward_and_compensating_rollback](/crates/frappe-framework/src/saga/test_saga_forward_and_compensating_rollback.md) |
| related | [serde](/_dependencies/cargo/serde.md) |
