//! DMN 1.4 Decision Model and Notation engine with S-FEEL expression evaluator.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitPolicy {
    Unique,
    First,
    Priority,
    CollectSum,
    CollectMin,
    CollectMax,
    CollectCount,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOp {
    Equals(String),
    NotEquals(String),
    LessThan(Decimal),
    LessThanOrEqual(Decimal),
    GreaterThan(Decimal),
    GreaterThanOrEqual(Decimal),
    Between(Decimal, Decimal),
    In(Vec<String>),
    Any,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionRule {
    pub input_conditions: Vec<ConditionOp>,
    pub output_values: Vec<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionTable {
    pub id: String,
    pub name: String,
    pub hit_policy: HitPolicy,
    pub rules: Vec<DecisionRule>,
}

impl DecisionTable {
    pub fn new(id: String, name: String, hit_policy: HitPolicy) -> Self {
        Self {
            id,
            name,
            hit_policy,
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, conditions: Vec<ConditionOp>, outputs: Vec<serde_json::Value>) {
        self.rules.push(DecisionRule {
            input_conditions: conditions,
            output_values: outputs,
        });
    }

    /// Evaluates inputs against the decision table and applies hit policies.
    pub fn evaluate(&self, inputs: &[serde_json::Value]) -> Result<Vec<serde_json::Value>, String> {
        let mut matched_rules = Vec::new();
        for rule in &self.rules {
            if self.match_rule(rule, inputs) {
                matched_rules.push(rule.output_values.clone());
                if self.hit_policy == HitPolicy::First {
                    return Ok(rule.output_values.clone());
                }
            }
        }

        if matched_rules.is_empty() {
            return Err(format!("No matching DMN rule found in table '{}'", self.id));
        }

        match self.hit_policy {
            HitPolicy::Unique => {
                if matched_rules.len() > 1 {
                    Err("DMN HitPolicy::Unique violated: multiple rules matched".into())
                } else {
                    Ok(matched_rules.into_iter().flatten().collect())
                }
            }
            HitPolicy::First => Ok(matched_rules.into_iter().next().unwrap_or_default()),
            HitPolicy::Priority | HitPolicy::CollectCount | HitPolicy::CollectSum | HitPolicy::CollectMin | HitPolicy::CollectMax => {
                Ok(matched_rules.into_iter().flatten().collect())
            }
        }
    }

    fn match_rule(&self, rule: &DecisionRule, inputs: &[serde_json::Value]) -> bool {
        for (idx, cond) in rule.input_conditions.iter().enumerate() {
            let Some(val) = inputs.get(idx) else { return false };
            match cond {
                ConditionOp::Any => continue,
                ConditionOp::Equals(expected) => {
                    let s = val.as_str().map(ToString::to_string).unwrap_or_else(|| val.to_string());
                    if &s != expected { return false; }
                }
                ConditionOp::NotEquals(expected) => {
                    let s = val.as_str().map(ToString::to_string).unwrap_or_else(|| val.to_string());
                    if &s == expected { return false; }
                }
                ConditionOp::LessThan(limit) => {
                    let Ok(d) = val.to_string().trim_matches('"').parse::<Decimal>() else { return false };
                    if d >= *limit { return false; }
                }
                ConditionOp::LessThanOrEqual(limit) => {
                    let Ok(d) = val.to_string().trim_matches('"').parse::<Decimal>() else { return false };
                    if d > *limit { return false; }
                }
                ConditionOp::GreaterThan(limit) => {
                    let Ok(d) = val.to_string().trim_matches('"').parse::<Decimal>() else { return false };
                    if d <= *limit { return false; }
                }
                ConditionOp::GreaterThanOrEqual(limit) => {
                    let Ok(d) = val.to_string().trim_matches('"').parse::<Decimal>() else { return false };
                    if d < *limit { return false; }
                }
                ConditionOp::Between(low, high) => {
                    let Ok(d) = val.to_string().trim_matches('"').parse::<Decimal>() else { return false };
                    if d < *low || d > *high { return false; }
                }
                ConditionOp::In(list) => {
                    let s = val.as_str().map(ToString::to_string).unwrap_or_else(|| val.to_string());
                    if !list.iter().any(|item| item == &s) { return false; }
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_dmn_s_feel_evaluation() {
        let mut table = DecisionTable::new("pricing_discount".into(), "Discount Policy".into(), HitPolicy::Unique);
        table.add_rule(
            vec![ConditionOp::In(vec!["VIP".into(), "Enterprise".into()]), ConditionOp::GreaterThanOrEqual(dec!(10000))],
            vec![serde_json::json!(0.25), serde_json::json!("Tier 1 Approved")],
        );
        table.add_rule(
            vec![ConditionOp::Equals("Retail".into()), ConditionOp::Any],
            vec![serde_json::json!(0.05), serde_json::json!("Standard Rate")],
        );

        let res = table.evaluate(&[serde_json::json!("VIP"), serde_json::json!(15000)]).unwrap();
        assert_eq!(res[0], serde_json::json!(0.25));
        assert_eq!(res[1], serde_json::json!("Tier 1 Approved"));
    }
}
