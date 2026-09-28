//! Reliability-Centered Maintenance (RCM), Weibull Degradation & FMECA RPN Analysis.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeibullParameters {
    pub beta: f64,  // Shape parameter (<1: infant mortality, =1: random, >1: wear-out)
    pub eta: f64,   // Scale parameter (Characteristic life)
    pub gamma: f64, // Location parameter (failure-free operating time)
}

impl WeibullParameters {
    #[must_use]
    pub fn new(beta: f64, eta: f64, gamma: f64) -> Self {
        Self { beta, eta, gamma }
    }

    /// Reliability function R(t) = exp(-((t - gamma) / eta)^beta)
    #[must_use]
    pub fn reliability(&self, t: f64) -> f64 {
        if t <= self.gamma {
            return 1.0;
        }
        let term = (t - self.gamma) / self.eta;
        (-term.powf(self.beta)).exp()
    }

    /// Hazard rate / failure rate h(t) = (beta / eta) * ((t - gamma) / eta)^(beta - 1)
    #[must_use]
    pub fn hazard_rate(&self, t: f64) -> f64 {
        if t <= self.gamma {
            return 0.0;
        }
        let term = (t - self.gamma) / self.eta;
        (self.beta / self.eta) * term.powf(self.beta - 1.0)
    }

    /// Cumulative failure probability F(t) = 1.0 - R(t)
    #[must_use]
    pub fn cumulative_failure_probability(&self, t: f64) -> f64 {
        1.0 - self.reliability(t)
    }

    /// Determines if preventive maintenance should be triggered when F(t) >= threshold.
    #[must_use]
    pub fn requires_preventive_maintenance(&self, t: f64, threshold_prob: f64) -> bool {
        self.cumulative_failure_probability(t) >= threshold_prob
    }

    #[must_use]
    pub fn failure_regime(&self) -> &'static str {
        if self.beta < 0.95 {
            "Infant Mortality (Burn-In Period)"
        } else if self.beta <= 1.05 {
            "Random Exponential Failure (Useful Life)"
        } else {
            "Wear-Out Degradation (Preventive Replacement Indicated)"
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FmecaRecord {
    pub failure_mode: String,
    pub effect: String,
    pub cause: String,
    pub severity: u8,   // 1 to 10
    pub occurrence: u8, // 1 to 10
    pub detection: u8,  // 1 to 10 (1 = high detectability, 10 = undetectable)
}

impl FmecaRecord {
    #[must_use]
    pub fn risk_priority_number(&self) -> u32 {
        (self.severity as u32) * (self.occurrence as u32) * (self.detection as u32)
    }

    #[must_use]
    pub fn is_high_criticality(&self) -> bool {
        self.risk_priority_number() >= 200 || self.severity >= 9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weibull_reliability_and_fmeca() {
        let pump = WeibullParameters::new(2.5, 10000.0, 0.0);
        assert_eq!(
            pump.failure_regime(),
            "Wear-Out Degradation (Preventive Replacement Indicated)"
        );
        let r_5000 = pump.reliability(5000.0);
        assert!(r_5000 > 0.80 && r_5000 < 0.90);

        let f_5000 = pump.cumulative_failure_probability(5000.0);
        assert!((r_5000 + f_5000 - 1.0).abs() < 1e-6);
        assert!(pump.requires_preventive_maintenance(9500.0, 0.50));
        assert!(!pump.requires_preventive_maintenance(2000.0, 0.50));

        let fmeca = FmecaRecord {
            failure_mode: "Bearing Seizure".into(),
            effect: "Catastrophic Turbine Trip".into(),
            cause: "Lubrication Breakdown".into(),
            severity: 9,
            occurrence: 4,
            detection: 6,
        };
        assert_eq!(fmeca.risk_priority_number(), 216);
        assert!(fmeca.is_high_criticality());
    }
}
