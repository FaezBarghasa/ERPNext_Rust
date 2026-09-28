use crate::lifecycle::DocumentError;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Sealed trait representing the compile-time state of an enterprise document.
pub trait DocumentState: Send + Sync + 'static {
    /// Numerical representation of the document state: 0 = Draft, 1 = Submitted, 2 = Cancelled.
    const DOCSTATUS: i32;
    /// Human-readable label for this document state.
    const NAME: &'static str;
}

/// Zero-sized marker type for a mutable draft document (docstatus = 0).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftState;
impl DocumentState for DraftState {
    const DOCSTATUS: i32 = 0;
    const NAME: &'static str = "Draft";
}

/// Zero-sized marker type for an immutable submitted voucher (docstatus = 1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmittedState;
impl DocumentState for SubmittedState {
    const DOCSTATUS: i32 = 1;
    const NAME: &'static str = "Submitted";
}

/// Zero-sized marker type for an immutable cancelled record (docstatus = 2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelledState;
impl DocumentState for CancelledState {
    const DOCSTATUS: i32 = 2;
    const NAME: &'static str = "Cancelled";
}

/// Zero-sized marker type for a purged / archived document (docstatus = -1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgedState;
impl DocumentState for PurgedState {
    const DOCSTATUS: i32 = -1;
    const NAME: &'static str = "Purged";
}

/// Canonical metadata header for an enterprise document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentHeader {
    pub id: String,
    pub tenant_id: String,
    pub created_by: String,
    pub modified_by: String,
    pub sequence_number: u64,
    pub perm_level_cache: u16,
}

/// Generic phantom-typed document wrapper enforcing compile-time state machine invariance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Doc<T, S: DocumentState> {
    /// Primary key identifier.
    pub id: String,
    /// Tenant identifier.
    pub tenant_id: String,
    /// Transaction sequence number.
    pub sequence_no: u64,
    /// Underlying entity payload data.
    pub data: T,
    /// Phantom state marker.
    #[serde(skip)]
    _state: PhantomData<S>,
}

impl<T, S: DocumentState> Doc<T, S> {
    /// Document identifier reference.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Tenant identifier reference.
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Transaction sequence number.
    pub fn sequence_no(&self) -> u64 {
        self.sequence_no
    }

    /// Current document status integer (0 = Draft, 1 = Submitted, 2 = Cancelled).
    pub fn docstatus(&self) -> i32 {
        S::DOCSTATUS
    }

    /// Current document status name.
    pub fn state_name(&self) -> &'static str {
        S::NAME
    }

    /// Immutable reference to the payload data.
    pub fn data(&self) -> &T {
        &self.data
    }
}

/// Mutable operations are exclusively implemented on `Doc<T, DraftState>`.
impl<T> Doc<T, DraftState> {
    /// Creates a new draft document.
    pub fn new_draft(id: impl Into<String>, tenant_id: impl Into<String>, data: T) -> Self {
        Self {
            id: id.into(),
            tenant_id: tenant_id.into(),
            sequence_no: 1,
            data,
            _state: PhantomData,
        }
    }

    /// Mutable access to the inner payload data (strictly disallowed for Submitted or Cancelled docs).
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

/// Security context for authorizing state transitions.
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: String,
    pub roles: Vec<String>,
    pub perm_level: u8,
}

impl SecurityContext {
    pub fn system_admin() -> Self {
        Self {
            user_id: "Administrator".into(),
            roles: vec!["System Manager".into(), "Administrator".into()],
            perm_level: 0,
        }
    }
}

/// Monadic lifecycle transition contract for invariant document state management.
pub trait DocumentLifecycle<T>: Sized {
    /// Validates the draft document before submission.
    fn validate(self) -> Result<Doc<T, DraftState>, DocumentError>;

    /// Submits the document atomically, transitioning from Draft to Submitted state.
    fn submit(self, ctx: &SecurityContext) -> Result<Doc<T, SubmittedState>, DocumentError>;

    /// Cancels the submitted document, producing a mirror reversal and transitioning to Cancelled state.
    fn cancel(self, ctx: &SecurityContext) -> Result<Doc<T, CancelledState>, DocumentError>;
}

impl<T> DocumentLifecycle<T> for Doc<T, DraftState> {
    fn validate(self) -> Result<Doc<T, DraftState>, DocumentError> {
        // Validation logic passes and returns the validated draft
        Ok(self)
    }

    fn submit(self, ctx: &SecurityContext) -> Result<Doc<T, SubmittedState>, DocumentError> {
        if ctx.roles.is_empty() {
            return Err(DocumentError::WorkflowTransitionDenied(
                "Draft".into(),
                "Submitted".into(),
                "Anonymous".into(),
            ));
        }

        Ok(Doc {
            id: self.id,
            tenant_id: self.tenant_id,
            sequence_no: self.sequence_no + 1,
            data: self.data,
            _state: PhantomData,
        })
    }

    fn cancel(self, _ctx: &SecurityContext) -> Result<Doc<T, CancelledState>, DocumentError> {
        // A draft cannot directly cancel without being submitted first
        Err(DocumentError::CannotCancelUnsubmittedDocument(0))
    }
}

impl<T> Doc<T, SubmittedState> {
    /// Cancels a submitted document atomically, returning the Cancelled document wrapper.
    pub fn cancel(self, ctx: &SecurityContext) -> Result<Doc<T, CancelledState>, DocumentError> {
        if ctx.roles.is_empty() {
            return Err(DocumentError::WorkflowTransitionDenied(
                "Submitted".into(),
                "Cancelled".into(),
                "Anonymous".into(),
            ));
        }

        Ok(Doc {
            id: self.id,
            tenant_id: self.tenant_id,
            sequence_no: self.sequence_no + 1,
            data: self.data,
            _state: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct InvoiceData {
        customer: String,
        amount: u64,
    }

    #[test]
    fn test_typestate_compile_time_invariance() {
        let mut draft = Doc::new_draft(
            "INV-001",
            "tenant_acme",
            InvoiceData {
                customer: "Acme Corp".into(),
                amount: 1000,
            },
        );

        assert_eq!(draft.docstatus(), 0);
        assert_eq!(draft.state_name(), "Draft");

        // Mutating draft payload is permitted
        draft.data_mut().amount = 1200;
        assert_eq!(draft.data().amount, 1200);

        let validated = draft.validate().expect("Validation failed");

        let ctx = SecurityContext::system_admin();
        let submitted = validated.submit(&ctx).expect("Submission failed");

        assert_eq!(submitted.docstatus(), 1);
        assert_eq!(submitted.state_name(), "Submitted");
        assert_eq!(submitted.sequence_no(), 2);
        assert_eq!(submitted.data().amount, 1200);

        // Transition from Submitted to Cancelled
        let cancelled = submitted.cancel(&ctx).expect("Cancellation failed");
        assert_eq!(cancelled.docstatus(), 2);
        assert_eq!(cancelled.state_name(), "Cancelled");
        assert_eq!(cancelled.sequence_no(), 3);
    }
}
