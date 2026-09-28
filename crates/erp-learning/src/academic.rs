//! Academic Education Lifecycle, Assessment Plans, and Student Fee Governance.
//!
//! Provides:
//! - Student admission lifecycle (`StudentApplicant` -> `ProgramEnrollment` -> `Student`).
//! - Academic Program, Course, and Student Batch hierarchies.
//! - Grading scales, assessment evaluations, and GPA calculations.
//! - Student fee structures, category breakdowns, and tuition billing schedules.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Academic errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AcademicError {
    #[error("Student {student_id} is already enrolled in program {program_id}")]
    AlreadyEnrolled {
        student_id: String,
        program_id: String,
    },
    #[error("Assessment score {score} is invalid (must be 0-100)")]
    InvalidScore { score: u32 },
    #[error("Fee category total does not balance with grand total")]
    FeeImbalance,
}

/// Student admission application status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplicationStatus {
    Applied,
    InterviewScheduled,
    Approved,
    Enrolled,
    Rejected,
}

/// A prospective student applicant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentApplicant {
    pub applicant_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub target_program_id: String,
    pub status: ApplicationStatus,
    pub application_date: DateTime<Utc>,
}

/// Active student record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentMaster {
    pub student_id: String,
    pub applicant_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub active: bool,
}

/// An academic program containing mandatory and elective courses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicProgram {
    pub program_id: String,
    pub program_name: String,
    pub department: String,
    pub total_credits_required: u32,
    pub course_ids: Vec<String>,
}

/// Program enrollment registering a student to a program and batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramEnrollment {
    pub enrollment_id: String,
    pub student_id: String,
    pub program_id: String,
    pub batch_id: String,
    pub academic_year: String,
    pub enrollment_date: DateTime<Utc>,
    pub completed: bool,
}

/// A letter grade band with grade points and score thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeInterval {
    pub grade_symbol: String,
    pub min_score: u32,
    pub max_score: u32,
    pub grade_points: Decimal,
}

/// Grading scale configuration for GPA evaluations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingScale {
    pub scale_name: String,
    pub intervals: Vec<GradeInterval>,
}

impl GradingScale {
    /// Resolves a numeric score (0-100) to its corresponding letter grade and points.
    pub fn resolve_grade(&self, score: u32) -> Result<(&str, Decimal), AcademicError> {
        if score > 100 {
            return Err(AcademicError::InvalidScore { score });
        }
        for interval in &self.intervals {
            if score >= interval.min_score && score <= interval.max_score {
                return Ok((interval.grade_symbol.as_str(), interval.grade_points));
            }
        }
        Ok(("F", Decimal::ZERO))
    }
}

/// An individual assessment result for a student in a course.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResult {
    pub result_id: String,
    pub student_id: String,
    pub course_id: String,
    pub raw_score: u32,
    pub grade: String,
    pub grade_points: Decimal,
}

/// A line item in a student fee structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeCategoryItem {
    pub category_name: String,
    pub amount: Decimal,
}

/// A multi-category tuition fee structure for an academic term.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    pub fee_structure_id: String,
    pub program_id: String,
    pub academic_term: String,
    pub currency: String,
    pub components: Vec<FeeCategoryItem>,
}

impl FeeStructure {
    /// Computes the total tuition and fee amount for the term.
    pub fn grand_total(&self) -> Decimal {
        self.components.iter().map(|c| c.amount).sum()
    }
}

/// A generated student fee invoice schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentFeeSchedule {
    pub schedule_id: String,
    pub student_id: String,
    pub fee_structure_id: String,
    pub due_date: DateTime<Utc>,
    pub total_amount: Decimal,
    pub outstanding_amount: Decimal,
    pub is_paid: bool,
}

impl StudentFeeSchedule {
    /// Creates a new fee schedule for an enrolled student.
    pub fn new(
        schedule_id: &str,
        student_id: &str,
        fee_structure: &FeeStructure,
        due_date: DateTime<Utc>,
    ) -> Self {
        let total = fee_structure.grand_total();
        Self {
            schedule_id: schedule_id.to_string(),
            student_id: student_id.to_string(),
            fee_structure_id: fee_structure.fee_structure_id.clone(),
            due_date,
            total_amount: total,
            outstanding_amount: total,
            is_paid: false,
        }
    }

    /// Records a payment towards the fee schedule.
    pub fn record_payment(&mut self, paid_amount: Decimal) {
        if paid_amount >= self.outstanding_amount {
            self.outstanding_amount = Decimal::ZERO;
            self.is_paid = true;
        } else {
            self.outstanding_amount -= paid_amount;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_grading_scale_resolution() {
        let scale = GradingScale {
            scale_name: "Standard Academic 4.0 Scale".into(),
            intervals: vec![
                GradeInterval {
                    grade_symbol: "A+".into(),
                    min_score: 95,
                    max_score: 100,
                    grade_points: dec!(4.0),
                },
                GradeInterval {
                    grade_symbol: "A".into(),
                    min_score: 85,
                    max_score: 94,
                    grade_points: dec!(3.7),
                },
                GradeInterval {
                    grade_symbol: "B".into(),
                    min_score: 75,
                    max_score: 84,
                    grade_points: dec!(3.0),
                },
                GradeInterval {
                    grade_symbol: "C".into(),
                    min_score: 60,
                    max_score: 74,
                    grade_points: dec!(2.0),
                },
            ],
        };

        let (grade, points) = scale.resolve_grade(97).unwrap();
        assert_eq!(grade, "A+");
        assert_eq!(points, dec!(4.0));

        let (grade_b, points_b) = scale.resolve_grade(80).unwrap();
        assert_eq!(grade_b, "B");
        assert_eq!(points_b, dec!(3.0));

        let (grade_f, points_f) = scale.resolve_grade(45).unwrap();
        assert_eq!(grade_f, "F");
        assert_eq!(points_f, Decimal::ZERO);

        assert!(scale.resolve_grade(105).is_err());
    }

    #[test]
    fn test_student_fee_schedule_and_payment_offset() {
        let fee_struct = FeeStructure {
            fee_structure_id: "FEE-2026-CS".into(),
            program_id: "BS-CS".into(),
            academic_term: "Fall 2026".into(),
            currency: "USD".into(),
            components: vec![
                FeeCategoryItem {
                    category_name: "Tuition".into(),
                    amount: dec!(4500.00),
                },
                FeeCategoryItem {
                    category_name: "Lab Access".into(),
                    amount: dec!(350.00),
                },
                FeeCategoryItem {
                    category_name: "Library & Athletics".into(),
                    amount: dec!(150.00),
                },
            ],
        };

        assert_eq!(fee_struct.grand_total(), dec!(5000.00));

        let mut schedule =
            StudentFeeSchedule::new("SCHED-001", "STU-8821", &fee_struct, Utc::now());

        assert_eq!(schedule.total_amount, dec!(5000.00));
        assert_eq!(schedule.outstanding_amount, dec!(5000.00));
        assert!(!schedule.is_paid);

        // Partial payment of $2000
        schedule.record_payment(dec!(2000.00));
        assert_eq!(schedule.outstanding_amount, dec!(3000.00));
        assert!(!schedule.is_paid);

        // Settle remaining $3000
        schedule.record_payment(dec!(3000.00));
        assert_eq!(schedule.outstanding_amount, dec!(0.00));
        assert!(schedule.is_paid);
    }
}
