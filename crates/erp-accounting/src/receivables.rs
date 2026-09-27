use crate::ledger::AccountingError;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Unsettled customer sales invoice for payment allocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenInvoice {
    /// Invoice identifier (e.g. "ACC-SINV-2026-00001").
    pub voucher_no: String,
    /// Invoice posting date.
    pub posting_date: NaiveDate,
    /// Total invoice amount.
    pub grand_total: Decimal,
    /// Remaining unpaid balance.
    pub outstanding_amount: Decimal,
}

/// Result of an individual invoice payment allocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentAllocationResult {
    /// Allocated invoice identifier.
    pub voucher_no: String,
    /// Amount allocated from incoming payment.
    pub allocated_amount: Decimal,
    /// Remaining outstanding balance on the invoice after allocation.
    pub remaining_outstanding: Decimal,
}

/// Accounts Receivable / Payable (AR/AP) Engine (Milestone 2.5).
pub struct ArApEngine;

impl ArApEngine {
    /// FIFO Payment Allocation: matches incoming payment against oldest open invoices sequentially.
    /// Returns allocation details and unallocated remaining payment amount.
    pub fn allocate_fifo(
        invoices: &mut [OpenInvoice],
        mut payment_amount: Decimal,
    ) -> (Vec<PaymentAllocationResult>, Decimal) {
        // Sort oldest first
        invoices.sort_by_key(|inv| inv.posting_date);

        let mut allocations = Vec::new();

        for inv in invoices.iter_mut() {
            if payment_amount <= Decimal::ZERO {
                break;
            }

            if inv.outstanding_amount > Decimal::ZERO {
                if inv.outstanding_amount <= payment_amount {
                    let allocated = inv.outstanding_amount;
                    payment_amount -= allocated;
                    inv.outstanding_amount = Decimal::ZERO;

                    allocations.push(PaymentAllocationResult {
                        voucher_no: inv.voucher_no.clone(),
                        allocated_amount: allocated,
                        remaining_outstanding: Decimal::ZERO,
                    });
                } else {
                    inv.outstanding_amount -= payment_amount;
                    allocations.push(PaymentAllocationResult {
                        voucher_no: inv.voucher_no.clone(),
                        allocated_amount: payment_amount,
                        remaining_outstanding: inv.outstanding_amount,
                    });
                    payment_amount = Decimal::ZERO;
                }
            }
        }

        (allocations, payment_amount)
    }

    /// Validates customer credit limit constraint:
    /// $\text{Current Outstanding} + \text{Unbilled Orders} + \text{New Invoice Amount} \le \text{Credit Limit}$
    pub fn validate_credit_limit(
        current_outstanding: Decimal,
        unbilled_orders: Decimal,
        new_invoice_amount: Decimal,
        credit_limit: Decimal,
    ) -> Result<(), AccountingError> {
        let total_exposure = current_outstanding + unbilled_orders + new_invoice_amount;
        if total_exposure > credit_limit {
            return Err(AccountingError::CreditLimitExceeded {
                current: current_outstanding + unbilled_orders,
                new_amount: new_invoice_amount,
                limit: credit_limit,
            });
        }
        Ok(())
    }
}
