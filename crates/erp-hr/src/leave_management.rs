//! Leave Management, Accrual & Holiday Engine (`erp-hr::leave_management`).
//!
//! Implements Phase 6 Leave Allocation and Balances:
//! - Leave Types: Casual, Sick, Annual, Unpaid LWP
//! - Automated monthly accrual based on employment duration
//! - Holiday calendar exclusions
//! - Balance deduction upon leave application approval.

use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::attendance::HrError;

/// Leave classification type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaveType {
    Casual,
    Sick,
    Annual,
    LeaveWithoutPay,
}

/// Employee Leave Allocation Record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaveAllocation {
    pub employee: CompactString,
    pub leave_type: LeaveType,
    pub allocated_days: Decimal,
    pub accrued_days: Decimal,
    pub used_days: Decimal,
}

impl LeaveAllocation {
    #[must_use]
    pub fn available_balance(&self) -> Decimal {
        (self.allocated_days + self.accrued_days - self.used_days).max(Decimal::ZERO)
    }

    /// Deducts approved leave days, verifying balance sufficiency unless LWP.
    pub fn deduct_leave(&mut self, requested_days: Decimal) -> Result<(), HrError> {
        if self.leave_type != LeaveType::LeaveWithoutPay && requested_days > self.available_balance() {
            return Err(HrError::InsufficientLeaveBalance {
                leave_type: format!("{:?}", self.leave_type),
                requested: requested_days,
                available: self.available_balance(),
            });
        }
        self.used_days += requested_days;
        Ok(())
    }

    /// Applies monthly accrual.
    pub fn apply_accrual(&mut self, monthly_rate: Decimal) {
        self.accrued_days += monthly_rate;
    }
}

/// Holiday List Calendar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HolidayList {
    pub name: CompactString,
    pub holidays: Vec<NaiveDate>,
}

impl HolidayList {
    /// Counts working days between two dates, excluding weekends and registered holidays.
    pub fn count_working_days(&self, from_date: NaiveDate, to_date: NaiveDate) -> Decimal {
        let mut count = 0;
        let mut curr = from_date;
        while curr <= to_date {
            let weekday = curr.format("%u").to_string().parse::<u32>().unwrap_or(1);
            let is_weekend = weekday == 6 || weekday == 7; // Saturday/Sunday
            let is_holiday = self.holidays.contains(&curr);

            if !is_weekend && !is_holiday {
                count += 1;
            }
            curr = curr.succ_opt().unwrap_or(curr);
        }
        Decimal::from(count)
    }
}
