//! Professional Services Automation (PSA) & Time Tracking Engine.
//!
//! Tracks billable/non-billable consulting hours, computes employee utilization rates,
//! calculates labor gross margin, and generates Time-and-Materials (T&M) and Fixed-Price invoices.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::errors::SoftwareBillingError;
use crate::subscription::InvoiceLineItem;

/// Individual recorded time log entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeLog {
    pub id: String,
    pub employee_id: String,
    pub project_id: String,
    pub task_id: Option<String>,
    pub contract_id: Option<String>,
    pub billable_hours: Decimal,
    pub non_billable_hours: Decimal,
    pub billable_rate: Decimal,
    pub cost_rate: Decimal,
    pub log_date: DateTime<Utc>,
    pub description: String,
    pub is_invoiced: bool,
}

impl TimeLog {
    /// Total duration logged across billable and non-billable hours.
    #[must_use]
    pub fn total_hours(&self) -> Decimal {
        self.billable_hours + self.non_billable_hours
    }

    /// Calculated standard billable value (billable hours * standard hourly billing rate).
    #[must_use]
    pub fn billable_value(&self) -> Decimal {
        self.billable_hours * self.billable_rate
    }

    /// Internal direct labor cost (total hours * employee internal hourly cost rate).
    #[must_use]
    pub fn internal_labor_cost(&self) -> Decimal {
        self.total_hours() * self.cost_rate
    }
}

/// Employee utilization and productivity scorecard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeeUtilizationScorecard {
    pub employee_id: String,
    pub total_billable_hours: Decimal,
    pub total_non_billable_hours: Decimal,
    pub total_logged_hours: Decimal,
    pub available_capacity_hours: Decimal,
    /// Utilization percentage: (Billable Hours / Total Available Capacity) * 100
    pub utilization_rate_percent: Decimal,
    /// Total standard billable value produced.
    pub total_billable_value: Decimal,
    /// Total direct employee labor cost incurred.
    pub total_labor_cost: Decimal,
}

/// Project financial metrics summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectPsaSummary {
    pub project_id: String,
    pub total_billable_hours: Decimal,
    pub total_billable_revenue: Decimal,
    pub total_internal_cost: Decimal,
    pub gross_profit: Decimal,
    /// Gross margin percentage: ((Revenue - Cost) / Revenue) * 100
    pub gross_margin_percent: Decimal,
}

/// Generated PSA Invoice for client services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PsaInvoice {
    pub invoice_id: String,
    pub customer_id: String,
    pub project_id: String,
    pub invoice_type: PsaInvoiceType,
    pub line_items: Vec<InvoiceLineItem>,
    pub total_amount: Decimal,
    pub invoiced_time_log_ids: Vec<String>,
    pub issued_at: DateTime<Utc>,
}

/// PSA Invoicing modality.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PsaInvoiceType {
    TimeAndMaterials,
    FixedPriceMilestone {
        milestone_id: String,
        milestone_name: String,
    },
    RetainerOverage {
        included_hours: Decimal,
        overage_hours: Decimal,
    },
}

/// Parameters for generating a Fixed-Price Milestone Invoice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MilestoneInvoiceRequest {
    pub invoice_id: String,
    pub customer_id: String,
    pub project_id: String,
    pub milestone_id: String,
    pub milestone_name: String,
    pub amount: Decimal,
    pub revenue_account: String,
}

/// PSA & Professional Services automation engine.
pub struct PsaEngine;

impl PsaEngine {
    /// Computes employee utilization scorecard for a given period:
    /// $$\text{Utilization Rate} = \frac{\text{Billable Hours}}{\text{Total Available Capacity}} \times 100\%$$
    pub fn compute_employee_utilization(
        employee_id: &str,
        time_logs: &[TimeLog],
        available_capacity_hours: Decimal,
    ) -> Result<EmployeeUtilizationScorecard, SoftwareBillingError> {
        if available_capacity_hours <= Decimal::ZERO {
            return Err(SoftwareBillingError::ZeroCapacity);
        }

        let mut billable = Decimal::ZERO;
        let mut non_billable = Decimal::ZERO;
        let mut billable_val = Decimal::ZERO;
        let mut labor_cost = Decimal::ZERO;

        for log in time_logs.iter().filter(|l| l.employee_id == employee_id) {
            billable += log.billable_hours;
            non_billable += log.non_billable_hours;
            billable_val += log.billable_value();
            labor_cost += log.internal_labor_cost();
        }

        let total_logged = billable + non_billable;
        let utilization = ((billable / available_capacity_hours) * dec!(100)).round_dp(2);

        Ok(EmployeeUtilizationScorecard {
            employee_id: employee_id.to_string(),
            total_billable_hours: billable,
            total_non_billable_hours: non_billable,
            total_logged_hours: total_logged,
            available_capacity_hours,
            utilization_rate_percent: utilization,
            total_billable_value: billable_val,
            total_labor_cost: labor_cost,
        })
    }

