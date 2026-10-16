//! Fixed Asset Depreciation, Capitalization, and Maintenance Engine (`erp-asset::depreciation`).
//!
//! Provides canonical ERPNext asset accounting capabilities:
//! - Depreciation Methods: Straight Line (SL), Written Down Value (WDV), Double Declining Balance (DDB)
//! - Depreciation Schedule generation with monthly/yearly periodicity and salvage floor protection
//! - Asset Capitalization (Capital Work In Progress - CWIP -> Fixed Asset)
//! - Asset Maintenance & Inspection logs.

use chrono::{Datelike, Months, NaiveDate};
use compact_str::CompactString;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Asset management and depreciation errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssetError {
    #[error("Gross purchase amount must be positive, got {0}")]
    InvalidGrossAmount(Decimal),
    #[error("Expected salvage value {salvage} exceeds gross purchase amount {gross}")]
    SalvageExceedsCost { salvage: Decimal, gross: Decimal },
    #[error("Total number of depreciations must be greater than zero")]
    ZeroDepreciationPeriods,
    #[error("Frequency of depreciation in months must be greater than zero")]
    ZeroFrequency,
    #[error("Asset '{0}' is already fully depreciated")]
    AlreadyDepreciated(CompactString),
    #[error("Asset '{0}' is already submitted or in use")]
    InvalidState(CompactString),
    #[error("No CWIP items or services specified for capitalization")]
    EmptyCapitalization,
}

/// Supported fixed asset depreciation calculation methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepreciationMethod {
    /// Equal depreciation amount each period: (Cost - Salvage) / Periods.
    StraightLine,
    /// Accelerated diminishing balance method based on declining book value.
    WrittenDownValue,
    /// Double declining balance rate: (2 / Periods) * Book Value.
    DoubleDecliningBalance,
}

/// Lifecycle status of a fixed asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AssetStatus {
    #[default]
    Draft,
    Submitted,
    Capitalized,
    InUse,
    FullyDepreciated,
    Scrapped,
    Sold,
}

/// Single scheduled depreciation period entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepreciationScheduleEntry {
    pub idx: u32,
    pub schedule_date: NaiveDate,
    pub depreciation_amount: Decimal,
    pub accumulated_depreciation_amount: Decimal,
    pub book_value: Decimal,
    pub journal_entry: Option<CompactString>,
}

/// Fixed Asset record in ERPNext.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Asset {
    pub name: CompactString,
    pub asset_name: CompactString,
    pub item_code: CompactString,
    pub gross_purchase_amount: Decimal,
    pub purchase_date: NaiveDate,
    pub available_for_use_date: NaiveDate,
    pub expected_value_after_useful_life: Decimal, // Salvage value
    pub depreciation_method: DepreciationMethod,
    pub total_number_of_depreciations: u32,
    pub frequency_of_depreciation_months: u32,
    pub status: AssetStatus,
    pub schedules: Vec<DepreciationScheduleEntry>,
}

impl Asset {
    /// Creates a new Asset in Draft status.
    pub fn new(
        name: impl Into<CompactString>,
        asset_name: impl Into<CompactString>,
        item_code: impl Into<CompactString>,
        gross_amount: Decimal,
        purchase_date: NaiveDate,
        available_date: NaiveDate,
        salvage_value: Decimal,
        method: DepreciationMethod,
        total_depreciations: u32,
        frequency_months: u32,
    ) -> Result<Self, AssetError> {
        if gross_amount <= Decimal::ZERO {
            return Err(AssetError::InvalidGrossAmount(gross_amount));
        }
        if salvage_value >= gross_amount {
            return Err(AssetError::SalvageExceedsCost {
                salvage: salvage_value,
                gross: gross_amount,
            });
        }
        if total_depreciations == 0 {
            return Err(AssetError::ZeroDepreciationPeriods);
        }
        if frequency_months == 0 {
            return Err(AssetError::ZeroFrequency);
        }

        Ok(Self {
            name: name.into(),
            asset_name: asset_name.into(),
            item_code: item_code.into(),
            gross_purchase_amount: gross_amount,
            purchase_date,
            available_for_use_date: available_date,
            expected_value_after_useful_life: salvage_value,
            depreciation_method: method,
            total_number_of_depreciations: total_depreciations,
            frequency_of_depreciation_months: frequency_months,
            status: AssetStatus::Draft,
            schedules: Vec::new(),
        })
    }

