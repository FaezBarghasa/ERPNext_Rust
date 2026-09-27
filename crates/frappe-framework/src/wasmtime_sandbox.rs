//! Real Wasmtime engine: fuel metering + memory cap (Stage 2.3.1).
use anyhow::{Context, Result};
use wasmtime::{Config, Engine, Module, Store};

pub struct RealSandbox {
    engine: Engine,
    fuel: u64,
    max_bytes: usize,
}

struct Limits {
    max_bytes: usize,
    used: usize,
}

impl wasmtime::ResourceLimiter for Limits {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        self.used = desired;
        Ok(desired <= self.max_bytes)
    }
    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        Ok(true)
    }
}

impl RealSandbox {
    pub fn new(fuel: u64, max_mem_bytes: usize) -> Result<Self> {
        let mut cfg = Config::new();
        cfg.consume_fuel(true);
        Ok(Self {
            engine: Engine::new(&cfg)?,
            fuel,
            max_bytes: max_mem_bytes,
        })
    }

    /// Compile + run a wat module exporting `run() -> i32`; proves fuel termination.
    pub fn run_wat(&self, wat: &str) -> Result<i32> {
        let module = Module::new(&self.engine, wat).context("compile")?;
        let limits = Limits {
            max_bytes: self.max_bytes,
            used: 0,
        };
        let mut store = Store::new(&self.engine, limits);
        store.set_fuel(self.fuel).context("set fuel")?;
        store.limiter(|state| state as _);
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).context("instantiate")?;
        let f = instance
            .get_typed_func::<(), i32>(&mut store, "run")
            .context("missing export `run`")?;
        f.call(&mut store, ()).context("trap")
    }
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn wat_runs() {
        let s = RealSandbox::new(1_000_000, 32 * 1024 * 1024).expect("engine");
        let v = s.run_wat("(module (func (export \"run\") (result i32) i32.const 42))");
        assert_eq!(v.unwrap(), 42);
    }
    #[test]
    fn fuel_trips_on_loop() {
        let s = RealSandbox::new(100, 32 * 1024 * 1024).expect("engine");
        let v = s.run_wat("(module (func (export \"run\") (result i32) (local $i i32) (loop $l (local.set $i (i32.add (local.get $i) (i32.const 1))) (br_if $l (i32.lt_s (local.get $i) (i32.const 1000000)))) (local.get $i)))");
        assert!(v.is_err(), "expected out-of-fuel trap, got {:?}", v.unwrap());
    }
}
