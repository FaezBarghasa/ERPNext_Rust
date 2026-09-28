use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Catch-weight scale tare subtraction and dual-variable inventory calculation record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatchWeightItem {
    /// Item identifier.
    pub item_code: String,
    /// Inventory unit of measure (e.g. "Carcass", "Bundle", "Drum").
    pub inventory_uom: String,
    /// Commercial pricing weight unit (e.g. "Kg", "Lb", "Metric Ton").
    pub weight_uom: String,
    /// Number of inventory pieces.
    pub piece_count: u32,
    /// Gross scale weight measured.
    pub gross_weight: Decimal,
    /// List of tare weights (pallets, hooks, packaging).
    pub tare_weights: Vec<Decimal>,
}

impl CatchWeightItem {
    /// Total tare weight.
    pub fn total_tare(&self) -> Decimal {
        self.tare_weights.iter().copied().sum()
    }

    /// Net scale weight: `Gross Weight - Σ Tare Weights`.
    pub fn net_weight(&self) -> Decimal {
        (self.gross_weight - self.total_tare()).max(Decimal::ZERO)
    }

    /// Computes total commercial billing amount given price per weight unit: `Net Weight * Unit Price`.
    pub fn calculate_billable_amount(&self, price_per_weight_unit: Decimal) -> Decimal {
        self.net_weight() * price_per_weight_unit
    }

    /// Computes average weight per piece: `Net Weight / Piece Count`.
    pub fn average_piece_weight(&self) -> Decimal {
        if self.piece_count == 0 {
            Decimal::ZERO
        } else {
            self.net_weight() / Decimal::from(self.piece_count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_catch_weight_tare_subtraction_and_billing() {
        let item = CatchWeightItem {
            item_code: "BEEF-CARCASS-01".into(),
            inventory_uom: "Carcass".into(),
            weight_uom: "Kg".into(),
            piece_count: 5,
            gross_weight: dec!(1550.0), // 1550 kg gross
            tare_weights: vec![
                dec!(20.0), // Pallet tare
                dec!(5.0),  // Hook tares
                dec!(2.5),  // Packaging plastic
            ],
        };

        // Total tare = 27.5 kg
        assert_eq!(item.total_tare(), dec!(27.5));
        // Net weight = 1522.5 kg
        assert_eq!(item.net_weight(), dec!(1522.5));
        // Average piece weight = 1522.5 / 5 = 304.5 kg
        assert_eq!(item.average_piece_weight(), dec!(304.5));

        // Billable amount @ $8.50/kg = 1522.5 * 8.50 = $12,941.25
        let total_amount = item.calculate_billable_amount(dec!(8.50));
        assert_eq!(total_amount, dec!(12941.25));
    }
}
