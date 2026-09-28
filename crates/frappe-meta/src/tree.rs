//! Nested Set Model Tree Engine (`frappe-meta::tree`).
//!
//! Provides deterministic $O(1)$ ancestor/descendant queries and dynamic `lft`/`rgt`
//! reindexing for hierarchical DocTypes (e.g. Account, Cost Center, Warehouse, Department).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TreeError {
    #[error("Node '{0}' not found in tree")]
    NodeNotFound(String),
    #[error("Cyclic parentage detected: node '{0}' cannot be child of '{1}'")]
    CyclicParentage(String, String),
    #[error("Invalid root node specification")]
    InvalidRoot,
}

/// A node in a Nested Set hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TreeNode {
    pub name: CompactString,
    pub parent_node: Option<CompactString>,
    pub is_group: bool,
    pub lft: u64,
    pub rgt: u64,
    pub disabled: bool,
}

impl TreeNode {
    #[must_use]
    pub fn new(
        name: impl Into<CompactString>,
        parent: Option<CompactString>,
        is_group: bool,
    ) -> Self {
        Self {
            name: name.into(),
            parent_node: parent,
            is_group,
            lft: 0,
            rgt: 0,
            disabled: false,
        }
    }
}

/// In-memory Nested Set tree indexer and query manager.
#[derive(Debug, Clone, Default)]
pub struct NestedSetTree {
    pub doctype: CompactString,
    nodes: HashMap<CompactString, TreeNode>,
    children_map: HashMap<Option<CompactString>, Vec<CompactString>>,
}

impl NestedSetTree {
    #[must_use]
    pub fn new(doctype: impl Into<CompactString>) -> Self {
        Self {
            doctype: doctype.into(),
            nodes: HashMap::new(),
            children_map: HashMap::new(),
        }
    }

    /// Adds or updates a node in the tree.
    pub fn add_node(&mut self, node: TreeNode) {
        let name = node.name.clone();
        let parent = node.parent_node.clone();

        self.nodes.insert(name.clone(), node);
        self.children_map.entry(parent).or_default().push(name);
    }

    /// Re-indexes all `lft` and `rgt` boundaries using depth-first pre-order traversal.
    pub fn rebuild_tree(&mut self) -> Result<(), TreeError> {
        let mut counter = 1u64;
        let roots = self.children_map.get(&None).cloned().unwrap_or_default();

        for root in roots {
            self.rebuild_subtree(&root, &mut counter)?;
        }

        Ok(())
    }

    fn rebuild_subtree(
        &mut self,
        node_name: &CompactString,
        counter: &mut u64,
    ) -> Result<(), TreeError> {
        let lft = *counter;
        *counter += 1;

        let children = self
            .children_map
            .get(&Some(node_name.clone()))
            .cloned()
            .unwrap_or_default();
        for child in children {
            self.rebuild_subtree(&child, counter)?;
        }

        let rgt = *counter;
        *counter += 1;

        if let Some(node) = self.nodes.get_mut(node_name) {
            node.lft = lft;
            node.rgt = rgt;
        } else {
            return Err(TreeError::NodeNotFound(node_name.to_string()));
        }

        Ok(())
    }

    /// Returns true if `candidate_descendant` is a descendant of `ancestor`.
    #[must_use]
    pub fn is_descendant(&self, ancestor: &str, candidate_descendant: &str) -> bool {
        if let (Some(a), Some(d)) = (
            self.nodes.get(ancestor),
            self.nodes.get(candidate_descendant),
        ) {
            d.lft > a.lft && d.rgt < a.rgt
        } else {
            false
        }
    }

    /// Returns all descendant nodes of a given node (excluding disabled if requested).
    #[must_use]
    pub fn get_descendants(&self, ancestor: &str, suppress_disabled: bool) -> Vec<&TreeNode> {
        if let Some(a) = self.nodes.get(ancestor) {
            self.nodes
                .values()
                .filter(|n| n.lft > a.lft && n.rgt < a.rgt && (!suppress_disabled || !n.disabled))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns all ancestors of a given node from root down to parent.
    #[must_use]
    pub fn get_ancestors(&self, node: &str) -> Vec<&TreeNode> {
        if let Some(n) = self.nodes.get(node) {
            let mut ancestors: Vec<&TreeNode> = self
                .nodes
                .values()
                .filter(|a| a.lft < n.lft && a.rgt > n.rgt)
                .collect();
            ancestors.sort_by_key(|a| a.lft);
            ancestors
        } else {
            Vec::new()
        }
    }

    /// Retrieves a reference to a node.
    #[must_use]
    pub fn get_node(&self, name: &str) -> Option<&TreeNode> {
        self.nodes.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_set_rebuild_and_descendants() {
        let mut tree = NestedSetTree::new("Account");

        tree.add_node(TreeNode::new("Application of Funds (Assets)", None, true));
        tree.add_node(TreeNode::new(
            "Current Assets",
            Some("Application of Funds (Assets)".into()),
            true,
        ));
        tree.add_node(TreeNode::new(
            "Bank Accounts",
            Some("Current Assets".into()),
            true,
        ));
        tree.add_node(TreeNode::new(
            "HDFC Bank",
            Some("Bank Accounts".into()),
            false,
        ));
        tree.add_node(TreeNode::new(
            "Cash in Hand",
            Some("Current Assets".into()),
            false,
        ));

        tree.rebuild_tree().unwrap();

        let root = tree.get_node("Application of Funds (Assets)").unwrap();
        let _hdfc = tree.get_node("HDFC Bank").unwrap();

        assert_eq!(root.lft, 1);
        assert_eq!(root.rgt, 10);
        assert!(tree.is_descendant("Current Assets", "HDFC Bank"));
        assert!(tree.is_descendant("Application of Funds (Assets)", "Cash in Hand"));
        assert!(!tree.is_descendant("Bank Accounts", "Cash in Hand"));

        let ancestors = tree.get_ancestors("HDFC Bank");
        assert_eq!(ancestors.len(), 3);
        assert_eq!(ancestors[0].name.as_str(), "Application of Funds (Assets)");
        assert_eq!(ancestors[1].name.as_str(), "Current Assets");
        assert_eq!(ancestors[2].name.as_str(), "Bank Accounts");
    }
}
