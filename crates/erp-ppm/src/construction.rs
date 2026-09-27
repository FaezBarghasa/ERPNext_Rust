//! EPC Contract Administration, AIA G702/G703 Billing, Retainage & Change Orders.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleOfValuesItem {
    pub item_no: String,
    pub description: String,
    pub scheduled_value: Decimal,
    pub work_completed_previous: Decimal,
    pub work_completed_this_period: Decimal,
    pub materials_presently_stored: Decimal,
    pub retainage_rate: Decimal, // e.g. 0.10 for 10%
}

impl ScheduleOfValuesItem {
    #[must_use]
    pub fn total_completed_and_stored(&self) -> Decimal {
        self.work_completed_previous + self.work_completed_this_period + self.materials_presently_stored
    }

    #[must_use]
    pub fn percent_complete(&self) -> Decimal {
        if self.scheduled_value.is_zero() {
            Decimal::ZERO
        } else {
            ((self.total_completed_and_stored() / self.scheduled_value) * Decimal::from(100)).round_dp(2)
        }
    }

    #[must_use]
    pub fn retainage_amount(&self) -> Decimal {
        (self.total_completed_and_stored() * self.retainage_rate).round_dp(2)
    }

    #[must_use]
    pub fn balance_to_finish(&self) -> Decimal {
        self.scheduled_value - self.total_completed_and_stored() + self.retainage_amount()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChangeOrderType {
    PotentialChangeOrder, // PCO
    ChangeOrderRequest,   // COR
    OwnerChangeOrder,     // OCO
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeOrder {
    pub id: String,
    pub co_type: ChangeOrderType,
    pub description: String,
    pub cost_impact: Decimal,
    pub schedule_impact_days: i64,
    pub is_approved: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LienWaiverStatus {
    ConditionalProgress,
    UnconditionalProgress,
    ConditionalFinal,
    UnconditionalFinal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LienWaiver {
    pub id: String,
    pub subcontractor_id: String,
    pub status: LienWaiverStatus,
    pub amount_released: Decimal,
    pub through_date: chrono::NaiveDate,
    pub signed: bool,
}

pub struct AiaG702Certificate {
    pub original_contract_sum: Decimal,
    pub net_change_orders: Decimal,
    pub contract_sum_to_date: Decimal,
    pub total_completed_and_stored: Decimal,
    pub total_retainage: Decimal,
    pub total_earned_less_retainage: Decimal,
    pub less_previous_certificates: Decimal,
    pub current_payment_due: Decimal,
    pub balance_to_finish_plus_retainage: Decimal,
}

impl AiaG702Certificate {
    pub fn calculate(
        original_contract: Decimal,
        change_orders: &[ChangeOrder],
        sov_items: &[ScheduleOfValuesItem],
        previous_certificates_total: Decimal,
    ) -> Self {
        let net_co: Decimal = change_orders
            .iter()
            .filter(|co| co.is_approved)
            .map(|co| co.cost_impact)
            .sum();

        let contract_sum = original_contract + net_co;
        let total_completed: Decimal = sov_items.iter().map(ScheduleOfValuesItem::total_completed_and_stored).sum();
        let total_ret: Decimal = sov_items.iter().map(ScheduleOfValuesItem::retainage_amount).sum();
        let earned_less_ret = total_completed - total_ret;
        let payment_due = earned_less_ret - previous_certificates_total;
        let balance_to_finish = contract_sum - earned_less_ret;

        Self {
            original_contract_sum: original_contract,
            net_change_orders: net_co,
            contract_sum_to_date: contract_sum,
            total_completed_and_stored: total_completed,
            total_retainage: total_ret,
            total_earned_less_retainage: earned_less_ret,
            less_previous_certificates: previous_certificates_total,
            current_payment_due: payment_due.max(Decimal::ZERO),
            balance_to_finish_plus_retainage: balance_to_finish,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_aia_g702_g703_generation() {
        let sov = vec![
            ScheduleOfValuesItem {
                item_no: "01-001".into(),
                description: "Structural Steel Frame".into(),
                scheduled_value: dec!(500000),
                work_completed_previous: dec!(100000),
                work_completed_this_period: dec!(150000),
                materials_presently_stored: dec!(50000),
                retainage_rate: dec!(0.10), // 10%
            },
        ];

        let cos = vec![ChangeOrder {
            id: "OCO-01".into(),
            co_type: ChangeOrderType::OwnerChangeOrder,
            description: "High Grade Coating".into(),
            cost_impact: dec!(25000),
            schedule_impact_days: 5,
            is_approved: true,
        }];

        let cert = AiaG702Certificate::calculate(dec!(500000), &cos, &sov, dec!(90000));
        assert_eq!(cert.contract_sum_to_date, dec!(525000));
        assert_eq!(cert.total_completed_and_stored, dec!(300000));
        assert_eq!(cert.total_retainage, dec!(30000)); // 10% of 300,000
        assert_eq!(cert.total_earned_less_retainage, dec!(270000));
        assert_eq!(cert.current_payment_due, dec!(180000)); // 270,000 - 90,000
    }
}
