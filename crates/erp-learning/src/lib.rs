pub mod academic;
pub mod lms;

pub use academic::{
    AcademicError, AcademicProgram, ApplicationStatus, AssessmentResult, FeeCategoryItem,
    FeeStructure, GradeInterval, GradingScale, ProgramEnrollment, StudentApplicant,
    StudentFeeSchedule, StudentMaster,
};
pub use lms::{Certificate, Course, CourseModule, Lesson, LmsError, StudentProgressTracker};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_lms_progress_and_certificate_issuance() {
        let course = Course {
            id: "RUST-101".into(),
            title: "Rust Systems Engineering".into(),
            description: "Deep dive into memory safety and systems".into(),
            passing_threshold: 80,
            modules: vec![
                CourseModule {
                    id: "MOD-1".into(),
                    title: "Ownership".into(),
                    lessons: vec![
                        Lesson {
                            id: "LES-1".into(),
                            title: "Move Semantics".into(),
                            content_markdown: "Markdown content".into(),
                            video_url: None,
                        },
                        Lesson {
                            id: "LES-2".into(),
                            title: "Borrowing".into(),
                            content_markdown: "Markdown content".into(),
                            video_url: None,
                        },
                    ],
                },
                CourseModule {
                    id: "MOD-2".into(),
                    title: "Lifetimes".into(),
                    lessons: vec![
                        Lesson {
                            id: "LES-3".into(),
                            title: "Explicit Lifetimes".into(),
                            content_markdown: "Markdown content".into(),
                            video_url: None,
                        },
                        Lesson {
                            id: "LES-4".into(),
                            title: "Subtyping".into(),
                            content_markdown: "Markdown content".into(),
                            video_url: None,
                        },
                    ],
                },
            ],
        };

        assert_eq!(course.total_lesson_count(), 4);

        let mut tracker = StudentProgressTracker::new();

        // Complete 3 out of 4 lessons -> 75%
        tracker.complete_lesson("LES-1");
        tracker.complete_lesson("LES-2");
        tracker.complete_lesson("LES-3");
        assert_eq!(tracker.calculate_progress_percentage(&course), 75);

        let date = NaiveDate::from_ymd_opt(2026, 9, 27).unwrap();

        // Attempt cert at 75% -> Fails
        let cert_early = tracker.evaluate_and_issue_certificate("STU-42", &course, 95, date);
        assert!(cert_early.is_err());

        // Complete 4th lesson -> 100%
        tracker.complete_lesson("LES-4");
        assert_eq!(tracker.calculate_progress_percentage(&course), 100);

        // Assessment score 70% (< 80% passing threshold) -> Fails
        let cert_failed_score = tracker.evaluate_and_issue_certificate("STU-42", &course, 70, date);
        assert!(matches!(
            cert_failed_score,
            Err(LmsError::AssessmentFailed {
                score: 70,
                passing_threshold: 80
            })
        ));

        // Assessment score 90% -> Certificate issued successfully
        let cert = tracker
            .evaluate_and_issue_certificate("STU-42", &course, 90, date)
            .expect("Certificate issuance failed");

        assert_eq!(cert.student_id, "STU-42");
        assert_eq!(cert.course_id, "RUST-101");
        assert_eq!(cert.final_grade_score, 90);
    }
}
