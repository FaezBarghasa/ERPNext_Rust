use serde::{Deserialize, Serialize};

/// Task status in collaborative workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    Review,
    Done,
}

/// Collaborative project task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceTask {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: TaskStatus,
    pub assigned_user: Option<String>,
}

/// Real-Time Collaborative Workspace Manager (Milestone 4.3).
#[derive(Default)]
pub struct WorkspaceManager {
    tasks: std::collections::HashMap<String, WorkspaceTask>,
}

impl WorkspaceManager {
    /// Creates a new workspace manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mutates task status.
    pub fn update_task_status(
        &mut self,
        task_id: &str,
        new_status: TaskStatus,
    ) -> Option<WorkspaceTask> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = new_status;
            Some(task.clone())
        } else {
            None
        }
    }

    /// Adds a task to workspace.
    pub fn add_task(&mut self, task: WorkspaceTask) {
        self.tasks.insert(task.id.clone(), task);
    }
}
