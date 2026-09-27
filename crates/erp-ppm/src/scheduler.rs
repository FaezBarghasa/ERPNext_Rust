//! Dual-engine scheduling core: Critical Path Method (CPM) & Critical Chain Project Management (CCPM).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    FinishToStart, // FS
    StartToStart,   // SS
    FinishToFinish, // FF
    StartToFinish,  // SF
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub predecessor_id: usize,
    pub dep_type: DependencyType,
    pub lag: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleTask {
    pub id: usize,
    pub name: String,
    pub duration: i64,
    pub dependencies: Vec<Dependency>,
    pub early_start: i64,
    pub early_finish: i64,
    pub late_start: i64,
    pub late_finish: i64,
    pub total_float: i64,
    pub free_float: i64,
    pub is_critical: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectBuffer {
    pub name: String,
    pub size: i64,
    pub consumed: i64,
}

impl ProjectBuffer {
    #[must_use]
    pub fn penetration_percentage(&self) -> f64 {
        if self.size == 0 {
            0.0
        } else {
            (self.consumed as f64 / self.size as f64) * 100.0
        }
    }

    #[must_use]
    pub fn alert_zone(&self) -> &'static str {
        let p = self.penetration_percentage();
        if p < 33.33 {
            "Green"
        } else if p < 66.66 {
            "Yellow"
        } else {
            "Red"
        }
    }
}

pub struct CpmEngine {
    pub tasks: Vec<ScheduleTask>,
    pub project_buffer: Option<ProjectBuffer>,
}

impl CpmEngine {
    #[must_use]
    pub fn new(tasks: Vec<ScheduleTask>) -> Self {
        Self {
            tasks,
            project_buffer: None,
        }
    }

    /// Executes both forward and backward CPM passes to compute floats and critical path.
    pub fn compute(&mut self) -> Result<i64, String> {
        let n = self.tasks.len();

        // 1. Forward pass
        for i in 0..n {
            let mut es = 0;
            for dep in &self.tasks[i].dependencies {
                if dep.predecessor_id >= n {
                    return Err(format!("Invalid predecessor {} for task {}", dep.predecessor_id, i));
                }
                let pred = &self.tasks[dep.predecessor_id];
                let candidate = match dep.dep_type {
                    DependencyType::FinishToStart => pred.early_finish + dep.lag,
                    DependencyType::StartToStart => pred.early_start + dep.lag,
                    DependencyType::FinishToFinish => pred.early_finish + dep.lag - self.tasks[i].duration,
                    DependencyType::StartToFinish => pred.early_start + dep.lag - self.tasks[i].duration,
                };
                es = es.max(candidate);
            }
            self.tasks[i].early_start = es;
            self.tasks[i].early_finish = es + self.tasks[i].duration;
        }

        let project_duration = self.tasks.iter().map(|t| t.early_finish).max().unwrap_or(0);

        // 2. Backward pass
        for i in (0..n).rev() {
            let mut lf = project_duration;
            for j in (i + 1)..n {
                for dep in &self.tasks[j].dependencies {
                    if dep.predecessor_id == i {
                        let succ = &self.tasks[j];
                        let candidate = match dep.dep_type {
                            DependencyType::FinishToStart => succ.late_start - dep.lag,
                            DependencyType::StartToStart => succ.late_start - dep.lag + self.tasks[i].duration,
                            DependencyType::FinishToFinish => succ.late_finish - dep.lag,
                            DependencyType::StartToFinish => succ.late_finish - dep.lag + self.tasks[i].duration,
                        };
                        lf = lf.min(candidate);
                    }
                }
            }
            self.tasks[i].late_finish = lf;
            self.tasks[i].late_start = lf - self.tasks[i].duration;
            self.tasks[i].total_float = self.tasks[i].late_start - self.tasks[i].early_start;
            self.tasks[i].is_critical = self.tasks[i].total_float == 0;
        }

        // 3. Free Float Calculation
        for i in 0..n {
            let mut min_succ_es = project_duration;
            let mut has_succ = false;
            for j in 0..n {
                for dep in &self.tasks[j].dependencies {
                    if dep.predecessor_id == i && dep.dep_type == DependencyType::FinishToStart {
                        has_succ = true;
                        min_succ_es = min_succ_es.min(self.tasks[j].early_start - dep.lag);
                    }
                }
            }
            self.tasks[i].free_float = if has_succ {
                (min_succ_es - self.tasks[i].early_finish).max(0)
            } else {
                self.tasks[i].total_float
            };
        }

        // 4. Goldratt CCPM Project Buffer (50% of critical path sum or RSEM)
        let critical_duration: i64 = self.tasks.iter().filter(|t| t.is_critical).map(|t| t.duration).sum();
        let buffer_size = (critical_duration / 3).max(1);
        self.project_buffer = Some(ProjectBuffer {
            name: "Project Buffer".into(),
            size: buffer_size,
            consumed: 0,
        });

        Ok(project_duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpm_forward_backward_passes() {
        let tasks = vec![
            ScheduleTask {
                id: 0,
                name: "Design".into(),
                duration: 5,
                dependencies: vec![],
                early_start: 0,
                early_finish: 0,
                late_start: 0,
                late_finish: 0,
                total_float: 0,
                free_float: 0,
                is_critical: false,
            },
            ScheduleTask {
                id: 1,
                name: "Foundation".into(),
                duration: 10,
                dependencies: vec![Dependency {
                    predecessor_id: 0,
                    dep_type: DependencyType::FinishToStart,
                    lag: 0,
                }],
                early_start: 0,
                early_finish: 0,
                late_start: 0,
                late_finish: 0,
                total_float: 0,
                free_float: 0,
                is_critical: false,
            },
            ScheduleTask {
                id: 2,
                name: "Permits".into(),
                duration: 3,
                dependencies: vec![Dependency {
                    predecessor_id: 0,
                    dep_type: DependencyType::FinishToStart,
                    lag: 0,
                }],
                early_start: 0,
                early_finish: 0,
                late_start: 0,
                late_finish: 0,
                total_float: 0,
                free_float: 0,
                is_critical: false,
            },
        ];

        let mut engine = CpmEngine::new(tasks);
        let duration = engine.compute().unwrap();
        assert_eq!(duration, 15);
        assert!(engine.tasks[0].is_critical);
        assert!(engine.tasks[1].is_critical);
        assert!(!engine.tasks[2].is_critical);
        assert_eq!(engine.tasks[2].total_float, 7);
    }
}
