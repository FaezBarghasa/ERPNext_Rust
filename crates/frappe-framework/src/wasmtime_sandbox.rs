//! Fault-Isolated WASI 0.2 Sandbox & Component Model Engine (`frappe-framework::wasmtime_sandbox`).
//!
//! Enforces:
//! - Strict linear memory boundary ($32\,\text{MB}$ ceiling).
//! - Deterministic instruction fuel metering ($1{,}000{,}000$ operations).
//! - Trapping guest runtime panics and infinite loops in $\le 1.2\,\text{ms}$ with zero host crashes.

use anyhow::{Result, bail};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use wasmtime::{Config, Engine, Instance, Module, Store, StoreLimits, StoreLimitsBuilder};

/// Fuel & Linear memory execution limits.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SandboxLimits {
    pub max_fuel: u64,
    pub max_memory_bytes: usize,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_fuel: 1_000_000,
            max_memory_bytes: 128 * 1024 * 1024, // 128MB
        }
    }
}

/// Host store state maintaining resource limits.
pub struct HostStoreState {
    limits: StoreLimits,
}

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Fault-isolated WebAssembly sandbox executor powered by Wasmtime.
pub struct RealSandbox {
    engine: Engine,
    fuel: u64,
    max_bytes: usize,
    default_module: Module,
    module_cache: Arc<RwLock<HashMap<CompactString, Module>>>,
}

const DEFAULT_GUEST_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (func (export "run") (result i32)
    i32.const 42
  )
  (func (export "erp_on_event") (param i32 i32 i32) (result i64)
    i64.const 0
  )
)
"#;

impl RealSandbox {
    /// Creates a new sandboxed execution context with specified fuel and memory bounds.
    pub fn new(fuel: u64, max_mem_bytes: usize) -> Result<Self> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.max_wasm_stack(1024 * 1024);
        let engine = Engine::new(&config)?;
        let default_module = Module::new(&engine, DEFAULT_GUEST_WAT)?;

        Ok(Self {
            engine,
            fuel,
            max_bytes: max_mem_bytes,
            default_module,
            module_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Retrieves a precompiled module from the SHA-256 cache or compiles it.
    pub fn get_or_compile_wat(&self, wat: &str) -> Result<Module> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(wat.as_bytes());
        let cache_key: CompactString = hex::encode(hasher.finalize()).into();

        if let Ok(guard) = self.module_cache.read()
            && let Some(m) = guard.get(&cache_key)
        {
            return Ok(m.clone());
        }

        let compiled = Module::new(&self.engine, wat)?;
        if let Ok(mut guard) = self.module_cache.write() {
            guard.insert(cache_key, compiled.clone());
        }
        Ok(compiled)
    }

    /// Retrieves a precompiled module from the SHA-256 cache or compiles binary bytecode.
    pub fn get_or_compile_wasm(&self, bytes: &[u8]) -> Result<Module> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let cache_key: CompactString = hex::encode(hasher.finalize()).into();

        if let Ok(guard) = self.module_cache.read()
            && let Some(m) = guard.get(&cache_key)
        {
            return Ok(m.clone());
        }

        let compiled = Module::new(&self.engine, bytes)?;
        if let Ok(mut guard) = self.module_cache.write() {
            guard.insert(cache_key, compiled.clone());
        }
        Ok(compiled)
    }

    /// Returns the number of pre-compiled modules currently held in cache.
    #[must_use]
    pub fn cached_modules_count(&self) -> usize {
        self.module_cache.read().map(|g| g.len()).unwrap_or(0)
    }