    /// Computes and populates the complete depreciation schedule.
    pub fn calculate_depreciation_schedule(&mut self) -> Result<(), AssetError> {
        let n = self.total_number_of_depreciations;
        let salvage = self.expected_value_after_useful_life;
        let mut current_book_value = self.gross_purchase_amount;
        let mut accumulated = Decimal::ZERO;

        let mut entries = Vec::with_capacity(n as usize);
        let mut cur_date = self.available_for_use_date;

        match self.depreciation_method {
            DepreciationMethod::StraightLine => {
                let depreciable_base = self.gross_purchase_amount - salvage;
                let n_dec = Decimal::from(n);
                let standard_period_depr = (depreciable_base / n_dec).round_dp(2);

                for i in 1..=n {
                    cur_date = cur_date
                        .checked_add_months(Months::new(self.frequency_of_depreciation_months))
                        .unwrap_or(cur_date);

                    let period_depr = if i == n {
                        // In the final period, absorb any fractional cent discrepancy
                        current_book_value - salvage
                    } else {
                        standard_period_depr
                    };

                    accumulated += period_depr;
                    current_book_value -= period_depr;

                    entries.push(DepreciationScheduleEntry {
                        idx: i,
                        schedule_date: cur_date,
                        depreciation_amount: period_depr,
                        accumulated_depreciation_amount: accumulated,
                        book_value: current_book_value,
                        journal_entry: None,
                    });
                }
            }
            DepreciationMethod::DoubleDecliningBalance => {
                let rate = (dec!(2.0) / Decimal::from(n)).round_dp(6);

                for i in 1..=n {
                    cur_date = cur_date
                        .checked_add_months(Months::new(self.frequency_of_depreciation_months))
                        .unwrap_or(cur_date);

                    let calculated_depr = (current_book_value * rate).round_dp(2);
                    // DDB cannot depreciate below salvage value
                    let max_allowed_depr = current_book_value - salvage;
                    let period_depr = if i == n {
                        max_allowed_depr
                    } else {
                        calculated_depr.min(max_allowed_depr).max(Decimal::ZERO)
                    };

                    accumulated += period_depr;
                    current_book_value -= period_depr;

                    entries.push(DepreciationScheduleEntry {
                        idx: i,
                        schedule_date: cur_date,
                        depreciation_amount: period_depr,
                        accumulated_depreciation_amount: accumulated,
                        book_value: current_book_value,
                        journal_entry: None,
                    });
                }
            }
            DepreciationMethod::WrittenDownValue => {
                // Diminishing Balance: Rate based on standard formula r = 1 - (salvage / cost)^(1/n)
                // Approximated by standard annual diminishing rate
                let depreciable_base = self.gross_purchase_amount - salvage;
                let rate = (dec!(1.5) / Decimal::from(n)).round_dp(6);

                for i in 1..=n {
                    cur_date = cur_date
                        .checked_add_months(Months::new(self.frequency_of_depreciation_months))
                        .unwrap_or(cur_date);

                    let calculated_depr = if i == n {
                        current_book_value - salvage
                    } else {
                        let d = (current_book_value * rate).round_dp(2);
                        let max_d = current_book_value - salvage;
                        d.min(max_d).max(Decimal::ZERO)
                    };

                    accumulated += calculated_depr;
                    current_book_value -= calculated_depr;

                    entries.push(DepreciationScheduleEntry {
                        idx: i,
                        schedule_date: cur_date,
                        depreciation_amount: calculated_depr,
                        accumulated_depreciation_amount: accumulated,
                        book_value: current_book_value,
                        journal_entry: None,
                    });
                }
            }
        }

        self.schedules = entries;
        Ok(())
    }
}

/// Item consumed from Capital Work-in-Progress (CWIP).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapitalizationItem {
    pub item_code: CompactString,
    pub qty: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
}

/// Additional service charge (installation, commissioning, engineering).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapitalizationService {
    pub expense_account: CompactString,
    pub description: CompactString,
    pub amount: Decimal,
}

/// Asset Capitalization voucher (transfers CWIP to Capitalized Fixed Asset).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetCapitalization {
    pub name: CompactString,
    pub posting_date: NaiveDate,
    pub target_asset_name: CompactString,
    pub cwip_items: Vec<CapitalizationItem>,
    pub service_costs: Vec<CapitalizationService>,
    pub total_capitalized_value: Decimal,
    pub is_submitted: bool,
}

