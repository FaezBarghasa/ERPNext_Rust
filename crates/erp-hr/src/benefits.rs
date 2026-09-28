use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Flexible Employee Benefit component (e.g. Wellness, Childcare, Remote Work Stipend).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlexBenefitCategory {
    /// Category name.
    pub name: String,
    /// Annual maximum allowable limit.
    pub annual_max_limit: Decimal,
    /// Whether pro-rated monthly or lump-sum.
    pub is_pro_rated: bool,
}

/// Flexible Employee Benefit Ledger record for an employee.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlexBenefitLedger {
    /// Employee identifier.
    pub employee_id: String,
    /// Fiscal year (e.g. "2026-2027").
    pub fiscal_year: String,
    /// Category name.
    pub category: String,
    /// Annual allocated allowance limit.
    pub annual_allowance: Decimal,
    /// Cumulative claims reimbursed so far.
    pub total_claimed: Decimal,
}

impl FlexBenefitLedger {
    /// Remaining flex benefit balance.
    pub fn remaining_balance(&self) -> Decimal {
        (self.annual_allowance - self.total_claimed).max(Decimal::ZERO)
    }

    /// Submits a claim against the flexible benefit ledger.
    pub fn claim_benefit(&mut self, claim_amount: Decimal) -> Result<(), String> {
        if claim_amount > self.remaining_balance() {
            return Err(format!(
                "Claim {} exceeds remaining flex allowance {}",
                claim_amount,
                self.remaining_balance()
            ));
        }
        self.total_claimed += claim_amount;
        Ok(())
    }
}

/// Holiday List for a department, location, or shift.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HolidayList {
    /// Holiday list name (e.g. "HQ Holidays 2026", "Branch Tehran 2026").
    pub name: String,
    /// Year.
    pub from_date: NaiveDate,
    /// End date.
    pub to_date: NaiveDate,
    /// List of official holiday dates.
    pub holidays: HashSet<NaiveDate>,
    /// Weekly off days (e.g. 5 for Friday, 6 for Saturday, 7 for Sunday).
    pub weekly_off_iso_weekdays: HashSet<u32>,
}

impl HolidayList {
    /// Checks if a given date is a non-working day (either holiday or weekly off).
    pub fn is_non_working_day(&self, date: NaiveDate) -> bool {
        if self.holidays.contains(&date) {
            return true;
        }

        use chrono::Datelike;
        let weekday = date.weekday().number_from_monday();
        self.weekly_off_iso_weekdays.contains(&weekday)
    }

    /// Counts total working days within an interval.
    pub fn count_working_days(&self, start: NaiveDate, end: NaiveDate) -> u32 {
        let mut count = 0;
        let mut curr = start;
        while curr <= end {
            if !self.is_non_working_day(curr) {
                count += 1;
            }
            if let Some(next) = curr.succ_opt() {
                curr = next;
            } else {
                break;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_flex_benefit_ledger() {
        let mut ledger = FlexBenefitLedger {
            employee_id: "EMP-001".into(),
            fiscal_year: "2026".into(),
            category: "Wellness".into(),
            annual_allowance: dec!(1200.0),
            total_claimed: dec!(400.0),
        };

        assert_eq!(ledger.remaining_balance(), dec!(800.0));
        assert!(ledger.claim_benefit(dec!(500.0)).is_ok());
        assert_eq!(ledger.remaining_balance(), dec!(300.0));
        assert!(ledger.claim_benefit(dec!(400.0)).is_err()); // Exceeds remaining 300
    }

    #[test]
    fn test_holiday_list_working_days() {
        let start = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 10, 7).unwrap(); // 7 days

        let mut holidays = HashSet::new();
        holidays.insert(NaiveDate::from_ymd_opt(2026, 10, 2).unwrap()); // Friday holiday

        let mut weekly_offs = HashSet::new();
        weekly_offs.insert(7); // Sunday is weekly off

        let list = HolidayList {
            name: "Default 2026".into(),
            from_date: start,
            to_date: end,
            holidays,
            weekly_off_iso_weekdays: weekly_offs,
        };

        let working_days = list.count_working_days(start, end);
        assert_eq!(working_days, 5); // 7 - 1 holiday - 1 sunday = 5
    }
}
