use rhai::{AST, Dynamic, Engine, EvalAltResult, Scope};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Scripting errors occurring during Rhai sandbox execution.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScriptError {
    /// Computation exceeded allowable instruction limit.
    #[error("Execution operation limit exceeded")]
    OperationLimitExceeded,
    /// Script syntax error.
    #[error("Script syntax compilation failed: {0}")]
    SyntaxError(String),
    /// Explicit error thrown by script user code.
    #[error("Script error [{0}]: {1}")]
    ThrownError(String, String),
    /// General script runtime execution failure.
    #[error("Script execution failed: {0}")]
    ExecutionFailed(String),
}

/// Document lifecycle state events dispatching to Rhai scripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleEvent {
    /// Before document AST validation.
    BeforeValidate,
    /// During document validation.
    Validate,
    /// Before saving document to database.
    BeforeSave,
    /// Immediately after saving document.
    AfterSave,
    /// Before submitting submittable document.
    BeforeSubmit,
    /// On submit: books accounting and inventory ledgers.
    OnSubmit,
    /// Before cancelling document.
    BeforeCancel,
    /// On cancel: reverses ledger postings.
    OnCancel,
}

/// Sandboxed Rhai execution engine configured with strict resource limits.
pub struct RhaiHookEngine {
    engine: Engine,
}

impl Default for RhaiHookEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RhaiHookEngine {
    /// Creates a hardened, sandboxed Rhai script engine.
    #[must_use]
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Architectural constraints from Milestone 1.5
        engine.set_max_operations(50_000);
        engine.set_max_call_levels(16);
        engine.set_max_string_size(64_000);
        engine.set_max_array_size(10_000);

        // Register logging functions
        engine.register_fn("log_info", |msg: &str| {
            tracing::info!(target: "rhai_script", "{}", msg);
        });
        engine.register_fn("log_warn", |msg: &str| {
            tracing::warn!(target: "rhai_script", "{}", msg);
        });
        engine.register_fn("log_error", |msg: &str| {
            tracing::error!(target: "rhai_script", "{}", msg);
        });

        // Register explicit throw function
        engine.register_fn(
            "throw_error",
            |code: &str, msg: &str| -> Result<(), Box<EvalAltResult>> {
                Err(format!("{code}:{msg}").into())
            },
        );

        Self { engine }
    }

    /// Compiles script string into an AST.
    pub fn compile(&self, script: &str) -> Result<AST, ScriptError> {
        self.engine
            .compile(script)
            .map_err(|e| ScriptError::SyntaxError(e.to_string()))
    }

    /// Executes a script on a mutable JSON document in the specified lifecycle event.
    pub fn dispatch_hook(
        &self,
        event: LifecycleEvent,
        doc: &mut serde_json::Value,
        script: &str,
    ) -> Result<(), ScriptError> {
        let ast = self.compile(script)?;
        let mut scope = Scope::new();

        // Convert serde_json::Value reference to Rhai Dynamic Map
        let doc_dynamic = rhai::serde::to_dynamic(&*doc)
            .map_err(|e| ScriptError::ExecutionFailed(e.to_string()))?;

        scope.push("doc", doc_dynamic);
        scope.push("event", format!("{event:?}"));

        let result: Result<Dynamic, Box<EvalAltResult>> =
            self.engine.eval_ast_with_scope(&mut scope, &ast);

        match result {
            Ok(_) => {
                if let Some(updated_doc) = scope.get_value::<Dynamic>("doc")
                    && let Ok(json_val) = rhai::serde::from_dynamic(&updated_doc)
                {
                    *doc = json_val;
                }

                Ok(())
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("Too many operations") || err_str.contains("operation limit") {
                    Err(ScriptError::OperationLimitExceeded)
                } else if err_str.contains(':') {
                    let mut parts = err_str.splitn(2, ':');
                    let code = parts.next().unwrap_or("ERR").trim().to_string();
                    let msg = parts.next().unwrap_or(&err_str).trim().to_string();
                    Err(ScriptError::ThrownError(code, msg))
                } else {
                    Err(ScriptError::ExecutionFailed(err_str))
                }
            }
        }
    }
}
