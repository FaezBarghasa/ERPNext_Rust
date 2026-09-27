# frappe-framework

Document model abstractions, strict lifecycle state machines, and sandboxed script execution runtimes.

---

## 📦 Overview

`frappe-framework` implements the core document lifecycle transitions and dual-engine sandboxing environment for ERPNext Rust.

### Key Capabilities

- **Document Model (`document/`, `lib.rs`)**: Strongly typed document wrappers with dirty-field tracking and mutation history.
- **Lifecycle Engine (`lifecycle.rs`)**: Rigid state machine transitions (`validate` → `before_save` → `on_update` → `before_submit` → `on_submit` → `on_cancel`).
- **Rhai Dynamic Scripting (`scripting.rs`)**: High-performance embedded scripting for dynamic server-side logic and validation hooks.
- **Wasmtime WASI Sandbox (`wasmtime_sandbox.rs`)**: Secure sandboxed WebAssembly plugin execution.

---

## 🛠 Usage Example

```rust
use frappe_framework::{Document, LifecycleEngine, DocumentState};

let mut doc = Document::new("Sales Invoice");
doc.set("grand_total", 1500.0);

let mut engine = LifecycleEngine::new();
engine.validate(&mut doc)?;
engine.submit(&mut doc)?;
assert_eq!(doc.state(), DocumentState::Submitted);
```