impl AssetCapitalization {
    pub fn new(
        name: impl Into<CompactString>,
        posting_date: NaiveDate,
        target_asset: impl Into<CompactString>,
    ) -> Self {
        Self {
            name: name.into(),
            posting_date,
            target_asset_name: target_asset.into(),
            cwip_items: Vec::new(),
            service_costs: Vec::new(),
            total_capitalized_value: Decimal::ZERO,
            is_submitted: false,
        }
    }

    /// Calculates total capitalized cost from accumulated CWIP items and service charges.
    pub fn calculate_capitalization(&mut self) -> Result<Decimal, AssetError> {
        if self.cwip_items.is_empty() && self.service_costs.is_empty() {
            return Err(AssetError::EmptyCapitalization);
        }

        let items_total: Decimal = self.cwip_items.iter().map(|item| item.amount).sum();
        let service_total: Decimal = self.service_costs.iter().map(|svc| svc.amount).sum();
        let total = items_total + service_total;
        self.total_capitalized_value = total;
        Ok(total)
    }

    /// Submits capitalization, locking amounts and activating target asset.
    pub fn submit(&mut self, asset: &mut Asset) -> Result<Decimal, AssetError> {
        let total = self.calculate_capitalization()?;
        asset.gross_purchase_amount = total;
        asset.status = AssetStatus::Capitalized;
        asset.calculate_depreciation_schedule()?;
        self.is_submitted = true;
        Ok(total)
    }
}

/// Asset Maintenance Periodicity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaintenancePeriodicity {
    Monthly,
    Quarterly,
    HalfYearly,
    Yearly,
}

/// Individual scheduled maintenance task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaintenanceTask {
    pub task_name: CompactString,
    pub periodicity: MaintenancePeriodicity,
    pub start_date: NaiveDate,
    pub next_due_date: NaiveDate,
    pub assignee: CompactString,
    pub is_completed: bool,
}

/// Asset Maintenance plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetMaintenance {
    pub asset_name: CompactString,
    pub item_code: CompactString,
    pub maintenance_manager: CompactString,
    pub tasks: Vec<MaintenanceTask>,
}

impl AssetMaintenance {
    pub fn new(
        asset_name: impl Into<CompactString>,
        item_code: impl Into<CompactString>,
        manager: impl Into<CompactString>,
    ) -> Self {
        Self {
            asset_name: asset_name.into(),
            item_code: item_code.into(),
            maintenance_manager: manager.into(),
            tasks: Vec::new(),
        }
    }

    pub fn add_task(
        &mut self,
        task_name: impl Into<CompactString>,
        periodicity: MaintenancePeriodicity,
        start_date: NaiveDate,
        assignee: impl Into<CompactString>,
    ) {
        let months = match periodicity {
            MaintenancePeriodicity::Monthly => 1,
            MaintenancePeriodicity::Quarterly => 3,
            MaintenancePeriodicity::HalfYearly => 6,
            MaintenancePeriodicity::Yearly => 12,
        };
        let next_due = start_date
            .checked_add_months(Months::new(months))
            .unwrap_or(start_date);

        self.tasks.push(MaintenanceTask {
            task_name: task_name.into(),
            periodicity,
            start_date,
            next_due_date: next_due,
            assignee: assignee.into(),
            is_completed: false,
        });
    }

