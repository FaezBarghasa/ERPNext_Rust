# 🔌 erpnext-plugin-sdk (`rust-web-land`)

> **Extensible WebAssembly & Native Plugin SDK for `rust-web-land` (`rwl`)**.  
> Safe, sandboxed extension system enabling custom DocType hooks, server scripts, workflow transition guards, and third-party API adapters.

---

## 📦 Key Capabilities

- **Sandboxed Execution**: Safe execution of tenant extensions via Wasmtime/WASI with strict memory budgets ($\le 16\,\text{MB}$ per hook invocation).
- **DocType Hook Lifecycle**: Seamless intercepts for `before_insert`, `validate`, `on_submit`, `on_cancel`, and `after_delete`.
- **Zero-Copy Host FFI**: Bounded buffer exchange between the `rwl-framework` kernel and compiled plugin binaries.

---

## 🛠️ Usage Example

```rust
use erpnext_plugin_sdk::prelude::*;

#[plugin_hook(doctype = "Sales Order", event = "validate")]
pub fn validate_order(doc: &mut Document) -> Result<(), PluginError> {
    if doc.get_f64("grand_total")? < 0.0 {
        return Err(PluginError::Validation("Grand total cannot be negative".into()));
    }
    Ok(())
}
```
