//! Social, Timeline & Version Engine (`frappe-framework::social`).
//!
//! Implements ToDo/Assignment tracking, document comments, activity timeline events,
//! document tagging, likes, and field-level Version diff history.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Type of comment or timeline interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommentType {
    Comment,
    Workflow,
    Created,
    Submitted,
    Cancelled,
    Assigned,
    AssignmentCompleted,
    Info,
    Edit,
}

/// Comment or Timeline Activity record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommentRecord {
    pub id: String,
    pub doctype: String,
    pub docname: String,
    pub comment_type: CommentType,
    pub comment_by: String,
    pub content: String,
    pub created_at: String,
}

/// Assignment (ToDo) record linking a document to an assignee.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssignmentRecord {
    pub id: String,
    pub doctype: String,
    pub docname: String,
    pub allocated_to: String,
    pub assigned_by: String,
    pub description: Option<String>,
    pub priority: String, // "Low", "Medium", "High", "Urgent"
    pub status: String,   // "Open", "Closed", "Cancelled"
    pub assigned_at: String,
}

/// Single field change delta between versions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldDelta {
    pub fieldname: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
}

/// Version Record storing historical diff of document mutations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionRecord {
    pub id: String,
    pub ref_doctype: String,
    pub docname: String,
    pub modified_by: String,
    pub timestamp: String,
    pub changed_fields: Vec<FieldDelta>,
}

/// Social and Collaboration Manager.
#[derive(Default)]
pub struct SocialEngine {
    comments: Vec<CommentRecord>,
    assignments: Vec<AssignmentRecord>,
    tags: Vec<(String, String, String)>, // (doctype, docname, tag)
    likes: HashSet<(String, String, String)>, // (doctype, docname, user)
    versions: Vec<VersionRecord>,
}

