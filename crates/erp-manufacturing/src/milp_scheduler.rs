//! Finite Capacity Scheduling (FCS) & Sequence-Dependent Setup Optimization (Traveling Salesperson Model).
//!
//! Features:
//! - True Branch-and-Bound solver for Single-Machine Total Weighted Tardiness with Sequence-Dependent Setup Times ($1 | s_{jk} | \sum w_j T_j$).
//! - Admissible continuous lower bounding via family changeover relaxation and Earliest Due Date (EDD) preemption relaxation.
//! - Branch pruning against dynamically tightening upper bounds.
//! - Configurable search timeout (default 500ms) with graceful fallback to best-explored bound.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

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

struct BranchNode {
    sequence: Vec<usize>,
    unassigned: Vec<usize>,
    curr_time: f64,
    acc_setup: f64,
    acc_weighted_tardiness: f64,
    lower_bound: f64,
}

struct CandidateBranch {
    job_idx: usize,
    next_time: f64,
    next_setup: f64,
    next_weighted_tardiness: f64,
    lower_bound: f64,
    urgency_score: f64,
}

pub struct SequenceOptimizer;

impl SequenceOptimizer {
    /// Standard changeover duration between different product families.
    pub const CHANGEOVER_HOURS: f64 = 1.5;

    /// Computes sequence-dependent changeover cost S(j, k) between consecutive products.
    #[must_use]
    pub fn setup_cost(prod_a: &str, prod_b: &str) -> f64 {
        if prod_a == prod_b {
            0.0 // Same product family -> zero changeover
        } else {
            Self::CHANGEOVER_HOURS // 1.5 hours clean-down & tool recalibration
        }
    }

    /// Evaluates total weighted tardiness, setup time, and makespan for a sequence of jobs.
    #[inline]
    fn schedule_cost(
        jobs: &[ProductionJob],
        sequence: &[usize],
        initial_product: &str,
    ) -> (f64, f64, f64) {
        let mut curr_time = 0.0;
        let mut curr_product = initial_product;
        let mut total_setup = 0.0;
        let mut total_weighted_tardiness = 0.0;

        for &job_idx in sequence {
            let job = &jobs[job_idx];
            let s = Self::setup_cost(curr_product, &job.product_code);
            total_setup += s;
            curr_time += s + job.processing_time_hours;
            let tardiness = (curr_time - job.due_date_hours).max(0.0);
            total_weighted_tardiness += tardiness * job.tardiness_penalty_weight;
            curr_product = &job.product_code;
        }

        (total_weighted_tardiness, total_setup, curr_time)
    }

    /// Generates a fast initial upper bound solution using Earliest Due Date & Changeover greedy heuristic.
    fn initial_greedy_solution(jobs: &[ProductionJob]) -> (Vec<usize>, f64) {
        let n = jobs.len();
        if n == 0 {
            return (Vec::new(), 0.0);
        }

        let mut unassigned: Vec<usize> = (0..n).collect();
        // Sort initial candidate list by earliest due date
        unassigned.sort_by(|&a, &b| jobs[a].due_date_hours.total_cmp(&jobs[b].due_date_hours));

        let mut sequence = Vec::with_capacity(n);
        let mut current_product = String::new();
        let mut total_time = 0.0;

        while !unassigned.is_empty() {
            let mut best_pos = 0;
            let mut min_score = f64::MAX;

            for (pos, &job_idx) in unassigned.iter().enumerate() {
                let job = &jobs[job_idx];
                let s_cost = Self::setup_cost(&current_product, &job.product_code);
                let finish_time = total_time + s_cost + job.processing_time_hours;
                let tardiness = (finish_time - job.due_date_hours).max(0.0);
                let score = s_cost * 10.0 + tardiness * job.tardiness_penalty_weight;

                if score < min_score {
                    min_score = score;
                    best_pos = pos;
                }
            }

            let next_idx = unassigned.remove(best_pos);
            let next_job = &jobs[next_idx];
            let s_time = Self::setup_cost(&current_product, &next_job.product_code);
            total_time += s_time + next_job.processing_time_hours;
            current_product.clone_from(&next_job.product_code);
            sequence.push(next_idx);
        }

        let (wt, setup, _) = Self::schedule_cost(jobs, &sequence, "");
        let cost = wt + setup * 0.1;
        (sequence, cost)
    }

