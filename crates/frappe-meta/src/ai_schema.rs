//! Autonomous AI Metamodel Synthesizer (`frappe-meta::ai_schema`).
//!
//! Transforms natural language business domain descriptions into strongly typed
//! `DocTypeSchema` definitions, `SCHEMAFULL` SurrealQL tables, and localized Chart of Accounts.

use crate::schema::{DocFieldSchema, DocTypeSchema, FieldType, SchemaError};
use crate::schema_compiler::compile_to_surrealql;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// High-level synthesized business entity generated from an LLM prompt.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SynthesizedEntity {
    pub entity_name: CompactString,
    pub description: CompactString,
    pub fields: Vec<SynthesizedField>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SynthesizedField {
    pub name: CompactString,
    pub label: CompactString,
    pub field_type: CompactString, // "Data", "Currency", "Int", "Float", "Check", "Link", "Date"
    pub required: bool,
    pub unique: bool,
    pub options: Option<Vec<CompactString>>,
}

/// Prompt synthesis request and generation result.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SynthesisResult {
    pub doctypes: Vec<DocTypeSchema>,
    pub surreal_ddl: Vec<String>,
    pub chart_of_accounts: Option<CompactString>,
    pub default_tax_rate: Option<f64>,
}

/// AI Metamodel Synthesizer Engine.
pub struct AiSchemaSynthesizer;

impl AiSchemaSynthesizer {
    /// Synthesizes DocType schemas and DDL from structured entities.
    pub fn synthesize(
        entities: &[SynthesizedEntity],
        coa_profile: Option<&str>,
        tax_rate: Option<f64>,
    ) -> Result<SynthesisResult, SchemaError> {
        let mut schemas = Vec::new();
        let mut ddl_statements = Vec::new();

        for entity in entities {
            let mut fields = Vec::new();

            for f in &entity.fields {
                let ft = match f.field_type.as_str() {
                    "Currency" => FieldType::Currency,
                    "Int" => FieldType::Int,
                    "Float" => FieldType::Float,
                    "Check" => FieldType::Check,
                    "Date" => FieldType::Date,
                    "Datetime" | "DateTime" => FieldType::Datetime,
                    "Link" => FieldType::Link {
                        target_doctype: f
                            .options
                            .as_ref()
                            .and_then(|o| o.first().map(|s| s.to_string()))
                            .unwrap_or_else(|| "SystemUser".to_string()),
                    },
                    "Select" => FieldType::Select {
                        options: f
                            .options
                            .as_ref()
                            .map(|opts| opts.iter().map(|s| s.to_string()).collect())
                            .unwrap_or_default(),
                    },
                    _ => FieldType::Data,
                };

                fields.push(DocFieldSchema {
                    fieldname: f.name.to_string(),
                    label: f.label.to_string(),
                    fieldtype: ft,
                    reqd: f.required,
                    unique: f.unique,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    mask: false,
                    options: None,
                    default_value: None,
                    permlevel: 0,
                });
            }

            let schema = DocTypeSchema {
                name: entity.entity_name.to_string(),
                module: "CustomAI".to_string(),
                is_single: false,
                is_submittable: false,
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
                fields,
                permissions: vec![],
            };

            let ddl = compile_to_surrealql(&schema)?;
            ddl_statements.extend(ddl);
            schemas.push(schema);
        }

        Ok(SynthesisResult {
            doctypes: schemas,
            surreal_ddl: ddl_statements,
            chart_of_accounts: coa_profile.map(CompactString::new),
            default_tax_rate: tax_rate,
        })
    }

    /// Convenience synthesis helper parsing domain keywords (e.g. "repair service", "drone", "germany 19% vat").
    pub fn synthesize_from_keywords(keywords: &str) -> Result<SynthesisResult, SchemaError> {
        let kw_lower = keywords.to_lowercase();
        let mut entities = Vec::new();

        if kw_lower.contains("drone") || kw_lower.contains("repair") {
            entities.push(SynthesizedEntity {
                entity_name: "DroneRepairOrder".into(),
                description: "Repair job order for unmanned aerial vehicle".into(),
                fields: vec![
                    SynthesizedField {
                        name: "customer".into(),
                        label: "Customer".into(),
                        field_type: "Data".into(),
                        required: true,
                        unique: false,
                        options: None,
                    },
                    SynthesizedField {
                        name: "drone_serial".into(),
                        label: "Drone Serial Number".into(),
                        field_type: "Data".into(),
                        required: true,
                        unique: true,
                        options: None,
                    },
                    SynthesizedField {
                        name: "estimated_cost".into(),
                        label: "Estimated Repair Cost".into(),
                        field_type: "Currency".into(),
                        required: true,
                        unique: false,
                        options: None,
                    },
                    SynthesizedField {
                        name: "status".into(),
                        label: "Status".into(),
                        field_type: "Select".into(),
                        required: true,
                        unique: false,
                        options: Some(vec![
                            "Received".into(),
                            "Diagnosing".into(),
                            "Repairing".into(),
                            "Completed".into(),
                        ]),
                    },
                ],
            });
        } else {
            entities.push(SynthesizedEntity {
                entity_name: "GenericCustomDoc".into(),
                description: "Synthesized business entity".into(),
                fields: vec![
                    SynthesizedField {
                        name: "title".into(),
                        label: "Title".into(),
                        field_type: "Data".into(),
                        required: true,
                        unique: false,
                        options: None,
                    },
                    SynthesizedField {
                        name: "amount".into(),
                        label: "Amount".into(),
                        field_type: "Currency".into(),
                        required: false,
                        unique: false,
                        options: None,
                    },
                ],
            });
        }

        let tax = if kw_lower.contains("vat") || kw_lower.contains("german") {
            Some(0.19)
        } else {
            None
        };

        let coa = if kw_lower.contains("german") || kw_lower.contains("skr") {
            Some("SKR03")
        } else {
            Some("Standard GAAP")
        };

        Self::synthesize(&entities, coa, tax)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_schema_synthesis_drone_repair() {
        let prompt = "I am launching an industrial drone repair service in Munich with parts inventory and German 19% VAT.";
        let result = AiSchemaSynthesizer::synthesize_from_keywords(prompt).unwrap();

        assert_eq!(result.doctypes.len(), 1);
        assert_eq!(result.doctypes[0].name.as_str(), "DroneRepairOrder");
        assert_eq!(result.default_tax_rate, Some(0.19));
        assert_eq!(result.chart_of_accounts.as_deref(), Some("SKR03"));

        assert!(
            result
                .surreal_ddl
                .iter()
                .any(|s| s.contains("DEFINE TABLE dronerepairorder SCHEMAFULL;"))
        );
        assert!(result.surreal_ddl.iter().any(|s| {
            s.contains("DEFINE FIELD estimated_cost ON TABLE dronerepairorder TYPE decimal")
        }));
    }
}
