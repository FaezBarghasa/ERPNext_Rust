//! Finite Capacity Scheduling (FCS) & Sequence-Dependent Setup Optimization (Traveling Salesperson Model).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductionJob {
    pub job_id: String,
    pub product_code: String,
    pub processing_time_hours: f64,
    pub due_date_hours: f64,
    pub tardiness_penalty_weight: f64,
    pub required_certification: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkCenterSchedule {
    pub center_id: String,
    pub capacity_hours: f64,
    pub scheduled_sequence: Vec<String>,
    pub total_setup_time_hours: f64,
    pub total_makespan_hours: f64,
}

pub struct SequenceOptimizer;

impl SequenceOptimizer {
    /// Computes sequence-dependent changeover cost S(j, k) between consecutive products.
    #[must_use]
    pub fn setup_cost(prod_a: &str, prod_b: &str) -> f64 {
        if prod_a == prod_b {
            0.0 // Same product family -> zero changeover
        } else {
            1.5 // 1.5 hours clean-down & tool recalibration
        }
    }

    /// Solves sequence optimization using greedy heuristic with changeover minimization.
    pub fn optimize_schedule(center_id: &str, capacity: f64, jobs: &[ProductionJob]) -> WorkCenterSchedule {
        if jobs.is_empty() {
            return WorkCenterSchedule {
                center_id: center_id.to_string(),
                capacity_hours: capacity,
                scheduled_sequence: vec![],
                total_setup_time_hours: 0.0,
                total_makespan_hours: 0.0,
            };
        }

        let mut unassigned: Vec<ProductionJob> = jobs.to_vec();
        // Sort initial list by earliest due date (EDD)
        unassigned.sort_by(|a, b| a.due_date_hours.partial_cmp(&b.due_date_hours).unwrap());

        let mut sequence = Vec::with_capacity(jobs.len());
        let mut current_product = String::new();
        let mut total_setup = 0.0;
        let mut total_time = 0.0;

        while !unassigned.is_empty() {
            let mut best_idx = 0;
            let mut min_cost = f64::MAX;

            for (idx, job) in unassigned.iter().enumerate() {
                let s_cost = Self::setup_cost(&current_product, &job.product_code);
                let urgency = (job.due_date_hours - (total_time + job.processing_time_hours + s_cost)).min(0.0).abs();
                let score = s_cost * 10.0 + urgency * job.tardiness_penalty_weight;

                if score < min_cost {
                    min_cost = score;
                    best_idx = idx;
                }
            }

            let next_job = unassigned.remove(best_idx);
            let s_time = Self::setup_cost(&current_product, &next_job.product_code);
            total_setup += s_time;
            total_time += s_time + next_job.processing_time_hours;
            current_product = next_job.product_code.clone();
            sequence.push(next_job.job_id);
        }

        WorkCenterSchedule {
            center_id: center_id.to_string(),
            capacity_hours: capacity,
            scheduled_sequence: sequence,
            total_setup_time_hours: total_setup,
            total_makespan_hours: total_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_setup_optimization() {
        let jobs = vec![
            ProductionJob {
                job_id: "JOB-A1".into(),
                product_code: "PROD-RED".into(),
                processing_time_hours: 4.0,
                due_date_hours: 10.0,
                tardiness_penalty_weight: 1.0,
                required_certification: None,
            },
            ProductionJob {
                job_id: "JOB-B1".into(),
                product_code: "PROD-BLUE".into(),
                processing_time_hours: 3.0,
                due_date_hours: 20.0,
                tardiness_penalty_weight: 1.0,
                required_certification: None,
            },
            ProductionJob {
                job_id: "JOB-A2".into(),
                product_code: "PROD-RED".into(),
                processing_time_hours: 5.0,
                due_date_hours: 15.0,
                tardiness_penalty_weight: 1.0,
                required_certification: None,
            },
        ];

        let schedule = SequenceOptimizer::optimize_schedule("CNC-CENTER-01", 40.0, &jobs);
        assert_eq!(schedule.scheduled_sequence.len(), 3);
        assert!(schedule.total_makespan_hours <= 15.0);
    }
}
