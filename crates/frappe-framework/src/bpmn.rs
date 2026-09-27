//! BPMN 2.0 Virtual Machine and workflow token execution engine.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    StartEvent,
    EndEvent,
    UserTask { assignee_role: String },
    ServiceTask { script: String },
    ExclusiveGateway,
    ParallelGateway,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SequenceFlow {
    pub id: String,
    pub source_ref: String,
    pub target_ref: String,
    pub condition_expression: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub name: String,
    pub activity_type: ActivityType,
    pub incoming: Vec<String>,
    pub outgoing: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BpmnProcessDefinition {
    pub id: String,
    pub name: String,
    pub nodes: HashMap<String, FlowNode>,
    pub flows: HashMap<String, SequenceFlow>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessInstance {
    pub id: String,
    pub process_id: String,
    pub active_tokens: HashSet<String>,
    pub completed_nodes: Vec<String>,
    pub variables: HashMap<String, serde_json::Value>,
    pub is_completed: bool,
}

impl ProcessInstance {
    pub fn new(id: String, process_id: String, start_node_id: String) -> Self {
        let mut active = HashSet::new();
        active.insert(start_node_id);
        Self {
            id,
            process_id,
            active_tokens: active,
            completed_nodes: Vec::new(),
            variables: HashMap::new(),
            is_completed: false,
        }
    }
}

pub struct BpmnEngine;

impl BpmnEngine {
    /// Advances execution tokens until a UserTask, EndEvent, or wait state is reached.
    pub fn step(
        def: &BpmnProcessDefinition,
        instance: &mut ProcessInstance,
    ) -> Result<Vec<String>, String> {
        let current_tokens: Vec<String> = instance.active_tokens.drain().collect();
        let mut next_tokens = HashSet::new();

        for node_id in current_tokens {
            let node = def
                .nodes
                .get(&node_id)
                .ok_or_else(|| format!("Node '{node_id}' not found in process definition"))?;

            instance.completed_nodes.push(node_id.clone());

            match &node.activity_type {
                ActivityType::StartEvent | ActivityType::ServiceTask { .. } => {
                    for flow_id in &node.outgoing {
                        if let Some(flow) = def.flows.get(flow_id) {
                            next_tokens.insert(flow.target_ref.clone());
                        }
                    }
                }
                ActivityType::ExclusiveGateway => {
                    let mut routed = false;
                    for flow_id in &node.outgoing {
                        if let Some(flow) = def.flows.get(flow_id) {
                            let should_take = match &flow.condition_expression {
                                Some(expr) => instance
                                    .variables
                                    .get(expr)
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                None => true,
                            };
                            if should_take {
                                next_tokens.insert(flow.target_ref.clone());
                                routed = true;
                                break;
                            }
                        }
                    }
                    if !routed && !node.outgoing.is_empty() {
                        if let Some(flow) = def.flows.get(&node.outgoing[0]) {
                            next_tokens.insert(flow.target_ref.clone());
                        }
                    }
                }
                ActivityType::ParallelGateway => {
                    for flow_id in &node.outgoing {
                        if let Some(flow) = def.flows.get(flow_id) {
                            next_tokens.insert(flow.target_ref.clone());
                        }
                    }
                }
                ActivityType::UserTask { .. } => {
                    // Stays at UserTask until user interaction / completion
                    next_tokens.insert(node_id);
                }
                ActivityType::EndEvent => {
                    if next_tokens.is_empty() {
                        instance.is_completed = true;
                    }
                }
            }
        }

        instance.active_tokens = next_tokens;
        if instance.active_tokens.is_empty() {
            instance.is_completed = true;
        }

        Ok(instance.active_tokens.iter().cloned().collect())
    }

    /// User completes a pending user task.
    pub fn complete_user_task(
        def: &BpmnProcessDefinition,
        instance: &mut ProcessInstance,
        task_id: &str,
    ) -> Result<(), String> {
        if !instance.active_tokens.remove(task_id) {
            return Err(format!("Task '{task_id}' is not an active token"));
        }
        let node = def
            .nodes
            .get(task_id)
            .ok_or_else(|| format!("Task node '{task_id}' not found"))?;

        instance.completed_nodes.push(task_id.to_string());
        for flow_id in &node.outgoing {
            if let Some(flow) = def.flows.get(flow_id) {
                instance.active_tokens.insert(flow.target_ref.clone());
            }
        }
        Self::step(def, instance)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpmn_workflow_step_execution() {
        let mut nodes = HashMap::new();
        let mut flows = HashMap::new();

        nodes.insert(
            "start".into(),
            FlowNode {
                id: "start".into(),
                name: "Start".into(),
                activity_type: ActivityType::StartEvent,
                incoming: vec![],
                outgoing: vec!["f1".into()],
            },
        );
        flows.insert(
            "f1".into(),
            SequenceFlow {
                id: "f1".into(),
                source_ref: "start".into(),
                target_ref: "task1".into(),
                condition_expression: None,
            },
        );
        nodes.insert(
            "task1".into(),
            FlowNode {
                id: "task1".into(),
                name: "Review Invoice".into(),
                activity_type: ActivityType::UserTask {
                    assignee_role: "Accountant".into(),
                },
                incoming: vec!["f1".into()],
                outgoing: vec!["f2".into()],
            },
        );
        flows.insert(
            "f2".into(),
            SequenceFlow {
                id: "f2".into(),
                source_ref: "task1".into(),
                target_ref: "end".into(),
                condition_expression: None,
            },
        );
        nodes.insert(
            "end".into(),
            FlowNode {
                id: "end".into(),
                name: "End".into(),
                activity_type: ActivityType::EndEvent,
                incoming: vec!["f2".into()],
                outgoing: vec![],
            },
        );

        let process = BpmnProcessDefinition {
            id: "proc_invoice".into(),
            name: "Invoice Approval".into(),
            nodes,
            flows,
        };

        let mut instance =
            ProcessInstance::new("inst_1".into(), "proc_invoice".into(), "start".into());
        let active = BpmnEngine::step(&process, &mut instance).unwrap();
        assert_eq!(active, vec!["task1".to_string()]);
        assert!(!instance.is_completed);

        BpmnEngine::complete_user_task(&process, &mut instance, "task1").unwrap();
        assert!(instance.is_completed);
    }
}
