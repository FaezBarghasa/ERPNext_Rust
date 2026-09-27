use std::sync::{Arc, RwLock};

/// Reactive Signal wrapper for client-side state management (Milestone 5.6).
#[derive(Debug, Clone)]
pub struct Signal<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> Signal<T> {
    /// Creates a new reactive signal initialized with the given value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    /// Reads the current signal value without tracking.
    pub fn peek<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.inner.read().expect("Signal read lock poisoned");
        f(&guard)
    }

    /// Mutates the signal value.
    pub fn set(&self, new_val: T) {
        let mut guard = self.inner.write().expect("Signal write lock poisoned");
        *guard = new_val;
    }

    /// Mutates the signal value in-place with a closure.
    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.inner.write().expect("Signal write lock poisoned");
        f(&mut guard)
    }
}

/// Pure kernel: case-sensitive substring filter over doc names.
#[must_use]
pub fn filter_docs<'a>(docs: &'a [String], query: &str) -> Vec<&'a String> {
    docs.iter().filter(|s| s.contains(query)).collect()
}

/// Helper reading filtered document list from a Signal.
#[must_use]
pub fn visible_from_signal(sig: &Signal<Vec<String>>, query: &str) -> Vec<String> {
    sig.peek(|docs| filter_docs(docs, query).into_iter().cloned().collect())
}

/// Helper appending a new doc name to a Signal list.
pub fn push_doc(sig: &Signal<Vec<String>>, name: String) {
    sig.update(|docs| docs.push(name));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_reactivity_and_filtering() {
        let docs_signal = Signal::new(vec![
            "INV-001".to_string(),
            "INV-002".to_string(),
            "DN-001".to_string(),
        ]);

        let inv_docs = visible_from_signal(&docs_signal, "INV");
        assert_eq!(inv_docs.len(), 2);

        push_doc(&docs_signal, "INV-003".to_string());
        let updated_inv = visible_from_signal(&docs_signal, "INV");
        assert_eq!(updated_inv.len(), 3);
    }
}
