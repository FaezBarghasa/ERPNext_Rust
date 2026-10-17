use serde::{Deserialize, Serialize};

/// Computes the visible row slice range for a virtualized high-density grid (60 FPS over 1M+ rows).
#[must_use]
pub fn visible_slice(
    total: usize,
    row_h: usize,
    scroll: usize,
    viewport_h: usize,
) -> (usize, usize) {
    if row_h == 0 || total == 0 {
        return (0, 0);
    }
    let start = (scroll / row_h).min(total);
    let count = viewport_h.div_ceil(row_h).min(total - start);
    (start, count)
}

/// 2D cell coordinate in the virtualized grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellCoordinate {
    pub row: usize,
    pub col: usize,
}

/// Rectangular cell selection range for spreadsheet operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellSelectionRange {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

impl CellSelectionRange {
    /// Normalizes range boundaries into `(min_row..=max_row, min_col..=max_col)`.
    pub fn normalized_bounds(&self) -> ((usize, usize), (usize, usize)) {
        let min_r = self.start_row.min(self.end_row);
        let max_r = self.start_row.max(self.end_row);
        let min_c = self.start_col.min(self.end_col);
        let max_c = self.start_col.max(self.end_col);
        ((min_r, max_r), (min_c, max_c))
    }

    /// Checks if a cell coordinate falls within the selection box.
    pub fn contains(&self, coord: CellCoordinate) -> bool {
        let ((min_r, max_r), (min_c, max_c)) = self.normalized_bounds();
        coord.row >= min_r && coord.row <= max_r && coord.col >= min_c && coord.col <= max_c
    }

    /// Counts total number of selected cells.
    pub fn cell_count(&self) -> usize {
        let ((min_r, max_r), (min_c, max_c)) = self.normalized_bounds();
        (max_r - min_r + 1) * (max_c - min_c + 1)
    }
}

/// Keyboard navigation movement direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridNavDirection {
    Up,
    Down,
    Left,
    Right,
    NextCell, // Tab
    PrevCell, // Shift+Tab
    NextRow,  // Enter
}

/// State model for high-density virtualized data grid with spreadsheet keyboard navigation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualizedGridState {
    pub total_rows: usize,
    pub total_cols: usize,
    pub active_cell: CellCoordinate,
    pub selection: Option<CellSelectionRange>,
}

impl VirtualizedGridState {
    pub fn new(total_rows: usize, total_cols: usize) -> Self {
        Self {
            total_rows,
            total_cols,
            active_cell: CellCoordinate { row: 0, col: 0 },
            selection: None,
        }
    }

    /// Moves the active cell in response to keyboard actions, respecting grid boundaries.
    pub fn navigate(&mut self, dir: GridNavDirection, extend_selection: bool) {
        let current = self.active_cell;
        let mut next = current;

        match dir {
            GridNavDirection::Up => {
                next.row = next.row.saturating_sub(1);
            }
            GridNavDirection::Down | GridNavDirection::NextRow => {
                if next.row + 1 < self.total_rows {
                    next.row += 1;
                }
            }
            GridNavDirection::Left => {
                next.col = next.col.saturating_sub(1);
            }
            GridNavDirection::Right | GridNavDirection::NextCell => {
                if next.col + 1 < self.total_cols {
                    next.col += 1;
                }
            }
            GridNavDirection::PrevCell => {
                if next.col > 0 {
                    next.col -= 1;
                } else if next.row > 0 {
                    next.row -= 1;
                    next.col = self.total_cols.saturating_sub(1);
                }
            }
        }

        self.active_cell = next;

        if extend_selection {
            if let Some(ref mut sel) = self.selection {
                sel.end_row = next.row;
                sel.end_col = next.col;
            } else {
                self.selection = Some(CellSelectionRange {
                    start_row: current.row,
                    start_col: current.col,
                    end_row: next.row,
                    end_col: next.col,
                });
            }
        } else {
            self.selection = None;
        }
    }
}

use compact_str::CompactString;
use rust_decimal::Decimal;
use std::str::FromStr;

/// In-cell editable child table grid with row mutations, spreadsheet keyboard navigation,
/// drag-and-drop row reordering, and real-time parent arithmetic triggers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditableChildTableGrid {
    pub child_doctype: CompactString,
    pub rows: Vec<serde_json::Map<String, serde_json::Value>>,
    pub nav_state: VirtualizedGridState,
    pub is_editing: bool,
}

impl EditableChildTableGrid {
    #[must_use]
    pub fn new(child_doctype: impl Into<CompactString>, num_cols: usize) -> Self {
        Self {
            child_doctype: child_doctype.into(),
            rows: Vec::new(),
            nav_state: VirtualizedGridState::new(0, num_cols),
            is_editing: false,
        }
    }

    /// Appends a new blank or template row to the child table.
    pub fn add_row(&mut self, mut row: serde_json::Map<String, serde_json::Value>) -> usize {
        let idx = self.rows.len();
        row.insert("idx".to_string(), serde_json::json!(idx + 1));
        self.rows.push(row);
        self.nav_state.total_rows = self.rows.len();
        idx
    }

    /// Removes a row at the given index.
    pub fn remove_row(&mut self, row_idx: usize) -> bool {
        if row_idx < self.rows.len() {
            self.rows.remove(row_idx);
            // Re-index remaining rows
            for (i, r) in self.rows.iter_mut().enumerate() {
                r.insert("idx".to_string(), serde_json::json!(i + 1));
            }
            self.nav_state.total_rows = self.rows.len();
            if self.nav_state.active_cell.row >= self.rows.len() && !self.rows.is_empty() {
                self.nav_state.active_cell.row = self.rows.len() - 1;
            }
            true
        } else {
            false
        }
    }

