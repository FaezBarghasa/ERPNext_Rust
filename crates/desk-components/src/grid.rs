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
}
