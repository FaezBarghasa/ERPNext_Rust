use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// LMS errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LmsError {
    /// Course not found.
    #[error("Course not found: {0}")]
    CourseNotFound(String),
    /// Assessment failed.
    #[error(
        "Assessment score {score}% is below required passing threshold of {passing_threshold}%"
    )]
    AssessmentFailed { score: u32, passing_threshold: u32 },
}

/// Educational lesson content node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub content_markdown: String,
    pub video_url: Option<String>,
}

/// Educational module containing lessons.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CourseModule {
    pub id: String,
    pub title: String,
    pub lessons: Vec<Lesson>,
}

/// Complete educational course curriculum (Milestone 4.6).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Course {
    pub id: String,
    pub title: String,
    pub description: String,
    pub modules: Vec<CourseModule>,
    pub passing_threshold: u32,
}

impl Course {
    /// Returns total lesson count across all modules.
    #[must_use]
    pub fn total_lesson_count(&self) -> usize {
        self.modules.iter().map(|m| m.lessons.len()).sum()
    }
}

/// Verified completion certificate (Milestone 4.7).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Certificate {
    pub certificate_no: String,
    pub student_id: String,
    pub course_id: String,
    pub issue_date: NaiveDate,
    pub final_grade_score: u32,
}

/// Student Progress Tracking & Certification Generator (Milestone 4.7).
#[derive(Debug, Clone, Default)]
pub struct StudentProgressTracker {
    completed_lessons: HashSet<String>,
}

impl StudentProgressTracker {
    /// Creates a new progress tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks a lesson as completed by the student.
    pub fn complete_lesson(&mut self, lesson_id: &str) {
        self.completed_lessons.insert(lesson_id.to_string());
    }

    /// Computes percentage progress: $\frac{\text{Completed}}{\text{Total}} \times 100$.
    #[must_use]
    pub fn calculate_progress_percentage(&self, course: &Course) -> u32 {
        let total = course.total_lesson_count();
        if total == 0 {
            return 100;
        }

        let mut completed_count = 0;
        for module in &course.modules {
            for lesson in &module.lessons {
                if self.completed_lessons.contains(&lesson.id) {
                    completed_count += 1;
                }
            }
        }

        ((completed_count as f64 / total as f64) * 100.0) as u32
    }

    /// Evaluates assessment and auto-issues certificate if 100% complete and passing grade achieved.
    pub fn evaluate_and_issue_certificate(
        &self,
        student_id: &str,
        course: &Course,
        assessment_score: u32,
        issue_date: NaiveDate,
    ) -> Result<Certificate, LmsError> {
        let progress = self.calculate_progress_percentage(course);
        if progress < 100 {
            return Err(LmsError::AssessmentFailed {
                score: progress,
                passing_threshold: 100,
            });
        }

        if assessment_score < course.passing_threshold {
            return Err(LmsError::AssessmentFailed {
                score: assessment_score,
                passing_threshold: course.passing_threshold,
            });
        }

        Ok(Certificate {
            certificate_no: format!(
                "CERT-{}-{}-{}",
                course.id,
                student_id,
                issue_date.format("%Y%m%d")
            ),
            student_id: student_id.to_string(),
            course_id: course.id.clone(),
            issue_date,
            final_grade_score: assessment_score,
        })
    }
}