    /// Computes project financial performance and gross profit margin.
    #[must_use]
    pub fn compute_project_summary(project_id: &str, time_logs: &[TimeLog]) -> ProjectPsaSummary {
        let mut total_billable_hours = Decimal::ZERO;
        let mut total_rev = Decimal::ZERO;
        let mut total_cost = Decimal::ZERO;

        for log in time_logs.iter().filter(|l| l.project_id == project_id) {
            total_billable_hours += log.billable_hours;
            total_rev += log.billable_value();
            total_cost += log.internal_labor_cost();
        }

        let gross_profit = total_rev - total_cost;
        let gross_margin_percent = if total_rev > Decimal::ZERO {
            ((gross_profit / total_rev) * dec!(100)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        ProjectPsaSummary {
            project_id: project_id.to_string(),
            total_billable_hours,
            total_billable_revenue: total_rev,
            total_internal_cost: total_cost,
            gross_profit,
            gross_margin_percent,
        }
    }

    /// Generates a Time-and-Materials (T&M) invoice for all uninvoiced billable hours on a project.
    pub fn generate_tm_invoice(
        invoice_id: String,
        customer_id: String,
        project_id: String,
        revenue_account: String,
        time_logs: &mut [TimeLog],
        now: DateTime<Utc>,
    ) -> Result<PsaInvoice, SoftwareBillingError> {
        let mut line_items = Vec::new();
        let mut invoiced_ids = Vec::new();
        let mut total_amount = Decimal::ZERO;

        for log in time_logs.iter_mut() {
            if log.project_id == project_id
                && !log.is_invoiced
                && log.billable_hours > Decimal::ZERO
            {
                let amount = log.billable_value();
                total_amount += amount;

                line_items.push(InvoiceLineItem {
                    description: format!(
                        "Time: {} - {} ({} hrs @ ${}/hr)",
                        log.employee_id, log.description, log.billable_hours, log.billable_rate
                    ),
                    quantity: log.billable_hours,
                    unit_price: log.billable_rate,
                    amount,
                    account: revenue_account.clone(),
                });

                log.is_invoiced = true;
                invoiced_ids.push(log.id.clone());
            }
        }

        if line_items.is_empty() {
            return Err(SoftwareBillingError::EmptyInvoice);
        }

        Ok(PsaInvoice {
            invoice_id,
            customer_id,
            project_id,
            invoice_type: PsaInvoiceType::TimeAndMaterials,
            line_items,
            total_amount,
            invoiced_time_log_ids: invoiced_ids,
            issued_at: now,
        })
    }

    /// Generates a Fixed-Price milestone invoice.
    pub fn generate_milestone_invoice(
        req: MilestoneInvoiceRequest,
        now: DateTime<Utc>,
    ) -> Result<PsaInvoice, SoftwareBillingError> {
        if req.amount <= Decimal::ZERO {
            return Err(SoftwareBillingError::InvalidContractTerms(
                "Milestone invoice amount must be positive".into(),
            ));
        }

        let line_items = vec![InvoiceLineItem {
            description: format!("Milestone: {} - {}", req.milestone_id, req.milestone_name),
            quantity: Decimal::ONE,
            unit_price: req.amount,
            amount: req.amount,
            account: req.revenue_account,
        }];

        Ok(PsaInvoice {
            invoice_id: req.invoice_id,
            customer_id: req.customer_id,
            project_id: req.project_id,
            invoice_type: PsaInvoiceType::FixedPriceMilestone {
                milestone_id: req.milestone_id,
                milestone_name: req.milestone_name,
            },
            line_items,
            total_amount: req.amount,
            invoiced_time_log_ids: Vec::new(),
            issued_at: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_employee_utilization_calculation() {
        let now = Utc::now();
        let logs = vec![TimeLog {
            id: "log-1".into(),
            employee_id: "emp-alice".into(),
            project_id: "proj-web".into(),
            task_id: Some("task-backend".into()),
            contract_id: Some("ctr-1".into()),
            billable_hours: dec!(32.0),
            non_billable_hours: dec!(8.0), // 40 hours total
            billable_rate: dec!(150.0),    // $150/hr
            cost_rate: dec!(60.0),         // $60/hr
            log_date: now,
            description: "Built Actix Web microservices".into(),
            is_invoiced: false,
        }];

        // 32 billable hours / 40 available capacity = 80.00% utilization
        let scorecard =
            PsaEngine::compute_employee_utilization("emp-alice", &logs, dec!(40.0)).unwrap();
        assert_eq!(scorecard.utilization_rate_percent, dec!(80.00));
        assert_eq!(scorecard.total_billable_value, dec!(4800.00)); // 32 * 150
        assert_eq!(scorecard.total_labor_cost, dec!(2400.00)); // 40 * 60

        // Project summary
        let proj = PsaEngine::compute_project_summary("proj-web", &logs);
        assert_eq!(proj.total_billable_revenue, dec!(4800.00));
        assert_eq!(proj.gross_profit, dec!(2400.00));
        assert_eq!(proj.gross_margin_percent, dec!(50.00)); // 2400 / 4800 = 50%
    }

    #[test]
    fn test_tm_invoice_generation() {
        let now = Utc::now();
        let mut logs = vec![TimeLog {
            id: "log-10".into(),
            employee_id: "emp-bob".into(),
            project_id: "proj-ai".into(),
            task_id: None,
            contract_id: None,
            billable_hours: dec!(10.0),
            non_billable_hours: dec!(0.0),
            billable_rate: dec!(200.0),
            cost_rate: dec!(80.0),
            log_date: now,
            description: "SurrealDB query optimization".into(),
            is_invoiced: false,
        }];

        let inv = PsaEngine::generate_tm_invoice(
            "INV-TM-01".into(),
            "cust-alpha".into(),
            "proj-ai".into(),
            "4100-Services".into(),
            &mut logs,
            now,
        )
        .unwrap();

        assert_eq!(inv.total_amount, dec!(2000.00));
        assert_eq!(inv.invoiced_time_log_ids, vec!["log-10".to_string()]);
        // Time log marked as invoiced
        assert!(logs[0].is_invoiced);
    }
}