impl SocialEngine {
    /// Creates a new social engine.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a new comment to a document timeline.
    pub fn add_comment(
        &mut self,
        doctype: &str,
        docname: &str,
        comment_by: &str,
        content: &str,
        comment_type: CommentType,
    ) -> CommentRecord {
        let record = CommentRecord {
            id: format!("COM-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            doctype: doctype.to_string(),
            docname: docname.to_string(),
            comment_type,
            comment_by: comment_by.to_string(),
            content: content.to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        self.comments.push(record.clone());
        record
    }

    /// Fetches chronological timeline events for a document.
    #[must_use]
    pub fn get_timeline(&self, doctype: &str, docname: &str) -> Vec<CommentRecord> {
        self.comments
            .iter()
            .filter(|c| c.doctype == doctype && c.docname == docname)
            .cloned()
            .collect()
    }

    /// Creates an assignment (ToDo) for a user.
    pub fn assign_to(
        &mut self,
        doctype: &str,
        docname: &str,
        allocated_to: &str,
        assigned_by: &str,
        description: Option<&str>,
        priority: &str,
    ) -> AssignmentRecord {
        let record = AssignmentRecord {
            id: format!("TODO-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            doctype: doctype.to_string(),
            docname: docname.to_string(),
            allocated_to: allocated_to.to_string(),
            assigned_by: assigned_by.to_string(),
            description: description.map(str::to_string),
            priority: priority.to_string(),
            status: "Open".to_string(),
            assigned_at: Utc::now().to_rfc3339(),
        };
        self.assignments.push(record.clone());

        // Emit timeline comment
        self.add_comment(
            doctype,
            docname,
            assigned_by,
            &format!("Assigned document to {allocated_to}"),
            CommentType::Assigned,
        );

        record
    }

    /// Closes an open assignment.
    pub fn complete_assignment(
        &mut self,
        doctype: &str,
        docname: &str,
        user: &str,
    ) -> Result<(), &'static str> {
        let assignment = self
            .assignments
            .iter_mut()
            .find(|a| a.doctype == doctype && a.docname == docname && a.allocated_to == user);

        if let Some(a) = assignment {
            a.status = "Closed".to_string();
            self.add_comment(
                doctype,
                docname,
                user,
                &format!("Assignment completed by {user}"),
                CommentType::AssignmentCompleted,
            );
            Ok(())
        } else {
            Err("No open assignment found for this user")
        }
    }

    /// Tags a document with a keyword.
    pub fn add_tag(&mut self, doctype: &str, docname: &str, tag: &str) {
        let entry = (doctype.to_string(), docname.to_string(), tag.to_string());
        if !self.tags.contains(&entry) {
            self.tags.push(entry);
        }
    }

    /// Removes a tag from a document.
    pub fn remove_tag(&mut self, doctype: &str, docname: &str, tag: &str) {
        self.tags
            .retain(|(dt, dn, t)| !(dt == doctype && dn == docname && t == tag));
    }

    /// Fetches all tags for a document.
    #[must_use]
    pub fn get_tags(&self, doctype: &str, docname: &str) -> Vec<String> {
        self.tags
            .iter()
            .filter(|(dt, dn, _)| dt == doctype && dn == docname)
            .map(|(_, _, t)| t.clone())
            .collect()
    }

    /// Toggles like status on a document for a given user.
    pub fn toggle_like(&mut self, doctype: &str, docname: &str, user: &str) -> bool {
        let key = (doctype.to_string(), docname.to_string(), user.to_string());
        if self.likes.contains(&key) {
            self.likes.remove(&key);
            false
        } else {
            self.likes.insert(key);
            true
        }
    }

    /// Calculates JSON diff and records a new version snapshot.
    pub fn record_version(
        &mut self,
        doctype: &str,
        docname: &str,
        modified_by: &str,
        old_data: &serde_json::Value,
        new_data: &serde_json::Value,
    ) -> Option<VersionRecord> {
        let mut deltas = Vec::new();

        if let (Some(old_map), Some(new_map)) = (old_data.as_object(), new_data.as_object()) {
            for (key, new_val) in new_map {
                // Ignore system volatile fields
                if key == "modified" || key == "creation" || key == "_comments" {
                    continue;
                }
                let old_val = old_map.get(key).unwrap_or(&serde_json::Value::Null);
                if old_val != new_val {
                    deltas.push(FieldDelta {
                        fieldname: key.clone(),
                        old_value: old_val.clone(),
                        new_value: new_val.clone(),
                    });
                }
            }
        }

        if deltas.is_empty() {
            return None;
        }

        let record = VersionRecord {
            id: format!("VER-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            ref_doctype: doctype.to_string(),
            docname: docname.to_string(),
            modified_by: modified_by.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            changed_fields: deltas,
        };

        self.versions.push(record.clone());
        Some(record)
    }

    /// Fetches all version changes for a document.
    #[must_use]
    pub fn get_versions(&self, doctype: &str, docname: &str) -> Vec<VersionRecord> {
        self.versions
            .iter()
            .filter(|v| v.ref_doctype == doctype && v.docname == docname)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comments_and_assignment_flow() {
        let mut social = SocialEngine::new();
        social.add_comment(
            "Sales Order",
            "SO-001",
            "analyst@corp.com",
            "Customer requested delivery priority bump",
            CommentType::Comment,
        );

        let timeline = social.get_timeline("Sales Order", "SO-001");
        assert_eq!(timeline.len(), 1);
        assert_eq!(
            timeline[0].content,
            "Customer requested delivery priority bump"
        );

        let assign = social.assign_to(
            "Sales Order",
            "SO-001",
            "logistics@corp.com",
            "analyst@corp.com",
            Some("Dispatch immediately"),
            "High",
        );
        assert_eq!(assign.status, "Open");

        let res = social.complete_assignment("Sales Order", "SO-001", "logistics@corp.com");
        assert!(res.is_ok());

        let updated_timeline = social.get_timeline("Sales Order", "SO-001");
        assert_eq!(updated_timeline.len(), 3); // initial comment + assigned + completed
    }

    #[test]
    fn test_version_diff_recording() {
        let mut social = SocialEngine::new();
        let old = serde_json::json!({ "status": "Draft", "grand_total": 100.0 });
        let new = serde_json::json!({ "status": "Draft", "grand_total": 150.0 });

        let version = social
            .record_version("Sales Order", "SO-001", "editor@corp.com", &old, &new)
            .expect("Version must be generated");

        assert_eq!(version.changed_fields.len(), 1);
        assert_eq!(version.changed_fields[0].fieldname, "grand_total");
        assert_eq!(
            version.changed_fields[0].old_value,
            serde_json::json!(100.0)
        );
        assert_eq!(
            version.changed_fields[0].new_value,
            serde_json::json!(150.0)
        );
    }
}