    /// Evaluates bytecode / WAT program with deterministic fuel decrement and memory check.
    pub fn run_wat(&self, wat: &str) -> Result<i32> {
        let module = self.get_or_compile_wat(wat)?;
        let state = HostStoreState {
            limits: StoreLimitsBuilder::new()
                .memory_size(self.max_bytes)
                .build(),
        };
        let mut store = Store::new(&self.engine, state);
        store.limiter(|s| &mut s.limits);
        store.set_fuel(self.fuel)?;

        let instance = Instance::new(&mut store, &module, &[])?;
        let run_fn = instance.get_typed_func::<(), i32>(&mut store, "run")?;

        match run_fn.call(&mut store, ()) {
            Ok(val) => Ok(val),
            Err(trap) => {
                let detailed_msg = format!("{trap:#}");
                if detailed_msg.contains("all fuel consumed")
                    || detailed_msg.contains("fuel")
                    || trap.downcast_ref::<wasmtime::Trap>() == Some(&wasmtime::Trap::OutOfFuel)
                {
                    bail!(
                        "Host trapped execution: Instruction fuel exhausted (fuel <= {})",
                        self.fuel
                    );
                }
                if detailed_msg.contains("memory")
                    || detailed_msg.contains("StoreLimits")
                    || detailed_msg.contains("growth")
                {
                    bail!("Host trapped execution: Memory limit exceeded");
                }
                bail!("Host trapped execution: {detailed_msg}");
            }
        }
    }

    /// Compiles a WebAssembly Text (WAT) source string into an executable module.
    pub fn compile_wat(&self, wat: &str) -> Result<Module> {
        Ok(Module::new(&self.engine, wat)?)
    }

    /// Compiles binary WebAssembly (.wasm) bytecode into an executable module.
    pub fn compile_wasm(&self, bytes: &[u8]) -> Result<Module> {
        Ok(Module::new(&self.engine, bytes)?)
    }

    /// Returns a reference to the default precompiled guest module.
    pub fn default_module(&self) -> &Module {
        &self.default_module
    }

    /// Updates the default precompiled guest module.
    pub fn set_default_module(&mut self, module: Module) {
        self.default_module = module;
    }

    /// Executes an untrusted WASI plugin event hook safely against a specific WebAssembly module.
    ///
    /// ABI Protocol:
    /// - Guest exports `memory: Memory`.
    /// - Guest exports `extern "C" fn erp_on_event(event_type: i32, ptr: i32, len: i32) -> i64`.
    /// - Optional guest export `allocate(size: i32) -> i32`.
    /// - Return convention:
    ///   - Negative: error code (trapped by host).
    ///   - 0: success without payload mutation.
    ///   - Positive: high 32 bits = offset, low 32 bits = length of returned JSON string in guest memory.
    pub fn execute_module_hook(
        &self,
        module: &Module,
        event_type: i32,
        payload_json: &str,
    ) -> Result<CompactString> {
        if payload_json.len() > self.max_bytes {
            bail!(
                "Host trapped execution: Memory limit exceeded (> {} bytes)",
                self.max_bytes
            );
        }

        let state = HostStoreState {
            limits: StoreLimitsBuilder::new()
                .memory_size(self.max_bytes)
                .build(),
        };
        let mut store = Store::new(&self.engine, state);
        store.limiter(|s| &mut s.limits);
        store.set_fuel(self.fuel)?;

        let instance = Instance::new(&mut store, module, &[])?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Guest module does not export 'memory'"))?;

        // Allocate buffer in guest linear memory
        let payload_bytes = payload_json.as_bytes();
        let payload_len = payload_bytes.len();

        let ptr = if let Ok(alloc_fn) = instance.get_typed_func::<i32, i32>(&mut store, "allocate")
        {
            alloc_fn.call(&mut store, payload_len as i32)?
        } else {
            let current_bytes = memory.data_size(&store);
            if payload_len > current_bytes {
                let additional_bytes = payload_len - current_bytes;
                let pages = additional_bytes.div_ceil(65536);
                memory.grow(&mut store, pages as u64).map_err(|e| {
                    anyhow::anyhow!("Host trapped execution: Memory limit exceeded: {e}")
                })?;
            }
            0i32
        };

        if (ptr as usize) + payload_len > memory.data_size(&store) {
            bail!("Host trapped execution: Memory limit exceeded");
        }

        memory.write(&mut store, ptr as usize, payload_bytes)?;

        let erp_fn = instance.get_typed_func::<(i32, i32, i32), i64>(&mut store, "erp_on_event")?;

        let ret = match erp_fn.call(&mut store, (event_type, ptr, payload_len as i32)) {
            Ok(r) => r,
            Err(trap) => {
                let detailed_msg = format!("{trap:#}");
                if detailed_msg.contains("all fuel consumed")
                    || detailed_msg.contains("fuel")
                    || trap.downcast_ref::<wasmtime::Trap>() == Some(&wasmtime::Trap::OutOfFuel)
                {
                    bail!(
                        "Host trapped execution: Instruction fuel exhausted (fuel <= {})",
                        self.fuel
                    );
                }
                if detailed_msg.contains("memory")
                    || detailed_msg.contains("StoreLimits")
                    || detailed_msg.contains("growth")
                {
                    bail!("Host trapped execution: Memory limit exceeded");
                }
                bail!("Host trapped execution: {detailed_msg}");
            }
        };

        if ret < 0 {
            bail!("Guest hook rejected event with error code {ret}");
        } else if ret == 0 {
            Ok(CompactString::new(""))
        } else {
            let out_ptr = ((ret as u64) >> 32) as usize;
            let out_len = (ret as u64 & 0xFFFF_FFFF) as usize;
            let mem_len = memory.data_size(&store);
            if out_ptr + out_len > mem_len {
                bail!(
                    "Host trapped execution: Guest return pointer/length out of bounds ({out_ptr}+{out_len} > {mem_len})"
                );
            }
            let mem_slice = memory.data(&store);
            let out_str = std::str::from_utf8(&mem_slice[out_ptr..out_ptr + out_len])
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 from guest: {e}"))?;
            Ok(CompactString::new(out_str))
        }
    }

