//! Wishlist & Saved-For-Later Engine (`erp-trade::wishlist`).
//!
//! Provides customer engagement & retention tools:
//! - Multi-wishlist support (Default, Birthday, Corporate registry)
//! - Public/Private shareable tokens for external gift registries
//! - Priority tags (Low, Medium, High) and personal notes
//! - Target price alerts tracking

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Wishlist operational errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WishlistError {
    #[error("Item '{0}' already exists in this wishlist")]
    ItemAlreadyPresent(String),
    #[error("Item '{0}' was not found in this wishlist")]
    ItemNotFound(String),
    #[error("Wishlist '{0}' was not found")]
    WishlistNotFound(String),
}

/// Item in a customer wishlist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WishlistItem {
    pub item_code: String,
    pub added_at: DateTime<Utc>,
    pub priority: u8, // 1 = Low, 2 = Medium, 3 = High
    pub desired_price: Option<Decimal>,
    pub notes: Option<String>,
}

impl WishlistItem {
    #[must_use]
    pub fn new(item_code: impl Into<String>) -> Self {
        Self {
            item_code: item_code.into(),
            added_at: Utc::now(),
            priority: 2, // Medium default
            desired_price: None,
            notes: None,
        }
    }

    #[must_use]
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.clamp(1, 3);
        self
    }

    #[must_use]
    pub fn with_desired_price(mut self, price: Decimal) -> Self {
        self.desired_price = Some(price);
        self
    }

    #[must_use]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Customer Wishlist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wishlist {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub is_public: bool,
    pub share_token: String,
    pub items: Vec<WishlistItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Wishlist {
    #[must_use]
    pub fn new(id: impl Into<String>, user_id: impl Into<String>, name: impl Into<String>) -> Self {
        let id = id.into();
        let user_id = user_id.into();
        let name = name.into();
        let now = Utc::now();

        // Deterministic public share token
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.update(user_id.as_bytes());
        hasher.update(name.as_bytes());
        let share_token = hex::encode(&hasher.finalize()[..16]);

        Self {
            id,
            user_id,
            name,
            is_public: false,
            share_token,
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Adds an item to the wishlist.
    pub fn add_item(&mut self, item: WishlistItem) -> Result<(), WishlistError> {
        if self
            .items
            .iter()
            .any(|i| i.item_code.eq_ignore_ascii_case(&item.item_code))
        {
            return Err(WishlistError::ItemAlreadyPresent(item.item_code));
        }
        self.items.push(item);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Removes an item from the wishlist.
    pub fn remove_item(&mut self, item_code: &str) -> Result<(), WishlistError> {
        let original_len = self.items.len();
        self.items
            .retain(|i| !i.item_code.eq_ignore_ascii_case(item_code));
        if self.items.len() == original_len {
            return Err(WishlistError::ItemNotFound(item_code.to_string()));
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Toggles the public shareability of the wishlist.
    pub fn set_public(&mut self, public: bool) {
        self.is_public = public;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_wishlist_lifecycle() {
        let mut list = Wishlist::new("w_01", "user_100", "Tech Upgrades");
        assert!(!list.is_public);
        assert_eq!(list.items.len(), 0);

        let item = WishlistItem::new("GPU-4090")
            .with_priority(3)
            .with_desired_price(dec!(1499.00))
            .with_notes("Wait for Black Friday discount");

        assert!(list.add_item(item.clone()).is_ok());
        assert_eq!(list.items.len(), 1);

        // Duplicate item addition must fail
        assert_eq!(
            list.add_item(item),
            Err(WishlistError::ItemAlreadyPresent("GPU-4090".into()))
        );

        // Toggle public share
        list.set_public(true);
        assert!(list.is_public);
        assert!(!list.share_token.is_empty());

        // Remove item
        assert!(list.remove_item("GPU-4090").is_ok());
        assert_eq!(list.items.len(), 0);
    }
}
