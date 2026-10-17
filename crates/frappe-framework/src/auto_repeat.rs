//! Automated Document Recurring Engine (`frappe-framework::auto_repeat`).
//!
//! Provides upstream ERPNext / Frappe Auto Repeat parity:
//! - Periodic recurring schedules: Daily, Weekly, Monthly, Quarterly, HalfYearly, Yearly
//! - Status transitions: Draft, Active, Disabled, Completed
//! - Document cloning and date advancement with boundary clamping
//! - Auto-submission support and notification triggers.

use chrono::{Datelike, Days, Duration, Months, NaiveDate};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Frequency of recurring document generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoRepeatFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    HalfYearly,
    Yearly,
}

/// Lifecycle status of an Auto Repeat configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoRepeatStatus {
    Draft,
    Active,
    Disabled,
    Completed,
}

/// Auto Repeat configuration and schedule tracker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoRepeatRecord {
    pub name: CompactString,
    pub reference_doctype: CompactString,
    pub reference_docname: CompactString,
    pub frequency: AutoRepeatFrequency,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub next_schedule_date: NaiveDate,
    pub status: AutoRepeatStatus,
    pub submit_on_creation: bool,
    pub notify_by_email: bool,
    pub recipients: Vec<CompactString>,
    pub template_data: serde_json::Value,
}

/// Resulting generated recurring document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneratedDocument {
    pub auto_repeat: CompactString,
    pub doctype: CompactString,
    pub reference_docname: CompactString,
    pub posting_date: NaiveDate,
    pub docstatus: i32,
    pub data: serde_json::Value,
}

/// Core engine orchestrating recurring document generation.
#[derive(Debug, Clone, Default)]
pub struct AutoRepeatEngine {
    records: Vec<AutoRepeatRecord>,
    generated: Vec<GeneratedDocument>,
}

impl AutoRepeatEngine {
    /// Creates a new Auto Repeat engine.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new Auto Repeat record.
    pub fn register(&mut self, record: AutoRepeatRecord) {
        self.records.push(record);
    }

    /// Advances a date based on the schedule frequency with calendar day clamping.
    #[must_use]
    pub fn advance_date(date: NaiveDate, frequency: AutoRepeatFrequency) -> NaiveDate {
        match frequency {
            AutoRepeatFrequency::Daily => date.checked_add_days(Days::new(1)).unwrap_or(date),
            AutoRepeatFrequency::Weekly => date + Duration::weeks(1),
            AutoRepeatFrequency::Monthly => {
                date.checked_add_months(Months::new(1)).unwrap_or_else(|| {
                    // Fallback day clamp if month addition overflows (e.g. Jan 31 -> Feb 28)
                    let year = if date.month() == 12 {
                        date.year() + 1
                    } else {
                        date.year()
                    };
                    let month = if date.month() == 12 {
                        1
                    } else {
                        date.month() + 1
                    };
                    NaiveDate::from_ymd_opt(year, month, 28).unwrap_or(date)
                })
            }
            AutoRepeatFrequency::Quarterly => {
                date.checked_add_months(Months::new(3)).unwrap_or_else(|| {
                    let month = (date.month() + 2) % 12 + 1;
                    let year = date.year() + (date.month() + 2) as i32 / 12;
                    NaiveDate::from_ymd_opt(year, month, 28).unwrap_or(date)
                })
            }
            AutoRepeatFrequency::HalfYearly => {
                date.checked_add_months(Months::new(6)).unwrap_or_else(|| {
                    let month = (date.month() + 5) % 12 + 1;
                    let year = date.year() + (date.month() + 5) as i32 / 12;
                    NaiveDate::from_ymd_opt(year, month, 28).unwrap_or(date)
                })
            }
            AutoRepeatFrequency::Yearly => {
                date.checked_add_months(Months::new(12)).unwrap_or_else(|| {
                    NaiveDate::from_ymd_opt(date.year() + 1, date.month(), 28).unwrap_or(date)
                })
            }
        }
    }