    /// Duplicates an existing row directly beneath it.
    pub fn duplicate_row(&mut self, row_idx: usize) -> Option<usize> {
        if row_idx < self.rows.len() {
            let cloned = self.rows[row_idx].clone();
            let insert_idx = row_idx + 1;
            self.rows.insert(insert_idx, cloned);
            for (i, r) in self.rows.iter_mut().enumerate() {
                r.insert("idx".to_string(), serde_json::json!(i + 1));
            }
            self.nav_state.total_rows = self.rows.len();
            Some(insert_idx)
        } else {
            None
        }
    }

    /// Reorders a row via drag-and-drop from `from_idx` to `to_idx`.
    pub fn move_row(&mut self, from_idx: usize, to_idx: usize) -> bool {
        if from_idx < self.rows.len() && to_idx < self.rows.len() {
            let row = self.rows.remove(from_idx);
            self.rows.insert(to_idx, row);
            for (i, r) in self.rows.iter_mut().enumerate() {
                r.insert("idx".to_string(), serde_json::json!(i + 1));
            }
            true
        } else {
            false
        }
    }

    /// Recalculates line amounts and returns `(net_total, tax_amount, grand_total)`.
    pub fn recalculate_totals(
        &mut self,
        qty_field: &str,
        rate_field: &str,
        amount_field: &str,
        tax_rate_pct: Decimal,
    ) -> (Decimal, Decimal, Decimal) {
        let mut net_total = Decimal::ZERO;

        for r in &mut self.rows {
            let qty = r
                .get(qty_field)
                .and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Decimal::from_str(s).ok()
                    } else if let Some(n) = v.as_i64() {
                        Some(Decimal::from(n))
                    } else {
                        v.as_f64()
                            .and_then(|f| Decimal::from_str(&format!("{f:.4}")).ok())
                    }
                })
                .unwrap_or(Decimal::ZERO);

            let rate = r
                .get(rate_field)
                .and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Decimal::from_str(s).ok()
                    } else if let Some(n) = v.as_i64() {
                        Some(Decimal::from(n))
                    } else {
                        v.as_f64()
                            .and_then(|f| Decimal::from_str(&format!("{f:.4}")).ok())
                    }
                })
                .unwrap_or(Decimal::ZERO);

            let amount = qty * rate;
            r.insert(
                amount_field.to_string(),
                serde_json::json!(amount.to_string()),
            );
            net_total += amount;
        }

        let tax_amount = (net_total * tax_rate_pct) / Decimal::from(100);
        let grand_total = net_total + tax_amount;
        (net_total, tax_amount, grand_total)
    }

    /// Navigates cell selection.
    pub fn navigate(&mut self, dir: GridNavDirection, extend_selection: bool) {
        self.nav_state.navigate(dir, extend_selection);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtualized_slice() {
        assert_eq!(visible_slice(1_000_000, 32, 3200, 640), (100, 20));
    }

    #[test]
    fn test_grid_navigation_and_selection() {
        let mut state = VirtualizedGridState::new(100, 10);
        assert_eq!(state.active_cell, CellCoordinate { row: 0, col: 0 });

        // Navigate Right with Tab
        state.navigate(GridNavDirection::NextCell, false);
        assert_eq!(state.active_cell, CellCoordinate { row: 0, col: 1 });

        // Navigate Down with Enter
        state.navigate(GridNavDirection::NextRow, false);
        assert_eq!(state.active_cell, CellCoordinate { row: 1, col: 1 });

        // Extend selection downwards
        state.navigate(GridNavDirection::Down, true);
        assert_eq!(state.active_cell, CellCoordinate { row: 2, col: 1 });

        let sel = state.selection.expect("Selection expected");
        assert_eq!(sel.cell_count(), 2); // Rows 1..=2, Col 1
        assert!(sel.contains(CellCoordinate { row: 1, col: 1 }));
        assert!(sel.contains(CellCoordinate { row: 2, col: 1 }));
        assert!(!sel.contains(CellCoordinate { row: 0, col: 1 }));
    }

    #[test]
    fn test_editable_child_table_mutations_and_recalc() {
        use rust_decimal_macros::dec;

        let mut grid = EditableChildTableGrid::new("Sales Invoice Item", 4);
        assert_eq!(grid.rows.len(), 0);

        let mut row1 = serde_json::Map::new();
        row1.insert("item_code".into(), serde_json::json!("LAPTOP"));
        row1.insert("qty".into(), serde_json::json!("2"));
        row1.insert("rate".into(), serde_json::json!("1000.00"));
        grid.add_row(row1);

        let mut row2 = serde_json::Map::new();
        row2.insert("item_code".into(), serde_json::json!("MOUSE"));
        row2.insert("qty".into(), serde_json::json!("1"));
        row2.insert("rate".into(), serde_json::json!("50.00"));
        grid.add_row(row2);

        assert_eq!(grid.rows.len(), 2);
        assert_eq!(grid.rows[0].get("idx").unwrap(), 1);
        assert_eq!(grid.rows[1].get("idx").unwrap(), 2);

        // Test real-time calculation triggers
        let (net, tax, grand) = grid.recalculate_totals("qty", "rate", "amount", dec!(10.0));
        assert_eq!(net, dec!(2050.00));
        assert_eq!(tax, dec!(205.00));
        assert_eq!(grand, dec!(2255.00));

        // Duplicate row 1
        let dup_idx = grid.duplicate_row(0).expect("duplicated");
        assert_eq!(dup_idx, 1);
        assert_eq!(grid.rows.len(), 3);

        // Move row 2 to 0
        assert!(grid.move_row(2, 0));
        assert_eq!(grid.rows[0].get("item_code").unwrap(), "MOUSE");

        // Remove row
        assert!(grid.remove_row(0));
        assert_eq!(grid.rows.len(), 2);
    }
}
