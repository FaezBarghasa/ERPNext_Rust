//! Hierarchical Categories, Tags & Multi-Taxonomy Engine (`erp-cms::taxonomy`).
//!
//! Provides WordPress and emdash parity for multi-taxonomy categorization:
//! - Hierarchical categories with cycle-free ancestor/descendant graph traversal
//! - Flat tagging systems and multi-faceted product brand taxonomies
//! - Automatic URL-safe slugification with collision deduplication
//! - Breadcrumb trail generation for SEO navigation

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Taxonomy operational errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TaxonomyError {
    #[error("Term '{0}' not found")]
    NotFound(String),
    #[error("Parent term '{0}' does not exist")]
    ParentNotFound(String),
    #[error("Circular parent relationship detected for term '{0}'")]
    CircularHierarchy(String),
    #[error("Slug '{0}' already exists in taxonomy '{1}'")]
    DuplicateSlug(String, String),
    #[error("Term name cannot be empty")]
    EmptyName,
}

/// Standard or custom taxonomy taxonomy type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaxonomyType {
    Category,
    Tag,
    ProductCategory,
    Brand,
    Custom(String),
}

impl TaxonomyType {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Category => "category",
            Self::Tag => "tag",
            Self::ProductCategory => "product_cat",
            Self::Brand => "brand",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// Individual taxonomy term / category / tag node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxonomyTerm {
    pub id: String,
    pub taxonomy: String,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
    pub count: usize,
    pub meta: HashMap<String, String>,
}

