use chrono::{NaiveDate, NaiveTime};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Overtime rate multiplier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OvertimeMultiplier {
    /// Standard working day overtime (1.5x base hourly rate).
    Standard1Point5,
    /// Weekend or holiday overtime (2.0x base hourly rate).
    HolidayDouble2Point0,
    /// Custom multiplier.
    Custom(Decimal),
}

impl OvertimeMultiplier {
    /// Multiplier value as Decimal.
    pub fn factor(&self) -> Decimal {
        match self {
            Self::Standard1Point5 => dec!(1.5),
            Self::HolidayDouble2Point0 => dec!(2.0),
            Self::Custom(d) => *d,
        }
    }
}

/// Overtime Slip record generated from biometric punch reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OvertimeSlip {
    /// Slip document ID.
    pub slip_id: String,
    /// Employee identifier.
    pub employee_id: String,
    /// Attendance date.
    pub attendance_date: NaiveDate,
    /// Shift scheduled end time.
    pub shift_end_time: NaiveTime,
    /// Actual biometric out punch.
    pub actual_out_time: NaiveTime,
    /// Overtime duration in hours.
    pub overtime_hours: Decimal,
    /// Base hourly wage rate.
    pub base_hourly_rate: Decimal,
    /// Applied multiplier.
    pub multiplier: OvertimeMultiplier,
    /// Calculated total overtime pay amount: `overtime_hours * base_hourly_rate * multiplier`.
    pub overtime_amount: Decimal,
}

/// Overtime calculation and slip generator.
pub struct OvertimeEngine;

impl OvertimeEngine {
    /// Evaluates actual checkout vs shift end time and generates an overtime slip if threshold exceeded.
    pub fn generate_overtime_slip(
        slip_id: &str,
        employee_id: &str,
        attendance_date: NaiveDate,
        shift_end_time: NaiveTime,
        actual_out_time: NaiveTime,
        base_hourly_rate: Decimal,
        multiplier: OvertimeMultiplier,
        min_overtime_mins_threshold: i64,
    ) -> Option<OvertimeSlip> {
        if actual_out_time <= shift_end_time {
            return None;
        }

        let diff_mins = (actual_out_time - shift_end_time).num_minutes();
        if diff_mins < min_overtime_mins_threshold {
            return None;
        }

        let hours = Decimal::from(diff_mins) / dec!(60.0);
        let amount = hours * base_hourly_rate * multiplier.factor();

        Some(OvertimeSlip {
            slip_id: slip_id.to_string(),
            employee_id: employee_id.to_string(),
            attendance_date,
            shift_end_time,
            actual_out_time,
            overtime_hours: hours,
            base_hourly_rate,
            multiplier,
            overtime_amount: amount,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overtime_slip_calculation() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();
        let shift_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        let actual_out = NaiveTime::from_hms_opt(19, 0, 0).unwrap(); // 2 hours (120 mins) OT

        let slip = OvertimeEngine::generate_overtime_slip(
            "OT-001",
            "EMP-001",
            date,
            shift_end,
            actual_out,
            dec!(20.0), // $20/hr
            OvertimeMultiplier::Standard1Point5, // 1.5x -> $30/hr
            30, // threshold 30 mins
        )
        .expect("Overtime slip generation expected");

        assert_eq!(slip.overtime_hours, dec!(2.0));
        assert_eq!(slip.overtime_amount, dec!(60.0)); // 2 hrs * $20 * 1.5 = $60
    }
}
