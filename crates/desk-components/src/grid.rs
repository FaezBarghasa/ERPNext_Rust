/// Visible-row slice for virtualized grid (Stage 6.1.1): 60fps over 1M rows.
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
#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn slice() {
        assert_eq!(visible_slice(1_000_000, 32, 3200, 640), (100, 20));
    }
}