impl TaxonomyTerm {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        taxonomy: impl Into<String>,
        name: impl Into<String>,
        slug: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            taxonomy: taxonomy.into(),
            name: name.into(),
            slug: slug.into(),
            parent_id: None,
            description: None,
            count: 0,
            meta: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// In-memory Taxonomy Registry and Hierarchy Graph Resolver.
#[derive(Debug, Clone, Default)]
pub struct TaxonomyRegistry {
    terms: HashMap<String, TaxonomyTerm>,
}

impl TaxonomyRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            terms: HashMap::new(),
        }
    }

    /// Generates a clean URL-safe slug from a human-readable title.
    #[must_use]
    pub fn slugify(text: &str) -> String {
        let mut slug = String::with_capacity(text.len());
        let mut prev_dash = false;

        for ch in text.trim().chars() {
            if ch.is_alphanumeric() {
                slug.push(ch.to_ascii_lowercase());
                prev_dash = false;
            } else if (ch.is_whitespace() || ch == '-' || ch == '_')
                && !prev_dash
                && !slug.is_empty()
            {
                slug.push('-');
                prev_dash = true;
            }
        }

        if slug.ends_with('-') {
            slug.pop();
        }

        if slug.is_empty() {
            "item".to_string()
        } else {
            slug
        }
    }

    /// Adds or registers a new taxonomy term, validating against duplicates and cycles.
    pub fn register_term(&mut self, term: TaxonomyTerm) -> Result<(), TaxonomyError> {
        if term.name.trim().is_empty() {
            return Err(TaxonomyError::EmptyName);
        }

        // Validate parent exists if specified
        if let Some(ref p_id) = term.parent_id {
            if p_id == &term.id {
                return Err(TaxonomyError::CircularHierarchy(term.id));
            }
            if !self.terms.contains_key(p_id) {
                return Err(TaxonomyError::ParentNotFound(p_id.clone()));
            }

            // Check if adding this parent introduces a loop
            let mut current = p_id.clone();
            let mut visited = HashSet::new();
            visited.insert(term.id.clone());
            while let Some(parent_term) = self.terms.get(&current) {
                if !visited.insert(current.clone()) {
                    return Err(TaxonomyError::CircularHierarchy(term.id));
                }
                if let Some(ref next_p) = parent_term.parent_id {
                    current = next_p.clone();
                } else {
                    break;
                }
            }
        }

        // Check duplicate slug within the same taxonomy
        for existing in self.terms.values() {
            if existing.taxonomy == term.taxonomy
                && existing.slug == term.slug
                && existing.id != term.id
            {
                return Err(TaxonomyError::DuplicateSlug(term.slug, term.taxonomy));
            }
        }

        self.terms.insert(term.id.clone(), term);
        Ok(())
    }

    /// Gets a term by ID.
    #[must_use]
    pub fn get_term(&self, id: &str) -> Option<&TaxonomyTerm> {
        self.terms.get(id)
    }

    /// Resolves all ancestors of a term up to the root (root first).
    #[must_use]
    pub fn get_ancestors(&self, term_id: &str) -> Vec<&TaxonomyTerm> {
        let mut ancestors = Vec::new();
        let mut current_id = term_id;

        while let Some(term) = self.terms.get(current_id) {
            let maybe_parent = term
                .parent_id
                .as_deref()
                .and_then(|p_id| self.terms.get(p_id));
            if let Some(parent) = maybe_parent {
                ancestors.push(parent);
                current_id = &parent.id;
                continue;
            }
            break;
        }

        ancestors.reverse();
        ancestors
    }

    /// Resolves all descendants of a term down the tree.
    #[must_use]
    pub fn get_descendants(&self, term_id: &str) -> Vec<&TaxonomyTerm> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![term_id];

        while let Some(current) = to_visit.pop() {
            for term in self.terms.values() {
                if term.parent_id.as_deref() == Some(current) {
                    descendants.push(term);
                    to_visit.push(&term.id);
                }
            }
        }

        descendants
    }

    /// Generates breadcrumb trail pairs: `[(name, slug), ...]` from root down to this term.
    #[must_use]
    pub fn get_breadcrumbs(&self, term_id: &str) -> Vec<(String, String)> {
        let mut crumbs = Vec::new();
        for ancestor in self.get_ancestors(term_id) {
            crumbs.push((ancestor.name.clone(), ancestor.slug.clone()));
        }
        if let Some(term) = self.get_term(term_id) {
            crumbs.push((term.name.clone(), term.slug.clone()));
        }
        crumbs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugification() {
        assert_eq!(
            TaxonomyRegistry::slugify("Electronics & Smart Gadgets!"),
            "electronics-smart-gadgets"
        );
        assert_eq!(
            TaxonomyRegistry::slugify("  Laptops / High-End PC  "),
            "laptops-high-end-pc"
        );
        assert_eq!(TaxonomyRegistry::slugify(""), "item");
    }

    #[test]
    fn test_hierarchical_taxonomy_traversal_and_breadcrumbs() {
        let mut registry = TaxonomyRegistry::new();

        let root = TaxonomyTerm::new("cat_1", "product_cat", "Electronics", "electronics");
        let sub = TaxonomyTerm::new("cat_2", "product_cat", "Computers", "computers")
            .with_parent("cat_1");
        let leaf = TaxonomyTerm::new("cat_3", "product_cat", "Gaming Laptops", "gaming-laptops")
            .with_parent("cat_2");

        assert!(registry.register_term(root).is_ok());
        assert!(registry.register_term(sub).is_ok());
        assert!(registry.register_term(leaf).is_ok());

        // Ancestors of cat_3 -> [cat_1, cat_2]
        let ancestors = registry.get_ancestors("cat_3");
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0].id, "cat_1");
        assert_eq!(ancestors[1].id, "cat_2");

        // Descendants of cat_1 -> [cat_2, cat_3]
        let descendants = registry.get_descendants("cat_1");
        assert_eq!(descendants.len(), 2);

        // Breadcrumbs for cat_3: Electronics -> Computers -> Gaming Laptops
        let crumbs = registry.get_breadcrumbs("cat_3");
        assert_eq!(crumbs.len(), 3);
        assert_eq!(crumbs[0].0, "Electronics");
        assert_eq!(crumbs[1].0, "Computers");
        assert_eq!(crumbs[2].0, "Gaming Laptops");
    }

    #[test]
    fn test_circular_hierarchy_prevention() {
        let mut registry = TaxonomyRegistry::new();
        let t1 = TaxonomyTerm::new("t1", "category", "Node 1", "node-1");
        assert!(registry.register_term(t1).is_ok());

        let t2 = TaxonomyTerm::new("t2", "category", "Node 2", "node-2").with_parent("t1");
        assert!(registry.register_term(t2).is_ok());

        // Self-parenting must fail
        let t3 = TaxonomyTerm::new("t3", "category", "Node 3", "node-3").with_parent("t3");
        assert_eq!(
            registry.register_term(t3),
            Err(TaxonomyError::CircularHierarchy("t3".into()))
        );
    }
}