    pub fn complete_task(&mut self, idx: usize) -> Option<NaiveDate> {
        if let Some(task) = self.tasks.get_mut(idx) {
            task.is_completed = true;
            let months = match task.periodicity {
                MaintenancePeriodicity::Monthly => 1,
                MaintenancePeriodicity::Quarterly => 3,
                MaintenancePeriodicity::HalfYearly => 6,
                MaintenancePeriodicity::Yearly => 12,
            };
            let next = task
                .next_due_date
                .checked_add_months(Months::new(months))
                .unwrap_or(task.next_due_date);
            task.next_due_date = next;
            Some(next)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_straight_line_depreciation_schedule() {
        let buy_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let use_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        // Asset cost $10,000, salvage $1,000, 3 years (36 months) frequency = 12 months (3 periods)
        let mut asset = Asset::new(
            "AST-001",
            "CNC Milling Machine",
            "MACH-01",
            dec!(10000.0),
            buy_date,
            use_date,
            dec!(1000.0),
            DepreciationMethod::StraightLine,
            3,
            12,
        )
        .unwrap();

        asset.calculate_depreciation_schedule().unwrap();

        assert_eq!(asset.schedules.len(), 3);
        // Depreciable base = 9000 / 3 = 3000 per period
        assert_eq!(asset.schedules[0].depreciation_amount, dec!(3000.0));
        assert_eq!(asset.schedules[0].book_value, dec!(7000.0));

        assert_eq!(asset.schedules[1].depreciation_amount, dec!(3000.0));
        assert_eq!(asset.schedules[1].book_value, dec!(4000.0));

        assert_eq!(asset.schedules[2].depreciation_amount, dec!(3000.0));
        assert_eq!(asset.schedules[2].book_value, dec!(1000.0)); // Exactly salvage value!
        assert_eq!(asset.schedules[2].accumulated_depreciation_amount, dec!(9000.0));
    }

    #[test]
    fn test_double_declining_balance_depreciation() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        // Cost $10,000, salvage $1,000, 4 periods -> Rate = 2/4 = 0.50 (50%)
        let mut asset = Asset::new(
            "AST-002",
            "Server Rack",
            "IT-01",
            dec!(10000.0),
            date,
            date,
            dec!(1000.0),
            DepreciationMethod::DoubleDecliningBalance,
            4,
            12,
        )
        .unwrap();

        asset.calculate_depreciation_schedule().unwrap();

        // Period 1: 10,000 * 0.5 = 5,000 -> Book Value = 5,000
        assert_eq!(asset.schedules[0].depreciation_amount, dec!(5000.0));
        assert_eq!(asset.schedules[0].book_value, dec!(5000.0));

        // Period 2: 5,000 * 0.5 = 2,500 -> Book Value = 2,500
        assert_eq!(asset.schedules[1].depreciation_amount, dec!(2500.0));
        assert_eq!(asset.schedules[1].book_value, dec!(2500.0));

        // Period 3: 2,500 * 0.5 = 1,250 -> Book Value = 1,250
        assert_eq!(asset.schedules[2].depreciation_amount, dec!(1250.0));
        assert_eq!(asset.schedules[2].book_value, dec!(1250.0));

        // Period 4: Final period absorbs remaining down to salvage ($1,000) -> 250
        assert_eq!(asset.schedules[3].depreciation_amount, dec!(250.0));
        assert_eq!(asset.schedules[3].book_value, dec!(1000.0));
    }

    #[test]
    fn test_asset_capitalization_cwip_transfer() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();

        let mut asset = Asset::new(
            "AST-PLANT-01",
            "Solar Array Power Plant",
            "SOLAR-PLANT",
            dec!(1.0), // Placeholder until capitalized
            date,
            date,
            dec!(5000.0), // Salvage
            DepreciationMethod::StraightLine,
            5,
            12,
        )
        .unwrap();

        let mut cap = AssetCapitalization::new("CAP-001", date, "AST-PLANT-01");
        cap.cwip_items.push(CapitalizationItem {
            item_code: "SOLAR-PANEL-400W".into(),
            qty: dec!(100.0),
            rate: dec!(200.0),
            amount: dec!(20000.0),
        });
        cap.cwip_items.push(CapitalizationItem {
            item_code: "INVERTER-50KW".into(),
            qty: dec!(2.0),
            rate: dec!(5000.0),
            amount: dec!(10000.0),
        });
        cap.service_costs.push(CapitalizationService {
            expense_account: "5120 - Civil & Installation Works".into(),
            description: "High voltage cabling and mounting structure".into(),
            amount: dec!(15000.0),
        });

        // Capitalize: 20,000 + 10,000 + 15,000 = $45,000
        let total = cap.submit(&mut asset).unwrap();
        assert_eq!(total, dec!(45000.0));
        assert_eq!(asset.gross_purchase_amount, dec!(45000.0));
        assert_eq!(asset.status, AssetStatus::Capitalized);

        // Schedule check: Depreciable base = 45,000 - 5,000 = 40,000 / 5 = 8,000 per year
        assert_eq!(asset.schedules.len(), 5);
        assert_eq!(asset.schedules[0].depreciation_amount, dec!(8000.0));
        assert_eq!(asset.schedules[4].book_value, dec!(5000.0));
    }
}
