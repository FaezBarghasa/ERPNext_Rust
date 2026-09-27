//! Closed-Loop CAPA (Corrective and Preventive Action) & 8D Root-Cause Ishikawa Trees.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EightDPhase {
    D1EstablishTeam,
    D2DescribeProblem,
    D3ContainmentAction,
    D4RootCauseAnalysis,
    D5CorrectiveActionChosen,
    D6CorrectiveActionImplemented,
    D7PreventRecurrence,
    D8CongratulateTeam,
    Closed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IshikawaCategory {
    pub name: String, // Man, Machine, Method, Material, Measurement, Milieu (Environment)
    pub causes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EightDReport {
    pub ncr_number: String,
    pub title: String,
    pub current_phase: EightDPhase,
    pub root_cause_5whys: Vec<String>,
    pub ishikawa_tree: Vec<IshikawaCategory>,
    pub containment_actions: Vec<String>,
    pub permanent_corrective_actions: Vec<String>,
    pub verification_results: Option<String>,
}

impl EightDReport {
    #[must_use]
    pub fn new(ncr_number: String, title: String) -> Self {
        Self {
            ncr_number,
            title,
            current_phase: EightDPhase::D1EstablishTeam,
            root_cause_5whys: Vec::new(),
            ishikawa_tree: vec![
                IshikawaCategory {
                    name: "Man".into(),
                    causes: vec![],
                },
                IshikawaCategory {
                    name: "Machine".into(),
                    causes: vec![],
                },
                IshikawaCategory {
                    name: "Method".into(),
                    causes: vec![],
                },
                IshikawaCategory {
                    name: "Material".into(),
                    causes: vec![],
                },
                IshikawaCategory {
                    name: "Measurement".into(),
                    causes: vec![],
                },
                IshikawaCategory {
                    name: "Environment".into(),
                    causes: vec![],
                },
            ],
            containment_actions: Vec::new(),
            permanent_corrective_actions: Vec::new(),
            verification_results: None,
        }
    }

    pub fn advance_phase(&mut self, next: EightDPhase) {
        self.current_phase = next;
    }

    pub fn add_cause(&mut self, category: &str, cause: String) {
        if let Some(cat) = self
            .ishikawa_tree
            .iter_mut()
            .find(|c| c.name.eq_ignore_ascii_case(category))
        {
            cat.causes.push(cause);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_8d_capa_workflow() {
        let mut capa = EightDReport::new(
            "NCR-2026-088".into(),
            "Tooling Spindle Thermal Drift".into(),
        );
        capa.add_cause("Machine", "Bearing heat dissipation degradation".into());
        capa.add_cause(
            "Environment",
            "Ambient summer temperature spike in shopfloor".into(),
        );
        capa.root_cause_5whys = vec![
            "Why 1: Spindle overheated".into(),
            "Why 2: Coolant flow dropped".into(),
            "Why 3: Filter clogged".into(),
            "Why 4: Maintenance schedule missed".into(),
            "Why 5: Sensor threshold alarm was misconfigured".into(),
        ];
        capa.advance_phase(EightDPhase::D4RootCauseAnalysis);
        assert_eq!(capa.current_phase, EightDPhase::D4RootCauseAnalysis);
        assert_eq!(capa.root_cause_5whys.len(), 5);
    }
}
