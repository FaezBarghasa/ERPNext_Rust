//! Statistical Process Control (SPC), Real-Time X-bar / R Charts & Nelson Rules.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpcRuleViolation {
    Rule1PointBeyond3Sigma,
    Rule2NinePointsSameSideOfCenter,
    Rule3SixPointsTrendingUpOrDown,
    Rule4FourteenPointsAlternating,
    Rule5TwoOfThreePointsBeyond2Sigma,
    Rule6FourOfFivePointsBeyond1Sigma,
    Rule7FifteenConsecutiveWithin1Sigma,
    Rule8EightConsecutiveBeyond1SigmaBothSides,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpcSubgroup {
    pub subgroup_id: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub measurements: Vec<f64>,
    pub mean: f64,
    pub range: f64,
    pub std_dev: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControlLimits {
    pub grand_mean: f64,
    pub ucl: f64, // Upper Control Limit (+3 sigma)
    pub lcl: f64, // Lower Control Limit (-3 sigma)
    pub sigma: f64,
}

pub struct SpcEngine;

impl SpcEngine {
    pub fn compute_subgroup(id: usize, measurements: &[f64]) -> SpcSubgroup {
        let n = measurements.len() as f64;
        let mean = if measurements.is_empty() {
            0.0
        } else {
            measurements.iter().sum::<f64>() / n
        };

        let min = measurements.iter().copied().fold(f64::INFINITY, f64::min);
        let max = measurements.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let range = if measurements.is_empty() { 0.0 } else { max - min };

        let variance = if measurements.len() > 1 {
            measurements.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)
        } else {
            0.0
        };

        SpcSubgroup {
            subgroup_id: id,
            timestamp: chrono::Utc::now(),
            measurements: measurements.to_vec(),
            mean,
            range,
            std_dev: variance.sqrt(),
        }
    }

    pub fn compute_limits(subgroups: &[SpcSubgroup]) -> ControlLimits {
        if subgroups.is_empty() {
            return ControlLimits { grand_mean: 0.0, ucl: 0.0, lcl: 0.0, sigma: 0.0 };
        }
        let grand_mean: f64 = subgroups.iter().map(|s| s.mean).sum::<f64>() / subgroups.len() as f64;
        let pooled_std: f64 = subgroups.iter().map(|s| s.std_dev).sum::<f64>() / subgroups.len() as f64;
        let sigma = pooled_std;

        ControlLimits {
            grand_mean,
            ucl: grand_mean + 3.0 * sigma,
            lcl: grand_mean - 3.0 * sigma,
            sigma,
        }
    }

    /// Checks the latest subgroups for Nelson & Western Electric rule violations.
    pub fn check_rules(subgroups: &[SpcSubgroup], limits: &ControlLimits) -> Vec<SpcRuleViolation> {
        let mut violations = Vec::new();
        let n = subgroups.len();
        if n == 0 {
            return violations;
        }

        let latest = &subgroups[n - 1];

        // Rule 1: Point beyond 3 sigma (UCL or LCL)
        if latest.mean > limits.ucl || latest.mean < limits.lcl {
            violations.push(SpcRuleViolation::Rule1PointBeyond3Sigma);
        }

        // Rule 2: 9 points in a row on one side of center line
        if n >= 9 {
            let last_9 = &subgroups[(n - 9)..n];
            let all_above = last_9.iter().all(|s| s.mean > limits.grand_mean);
            let all_below = last_9.iter().all(|s| s.mean < limits.grand_mean);
            if all_above || all_below {
                violations.push(SpcRuleViolation::Rule2NinePointsSameSideOfCenter);
            }
        }

        // Rule 3: 6 points in a row continually increasing or decreasing
        if n >= 6 {
            let last_6 = &subgroups[(n - 6)..n];
            let increasing = last_6.windows(2).all(|w| w[1].mean > w[0].mean);
            let decreasing = last_6.windows(2).all(|w| w[1].mean < w[0].mean);
            if increasing || decreasing {
                violations.push(SpcRuleViolation::Rule3SixPointsTrendingUpOrDown);
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spc_nelson_rule_violations() {
        let mut subgroups = Vec::new();
        for i in 0..10 {
            subgroups.push(SpcEngine::compute_subgroup(i, &[10.1, 10.2, 10.15, 10.25]));
        }

        let limits = ControlLimits {
            grand_mean: 10.0,
            ucl: 10.5,
            lcl: 9.5,
            sigma: 0.166,
        };

        let violations = SpcEngine::check_rules(&subgroups, &limits);
        // All 9 points are strictly above grand mean 10.0
        assert!(violations.contains(&SpcRuleViolation::Rule2NinePointsSameSideOfCenter));
    }
}
