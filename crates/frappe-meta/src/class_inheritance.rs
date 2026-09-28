//! Class Inheritance & Override Engine (`frappe-meta::class_inheritance`).
//!
//! Validates DocType class overrides and extensions, rejecting orphan overrides
//! and cyclic inheritance chains at initialization time.

use crate::schema::{DocTypeSchema, SchemaError};
use compact_str::CompactString;
use std::collections::{HashMap, HashSet};

/// Registry and validator for DocType class inheritance and overrides.
#[derive(Debug, Clone, Default)]
pub struct ClassInheritanceRegistry {
    schemas: HashMap<CompactString, DocTypeSchema>,
    /// Map of overridden DocType -> Override implementing DocType
    overrides: HashMap<CompactString, CompactString>,
}

impl ClassInheritanceRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a base or extending DocType schema.
    pub fn register_schema(&mut self, schema: DocTypeSchema) -> Result<(), SchemaError> {
        schema.validate()?;
        let name = CompactString::new(&schema.name);

        if let Some(base_class) = &schema.extends_class {
            let base_name = CompactString::new(base_class);
            self.overrides.insert(base_name, name.clone());
        }

        self.schemas.insert(name, schema);
        Ok(())
    }

    /// Validates all registered class overrides to ensure no orphan or cyclic inheritance exists.
    pub fn validate_hierarchy(&self) -> Result<(), SchemaError> {
        for (target, schema) in &self.schemas {
            if let Some(base) = &schema.extends_class {
                let base_name = CompactString::new(base);
                if !self.schemas.contains_key(&base_name) {
                    return Err(SchemaError::BaseClassNotFound(
                        base.clone(),
                        target.to_string(),
                    ));
                }

                // Check cyclic inheritance
                let mut visited = HashSet::new();
                visited.insert(target.clone());
                let mut curr = base_name.clone();

                while let Some(parent_schema) = self.schemas.get(&curr) {
                    if visited.contains(&curr) {
                        return Err(SchemaError::CyclicInheritance(
                            target.to_string(),
                            curr.to_string(),
                        ));
                    }
                    visited.insert(curr.clone());
                    if let Some(next_base) = &parent_schema.extends_class {
                        curr = CompactString::new(next_base);
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Resolves the effective active DocType schema considering class overrides.
    #[must_use]
    pub fn resolve_effective_doctype<'a>(&'a self, doctype: &str) -> Option<&'a DocTypeSchema> {
        let key = CompactString::new(doctype);
        let effective_key = self.overrides.get(&key).unwrap_or(&key);
        self.schemas.get(effective_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_inheritance_resolution_and_cycle_detection() {
        let mut registry = ClassInheritanceRegistry::new();

        let base = DocTypeSchema {
            name: "Sales Invoice".into(),
            module: "Accounts".into(),
            is_single: false,
            is_submittable: true,
            is_child_table: false,
            is_tree: false,
            track_changes: true,
            quick_entry: false,
            allow_rename: false,
            allow_import: true,
            allow_auto_repeat: false,
            naming_rule: None,
            naming_rule_spec: None,
            virtual_child_tables: false,
            lazy_materialization: false,
            extends_class: None,
            fields: vec![],
            permissions: vec![],
        };

        let custom = DocTypeSchema {
            name: "Custom Sales Invoice".into(),
            module: "Custom Accounts".into(),
            is_single: false,
            is_submittable: true,
            is_child_table: false,
            is_tree: false,
            track_changes: true,
            quick_entry: false,
            allow_rename: false,
            allow_import: true,
            allow_auto_repeat: false,
            naming_rule: None,
            naming_rule_spec: None,
            virtual_child_tables: false,
            lazy_materialization: false,
            extends_class: Some("Sales Invoice".into()),
            fields: vec![],
            permissions: vec![],
        };

        registry.register_schema(base).unwrap();
        registry.register_schema(custom).unwrap();
        assert!(registry.validate_hierarchy().is_ok());

        let resolved = registry.resolve_effective_doctype("Sales Invoice").unwrap();
        assert_eq!(resolved.name, "Custom Sales Invoice");
    }
}
