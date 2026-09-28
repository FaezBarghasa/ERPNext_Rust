//! Domain error types for software subscription billing, ASC 606 revenue recognition, and PSA engine.

use thiserror::Error;

/// Error variants encountered in subscription billing, revenue recognition, and PSA operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SoftwareBillingError {
    /// Invalid contract terms.
    #[error("Invalid contract terms: {0}")]
    InvalidContractTerms(String),

    /// Relative SSP allocation mismatch where allocated sum differs from transaction price.
    #[error("SSP allocation mismatch: total allocated revenue {allocated} does not match transaction price {expected}")]
    SspAllocationMismatch { allocated: String, expected: String },

    /// Standalone Selling Price sum is zero, preventing ratio computation.
    #[error("Total Standalone Selling Price (SSP) cannot be zero")]
    ZeroSellingPrice,

    /// Invalid tiered pricing configuration.
    #[error("Invalid tier configuration: {0}")]
    InvalidTierConfiguration(String),

    /// Schedule period bounds out of range.
    #[error("Schedule out of period bounds: start {start} > end {end}")]
    ScheduleOutOfPeriod { start: String, end: String },

    /// Duplicate usage idempotency key.
    #[error("Duplicate metered event idempotency key: {0}")]
    UsageIdempotencyViolation(String),

    /// Available capacity hours cannot be zero or negative.
    #[error("Total capacity hours must be strictly positive")]
    ZeroCapacity,

    /// Specified performance obligation was not found.
    #[error("Contract obligation not found: {0}")]
    ObligationNotFound(String),

    /// No billing line items generated.
    #[error("No billing line items generated")]
    EmptyInvoice,
}
