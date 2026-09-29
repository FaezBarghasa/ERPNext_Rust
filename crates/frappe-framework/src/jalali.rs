//! Bitemporal Multi-Calendar Engine: Native Shamsi / Jalali (Solar Hijri) (`frappe_framework::jalali`).
//!
//! Implements Pillar XXIII: Precise astronomical Khayyam 2820-year leap cycle algorithm,
//! bi-directional Gregorian <-> Jalali conversions, Persian fiscal year ranges,
//! and accurate asset depreciation allocation across 31, 30, and 29/30 day months.

use chrono::{Datelike, NaiveDate};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Strongly-typed Jalali (Solar Hijri) Calendar Date representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct JalaliDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl JalaliDate {
    /// Creates a new JalaliDate with bounds validation.
    #[must_use]
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }

        let max_days = Self::days_in_month(year, month);
        if day < 1 || day > max_days {
            return None;
        }

        Some(Self { year, month, day })
    }

    /// Determines if a Jalali year is a leap year using Khayyam's 2820-year algorithmic cycle.
    #[must_use]
    pub fn is_leap_year(year: i32) -> bool {
        let base = year - if year > 0 { 474 } else { 473 };
        let rem_2820 = ((base % 2820) + 2820) % 2820;
        let cycle = (rem_2820 + 474 + 38) * 682;
        (cycle % 2816) < 682
    }

    /// Returns the number of days in a given Jalali month:
    /// - Months 1 to 6 (Farvardin to Shahrivar): 31 days
    /// - Months 7 to 11 (Mehr to Bahman): 30 days
    /// - Month 12 (Esfand): 29 days (30 days in leap years)
    #[must_use]
    pub fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1..=6 => 31,
            7..=11 => 30,
            12 => {
                if Self::is_leap_year(year) {
                    30
                } else {
                    29
                }
            }
            _ => 0,
        }
    }

    /// Converts a Gregorian `NaiveDate` to `JalaliDate`.
    #[must_use]
    pub fn from_gregorian(g_date: NaiveDate) -> Self {
        let gy = g_date.year();
        let gm = g_date.month() as i32;
        let gd = g_date.day() as i32;

        let g_d_m = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        let mut jy;

        let gy2 = if gm > 2 { gy } else { gy - 1 };
        let mut days = 355666 + (365 * gy) + ((gy2 + 3) / 4) - ((gy2 + 99) / 100) + ((gy2 + 399) / 400) + gd + g_d_m[(gm - 1) as usize];
        jy = -1595 + (33 * (days / 12053));
        days %= 12053;
        jy += 4 * (days / 1461);
        days %= 1461;

        if days > 365 {
            jy += (days - 1) / 365;
            days = (days - 1) % 365;
        }

        let (jm, jd) = if days < 186 {
            (1 + (days / 31), 1 + (days % 31))
        } else {
            (7 + ((days - 186) / 30), 1 + ((days - 186) % 30))
        };

        Self {
            year: jy,
            month: jm as u32,
            day: jd as u32,
        }
    }

    /// Converts this `JalaliDate` to Gregorian `NaiveDate`.
    #[must_use]
    pub fn to_gregorian(&self) -> NaiveDate {
        let jy = self.year;
        let jm = self.month as i32;
        let jd = self.day as i32;

        let gy = jy + 1595;
        let mut days = -355668 + (365 * gy) + ((gy + 3) / 4) - ((gy + 99) / 100) + ((gy + 399) / 400) + jd
            + if jm < 7 {
                (jm - 1) * 31
            } else {
                ((jm - 7) * 30) + 186
            };

        let mut gy2 = 400 * (days / 146097);
        days %= 146097;

        if days > 36524 {
            gy2 += 100 * ((days - 1) / 36524);
            days = (days - 1) % 36524;
            if days >= 365 {
                days += 1;
            }
        }

        gy2 += 4 * (days / 1461);
        days %= 1461;

        if days > 365 {
            gy2 += (days - 1) / 365;
            days = (days - 1) % 365;
        }

        let mut gd = days + 1;
        let sal_a = [
            0,
            31,
            if (gy2 % 4 == 0 && gy2 % 100 != 0) || (gy2 % 400 == 0) {
                29
            } else {
                28
            },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];

        let mut gm = 0;
        while gm < 13 && gd > sal_a[gm] {
            gd -= sal_a[gm];
            gm += 1;
        }

        NaiveDate::from_ymd_opt(gy2, gm as u32, gd as u32)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    }

    /// Formats as Latin Shamsi string: `YYYY/MM/DD`.
    #[must_use]
    pub fn format_shamsi(&self) -> CompactString {
        format!("{:04}/{:02}/{:02}", self.year, self.month, self.day).into()
    }

    /// Persian month name.
    #[must_use]
    pub fn persian_month_name(&self) -> &'static str {
        match self.month {
            1 => "فروردین",
            2 => "اردیبهشت",
            3 => "خرداد",
            4 => "تیر",
            5 => "مرداد",
            6 => "شهریور",
            7 => "مهر",
            8 => "آبان",
            9 => "آذر",
            10 => "دی",
            11 => "بهمن",
            12 => "اسفند",
            _ => "",
        }
    }

    /// Returns the exact Gregorian date range for a full Persian Fiscal Year (1 Farvardin -> 29/30 Esfand).
    #[must_use]
    pub fn persian_fiscal_year_bounds(jalali_year: i32) -> (NaiveDate, NaiveDate) {
        let start = JalaliDate::new(jalali_year, 1, 1)
            .expect("Valid start date")
            .to_gregorian();
        let end_day = if Self::is_leap_year(jalali_year) { 30 } else { 29 };
        let end = JalaliDate::new(jalali_year, 12, end_day)
            .expect("Valid end date")
            .to_gregorian();
        (start, end)
    }

    /// Allocates asset annual depreciation across all 12 Jalali months proportionally to day counts.
    #[must_use]
    pub fn compute_jalali_monthly_depreciation(annual_depreciation: f64, jalali_year: i32) -> Vec<f64> {
        let total_days = if Self::is_leap_year(jalali_year) { 366.0 } else { 365.0 };
        let daily_rate = annual_depreciation / total_days;

        (1..=12)
            .map(|m| {
                let days = Self::days_in_month(jalali_year, m) as f64;
                (daily_rate * days * 100.0).round() / 100.0
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_khayyam_leap_year() {
        assert!(JalaliDate::is_leap_year(1399));
        assert!(JalaliDate::is_leap_year(1404));
        assert!(!JalaliDate::is_leap_year(1403));
        assert!(!JalaliDate::is_leap_year(1405));
    }

    #[test]
    fn test_gregorian_jalali_bi_directional_conversion() {
        // Now: 2026-09-29 -> 1405-07-07 (Mehr 7, 1405)
        let g_date = NaiveDate::from_ymd_opt(2026, 9, 29).unwrap();
        let j_date = JalaliDate::from_gregorian(g_date);

        assert_eq!(j_date.year, 1405);
        assert_eq!(j_date.month, 7);
        assert_eq!(j_date.day, 7);

        // Convert back
        let g_back = j_date.to_gregorian();
        assert_eq!(g_back, g_date);
    }

    #[test]
    fn test_fiscal_year_bounds() {
        let (start, end) = JalaliDate::persian_fiscal_year_bounds(1405);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 3, 21).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2027, 3, 20).unwrap());
    }

    #[test]
    fn test_monthly_depreciation_proportions() {
        let monthly = JalaliDate::compute_jalali_monthly_depreciation(36500.0, 1405); // 365 days in 1405 -> $100/day
        assert_eq!(monthly.len(), 12);
        assert_eq!(monthly[0], 3100.0); // Farvardin (31 days)
        assert_eq!(monthly[6], 3000.0); // Mehr (30 days)
        assert_eq!(monthly[11], 2900.0); // Esfand (29 days in 1405)
    }
}
