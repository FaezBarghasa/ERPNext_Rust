//! Stock Valuation Engine (`erp-inventory::valuation`).
//!
//! Implements canonical upstream ERPNext stock valuation algorithms:
//! - FIFO (First In, First Out) layer consumption
//! - Moving Average continuous unit cost recalculation
//! - LIFO (Last In, First Out) layer consumption
//! - Real-time Bin updates (actual_qty, stock_value, valuation_rate)
//! - Stock Value Difference (SVD) calculation for General Ledger synchronization.

use chrono::NaiveDateTime;
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Stock valuation operational errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValuationError {
    #[error(
        "Negative stock error: Warehouse '{warehouse}' has insufficient quantity {available} for Item '{item_code}' (requested {requested})"
    )]
    InsufficientStock {
        item_code: CompactString,
        warehouse: CompactString,
        available: Decimal,
        requested: Decimal,
    },
    #[error("Invalid incoming rate: {0}")]
    InvalidRate(Decimal),
}

/// Supported stock valuation methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValuationMethod {
    FIFO,
    MovingAverage,
    LIFO,
}

/// An individual FIFO/LIFO receipt layer in inventory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockLayer {
    pub qty: Decimal,
    pub rate: Decimal,
}

/// Real-time warehouse inventory Bin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StockBin {
    pub item_code: CompactString,
    pub warehouse: CompactString,
    pub actual_qty: Decimal,
    pub valuation_rate: Decimal,
    pub stock_value: Decimal,
    pub layers: Vec<StockLayer>,
}

/// Stock Ledger Entry (SLE) tracking granular inventory movements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockLedgerEntry {
    pub name: CompactString,
    pub item_code: CompactString,
    pub warehouse: CompactString,
    pub posting_datetime: NaiveDateTime,
    pub voucher_type: CompactString,
    pub voucher_no: CompactString,
    pub actual_qty: Decimal, // Positive for inward/receipt, negative for outward/dispatch
    pub incoming_rate: Decimal,
    pub outgoing_rate: Decimal,
    pub stock_value_difference: Decimal,
    pub qty_after_transaction: Decimal,
    pub stock_value: Decimal,
    pub valuation_rate: Decimal,
}

/// Parameters for processing a Stock Ledger Entry against a Bin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SleParams<'a> {
    pub sle_name: &'a str,
    pub voucher_type: &'a str,
    pub voucher_no: &'a str,
    pub posting_datetime: NaiveDateTime,
    pub qty: Decimal,
    pub incoming_rate: Decimal,
    pub method: ValuationMethod,
}

/// Stock Valuation Engine.
pub struct ValuationEngine;

