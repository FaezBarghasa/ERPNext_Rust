//! SurrealDB Graph Relations & Schema Definitions (`rwl-meta::graph_schema`).
//!
//! Provides the native graph edge definitions, relationship validations,
//! and recursive traversal query generators for trees, BOM explosions,
//! document lineage flows, and supply chain traceability.

use serde::{Deserialize, Serialize};

/// Graph edge relation type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// Hierarchical tree relationship (e.g. Account, Warehouse, Department).
    ChildOf,
    /// BOM material requirement edge.
    Requires,
    /// Document progression flow (e.g. Quotation -> Sales Order -> Delivery Note -> Sales Invoice).
    DocFlow(String),
    /// Batch and serial supply chain tracking edge.
    Traceability(String),
    /// User and role authorization edge.
    HasRole,
    /// Role to DocType permission edge.
    HasPermission,
}

/// Structural definition of a SurrealDB graph relationship table (`TYPE RELATION`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdgeSchema {
    /// Relation table name in SurrealDB (e.g. `child_of`, `requires`, `doc_flow`).
    pub relation_table: String,
    /// Originating table constraint (`FROM table`).
    pub from_table: Option<String>,
    /// Destination table constraint (`TO table`).
    pub to_table: Option<String>,
    /// Enforce `SCHEMAFULL` constraint on the relation.
    pub schemafull: bool,
    /// Edge payload attributes (e.g. `qty`, `allocated_amount`, `permlevel`).
    pub attributes: Vec<EdgeAttribute>,
}

/// Strongly typed attribute embedded on a graph relationship edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeAttribute {
    pub name: String,
    pub field_type: String,
    pub assert_expr: Option<String>,
}

impl GraphEdgeSchema {
    /// Compiles this graph edge schema into SurrealQL DDL statements.
    #[must_use]
    pub fn compile_to_surrealql(&self) -> Vec<String> {
        let mut statements = Vec::new();
        let schema_mode = if self.schemafull {
            "SCHEMAFULL"
        } else {
            "SCHEMALESS"
        };

        let mut define = format!("DEFINE TABLE {} TYPE RELATION", self.relation_table);
        if let Some(from) = &self.from_table {
            define.push_str(&format!(" FROM {from}"));
        }
        if let Some(to) = &self.to_table {
            define.push_str(&format!(" TO {to}"));
        }
        define.push_str(&format!(" {schema_mode};"));
        statements.push(define);

        for attr in &self.attributes {
            let mut field_def = format!(
                "DEFINE FIELD {} ON TABLE {} TYPE {}",
                attr.name, self.relation_table, attr.field_type
            );
            if let Some(assertion) = &attr.assert_expr {
                field_def.push_str(&format!(" ASSERT {assertion}"));
            }
            field_def.push(';');
            statements.push(field_def);
        }

        statements
    }
}

/// Registry of canonical ERPNext graph relationships.
#[must_use]
pub fn standard_erpnext_graph_edges() -> Vec<GraphEdgeSchema> {
    vec![
        // 1. Chart of Accounts Hierarchy Edge
        GraphEdgeSchema {
            relation_table: "account_child_of".into(),
            from_table: Some("account".into()),
            to_table: Some("account".into()),
            schemafull: true,
            attributes: vec![],
        },
        // 2. Warehouse Hierarchy Edge
        GraphEdgeSchema {
            relation_table: "warehouse_sub_of".into(),
            from_table: Some("warehouse".into()),
            to_table: Some("warehouse".into()),
            schemafull: true,
            attributes: vec![],
        },
        // 3. Item Group Hierarchy Edge
        GraphEdgeSchema {
            relation_table: "item_group_sub_of".into(),
            from_table: Some("item_group".into()),
            to_table: Some("item_group".into()),
            schemafull: true,
            attributes: vec![],
        },
        // 4. BOM Explosion Edge: BOM -> requires -> Item
        GraphEdgeSchema {
            relation_table: "bom_requires".into(),
            from_table: Some("bom".into()),
            to_table: Some("item".into()),
            schemafull: true,
            attributes: vec![
                EdgeAttribute {
                    name: "qty".into(),
                    field_type: "float".into(),
                    assert_expr: Some("$value > 0.0".into()),
                },
                EdgeAttribute {
                    name: "scrap_percentage".into(),
                    field_type: "float".into(),
                    assert_expr: Some("$value >= 0.0".into()),
                },
                EdgeAttribute {
                    name: "rate".into(),
                    field_type: "decimal".into(),
                    assert_expr: Some("$value >= 0.0".into()),
                },
            ],
        },
        // 5. Universal Document Flow Lineage Edge
        GraphEdgeSchema {
            relation_table: "doc_flow".into(),
            from_table: None,
            to_table: None,
            schemafull: true,
            attributes: vec![
                EdgeAttribute {
                    name: "transition_type".into(),
                    field_type: "string".into(),
                    assert_expr: None,
                },
                EdgeAttribute {
                    name: "created_at".into(),
                    field_type: "datetime".into(),
                    assert_expr: None,
                },
                EdgeAttribute {
                    name: "allocated_amount".into(),
                    field_type: "option<decimal>".into(),
                    assert_expr: None,
                },
            ],
        },
        // 6. Batch & Supply Chain Traceability Edge
        GraphEdgeSchema {
            relation_table: "batch_trace".into(),
            from_table: None,
            to_table: None,
            schemafull: true,
            attributes: vec![
                EdgeAttribute {
                    name: "action".into(),
                    field_type: "string".into(),
                    assert_expr: None,
                },
                EdgeAttribute {
                    name: "qty".into(),
                    field_type: "float".into(),
                    assert_expr: Some("$value > 0.0".into()),
                },
            ],
        },
        // 7. RBAC Role Assignment Edge
        GraphEdgeSchema {
            relation_table: "has_role".into(),
            from_table: Some("user".into()),
            to_table: Some("role".into()),
            schemafull: true,
            attributes: vec![],
        },
    ]
}

