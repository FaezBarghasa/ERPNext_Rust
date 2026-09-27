//! Real Dioxus signal state (Stage 6.1): doc-list reactive filter.
//! Signal reads/writes execute inside components at runtime; the pure
//! filter kernel below is unit-tested here.
use dioxus_signals::{ReadableExt, Signal, WritableExt};

/// Pure kernel: case-sensitive substring filter over doc names.
pub fn filter_docs<'a>(docs: &'a [String], query: &str) -> Vec<&'a String> {
    docs.iter().filter(|s| s.contains(query)).collect()
}

/// Component-side helper: read a doc-list signal through the reactive kernel.
pub fn visible_from_signal(sig: &Signal<Vec<String>>, query: &str) -> Vec<String> {
    filter_docs(&sig.peek(), query).into_iter().cloned().collect()
}

/// Component-side helper: push a newly created doc into the list signal.
pub fn push_doc(sig: &mut Signal<Vec<String>>, name: String) {
    sig.write().push(name);
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn filter_kernel() {
        let docs = vec![
            "INV-001".to_string(),
            "INV-002".to_string(),
            "DN-001".to_string(),
        ];
        assert_eq!(filter_docs(&docs, "INV").len(), 2);
        assert_eq!(filter_docs(&docs, "DN").len(), 1);
        assert!(filter_docs(&docs, "XYZ").is_empty());
    }
}