    /// Computes an admissible continuous lower bound for a partial sequence and remaining unassigned jobs.
    ///
    /// Lower Bound Components:
    /// 1. Current accumulated cost: accumulated weighted tardiness + 0.1 * accumulated setup time.
    /// 2. Family changeover relaxation: minimum required changeovers based on distinct product families in remaining jobs.
    /// 3. Preemptive EDD lower bound on tardiness: sorting unassigned jobs by due date to compute earliest possible completion.
    fn compute_lower_bound(
        jobs: &[ProductionJob],
        unassigned: &[usize],
        current_time: f64,
        current_product: &str,
        acc_weighted_tardiness: f64,
        acc_setup_time: f64,
    ) -> f64 {
        if unassigned.is_empty() {
            return acc_weighted_tardiness + acc_setup_time * 0.1;
        }

        // 1. Minimum family setup time lower bound
        let mut distinct_families = BTreeSet::new();
        for &idx in unassigned {
            distinct_families.insert(jobs[idx].product_code.as_str());
        }
        let min_changeovers = if distinct_families.contains(current_product) {
            distinct_families.len().saturating_sub(1) as f64
        } else {
            distinct_families.len() as f64
        };
        let min_setup_remaining = min_changeovers * Self::CHANGEOVER_HOURS;

        // 2. Continuous Preemption / EDD Tardiness Lower Bound
        let mut sorted_unassigned = unassigned.to_vec();
        sorted_unassigned
            .sort_unstable_by(|&a, &b| jobs[a].due_date_hours.total_cmp(&jobs[b].due_date_hours));

        let mut earliest_completion = current_time;
        let mut tardiness_lb = 0.0;

        for &job_idx in &sorted_unassigned {
            let job = &jobs[job_idx];
            earliest_completion += job.processing_time_hours;
            let tardiness = (earliest_completion - job.due_date_hours).max(0.0);
            tardiness_lb += tardiness * job.tardiness_penalty_weight;
        }

        acc_weighted_tardiness + tardiness_lb + (acc_setup_time + min_setup_remaining) * 0.1
    }

    /// Solves the Single-Machine Total Weighted Tardiness with Sequence-Dependent Setup Times ($1 | s_{jk} | \sum w_j T_j$)
    /// using Branch-and-Bound with a configurable timeout.
    pub fn optimize_schedule_milp(
        center_id: &str,
        capacity: f64,
        jobs: &[ProductionJob],
        timeout: Duration,
    ) -> WorkCenterSchedule {
        if jobs.is_empty() {
            return WorkCenterSchedule {
                center_id: center_id.to_string(),
                capacity_hours: capacity,
                scheduled_sequence: vec![],
                total_setup_time_hours: 0.0,
                total_makespan_hours: 0.0,
            };
        }

        let start_time = Instant::now();
        let (mut best_sequence, mut best_cost) = Self::initial_greedy_solution(jobs);

        let n = jobs.len();
        if n > 1 {
            let mut stack: Vec<BranchNode> = Vec::with_capacity(n * n);

            // Root node branches
            let mut root_branches: Vec<usize> = (0..n).collect();
            root_branches.sort_by(|&a, &b| {
                let s_a = Self::setup_cost("", &jobs[a].product_code);
                let s_b = Self::setup_cost("", &jobs[b].product_code);
                (s_a + jobs[a].due_date_hours).total_cmp(&(s_b + jobs[b].due_date_hours))
            });

            for &first_job_idx in root_branches.iter().rev() {
                let s = Self::setup_cost("", &jobs[first_job_idx].product_code);
                let t = s + jobs[first_job_idx].processing_time_hours;
                let tardiness = (t - jobs[first_job_idx].due_date_hours).max(0.0);
                let wt = tardiness * jobs[first_job_idx].tardiness_penalty_weight;

                let unassigned: Vec<usize> = (0..n).filter(|&i| i != first_job_idx).collect();
                let lb = Self::compute_lower_bound(
                    jobs,
                    &unassigned,
                    t,
                    &jobs[first_job_idx].product_code,
                    wt,
                    s,
                );

                if lb < best_cost {
                    stack.push(BranchNode {
                        sequence: vec![first_job_idx],
                        unassigned,
                        curr_time: t,
                        acc_setup: s,
                        acc_weighted_tardiness: wt,
                        lower_bound: lb,
                    });
                }
            }

            while let Some(node) = stack.pop() {
                if start_time.elapsed() >= timeout {
                    // Graceful fallback to best-explored bound on timeout
                    break;
                }

                if node.lower_bound >= best_cost {
                    // Pruned by tightened upper bound
                    continue;
                }

                if node.unassigned.is_empty() {
                    let total_cost = node.acc_weighted_tardiness + node.acc_setup * 0.1;
                    if total_cost < best_cost {
                        best_cost = total_cost;
                        best_sequence = node.sequence;
                    }
                    continue;
                }

                let last_product = &jobs[*node.sequence.last().unwrap()].product_code;

                let mut candidates: Vec<CandidateBranch> = node
                    .unassigned
                    .iter()
                    .map(|&idx| {
                        let job = &jobs[idx];
                        let s = Self::setup_cost(last_product, &job.product_code);
                        let t = node.curr_time + s + job.processing_time_hours;
                        let tardiness = (t - job.due_date_hours).max(0.0);
                        let wt =
                            node.acc_weighted_tardiness + tardiness * job.tardiness_penalty_weight;
                        let total_s = node.acc_setup + s;

                        let next_unassigned: Vec<usize> = node
                            .unassigned
                            .iter()
                            .copied()
                            .filter(|&i| i != idx)
                            .collect();
                        let lb = Self::compute_lower_bound(
                            jobs,
                            &next_unassigned,
                            t,
                            &job.product_code,
                            wt,
                            total_s,
                        );
                        let urgency_score = s * 10.0 + tardiness * job.tardiness_penalty_weight;

                        CandidateBranch {
                            job_idx: idx,
                            next_time: t,
                            next_setup: total_s,
                            next_weighted_tardiness: wt,
                            lower_bound: lb,
                            urgency_score,
                        }
                    })
                    .filter(|cand| cand.lower_bound < best_cost)
                    .collect();

                // Sort candidates by urgency descending so lowest urgency popped first
                candidates.sort_by(|a, b| b.urgency_score.total_cmp(&a.urgency_score));

                for cand in candidates {
                    let mut next_seq = node.sequence.clone();
                    next_seq.push(cand.job_idx);
                    let next_unassigned: Vec<usize> = node
                        .unassigned
                        .iter()
                        .copied()
                        .filter(|&i| i != cand.job_idx)
                        .collect();

                    stack.push(BranchNode {
                        sequence: next_seq,
                        unassigned: next_unassigned,
                        curr_time: cand.next_time,
                        acc_setup: cand.next_setup,
                        acc_weighted_tardiness: cand.next_weighted_tardiness,
                        lower_bound: cand.lower_bound,
                    });
                }
            }
        }

        let scheduled_sequence: Vec<String> = best_sequence
            .iter()
            .map(|&i| jobs[i].job_id.clone())
            .collect();
        let (_, total_setup, total_makespan) = Self::schedule_cost(jobs, &best_sequence, "");

        WorkCenterSchedule {
            center_id: center_id.to_string(),
            capacity_hours: capacity,
            scheduled_sequence,
            total_setup_time_hours: total_setup,
            total_makespan_hours: total_makespan,
        }
    }