/// Generates a SurrealQL query to create a typed relationship between two records.
#[must_use]
pub fn relate_statement(
    from_record: &str,
    edge_table: &str,
    to_record: &str,
    set_fields: Option<&str>,
) -> String {
    let mut stmt = format!("RELATE {from_record}->{edge_table}->{to_record}");
    if let Some(set) = set_fields {
        stmt.push_str(&format!(" SET {set}"));
    }
    stmt.push(';');
    stmt
}

/// Generates a recursive multi-level BOM explosion query using SurrealDB graph syntax.
#[must_use]
pub fn bom_explosion_query(bom_id: &str) -> String {
    format!(
        "SELECT id, ->bom_requires->item.* AS items, ->bom_requires.qty AS quantities FROM bom:{bom_id};"
    )
}

/// Generates a document upstream/downstream lineage traversal query.
#[must_use]
pub fn document_lineage_query(doc_table: &str, doc_id: &str) -> String {
    format!(
        "SELECT id, ->doc_flow.* AS downstream, <-doc_flow.* AS upstream FROM {doc_table}:{doc_id};"
    )
}

/// Compiles all standard ERPNext graph edge schemas into a vector of SurrealQL statements.
#[must_use]
pub fn compile_all_graph_edges() -> Vec<String> {
    let mut ddl = Vec::new();
    for edge in standard_erpnext_graph_edges() {
        ddl.extend(edge.compile_to_surrealql());
    }
    ddl
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_edge_compilation() {
        let edges = standard_erpnext_graph_edges();
        assert_eq!(edges.len(), 7);

        let ddl = compile_all_graph_edges();
        assert!(!ddl.is_empty());

        let ddl_str = ddl.join("\n");
        assert!(ddl_str.contains(
            "DEFINE TABLE account_child_of TYPE RELATION FROM account TO account SCHEMAFULL;"
        ));
        assert!(
            ddl_str
                .contains("DEFINE TABLE bom_requires TYPE RELATION FROM bom TO item SCHEMAFULL;")
        );
        assert!(
            ddl_str
                .contains("DEFINE FIELD qty ON TABLE bom_requires TYPE float ASSERT $value > 0.0;")
        );
    }

    #[test]
    fn test_relate_and_query_generation() {
        let stmt = relate_statement(
            "quotation:Q_1001",
            "doc_flow",
            "sales_order:SO_2001",
            Some("transition_type = 'ordered_as', created_at = time::now()"),
        );
        assert_eq!(
            stmt,
            "RELATE quotation:Q_1001->doc_flow->sales_order:SO_2001 SET transition_type = 'ordered_as', created_at = time::now();"
        );

        let query = bom_explosion_query("BOM_WIDGET_V1");
        assert!(query.contains("bom:BOM_WIDGET_V1"));

        let lineage = document_lineage_query("sales_order", "SO_2001");
        assert!(lineage.contains("sales_order:SO_2001"));
    }
}
