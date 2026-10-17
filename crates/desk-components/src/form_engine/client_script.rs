//! Dynamic Form Engine & Client Script Runtime (`desk-components::form_engine::client_script`).
//!
//! Provides upstream ERPNext client-side scripting and dynamic form evaluation:
//! - Expression evaluator for `depends_on`, `mandatory_depends_on`, and `read_only_depends_on`
//! - Cascading `fetch_from` Link field value resolution
//! - Sandboxed Rhai Client Script execution for form events (`onload`, `refresh`, `validate`, `on_change`).

use compact_str::CompactString;
use rhai::{Dynamic, Engine, Scope};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Client script execution errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ClientScriptError {
    #[error("Expression evaluation error: {0}")]
    EvaluationError(String),
    #[error("Client validation failed (throw): {0}")]
    ValidationError(String),
    #[error("Script operation limit exceeded (possible infinite loop)")]
    OperationLimitExceeded,
}

/// Dynamic Field State evaluated for a form field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DynamicFieldState {
    pub is_visible: bool,
    pub is_mandatory: bool,
    pub is_read_only: bool,
    pub value: Value,
}

/// Dynamic Form Expression Evaluator.
pub struct DynamicFormEvaluator;

impl DynamicFormEvaluator {
    /// Evaluates a `depends_on` rule against current document state.
    ///
    /// Supports:
    /// - `eval:<expression>` (e.g. `eval:doc.status == 'Submitted'`, `eval:doc.grand_total > 500`)
    /// - Simple field name truthiness (e.g. `is_tax_exempt`)
    #[must_use]
    pub fn evaluate_condition(condition: &str, doc: &Value) -> bool {
        let condition = condition.trim();
        if condition.is_empty() {
            return true;
        }

        if let Some(expr) = condition.strip_prefix("eval:") {
            Self::eval_rhai_bool(expr.trim(), doc)
        } else {
            // Field truthiness check
            match doc.get(condition) {
                Some(Value::Bool(b)) => *b,
                Some(Value::Number(n)) => n.as_f64().is_some_and(|v| v != 0.0),
                Some(Value::String(s)) => !s.trim().is_empty(),
                Some(Value::Array(a)) => !a.is_empty(),
                Some(Value::Object(o)) => !o.is_empty(),
                _ => false,
            }
        }
    }

    /// Evaluates full dynamic state for a field.
    #[must_use]
    pub fn evaluate_field_state(
        fieldname: &str,
        depends_on: Option<&str>,
        mandatory_depends_on: Option<&str>,
        read_only_depends_on: Option<&str>,
        default_reqd: bool,
        default_read_only: bool,
        doc: &Value,
    ) -> DynamicFieldState {
        let is_visible = depends_on.is_none_or(|c| Self::evaluate_condition(c, doc));
        let is_mandatory =
            default_reqd || mandatory_depends_on.is_some_and(|c| Self::evaluate_condition(c, doc));
        let is_read_only = default_read_only
            || read_only_depends_on.is_some_and(|c| Self::evaluate_condition(c, doc));
        let value = doc.get(fieldname).cloned().unwrap_or(Value::Null);

        DynamicFieldState {
            is_visible,
            is_mandatory,
            is_read_only,
            value,
        }
    }

    /// Resolves `fetch_from` cascade when a linked document field changes.
    ///
    /// Format: `<link_fieldname>.<source_fieldname>` (e.g. `customer.tax_id` or `item.standard_rate`).
    pub fn resolve_fetch_from<F>(
        fetch_from: &str,
        doc: &Value,
        mut get_linked_doc: F,
    ) -> Option<(CompactString, Value)>
    where
        F: FnMut(&str, &str) -> Option<Value>, // (link_doctype, link_value) -> linked_doc
    {
        let mut parts = fetch_from.split('.');
        let link_fieldname = parts.next()?;
        let source_fieldname = parts.next()?;

        let link_val = doc.get(link_fieldname)?.as_str()?;
        if link_val.is_empty() {
            return None;
        }

        let linked_doc = get_linked_doc(link_fieldname, link_val)?;
        let fetched_val = linked_doc.get(source_fieldname)?.clone();

        Some((source_fieldname.into(), fetched_val))
    }

    fn eval_rhai_bool(expr: &str, doc: &Value) -> bool {
        let mut engine = Engine::new();
        engine.set_max_operations(10_000);

        let mut scope = Scope::new();

        // Convert JSON value to Rhai Dynamic map
        if let Ok(dynamic_doc) = serde_json::from_value::<Dynamic>(doc.clone()) {
            scope.push("doc", dynamic_doc);
        }

        engine
            .eval_with_scope::<bool>(&mut scope, expr)
            .unwrap_or(false)
    }
}

/// Sandboxed Client Script Engine executing custom form hooks in Rhai.
pub struct ClientScriptEngine {
    engine: Engine,
    scripts: HashMap<CompactString, CompactString>, // doctype -> script
}

impl Default for ClientScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientScriptEngine {
    /// Creates a new sandboxed ClientScriptEngine.
    #[must_use]
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(25_000);

