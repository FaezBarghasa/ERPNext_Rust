//! Distributed Saga Orchestration Coordinator with forward execution and compensating backward rollbacks.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SagaStep {
    pub name: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SagaStatus {
    Pending,
    InProgress,
    Completed,
    Compensating,
    RolledBack,
    Failed,
}

pub trait SagaAction: Send + Sync {
    fn execute(&self, payload: &serde_json::Value) -> Result<serde_json::Value, String>;
    fn compensate(&self, payload: &serde_json::Value) -> Result<(), String>;
}

pub struct SagaTransaction {
    pub id: String,
    pub name: String,
    pub forward_action: Box<dyn SagaAction>,
    pub payload: serde_json::Value,
}

pub struct SagaCoordinator {
    idempotent_inbox: HashSet<String>,
}

impl Default for SagaCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl SagaCoordinator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            idempotent_inbox: HashSet::new(),
        }
    }

    /// Executes a list of saga steps sequentially. On failure, triggers backward compensation.
    pub fn execute_saga(
        &mut self,
        saga_id: &str,
        transactions: &[SagaTransaction],
    ) -> Result<Vec<serde_json::Value>, String> {
        if self.idempotent_inbox.contains(saga_id) {
            return Err(format!(
                "Saga '{saga_id}' has already been processed (Idempotency Guard)"
            ));
        }

        let mut executed_indices = Vec::new();
        let mut results = Vec::new();

        for (idx, tx) in transactions.iter().enumerate() {
            match tx.forward_action.execute(&tx.payload) {
                Ok(res) => {
                    executed_indices.push(idx);
                    results.push(res);
                }
                Err(err) => {
                    // Trigger backward compensation
                    for &rev_idx in executed_indices.iter().rev() {
                        let comp_tx = &transactions[rev_idx];
                        let _ = comp_tx.forward_action.compensate(&comp_tx.payload);
                    }
                    return Err(format!(
                        "Saga '{saga_id}' failed at step {idx} ('{}'): {err}. Compensating rollback executed.",
                        tx.name
                    ));
                }
            }
        }

        self.idempotent_inbox.insert(saga_id.to_string());
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct MockStep {
        should_fail: bool,
        compensated: Arc<AtomicBool>,
    }

    impl SagaAction for MockStep {
        fn execute(&self, _payload: &serde_json::Value) -> Result<serde_json::Value, String> {
            if self.should_fail {
                Err("Forced failure".into())
            } else {
                Ok(serde_json::json!({ "status": "ok" }))
            }
        }

        fn compensate(&self, _payload: &serde_json::Value) -> Result<(), String> {
            self.compensated.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_saga_forward_and_compensating_rollback() {
        let mut coordinator = SagaCoordinator::new();
        let comp1 = Arc::new(AtomicBool::new(false));

        let tx1 = SagaTransaction {
            id: "tx1".into(),
            name: "Reserve Inventory".into(),
            forward_action: Box::new(MockStep {
                should_fail: false,
                compensated: comp1.clone(),
            }),
            payload: serde_json::json!({ "item": "SKU-1" }),
        };

        let tx2 = SagaTransaction {
            id: "tx2".into(),
            name: "Authorize Payment".into(),
            forward_action: Box::new(MockStep {
                should_fail: true,
                compensated: Arc::new(AtomicBool::new(false)),
            }),
            payload: serde_json::json!({ "amount": 100 }),
        };

        let res = coordinator.execute_saga("saga_order_001", &[tx1, tx2]);
        assert!(res.is_err());
        assert!(comp1.load(Ordering::SeqCst));
    }
}
