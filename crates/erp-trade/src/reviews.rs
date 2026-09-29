//! Product Reviews, Ratings & Moderation Engine (`erp-trade::reviews`).
//!
//! Provides comprehensive review & rating management matching WooCommerce and Odoo:
//! - 1 to 5 star ratings with verified purchase verification
//! - Moderation workflows: Pending, Approved, Rejected, Flagged
//! - Community helpfulness voting (upvotes/downvotes)
//! - Aggregated rating metrics & distribution breakdown calculation
//! - Official merchant replies

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Review operational and validation errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReviewError {
    #[error("Rating {0} is invalid; must be between 1 and 5 stars")]
    InvalidRating(u8),
    #[error("Review title cannot be empty")]
    EmptyTitle,
    #[error("Review content cannot be empty (minimum 10 characters)")]
    ContentTooShort,
    #[error("Review '{0}' was not found")]
    NotFound(String),
    #[error("User has already submitted a review for item '{0}'")]
    DuplicateReview(String),
}

/// Moderation status for a review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    Flagged,
}

/// Product Review model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductReview {
    pub id: String,
    pub item_code: String,
    pub user_id: String,
    pub user_display_name: String,
    pub rating: u8,
    pub title: String,
    pub content: String,
    pub verified_purchase: bool,
    pub status: ReviewStatus,
    pub helpful_votes: u32,
    pub unhelpful_votes: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub admin_reply: Option<String>,
    pub admin_reply_at: Option<DateTime<Utc>>,
}

impl ProductReview {
    /// Creates a new review in `Pending` state for moderation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        item_code: impl Into<String>,
        user_id: impl Into<String>,
        user_display_name: impl Into<String>,
        rating: u8,
        title: impl Into<String>,
        content: impl Into<String>,
        verified_purchase: bool,
    ) -> Result<Self, ReviewError> {
        if !(1..=5).contains(&rating) {
            return Err(ReviewError::InvalidRating(rating));
        }

        let title = title.into().trim().to_string();
        if title.is_empty() {
            return Err(ReviewError::EmptyTitle);
        }

        let content = content.into().trim().to_string();
        if content.len() < 10 {
            return Err(ReviewError::ContentTooShort);
        }

        let now = Utc::now();
        Ok(Self {
            id: id.into(),
            item_code: item_code.into(),
            user_id: user_id.into(),
            user_display_name: user_display_name.into(),
            rating,
            title,
            content,
            verified_purchase,
            status: ReviewStatus::Pending,
            helpful_votes: 0,
            unhelpful_votes: 0,
            created_at: now,
            updated_at: now,
            admin_reply: None,
            admin_reply_at: None,
        })
    }

    /// Auto-approves the review if merchant policy permits verified reviews.
    #[must_use]
    pub fn auto_approve(mut self) -> Self {
        self.status = ReviewStatus::Approved;
        self
    }
}

/// Aggregated Review summary and star ratings distribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub item_code: String,
    pub total_reviews: usize,
    pub average_rating: f64,
    /// Count of [1-star, 2-star, 3-star, 4-star, 5-star] reviews.
    pub rating_distribution: [usize; 5],
    pub verified_purchases_count: usize,
    pub recommendation_percentage: f64, // % of reviews with 4 or 5 stars
}

/// Product Review Management Engine.
pub struct ReviewManager;