    /// Evaluates all active Auto Repeat records and generates documents due on or before `current_date`.
    pub fn process_due_records(&mut self, current_date: NaiveDate) -> Vec<GeneratedDocument> {
        let mut newly_generated = Vec::new();

        for record in &mut self.records {
            if record.status != AutoRepeatStatus::Active {
                continue;
            }

            while record.next_schedule_date <= current_date
                && record.status == AutoRepeatStatus::Active
            {
                if record
                    .end_date
                    .is_some_and(|end| record.next_schedule_date > end)
                {
                    record.status = AutoRepeatStatus::Completed;
                    break;
                }

                // Clone template data and update transaction/posting dates
                let mut doc_data = record.template_data.clone();
                if let serde_json::Value::Object(ref mut map) = doc_data {
                    map.remove("name");
                    map.remove("creation");
                    map.remove("modified");
                    map.insert(
                        "posting_date".into(),
                        serde_json::Value::String(record.next_schedule_date.to_string()),
                    );
                    map.insert(
                        "auto_repeat".into(),
                        serde_json::Value::String(record.name.to_string()),
                    );
                    let docstatus = if record.submit_on_creation { 1 } else { 0 };
                    map.insert(
                        "docstatus".into(),
                        serde_json::Value::Number(docstatus.into()),
                    );
                }

                let generated_doc = GeneratedDocument {
                    auto_repeat: record.name.clone(),
                    doctype: record.reference_doctype.clone(),
                    reference_docname: record.reference_docname.clone(),
                    posting_date: record.next_schedule_date,
                    docstatus: if record.submit_on_creation { 1 } else { 0 },
                    data: doc_data,
                };

                newly_generated.push(generated_doc.clone());
                self.generated.push(generated_doc);

                // Advance next schedule date
                let next_date = Self::advance_date(record.next_schedule_date, record.frequency);
                if record.end_date.is_some_and(|end| next_date > end) {
                    record.status = AutoRepeatStatus::Completed;
                }
                record.next_schedule_date = next_date;
            }
        }

        newly_generated
    }

    /// Returns all registered Auto Repeat records.
    #[must_use]
    pub fn records(&self) -> &[AutoRepeatRecord] {
        &self.records
    }

    /// Returns all generated recurring documents.
    #[must_use]
    pub fn generated(&self) -> &[GeneratedDocument] {
        &self.generated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_auto_repeat_monthly_schedule() {
        let mut engine = AutoRepeatEngine::new();

        let template = json!({
            "doctype": "Sales Invoice",
            "customer": "Cust-001",
            "grand_total": 1200.0,
            "docstatus": 0
        });

        let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();

        engine.register(AutoRepeatRecord {
            name: "AR-SINV-001".into(),
            reference_doctype: "Sales Invoice".into(),
            reference_docname: "ACC-SINV-2026-00001".into(),
            frequency: AutoRepeatFrequency::Monthly,
            start_date,
            end_date: Some(end_date),
            next_schedule_date: start_date,
            status: AutoRepeatStatus::Active,
            submit_on_creation: true,
            notify_by_email: false,
            recipients: Vec::new(),
            template_data: template,
        });

        // 1. Process as of Jan 15 -> Generates Jan 1 invoice, advances to Feb 1
        let gen_jan = engine.process_due_records(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
        assert_eq!(gen_jan.len(), 1);
        assert_eq!(
            gen_jan[0].posting_date,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
        assert_eq!(gen_jan[0].docstatus, 1);
        assert_eq!(
            engine.records()[0].next_schedule_date,
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()
        );
        assert_eq!(engine.records()[0].status, AutoRepeatStatus::Active);

        // 2. Process as of April 15 -> Generates Feb 1 and Mar 1 invoices, then completes!
        let gen_rem = engine.process_due_records(NaiveDate::from_ymd_opt(2026, 4, 15).unwrap());
        assert_eq!(gen_rem.len(), 2);
        assert_eq!(
            gen_rem[0].posting_date,
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()
        );
        assert_eq!(
            gen_rem[1].posting_date,
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
        );
        assert_eq!(engine.records()[0].status, AutoRepeatStatus::Completed);
        assert_eq!(engine.generated().len(), 3);
    }
}
