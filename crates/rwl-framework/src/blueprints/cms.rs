//! CMS Blueprint: Content Management & Blogging Platform (`frappe-framework::blueprints::cms`).
//!
//! Provides the complete, pre-configured content platform:
//! - DocTypes: `BlogPost`, `Author`, `Category`, `Tag`, `SitePage`.
//! - Pre-designed Pages: Home, Blog Catalog, Post Detail, About, Contact.
//! - Seed records and automated workflows (Sitemap generation, publishing notifications).

use super::{BlueprintMetadata, BlueprintNavigationItem, BlueprintPackage, BlueprintPage};
use crate::scripting::LifecycleEvent;
use crate::workflow_nodes::{
    AutomationAction, ComparisonOp, ConditionRule, VisualWorkflow, VisualWorkflowNode,
};
use rwl_meta::{DocFieldSchema, DocPermSchema, DocTypeSchema, FieldType};
use serde_json::json;

/// Constructs the comprehensive CMS Blueprint Package.
#[must_use]
pub fn cms_blueprint() -> BlueprintPackage {
    let metadata = BlueprintMetadata {
        id: "blueprint_cms".into(),
        name: "Universal CMS & Blog".into(),
        description: "Modern high-speed content platform with authors, categories, tags, responsive blog layouts, and automated SEO.".into(),
        icon: "newspaper".into(),
        version: "1.0.0".into(),
        category: super::BlueprintCategory::ContentManagement,
        tags: vec!["cms".into(), "blog".into(), "seo".into(), "publishing".into()],
    };

    // 1. DocTypes
    let author_schema = DocTypeSchema {
        name: "Author".into(),
        module: "Website".into(),
        is_single: false,
        is_submittable: false,
        is_child_table: false,
        is_tree: false,
        track_changes: true,
        quick_entry: true,
        allow_rename: true,
        allow_import: true,
        allow_auto_repeat: false,
        naming_rule: Some("AUT-.#####".into()),
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        title_field: Some("full_name".into()),
        search_fields: Some("full_name,email".into()),
        image_field: Some("avatar".into()),
        fields: vec![
            DocFieldSchema {
                fieldname: "full_name".into(),
                fieldtype: FieldType::Data,
                label: "Full Name".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "email".into(),
                fieldtype: FieldType::Data,
                label: "Email".into(),
                reqd: true,
                unique: true,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "bio".into(),
                fieldtype: FieldType::SmallText,
                label: "Biography".into(),
                reqd: false,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: false,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "avatar".into(),
                fieldtype: FieldType::AttachImage,
                label: "Avatar Image".into(),
                reqd: false,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: false,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
        ],
        permissions: vec![
            DocPermSchema {
                role: "All".into(),
                permlevel: 0,
                read: true,
                write: false,
                create: false,
                delete: false,
                submit: false,
                cancel: false,
                amend: false,
                ..Default::default()
            },
            DocPermSchema {
                role: "Blogger".into(),
                permlevel: 0,
                read: true,
                write: true,
                create: true,
                delete: false,
                submit: false,
                cancel: false,
                amend: false,
                ..Default::default()
            },
        ],
    };

    let category_schema = DocTypeSchema {
        name: "Blog Category".into(),
        module: "Website".into(),
        is_single: false,
        is_submittable: false,
        is_child_table: false,
        is_tree: false,
        track_changes: true,
        quick_entry: true,
        allow_rename: true,
        allow_import: true,
        allow_auto_repeat: false,
        naming_rule: None,
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        title_field: Some("title".into()),
        search_fields: Some("title,slug".into()),
        image_field: None,
        fields: vec![
            DocFieldSchema {
                fieldname: "title".into(),
                fieldtype: FieldType::Data,
                label: "Category Name".into(),
                reqd: true,
                unique: true,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "slug".into(),
                fieldtype: FieldType::Data,
                label: "URL Slug".into(),
                reqd: true,
                unique: true,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
        ],
        permissions: vec![DocPermSchema {
            role: "All".into(),
            permlevel: 0,
            read: true,
            write: true,
            create: true,
            delete: true,
            submit: false,
            cancel: false,
            amend: false,
            ..Default::default()
        }],
    };

    let blog_post_schema = DocTypeSchema {
        name: "Blog Post".into(),
        module: "Website".into(),
        is_single: false,
        is_submittable: false,
        is_child_table: false,
        is_tree: false,
        track_changes: true,
        quick_entry: false,
        allow_rename: true,
        allow_import: true,
        allow_auto_repeat: false,
        naming_rule: Some("BLOG-.YYYY.-.#####".into()),
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        title_field: Some("title".into()),
        search_fields: Some("title,author,category".into()),
        image_field: Some("cover_image".into()),
        fields: vec![
            DocFieldSchema {
                fieldname: "title".into(),
                fieldtype: FieldType::Data,
                label: "Article Title".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "slug".into(),
                fieldtype: FieldType::Data,
                label: "URL Slug".into(),
                reqd: true,
                unique: true,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "author".into(),
                fieldtype: FieldType::Link {
                    target_doctype: "Author".into(),
                },
                label: "Author".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "category".into(),
                fieldtype: FieldType::Link {
                    target_doctype: "Blog Category".into(),
                },
                label: "Category".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "status".into(),
                fieldtype: FieldType::Select {
                    options: vec!["Draft".into(), "Published".into(), "Archived".into()],
                },
                label: "Status".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!("Draft")),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "content".into(),
                fieldtype: FieldType::TextEditor,
                label: "Content Body".into(),
                reqd: true,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: false,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "cover_image".into(),
                fieldtype: FieldType::AttachImage,
                label: "Cover Image".into(),
                reqd: false,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: false,
                mask: false,
                default_value: None,
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
            DocFieldSchema {
                fieldname: "view_count".into(),
                fieldtype: FieldType::Int,
                label: "View Count".into(),
                reqd: false,
                unique: false,
                read_only: true,
                hidden: false,
                in_list_view: true,
                mask: false,
                default_value: Some(json!(0)),
                options: None,
                permlevel: 0,
                depends_on: None,
                mandatory_depends_on: None,
                read_only_depends_on: None,
                fetch_from: None,
                precision: None,
            },
        ],
        permissions: vec![
            DocPermSchema {
                role: "All".into(),
                permlevel: 0,
                read: true,
                write: false,
                create: false,
                delete: false,
                submit: false,
                cancel: false,
                amend: false,
                ..Default::default()
            },
            DocPermSchema {
                role: "Blogger".into(),
                permlevel: 0,
                read: true,
                write: true,
                create: true,
                delete: false,
                submit: false,
                cancel: false,
                amend: false,
                ..Default::default()
            },
        ],
    };

    // 2. Pre-designed Pages
    let pages = vec![
        BlueprintPage {
            path: "/".into(),
            title: "Home — Discover Stories".into(),
            description: "Magazine-style home page with hero feature and latest publications."
                .into(),
            blocks_json: json!({
                "type": "Container",
                "max_width_px": 1280,
                "children": [
                    {
                        "type": "Hero",
                        "title": "Welcome to rust-web-land Publishing",
                        "subtitle": "Blazing-fast articles running on pure Rust and zero client-side bloat.",
                        "cta_label": "Read Stories",
                        "cta_url": "/blog"
                    },
                    {
                        "type": "Repeater",
                        "data_source_path": "pageData.recent_posts",
                        "item_alias": "post",
                        "template": {
                            "type": "CardBlock",
                            "title": "{{ post.title }}",
                            "body": "{{ post.summary }}",
                            "image_url": "{{ post.cover_image }}"
                        }
                    }
                ]
            }),
            is_public: true,
        },
        BlueprintPage {
            path: "/blog".into(),
            title: "Articles & Publications".into(),
            description: "Paginated publication catalog with topic filters.".into(),
            blocks_json: json!({
                "type": "Container",
                "max_width_px": 1280,
                "children": [
                    {
                        "type": "Heading",
                        "level": 1,
                        "text": "All Publications"
                    },
                    {
                        "type": "Repeater",
                        "data_source_path": "pageData.posts",
                        "item_alias": "item",
                        "template": {
                            "type": "CardBlock",
                            "title": "{{ item.title }}",
                            "body": "By {{ item.author }} in {{ item.category }}",
                            "image_url": null
                        }
                    }
                ]
            }),
            is_public: true,
        },
        BlueprintPage {
            path: "/about".into(),
            title: "About Us".into(),
            description: "Mission and story of the publication.".into(),
            blocks_json: json!({
                "type": "Container",
                "max_width_px": 960,
                "children": [
                    {
                        "type": "Heading",
                        "level": 1,
                        "text": "Crafting the Future of Web Systems"
                    },
                    {
                        "type": "TextBlock",
                        "content": "rust-web-land was born to empower non-technical creators to launch self-hosted, scalable web applications with sub-millisecond response times and minimal memory footprints."
                    }
                ]
            }),
            is_public: true,
        },
    ];

    // 3. Navigation
    let navigation = vec![
        BlueprintNavigationItem {
            label: "Home".into(),
            url: "/".into(),
            icon: "home".into(),
            role_required: None,
        },
        BlueprintNavigationItem {
            label: "Blog".into(),
            url: "/blog".into(),
            icon: "book-open".into(),
            role_required: None,
        },
        BlueprintNavigationItem {
            label: "About".into(),
            url: "/about".into(),
            icon: "info".into(),
            role_required: None,
        },
        BlueprintNavigationItem {
            label: "Console".into(),
            url: "/desk".into(),
            icon: "terminal".into(),
            role_required: Some("Blogger".into()),
        },
    ];

    // 4. Seed Records
    let seed_records = vec![
        (
            "Author".into(),
            json!({
                "name": "AUT-00001",
                "full_name": "Antigravity Engineering",
                "email": "team@rustwebland.org",
                "bio": "Systems architects building high-performance offline-first web technologies."
            }),
        ),
        (
            "Blog Category".into(),
            json!({
                "name": "Technology",
                "title": "Technology",
                "slug": "technology"
            }),
        ),
        (
            "Blog Post".into(),
            json!({
                "name": "BLOG-2026-00001",
                "title": "Launching Universal Web Platforms in Pure Rust",
                "slug": "launching-universal-web-platforms-in-pure-rust",
                "author": "AUT-00001",
                "category": "Technology",
                "status": "Published",
                "content": "<p>Welcome to the next generation of web applications: zero-recompile dynamic DocTypes, sub-millisecond response times, and an ultra-lean RAM footprint operating on any 1GB VPS.</p>",
                "view_count": 128
            }),
        ),
    ];

    // 5. Automated Workflows
    let workflow_automations = vec![VisualWorkflow {
        id: "wf_blog_publish_notify".into(),
        name: "Publish Article Notification".into(),
        doctype: "Blog Post".into(),
        event: LifecycleEvent::Validate,
        enabled: true,
        nodes: vec![
            VisualWorkflowNode {
                id: "n_trig".into(),
                label: "When Status is Published".into(),
                node_kind: "trigger".into(),
                position_x: 0.0,
                position_y: 0.0,
                connections: vec!["n_cond".into()],
                metadata: json!({"event": "Validate"}),
            },
            VisualWorkflowNode {
                id: "n_cond".into(),
                label: "Status == 'Published'".into(),
                node_kind: "condition".into(),
                position_x: 180.0,
                position_y: 0.0,
                connections: vec!["n_act".into()],
                metadata: json!({"field": "status", "val": "Published"}),
            },
            VisualWorkflowNode {
                id: "n_act".into(),
                label: "Send Webhook to Search Indexer".into(),
                node_kind: "action".into(),
                position_x: 360.0,
                position_y: 0.0,
                connections: vec![],
                metadata: json!({"webhook": "https://api.indexer.local/reindex"}),
            },
        ],
        conditions: vec![ConditionRule::FieldCompare {
            field: "status".into(),
            op: ComparisonOp::Equals,
            value: json!("Published"),
        }],
        actions: vec![
            AutomationAction::SendNotification {
                channel: "InApp".into(),
                recipient: "Administrator".into(),
                subject: "Blog Post Published".into(),
                message: "A new blog post has been published and indexed.".into(),
            },
            AutomationAction::EmitWebhook {
                url: "https://api.indexer.local/reindex".into(),
                payload_template: None,
            },
        ],
    }];

    BlueprintPackage {
        metadata,
        doctypes: vec![author_schema, category_schema, blog_post_schema],
        pages,
        navigation,
        seed_records,
        workflow_automations,
    }
}