impl ReviewManager {
    /// Calculates aggregated metrics from a collection of approved reviews.
    #[must_use]
    pub fn calculate_summary(item_code: &str, reviews: &[ProductReview]) -> ReviewSummary {
        let approved: Vec<&ProductReview> = reviews
            .iter()
            .filter(|r| r.status == ReviewStatus::Approved)
            .collect();

        if approved.is_empty() {
            return ReviewSummary {
                item_code: item_code.to_string(),
                total_reviews: 0,
                average_rating: 0.0,
                rating_distribution: [0, 0, 0, 0, 0],
                verified_purchases_count: 0,
                recommendation_percentage: 0.0,
            };
        }

        let mut distribution = [0usize; 5];
        let mut total_score: u64 = 0;
        let mut verified_count = 0;
        let mut positive_count = 0;

        for r in &approved {
            if (1..=5).contains(&r.rating) {
                distribution[(r.rating - 1) as usize] += 1;
                total_score += r.rating as u64;
                if r.rating >= 4 {
                    positive_count += 1;
                }
            }
            if r.verified_purchase {
                verified_count += 1;
            }
        }

        let count = approved.len();
        let avg = total_score as f64 / count as f64;
        let rec_pct = (positive_count as f64 / count as f64) * 100.0;

        ReviewSummary {
            item_code: item_code.to_string(),
            total_reviews: count,
            average_rating: (avg * 100.0).round() / 100.0,
            rating_distribution: distribution,
            verified_purchases_count: verified_count,
            recommendation_percentage: (rec_pct * 10.0).round() / 10.0,
        }
    }

    /// Moderates a review by setting its state.
    pub fn moderate_review(review: &mut ProductReview, new_status: ReviewStatus) {
        review.status = new_status;
        review.updated_at = Utc::now();
    }

    /// Casts an upvote or downvote for helpfulness.
    pub fn vote_helpfulness(review: &mut ProductReview, helpful: bool) {
        if helpful {
            review.helpful_votes += 1;
        } else {
            review.unhelpful_votes += 1;
        }
        review.updated_at = Utc::now();
    }

    /// Attaches an official merchant response to the review.
    pub fn add_admin_reply(review: &mut ProductReview, reply: impl Into<String>) {
        review.admin_reply = Some(reply.into());
        review.admin_reply_at = Some(Utc::now());
        review.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_review_and_validations() {
        let review_err = ProductReview::new(
            "rev_1",
            "LAPTOP-01",
            "user_42",
            "John D.",
            6, // Invalid rating > 5
            "Great laptop",
            "This laptop runs Rust code at light speed!",
            true,
        );
        assert_eq!(review_err, Err(ReviewError::InvalidRating(6)));

        let valid = ProductReview::new(
            "rev_1",
            "LAPTOP-01",
            "user_42",
            "John D.",
            5,
            "Exceptional quality",
            "This laptop runs Rust code at light speed and has amazing battery life.",
            true,
        )
        .expect("valid review")
        .auto_approve();

        assert_eq!(valid.status, ReviewStatus::Approved);
        assert!(valid.verified_purchase);
    }

    #[test]
    fn test_review_summary_and_distribution_calculation() {
        let r1 = ProductReview::new(
            "r1",
            "ITEM-X",
            "u1",
            "Alice",
            5,
            "Amazing",
            "Super high performance, love it!",
            true,
        )
        .unwrap()
        .auto_approve();

        let r2 = ProductReview::new(
            "r2",
            "ITEM-X",
            "u2",
            "Bob",
            4,
            "Good value",
            "Solid product for the price point.",
            false,
        )
        .unwrap()
        .auto_approve();

        let r3 = ProductReview::new(
            "r3",
            "ITEM-X",
            "u3",
            "Charlie",
            2,
            "Disappointed",
            "Broke after three days of use.",
            true,
        )
        .unwrap()
        .auto_approve();

        let reviews = vec![r1, r2, r3];
        let summary = ReviewManager::calculate_summary("ITEM-X", &reviews);

        assert_eq!(summary.total_reviews, 3);
        // Average: (5 + 4 + 2) / 3 = 11 / 3 = 3.67
        assert_eq!(summary.average_rating, 3.67);
        // Distribution: [0 (1-star), 1 (2-star), 0 (3-star), 1 (4-star), 1 (5-star)]
        assert_eq!(summary.rating_distribution, [0, 1, 0, 1, 1]);
        assert_eq!(summary.verified_purchases_count, 2);
        // 2 out of 3 reviews are 4 or 5 stars -> 66.7%
        assert_eq!(summary.recommendation_percentage, 66.7);
    }
}
