use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 4 Standard ERPNext Project % Complete Calculation Methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PercentCompleteMethod {
    /// Manually updated by project manager.
    Manual,
    /// Percentage of completed tasks: `(Completed Tasks / Total Tasks) * 100`.
    TaskCompletion,
    /// Average task progress across all tasks: `Σ task.progress / Total Tasks`.
    TaskProgress,
    /// Weighted task progress: `Σ (task.progress * task.weight) / Σ task.weight`.
    TaskWeight,
}

/// Task state required for project completion evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectTaskState {
    /// Task identifier.
    pub task_id: String,
    /// Is the task marked as completed.
    pub is_completed: bool,
    /// Progress percentage (0 - 100).
    pub progress: Decimal,
    /// Custom task weight (e.g. effort/cost weighting).
    pub weight: Decimal,
}

/// Project completion calculation engine.
pub struct ProjectCompletionEngine;

impl ProjectCompletionEngine {
    /// Computes project completion percentage according to the chosen method.
    pub fn calculate_completion(
        method: PercentCompleteMethod,
        manual_percentage: Decimal,
        tasks: &[ProjectTaskState],
    ) -> Decimal {
        if tasks.is_empty() {
            return manual_percentage;
        }

        match method {
            PercentCompleteMethod::Manual => manual_percentage,
            PercentCompleteMethod::TaskCompletion => {
                let completed_count = tasks.iter().filter(|t| t.is_completed).count();
                (Decimal::from(completed_count) / Decimal::from(tasks.len())) * Decimal::from(100)
            }
            PercentCompleteMethod::TaskProgress => {
                let total_progress: Decimal = tasks.iter().map(|t| t.progress).sum();
                total_progress / Decimal::from(tasks.len())
            }
            PercentCompleteMethod::TaskWeight => {
                let total_weight: Decimal = tasks.iter().map(|t| t.weight).sum();
                if total_weight.is_zero() {
                    Decimal::ZERO
                } else {
                    let weighted_sum: Decimal = tasks.iter().map(|t| t.progress * t.weight).sum();
                    weighted_sum / total_weight
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_all_four_completion_methods() {
        let tasks = vec![
            ProjectTaskState {
                task_id: "T1".into(),
                is_completed: true,
                progress: dec!(100.0),
                weight: dec!(10.0),
            },
            ProjectTaskState {
                task_id: "T2".into(),
                is_completed: false,
                progress: dec!(50.0),
                weight: dec!(30.0),
            },
            ProjectTaskState {
                task_id: "T3".into(),
                is_completed: false,
                progress: dec!(0.0),
                weight: dec!(60.0),
            },
        ];

        // 1. Manual
        assert_eq!(
            ProjectCompletionEngine::calculate_completion(
                PercentCompleteMethod::Manual,
                dec!(45.0),
                &tasks
            ),
            dec!(45.0)
        );

        // 2. Task Completion: 1 completed out of 3 = 33.33333333333333333333333333%
        let comp = ProjectCompletionEngine::calculate_completion(
            PercentCompleteMethod::TaskCompletion,
            Decimal::ZERO,
            &tasks,
        );
        assert_eq!(comp.round_dp(2), dec!(33.33));

        // 3. Task Progress: (100 + 50 + 0) / 3 = 50.0%
        assert_eq!(
            ProjectCompletionEngine::calculate_completion(
                PercentCompleteMethod::TaskProgress,
                Decimal::ZERO,
                &tasks
            ),
            dec!(50.0)
        );

        // 4. Task Weight: (100*10 + 50*30 + 0*60) / 100 = (1000 + 1500) / 100 = 25.0%
        assert_eq!(
            ProjectCompletionEngine::calculate_completion(
                PercentCompleteMethod::TaskWeight,
                Decimal::ZERO,
                &tasks
            ),
            dec!(25.0)
        );
    }
}
