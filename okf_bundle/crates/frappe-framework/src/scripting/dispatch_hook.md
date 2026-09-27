---
okf_version: "0.2"
type: Function
title: dispatch_hook
description: Executes a script on a mutable JSON document in the specified lifecycle event.
resource: crates/frappe-framework/src/scripting.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:56:51Z"
concept_id: crates/frappe-framework/src/scripting/dispatch_hook
language: rust
---

# dispatch_hook

Executes a script on a mutable JSON document in the specified lifecycle event.

## Signature

```rust
impl RhaiHookEngine { pub fn dispatch_hook(
        &self,
        event: LifecycleEvent,
        doc: &mut serde_json::Value,
        script: &str,
    ) -> Result<(), ScriptError> }
```

## Visibility

- `pub`

## Docstring

Executes a script on a mutable JSON document in the specified lifecycle event.

## Source
Lines 93–134 in `crates/frappe-framework/src/scripting.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [scripting](/crates/frappe-framework/src/scripting.md) |
