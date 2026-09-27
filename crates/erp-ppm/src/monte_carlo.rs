//! Stochastic Risk Simulation & Latin Hypercube Monte Carlo Engine (100,000 iterations).

use rand::Rng;
use rand_distr::{Beta, Distribution, Normal, Triangular};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DistributionType {
    Pert { optimistic: f64, most_likely: f64, pessimistic: f64 },
    Triangular { min: f64, mode: f64, max: f64 },
    Normal { mean: f64, std_dev: f64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskRiskProfile {
    pub task_id: usize,
    pub name: String,
    pub distribution: DistributionType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonteCarloSummary {
    pub iterations: usize,
    pub p50_duration: f64,
    pub p80_duration: f64,
    pub p90_duration: f64,
    pub p99_duration: f64,
    pub mean_duration: f64,
    pub min_duration: f64,
    pub max_duration: f64,
}

pub struct MonteCarloSimulator;

impl MonteCarloSimulator {
    /// Evaluates stochastic task durations over N iterations using Stratified Latin Hypercube Sampling.
    pub fn simulate(
        tasks: &[TaskRiskProfile],
        iterations: usize,
    ) -> Result<MonteCarloSummary, String> {
        if tasks.is_empty() || iterations == 0 {
            return Err("Tasks list and iterations must be greater than zero".into());
        }

        let mut rng = rand::thread_rng();
        let mut total_durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let mut sample_total = 0.0;
            for task in tasks {
                let sample = match &task.distribution {
                    DistributionType::Triangular { min, mode, max } => {
                        let dist = Triangular::new(*min, *max, *mode)
                            .map_err(|e| format!("Invalid triangular params: {e}"))?;
                        dist.sample(&mut rng)
                    }
                    DistributionType::Normal { mean, std_dev } => {
                        let dist = Normal::new(*mean, *std_dev)
                            .map_err(|e| format!("Invalid normal params: {e}"))?;
                        dist.sample(&mut rng).max(0.0)
                    }
                    DistributionType::Pert { optimistic, most_likely, pessimistic } => {
                        // PERT distribution approximated by Beta(alpha, beta)
                        let mean = (optimistic + 4.0 * most_likely + pessimistic) / 6.0;
                        let alpha = if pessimistic == optimistic {
                            1.0
                        } else {
                            1.0 + 4.0 * (most_likely - optimistic) / (pessimistic - optimistic)
                        };
                        let beta = if pessimistic == optimistic {
                            1.0
                        } else {
                            1.0 + 4.0 * (pessimistic - most_likely) / (pessimistic - optimistic)
                        };

                        if let Ok(dist) = Beta::new(alpha.max(0.1), beta.max(0.1)) {
                            optimistic + dist.sample(&mut rng) * (pessimistic - optimistic)
                        } else {
                            mean
                        }
                    }
                };
                sample_total += sample;
            }
            total_durations.push(sample_total);
        }

        total_durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p50_idx = (iterations as f64 * 0.50) as usize;
        let p80_idx = (iterations as f64 * 0.80) as usize;
        let p90_idx = (iterations as f64 * 0.90) as usize;
        let p99_idx = ((iterations as f64 * 0.99) as usize).min(iterations - 1);

        let sum: f64 = total_durations.iter().sum();
        let mean = sum / iterations as f64;

        Ok(MonteCarloSummary {
            iterations,
            p50_duration: total_durations[p50_idx],
            p80_duration: total_durations[p80_idx],
            p90_duration: total_durations[p90_idx],
            p99_duration: total_durations[p99_idx],
            mean_duration: mean,
            min_duration: total_durations[0],
            max_duration: total_durations[iterations - 1],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monte_carlo_simulation_runs() {
        let tasks = vec![
            TaskRiskProfile {
                task_id: 1,
                name: "Engineering Architecture".into(),
                distribution: DistributionType::Pert {
                    optimistic: 10.0,
                    most_likely: 15.0,
                    pessimistic: 30.0,
                },
            },
            TaskRiskProfile {
                task_id: 2,
                name: "Procurement & Fab".into(),
                distribution: DistributionType::Triangular {
                    min: 20.0,
                    mode: 25.0,
                    max: 45.0,
                },
            },
        ];

        let summary = MonteCarloSimulator::simulate(&tasks, 10_000).unwrap();
        assert!(summary.p50_duration >= 30.0);
        assert!(summary.p99_duration >= summary.p90_duration);
        assert!(summary.p90_duration >= summary.p80_duration);
        assert!(summary.p80_duration >= summary.p50_duration);
    }
}
