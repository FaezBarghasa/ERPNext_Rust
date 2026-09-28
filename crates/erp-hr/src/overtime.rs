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

/// Parameters for generating an overtime slip from biometric punch logs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OvertimeCalculationRequest<'a> {
    pub slip_id: &'a str,
    pub employee_id: &'a str,
    pub attendance_date: NaiveDate,
    pub shift_end_time: NaiveTime,
    pub actual_out_time: NaiveTime,
    pub base_hourly_rate: Decimal,
    pub multiplier: OvertimeMultiplier,
    pub min_overtime_mins_threshold: i64,
}

/// Overtime calculation and slip generator.
pub struct OvertimeEngine;

impl OvertimeEngine {
    /// Evaluates actual checkout vs shift end time and generates an overtime slip if threshold exceeded.
    pub fn generate_overtime_slip(req: &OvertimeCalculationRequest) -> Option<OvertimeSlip> {
        if req.actual_out_time <= req.shift_end_time {
            return None;
        }

        let diff_mins = (req.actual_out_time - req.shift_end_time).num_minutes();
        if diff_mins < req.min_overtime_mins_threshold {
            return None;
        }

        let hours = Decimal::from(diff_mins) / dec!(60.0);
        let amount = hours * req.base_hourly_rate * req.multiplier.factor();

        Some(OvertimeSlip {
            slip_id: req.slip_id.to_string(),
            employee_id: req.employee_id.to_string(),
            attendance_date: req.attendance_date,
            shift_end_time: req.shift_end_time,
            actual_out_time: req.actual_out_time,
            overtime_hours: hours,
            base_hourly_rate: req.base_hourly_rate,
            multiplier: req.multiplier,
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

        let req = OvertimeCalculationRequest {
            slip_id: "OT-001",
            employee_id: "EMP-001",
            attendance_date: date,
            shift_end_time: shift_end,
            actual_out_time: actual_out,
            base_hourly_rate: dec!(20.0),                    // $20/hr
            multiplier: OvertimeMultiplier::Standard1Point5, // 1.5x -> $30/hr
            min_overtime_mins_threshold: 30,                 // threshold 30 mins
        };

        let slip = OvertimeEngine::generate_overtime_slip(&req)
            .expect("Overtime slip generation expected");

        assert_eq!(slip.overtime_hours, dec!(2.0));
        assert_eq!(slip.overtime_amount, dec!(60.0)); // 2 hrs * $20 * 1.5 = $60
    }
}