        Self {
            engine,
            scripts: HashMap::new(),
        }
    }

    /// Registers a client script for a given DocType.
    pub fn register_script(&mut self, doctype: CompactString, script: CompactString) {
        self.scripts.insert(doctype, script);
    }

    /// Executes `validate` client script hook on document mutation.
    pub fn run_validate(&self, doctype: &str, doc: &mut Value) -> Result<(), ClientScriptError> {
        let Some(script) = self.scripts.get(doctype) else {
            return Ok(());
        };

        let mut scope = Scope::new();
        let dynamic_doc: Dynamic = serde_json::from_value(doc.clone())
            .map_err(|e| ClientScriptError::EvaluationError(e.to_string()))?;
        scope.push("doc", dynamic_doc);

        // Execute script with validate wrapper
        let wrapped_script = format!(
            "{script}\nif is_def_fn(\"validate\", 1) {{\n    let res = validate(doc);\n    if type_of(res) == \"map\" {{ doc = res; }}\n}}\n"
        );

        let ast = self
            .engine
            .compile(&wrapped_script)
            .map_err(|e| ClientScriptError::EvaluationError(e.to_string()))?;

        self.engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("Runtime error")
                    || err_str.contains("throw")
                    || err_str.contains("Validation")
                {
                    ClientScriptError::ValidationError(err_str)
                } else if err_str.contains("Too many operations") {
                    ClientScriptError::OperationLimitExceeded
                } else {
                    ClientScriptError::EvaluationError(err_str)
                }
            })?;

        // Pull back mutated doc
        if let Some(json_doc) = scope
            .get_value::<Dynamic>("doc")
            .and_then(|d| serde_json::to_value(d).ok())
        {
            *doc = json_doc;
        }

        Ok(())
    }

    /// Executes `on_change` client script hook when a specific field value changes.
    pub fn run_on_change(
        &self,
        doctype: &str,
        fieldname: &str,
        doc: &mut Value,
    ) -> Result<(), ClientScriptError> {
        let Some(script) = self.scripts.get(doctype) else {
            return Ok(());
        };

        let mut scope = Scope::new();
        let dynamic_doc: Dynamic = serde_json::from_value(doc.clone())
            .map_err(|e| ClientScriptError::EvaluationError(e.to_string()))?;
        scope.push("doc", dynamic_doc);
        scope.push("fieldname", fieldname.to_string());

        let wrapped_script = format!(
            "{script}\nif is_def_fn(\"on_change\", 2) {{\n    let res = on_change(doc, fieldname);\n    if type_of(res) == \"map\" {{ doc = res; }}\n}}\n"
        );

        let ast = self
            .engine
            .compile(&wrapped_script)
            .map_err(|e| ClientScriptError::EvaluationError(e.to_string()))?;

        self.engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(|e| ClientScriptError::EvaluationError(e.to_string()))?;

        if let Some(json_doc) = scope
            .get_value::<Dynamic>("doc")
            .and_then(|d| serde_json::to_value(d).ok())
        {
            *doc = json_doc;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_dynamic_depends_on_and_mandatory_evaluation() {
        let doc = json!({
            "status": "Draft",
            "grand_total": 1500,
            "has_discount": true,
            "discount_code": ""
        });

        // 1. depends_on expression eval
        assert!(DynamicFormEvaluator::evaluate_condition(
            "eval:doc.grand_total > 1000",
            &doc
        ));
        assert!(!DynamicFormEvaluator::evaluate_condition(
            "eval:doc.status == 'Submitted'",
            &doc
        ));

        // 2. Simple truthiness
        assert!(DynamicFormEvaluator::evaluate_condition(
            "has_discount",
            &doc
        ));
        assert!(!DynamicFormEvaluator::evaluate_condition(
            "discount_code",
            &doc
        ));

        // 3. Full dynamic field state
        let state = DynamicFormEvaluator::evaluate_field_state(
            "discount_percentage",
            Some("has_discount"),
            Some("eval:doc.grand_total > 1000"),
            Some("eval:doc.status == 'Submitted'"),
            false,
            false,
            &doc,
        );

        assert!(state.is_visible);
        assert!(state.is_mandatory); // grand_total > 1000 triggers mandatory
        assert!(!state.is_read_only); // status is Draft, so not read-only
    }

    #[test]
    fn test_fetch_from_cascading_resolution() {
        let doc = json!({
            "customer": "CUST-0001",
            "tax_id": ""
        });

        let fetched = DynamicFormEvaluator::resolve_fetch_from(
            "customer.tax_id",
            &doc,
            |link_field, link_val| {
                if link_field == "customer" && link_val == "CUST-0001" {
                    Some(json!({
                        "name": "CUST-0001",
                        "customer_name": "Apex Global Inc",
                        "tax_id": "DE-998877665"
                    }))
                } else {
                    None
                }
            },
        );

        assert_eq!(
            fetched,
            Some(("tax_id".into(), Value::String("DE-998877665".into())))
        );
    }

    #[test]
    fn test_client_script_engine_mutation_and_validation() {
        let mut engine = ClientScriptEngine::new();

        let script = r#"
            fn on_change(doc, fieldname) {
                if fieldname == "qty" || fieldname == "rate" {
                    doc.amount = doc.qty * doc.rate;
                }
                return doc;
            }

            fn validate(doc) {
                if doc.amount < 0 {
                    throw "Amount cannot be negative!";
                }
            }
        "#;

        engine.register_script("Sales Invoice Item".into(), script.into());

        let mut doc = json!({
            "qty": 5,
            "rate": 20,
            "amount": 0
        });

        // 1. Run on_change hook -> computes amount = 100
        engine
            .run_on_change("Sales Invoice Item", "qty", &mut doc)
            .expect("on_change failed");
        assert_eq!(doc["amount"], 100);

        // 2. Run validate hook on positive amount -> passes
        assert!(engine.run_validate("Sales Invoice Item", &mut doc).is_ok());

        // 3. Set negative amount -> validate throws ValidationError
        doc["amount"] = json!(-50);
        let err = engine.run_validate("Sales Invoice Item", &mut doc);
        assert!(matches!(err, Err(ClientScriptError::ValidationError(_))));
    }
}
