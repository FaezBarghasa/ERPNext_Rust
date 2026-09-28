---
okf_version: "0.2"
type: Function
title: compile
description: Compiles script string into an AST.
resource: crates/frappe-framework/src/scripting.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/scripting/compile_1
language: rust
---

# compile

Compiles script string into an AST.

## Signature

```rust
pub fn compile(&self, script: &str) -> Result<AST, ScriptError>
```

## Visibility

- `pub`

## Docstring

Compiles script string into an AST.

## Source
Lines 89–93 in `crates/frappe-framework/src/scripting.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [scripting](/crates/frappe-framework/src/scripting.md) |