impl ValuationEngine {
    /// Processes a Stock Ledger Entry against a Bin, updating quantities, valuation rates,
    /// and stock value difference for accounting GL integration.
    pub fn process_sle(
        bin: &mut StockBin,
        params: SleParams<'_>,
    ) -> Result<StockLedgerEntry, ValuationError> {
        let qty = params.qty;
        let incoming_rate = params.incoming_rate;

        if qty.is_zero() {
            return Err(ValuationError::InvalidRate(incoming_rate));
        }

        let is_inward = qty > Decimal::ZERO;
        let mut outgoing_rate = Decimal::ZERO;
        let previous_stock_value = bin.stock_value;

        if is_inward {
            // INWARD RECEIPT
            match params.method {
                ValuationMethod::FIFO | ValuationMethod::LIFO => {
                    bin.layers.push(StockLayer {
                        qty,
                        rate: incoming_rate,
                    });
                    bin.actual_qty += qty;
                    bin.stock_value += qty * incoming_rate;
                    if !bin.actual_qty.is_zero() {
                        bin.valuation_rate = bin.stock_value / bin.actual_qty;
                    }
                }
                ValuationMethod::MovingAverage => {
                    let incoming_value = qty * incoming_rate;
                    let new_actual_qty = bin.actual_qty + qty;
                    let new_stock_value = bin.stock_value + incoming_value;
                    bin.actual_qty = new_actual_qty;
                    bin.stock_value = new_stock_value;
                    if !new_actual_qty.is_zero() {
                        bin.valuation_rate = new_stock_value / new_actual_qty;
                    }
                }
            }
        } else {
            // OUTWARD DISPATCH
            let consume_qty = qty.abs();
            if bin.actual_qty < consume_qty {
                return Err(ValuationError::InsufficientStock {
                    item_code: bin.item_code.clone(),
                    warehouse: bin.warehouse.clone(),
                    available: bin.actual_qty,
                    requested: consume_qty,
                });
            }

            match params.method {
                ValuationMethod::FIFO => {
                    let mut needed = consume_qty;
                    let mut cost_of_goods_sold = Decimal::ZERO;

                    while needed > Decimal::ZERO && !bin.layers.is_empty() {
                        let layer = &mut bin.layers[0];
                        if layer.qty <= needed {
                            cost_of_goods_sold += layer.qty * layer.rate;
                            needed -= layer.qty;
                            bin.layers.remove(0);
                        } else {
                            cost_of_goods_sold += needed * layer.rate;
                            layer.qty -= needed;
                            needed = Decimal::ZERO;
                        }
                    }

                    bin.actual_qty -= consume_qty;
                    bin.stock_value -= cost_of_goods_sold;
                    outgoing_rate = if !consume_qty.is_zero() {
                        cost_of_goods_sold / consume_qty
                    } else {
                        Decimal::ZERO
                    };

                    if !bin.actual_qty.is_zero() {
                        bin.valuation_rate = bin.stock_value / bin.actual_qty;
                    } else {
                        bin.valuation_rate = Decimal::ZERO;
                        bin.stock_value = Decimal::ZERO;
                    }
                }
                ValuationMethod::LIFO => {
                    let mut needed = consume_qty;
                    let mut cost_of_goods_sold = Decimal::ZERO;

                    while needed > Decimal::ZERO && !bin.layers.is_empty() {
                        let last_idx = bin.layers.len() - 1;
                        let layer = &mut bin.layers[last_idx];
                        if layer.qty <= needed {
                            cost_of_goods_sold += layer.qty * layer.rate;
                            needed -= layer.qty;
                            bin.layers.pop();
                        } else {
                            cost_of_goods_sold += needed * layer.rate;
                            layer.qty -= needed;
                            needed = Decimal::ZERO;
                        }
                    }

                    bin.actual_qty -= consume_qty;
                    bin.stock_value -= cost_of_goods_sold;
                    outgoing_rate = if !consume_qty.is_zero() {
                        cost_of_goods_sold / consume_qty
                    } else {
                        Decimal::ZERO
                    };

                    if !bin.actual_qty.is_zero() {
                        bin.valuation_rate = bin.stock_value / bin.actual_qty;
                    } else {
                        bin.valuation_rate = Decimal::ZERO;
                        bin.stock_value = Decimal::ZERO;
                    }
                }
                ValuationMethod::MovingAverage => {
                    outgoing_rate = bin.valuation_rate;
                    let cogs = consume_qty * outgoing_rate;
                    bin.actual_qty -= consume_qty;
                    bin.stock_value -= cogs;
                    if bin.actual_qty.is_zero() {
                        bin.valuation_rate = Decimal::ZERO;
                        bin.stock_value = Decimal::ZERO;
                    }
                }
            }
        }

        let stock_value_difference = bin.stock_value - previous_stock_value;

        Ok(StockLedgerEntry {
            name: CompactString::new(params.sle_name),
            item_code: bin.item_code.clone(),
            warehouse: bin.warehouse.clone(),
            posting_datetime: params.posting_datetime,
            voucher_type: CompactString::new(params.voucher_type),
            voucher_no: CompactString::new(params.voucher_no),
            actual_qty: qty,
            incoming_rate: if is_inward {
                incoming_rate
            } else {
                Decimal::ZERO
            },
            outgoing_rate,
            stock_value_difference,
            qty_after_transaction: bin.actual_qty,
            stock_value: bin.stock_value,
            valuation_rate: bin.valuation_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_fifo_valuation_layers_and_cogs() {
        let dt = NaiveDate::from_ymd_opt(2026, 10, 16)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        let mut bin = StockBin {
            item_code: "STEEL-ROD".into(),
            warehouse: "Main Store - CORP".into(),
            actual_qty: Decimal::ZERO,
            valuation_rate: Decimal::ZERO,
            stock_value: Decimal::ZERO,
            layers: Vec::new(),
        };

        // 1. Receipt 10 units @ $100 = $1,000
        let sle1 = ValuationEngine::process_sle(
            &mut bin,
            SleParams {
                sle_name: "SLE-001",
                voucher_type: "Purchase Receipt",
                voucher_no: "PR-001",
                posting_datetime: dt,
                qty: dec!(10.0),
                incoming_rate: dec!(100.0),
                method: ValuationMethod::FIFO,
            },
        )
        .unwrap();
        assert_eq!(sle1.stock_value_difference, dec!(1000.0));
        assert_eq!(bin.actual_qty, dec!(10.0));
        assert_eq!(bin.valuation_rate, dec!(100.0));

        // 2. Receipt 10 units @ $200 = $2,000
        let sle2 = ValuationEngine::process_sle(
            &mut bin,
            SleParams {
                sle_name: "SLE-002",
                voucher_type: "Purchase Receipt",
                voucher_no: "PR-002",
                posting_datetime: dt,
                qty: dec!(10.0),
                incoming_rate: dec!(200.0),
                method: ValuationMethod::FIFO,
            },
        )
        .unwrap();
        assert_eq!(sle2.stock_value_difference, dec!(2000.0));
        assert_eq!(bin.actual_qty, dec!(20.0));
        assert_eq!(bin.valuation_rate, dec!(150.0)); // (1000 + 2000) / 20 = 150

        // 3. Dispatch 15 units (FIFO: consumes 10 @ 100 + 5 @ 200 = 1,000 + 1,000 = 2,000 COGS)
        let sle3 = ValuationEngine::process_sle(
            &mut bin,
            SleParams {
                sle_name: "SLE-003",
                voucher_type: "Delivery Note",
                voucher_no: "DN-001",
                posting_datetime: dt,
                qty: dec!(-15.0),
                incoming_rate: Decimal::ZERO,
                method: ValuationMethod::FIFO,
            },
        )
        .unwrap();
        assert_eq!(sle3.outgoing_rate.round_dp(6), dec!(133.333333));
        assert_eq!(sle3.stock_value_difference, dec!(-2000.0));
        assert_eq!(bin.actual_qty, dec!(5.0));
        assert_eq!(bin.stock_value, dec!(1000.0));
        assert_eq!(bin.valuation_rate, dec!(200.0)); // Remaining 5 units are all from Layer 2 ($200)
    }

    #[test]
    fn test_moving_average_valuation() {
        let dt = NaiveDate::from_ymd_opt(2026, 10, 16)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        let mut bin = StockBin {
            item_code: "COPPER-WIRE".into(),
            warehouse: "Raw Material - CORP".into(),
            actual_qty: Decimal::ZERO,
            valuation_rate: Decimal::ZERO,
            stock_value: Decimal::ZERO,
            layers: Vec::new(),
        };

        // Receipt 10 @ 50 = $500
        ValuationEngine::process_sle(
            &mut bin,
            SleParams {
                sle_name: "SLE-01",
                voucher_type: "PR",
                voucher_no: "PR-1",
                posting_datetime: dt,
                qty: dec!(10.0),
                incoming_rate: dec!(50.0),
                method: ValuationMethod::MovingAverage,
            },
        )
        .unwrap();

        // Receipt 10 @ 70 = $700 -> Total Qty 20, Value $1200, Avg Rate $60
        ValuationEngine::process_sle(
            &mut bin,
            SleParams {
                sle_name: "SLE-02",
                voucher_type: "PR",
                voucher_no: "PR-2",
                posting_datetime: dt,
                qty: dec!(10.0),
                incoming_rate: dec!(70.0),
                method: ValuationMethod::MovingAverage,
            },
        )
        .unwrap();
        assert_eq!(bin.valuation_rate, dec!(60.0));

        // Dispatch 5 units -> COGS = 5 * 60 = 300, Remaining = 15 @ 60 = 900
        let sle_out = ValuationEngine::process_sle(
            &mut bin,
            SleParams {
                sle_name: "SLE-03",
                voucher_type: "DN",
                voucher_no: "DN-1",
                posting_datetime: dt,
                qty: dec!(-5.0),
                incoming_rate: Decimal::ZERO,
                method: ValuationMethod::MovingAverage,
            },
        )
        .unwrap();
        assert_eq!(sle_out.stock_value_difference, dec!(-300.0));
        assert_eq!(bin.actual_qty, dec!(15.0));
        assert_eq!(bin.stock_value, dec!(900.0));
        assert_eq!(bin.valuation_rate, dec!(60.0));
    }
}