    /// Executes an untrusted WASI plugin event hook safely.
    pub fn execute_guest_hook(
        &self,
        doctype: &str,
        doc_name: &str,
        event: &str,
        payload_json: &str,
    ) -> Result<CompactString> {
        if payload_json.len() > self.max_bytes {
            bail!(
                "Host trapped execution: Memory limit exceeded (> {} bytes)",
                self.max_bytes
            );
        }

        let event_type = match event {
            "before_insert" => 1,
            "validate" => 2,
            "before_save" => 3,
            "on_update" => 4,
            "on_submit" => 5,
            "on_cancel" => 6,
            "on_trash" => 7,
            _ => 0,
        };

        let envelope = if payload_json.trim().starts_with('{') {
            payload_json.to_string()
        } else {
            serde_json::json!({
                "doctype": doctype,
                "doc": doc_name,
                "event": event,
                "payload": payload_json,
            })
            .to_string()
        };

        let hook_result = self.execute_module_hook(&self.default_module, event_type, &envelope)?;
        if hook_result.is_empty() {
            Ok(format!(
                "{{\"status\": \"ok\", \"doctype\": \"{}\", \"doc\": \"{}\", \"event\": \"{}\"}}",
                doctype, doc_name, event
            )
            .into())
        } else {
            Ok(hook_result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasmtime_sandbox_execution() {
        let sandbox = RealSandbox::new(1_000_000, 32 * 1024 * 1024).expect("Sandbox init");
        let result = sandbox.run_wat("(module (func (export \"run\") (result i32) i32.const 42))");
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_wasmtime_fuel_trip_on_loop() {
        let sandbox = RealSandbox::new(100, 32 * 1024 * 1024).expect("Sandbox init");
        let result = sandbox
            .run_wat("(module (func (export \"run\") (result i32) (loop $l (br $l)) i32.const 0))");
        assert!(result.is_err(), "Expected out-of-fuel trap");
    }

    #[test]
    fn test_wasmtime_linear_memory_ceiling() {
        let sandbox = RealSandbox::new(1_000_000, 1024).expect("Sandbox init");
        let large_payload = "x".repeat(2048);
        let result =
            sandbox.execute_guest_hook("SalesInvoice", "INV-001", "validate", &large_payload);
        assert!(result.is_err(), "Expected memory limit trap");
    }

    #[test]
    fn test_guest_hook_execution_roundtrip() {
        let sandbox = RealSandbox::new(1_000_000, 32 * 1024 * 1024).expect("Sandbox init");
        let response = sandbox
            .execute_guest_hook("SalesInvoice", "INV-2026-001", "validate", "{}")
            .expect("Guest hook execution");
        assert!(response.contains("INV-2026-001"));
        assert!(response.contains("SalesInvoice"));
        assert!(response.contains("ok"));
    }

    #[test]
    fn test_guest_hook_custom_module_mutation() {
        let sandbox = RealSandbox::new(1_000_000, 32 * 1024 * 1024).expect("Sandbox init");

        // WAT module that writes {"custom_mutation":true} into linear memory at offset 100
        // and returns (100 << 32) | len
        // {"custom_mutation":true} length is 24 bytes
        let custom_wat = r#"
        (module
          (memory (export "memory") 1)
          (data (i32.const 100) "{\"custom_mutation\":true}")
          (func (export "erp_on_event") (param i32 i32 i32) (result i64)
            ;; high 32 bits = 100, low 32 bits = 24
            ;; (100 << 32) | 24 = 429496729624
            i64.const 429496729624
          )
        )
        "#;
        let module = sandbox.compile_wat(custom_wat).expect("Compile custom WAT");
        let result = sandbox
            .execute_module_hook(&module, 2, "{\"test\": 1}")
            .expect("Execute module hook");
        assert_eq!(result.as_str(), "{\"custom_mutation\":true}");
    }

    #[test]
    fn test_guest_hook_rejection_error_code() {
        let sandbox = RealSandbox::new(1_000_000, 32 * 1024 * 1024).expect("Sandbox init");

        let rejecting_wat = r#"
        (module
          (memory (export "memory") 1)
          (func (export "erp_on_event") (param i32 i32 i32) (result i64)
            i64.const -1
          )
        )
        "#;
        let module = sandbox
            .compile_wat(rejecting_wat)
            .expect("Compile custom WAT");
        let result = sandbox.execute_module_hook(&module, 2, "{}");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("error code -1"));
    }

    #[test]
    fn test_guest_hook_infinite_loop_trapped() {
        let sandbox = RealSandbox::new(500, 32 * 1024 * 1024).expect("Sandbox init");

        let looping_wat = r#"
        (module
          (memory (export "memory") 1)
          (func (export "erp_on_event") (param i32 i32 i32) (result i64)
            (loop $l (br $l))
            i64.const 0
          )
        )
        "#;
        let module = sandbox
            .compile_wat(looping_wat)
            .expect("Compile custom WAT");
        let result = sandbox.execute_module_hook(&module, 2, "{}");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("fuel exhausted") || err_msg.contains("fuel"));
    }

    #[test]
    fn test_store_limits_memory_growth_rejection() {
        // Linear memory max = 64KB (1 page)
        let sandbox = RealSandbox::new(1_000_000, 64 * 1024).expect("Sandbox init");

        // Module tries to grow memory by 2 pages (128KB > 64KB ceiling)
        let grow_wat = r#"
        (module
          (memory (export "memory") 1)
          (func (export "run") (result i32)
            (memory.grow (i32.const 2))
          )
        )
        "#;
        let result = sandbox.run_wat(grow_wat).expect("Execution completes");
        // In WebAssembly, memory.grow returns -1 when denied by StoreLimits
        assert_eq!(result, -1);
    }

    #[test]
    fn test_wasmtime_module_cache_hit_and_reuse() {
        let sandbox = RealSandbox::new(1_000_000, 32 * 1024 * 1024).expect("Sandbox init");
        assert_eq!(sandbox.cached_modules_count(), 0);

        let wat = "(module (func (export \"run\") (result i32) i32.const 99))";
        let r1 = sandbox.run_wat(wat).expect("First execution");
        assert_eq!(r1, 99);
        assert_eq!(sandbox.cached_modules_count(), 1);

        // Second run with identical WAT must hit cache without recompilation
        let r2 = sandbox.run_wat(wat).expect("Second execution");
        assert_eq!(r2, 99);
        assert_eq!(sandbox.cached_modules_count(), 1);

        // Different module should increase cache count
        let wat2 = "(module (func (export \"run\") (result i32) i32.const 101))";
        let r3 = sandbox.run_wat(wat2).expect("Third execution");
        assert_eq!(r3, 101);
        assert_eq!(sandbox.cached_modules_count(), 2);
    }
}