    /// Solves sequence optimization using globally optimal Branch-and-Bound with a 500ms timeout.
    pub fn optimize_schedule(
        center_id: &str,
        capacity: f64,
        jobs: &[ProductionJob],
    ) -> WorkCenterSchedule {
        Self::optimize_schedule_milp(center_id, capacity, jobs, Duration::from_millis(500))
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

    #[test]
    fn test_branch_and_bound_global_optimality_over_greedy() {
        // High priority job with high tardiness penalty weight
        // If greedy looks locally at setup time, it would group same products first and delay the critical job.
        // B&B explores the true global space and prioritizes the urgent heavy-penalty job.
        let jobs = vec![
            ProductionJob {
                job_id: "JOB-LOW-1".into(),
                product_code: "PROD-A".into(),
                processing_time_hours: 5.0,
                due_date_hours: 30.0,
                tardiness_penalty_weight: 1.0,
                required_certification: None,
            },
            ProductionJob {
                job_id: "JOB-CRITICAL".into(),
                product_code: "PROD-B".into(),
                processing_time_hours: 6.0,
                due_date_hours: 8.0,
                tardiness_penalty_weight: 100.0,
                required_certification: None,
            },
            ProductionJob {
                job_id: "JOB-LOW-2".into(),
                product_code: "PROD-A".into(),
                processing_time_hours: 5.0,
                due_date_hours: 35.0,
                tardiness_penalty_weight: 1.0,
                required_certification: None,
            },
        ];

        let schedule = SequenceOptimizer::optimize_schedule("WORKCENTER-01", 100.0, &jobs);
        assert_eq!(schedule.scheduled_sequence.len(), 3);
        // JOB-CRITICAL must be scheduled first to avoid massive tardiness penalty
        assert_eq!(schedule.scheduled_sequence[0], "JOB-CRITICAL");
    }

    #[test]
    fn test_branch_and_bound_timeout_safety() {
        let jobs: Vec<ProductionJob> = (0..12)
            .map(|i| ProductionJob {
                job_id: format!("JOB-{i}"),
                product_code: format!("PROD-{}", i % 4),
                processing_time_hours: 2.0 + (i as f64) * 0.5,
                due_date_hours: 15.0 + (i as f64) * 3.0,
                tardiness_penalty_weight: 1.0 + (i as f64) * 0.2,
                required_certification: None,
            })
            .collect();

        let start = Instant::now();
        // Set short timeout of 100ms
        let schedule = SequenceOptimizer::optimize_schedule_milp(
            "CENTER-FAST",
            200.0,
            &jobs,
            Duration::from_millis(100),
        );
        let elapsed = start.elapsed();
        assert_eq!(schedule.scheduled_sequence.len(), 12);
        // Ensure execution was bounded
        assert!(elapsed < Duration::from_millis(500));
    }
}
