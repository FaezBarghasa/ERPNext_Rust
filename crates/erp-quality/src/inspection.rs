//! Quality Inspection and Inspection Template Module (`erp-quality::inspection`).
//!
//! Provides enterprise quality control inspections:
//! - Incoming material inspection (Goods Receipt)
//! - In-process manufacturing inspection (Job Card / Work Order)
//! - Outgoing finished goods inspection (Delivery Note)

use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Quality control inspection error types.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum QualityError {
    #[error("Sample size must be greater than zero, got {0}")]
    InvalidSampleSize(u32),
    #[error("Inspection reading out of bounds for parameter '{0}': value {1} not in [{2}, {3}]")]
    ReadingOutOfBounds(String, Decimal, Decimal, Decimal),
    #[error("Inspection must have at least one test parameter")]
    EmptyReadings,
    #[error("Invalid status transition from {0:?} to {1:?}")]
    InvalidStatusTransition(InspectionStatus, InspectionStatus),
    #[error("Cannot modify submitted or cancelled inspection {0}")]
    InspectionLocked(String),
}

/// Inspection stage classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InspectionType {
    Incoming,
    InProcess,
    Outgoing,
}

/// Quality inspection final status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InspectionStatus {
    Draft,
    Accepted,
    Rejected,
    Cancelled,
}

/// A single parameter test reading.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityReading {
    /// Name of the inspected parameter (e.g. "Hardness", "Length", "Surface Roughness").
    pub parameter: CompactString,
    /// Minimum permissible value.
    pub min_value: Decimal,
    /// Maximum permissible value.
    pub max_value: Decimal,
    /// Actual measured value.
    pub reading_value: Decimal,
    /// Pass/Fail outcome for this parameter.
    pub status: InspectionStatus,
}

impl QualityReading {
    /// Evaluates whether the measured reading falls within the [min, max] range.
    #[must_use]
    pub fn evaluate(parameter: &str, min: Decimal, max: Decimal, reading: Decimal) -> Self {
        let is_pass = reading >= min && reading <= max;
        Self {
            parameter: parameter.into(),
            min_value: min,
            max_value: max,
            reading_value: reading,
            status: if is_pass {
                InspectionStatus::Accepted
            } else {
                InspectionStatus::Rejected
            },
        }
    }
}

/// Template defining standard parameters for an item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityInspectionTemplate {
    pub template_name: CompactString,
    pub item_code: CompactString,
    pub parameters: Vec<InspectionParameterDef>,
}

/// Parameter definition inside a template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InspectionParameterDef {
    pub parameter: CompactString,
    pub min_value: Decimal,
    pub max_value: Decimal,
    pub uom: Option<CompactString>,
}

/// Complete Quality Inspection document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityInspection {
    /// Unique inspection document name (e.g. "QI-2026-00001").
    pub name: CompactString,
    /// Inspection type.
    pub inspection_type: InspectionType,
    /// Target item code inspected.
    pub item_code: CompactString,
    /// Reference document type (e.g. "Purchase Receipt", "Delivery Note", "Job Card").
    pub reference_type: CompactString,
    /// Reference document identifier.
    pub reference_name: CompactString,
    /// Inspected batch number if applicable.
    pub batch_no: Option<CompactString>,
    /// Quantity of sample tested.
    pub sample_size: u32,
    /// List of parameter inspection readings.
    pub readings: Vec<QualityReading>,
    /// Inspected date.
    pub inspection_date: NaiveDate,
    /// Inspector user ID or name.
    pub inspected_by: CompactString,
    /// Final inspection verdict.
    pub status: InspectionStatus,
}

impl QualityInspection {
    /// Creates a new Draft Quality Inspection document.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        inspection_type: InspectionType,
        item_code: &str,
        reference_type: &str,
        reference_name: &str,
        sample_size: u32,
        inspection_date: NaiveDate,
        inspected_by: &str,
    ) -> Result<Self, QualityError> {
        if sample_size == 0 {
            return Err(QualityError::InvalidSampleSize(sample_size));
        }
        Ok(Self {
            name: name.into(),
            inspection_type,
            item_code: item_code.into(),
            reference_type: reference_type.into(),
            reference_name: reference_name.into(),
            batch_no: None,
            sample_size,
            readings: Vec::new(),
            inspection_date,
            inspected_by: inspected_by.into(),
            status: InspectionStatus::Draft,
        })
    }

    /// Adds a reading to the inspection.
    pub fn add_reading(&mut self, reading: QualityReading) -> Result<(), QualityError> {
        if self.status != InspectionStatus::Draft {
            return Err(QualityError::InspectionLocked(self.name.to_string()));
        }
        self.readings.push(reading);
        Ok(())
    }

    /// Finalizes and submits the inspection, calculating the overall status.
    /// If ANY parameter reading is Rejected, the overall inspection is Rejected.
    pub fn submit(&mut self) -> Result<InspectionStatus, QualityError> {
        if self.status != InspectionStatus::Draft {
            return Err(QualityError::InspectionLocked(self.name.to_string()));
        }
        if self.readings.is_empty() {
            return Err(QualityError::EmptyReadings);
        }

        let all_passed = self
            .readings
            .iter()
            .all(|r| r.status == InspectionStatus::Accepted);
        self.status = if all_passed {
            InspectionStatus::Accepted
        } else {
            InspectionStatus::Rejected
        };

        Ok(self.status)
    }
}
