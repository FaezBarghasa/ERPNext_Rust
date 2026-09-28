use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Timesheet record tracking employee project hours.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectTimesheet {
    /// Timesheet document name.
    pub name: String,
    /// Employee identifier.
    pub employee: String,
    /// Project identifier.
    pub project: String,
    /// Task identifier (optional).
    pub task: Option<String>,
    /// Activity date.
    pub activity_date: NaiveDate,
    /// Total logged hours.
    pub total_hours: Decimal,
    /// Billable hours.
    pub billable_hours: Decimal,
    /// Billing rate per hour.
    pub billing_rate: Decimal,
    /// Whether this timesheet has been invoiced to the customer.
    pub is_billed: bool,
    /// Linked Sales Invoice number (if billed).
    pub sales_invoice: Option<String>,
}

impl ProjectTimesheet {
    /// Total billable amount: `billable_hours * billing_rate`.
    pub fn billable_amount(&self) -> Decimal {
        self.billable_hours * self.billing_rate
    }
}

/// Timesheet billing accumulator and invoicing helper.
pub struct TimesheetBillingManager;

impl TimesheetBillingManager {
    /// Filters unbilled timesheets for a project and calculates aggregate hours and billable revenue.
    pub fn calculate_unbilled_summary(
        project: &str,
        timesheets: &[ProjectTimesheet],
    ) -> (Decimal, Decimal, Vec<String>) {
        let mut total_hours = Decimal::ZERO;
        let mut total_amount = Decimal::ZERO;
        let mut unbilled_ids = Vec::new();

        for ts in timesheets {
            if ts.project == project && !ts.is_billed {
                total_hours += ts.billable_hours;
                total_amount += ts.billable_amount();
                unbilled_ids.push(ts.name.clone());
            }
        }

        (total_hours, total_amount, unbilled_ids)
    }

    /// Marks timesheets as billed and links the Sales Invoice voucher number.
    pub fn mark_timesheets_billed(
        timesheets: &mut [ProjectTimesheet],
        invoice_no: &str,
        target_ids: &[String],
    ) {
        for ts in timesheets.iter_mut() {
            if target_ids.contains(&ts.name) {
                ts.is_billed = true;
                ts.sales_invoice = Some(invoice_no.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_unbilled_timesheet_aggregation_and_billing() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();
        let mut timesheets = vec![
            ProjectTimesheet {
                name: "TS-001".into(),
                employee: "EMP-01".into(),
                project: "PROJ-AI-01".into(),
                task: Some("Architecture".into()),
                activity_date: date,
                total_hours: dec!(8.0),
                billable_hours: dec!(8.0),
                billing_rate: dec!(150.0), // $1,200
                is_billed: false,
                sales_invoice: None,
            },
            ProjectTimesheet {
                name: "TS-002".into(),
                employee: "EMP-02".into(),
                project: "PROJ-AI-01".into(),
                task: Some("Frontend".into()),
                activity_date: date,
                total_hours: dec!(4.0),
                billable_hours: dec!(4.0),
                billing_rate: dec!(100.0), // $400
                is_billed: false,
                sales_invoice: None,
            },
            ProjectTimesheet {
                name: "TS-003".into(),
                employee: "EMP-01".into(),
                project: "PROJ-OTHER".into(),
                task: None,
                activity_date: date,
                total_hours: dec!(5.0),
                billable_hours: dec!(5.0),
                billing_rate: dec!(150.0),
                is_billed: false,
                sales_invoice: None,
            },
        ];

        let (hours, amount, ids) =
            TimesheetBillingManager::calculate_unbilled_summary("PROJ-AI-01", &timesheets);
        assert_eq!(hours, dec!(12.0));
        assert_eq!(amount, dec!(1600.0));
        assert_eq!(ids.len(), 2);

        TimesheetBillingManager::mark_timesheets_billed(&mut timesheets, "SINV-2026-001", &ids);
        assert!(timesheets[0].is_billed);
        assert_eq!(
            timesheets[0].sales_invoice,
            Some("SINV-2026-001".to_string())
        );
        assert!(timesheets[1].is_billed);
        assert!(!timesheets[2].is_billed);
    }
}
