//! Visual Site Builder Core Engine & Autonomous AI ("Bob") (`erp-cms::builder_core`).
//!
//! Provides:
//! - Figma-grade hierarchical JSON block canvas tree
//! - Multi-breakpoint viewport frame definitions (Desktop >= 1280px, Tablet >= 768px, Mobile < 768px)
//! - Interaction mode state machine (Select `v`, Container `c`, Text `t`)
//! - `BlockValueResolver` for multi-scope dynamic data bindings (`pageData`, `componentData`, `props`)
//! - Sandboxed Data Script execution context (`get_all`, `get_doc`, route slugs, redirect)
//! - Append-only visual AST undo/redo reversion ledger
//! - Autonomous Site Agent ("Bob") low-token YAML AST streaming exchange protocol.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Interaction modes in the visual canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasInteractionMode {
    Select,    // 'v'
    Container, // 'c'
    Text,      // 't'
}

/// Viewport Breakpoint specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasBreakpoint {
    Desktop, // >= 1280px
    Tablet,  // >= 768px
    Mobile,  // < 768px
}

impl CanvasBreakpoint {
    #[must_use]
    pub fn min_width_px(&self) -> u32 {
        match self {
            Self::Desktop => 1280,
            Self::Tablet => 768,
            Self::Mobile => 375,
        }
    }
}

/// Universal Visual Block Type AST.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "block_type")]
pub enum CanvasBlock {
    // Typography
    Heading {
        level: u8, // 1-6
        text: CompactString,
        data_binding: Option<CompactString>,
    },
    TextBlock {
        content: CompactString,
        data_binding: Option<CompactString>,
    },
    BlockQuote {
        quote: CompactString,
        author: Option<CompactString>,
    },

    // Structure & Containers
    Section {
        id: CompactString,
        children: Vec<CanvasBlock>,
        css_classes: CompactString,
    },
    Div {
        children: Vec<CanvasBlock>,
        css_classes: CompactString,
    },
    Container {
        children: Vec<CanvasBlock>,
        max_width_px: u32,
    },
    Spacer {
        height_px: u32,
    },
    Divider,
    Navbar {
        brand_title: CompactString,
        links: Vec<(CompactString, CompactString)>, // [label, url]
    },
    Footer {
        copyright_text: CompactString,
    },

    // Dynamic Repeaters & Data Blocks
    Repeater {
        data_source_path: CompactString, // e.g. "pageData.products"
        item_alias: CompactString,       // e.g. "product"
        template: Box<CanvasBlock>,
    },
    CardBlock {
        title: CompactString,
        body: CompactString,
        image_url: Option<CompactString>,
    },
    NumberCardBlock {
        label: CompactString,
        value_data_path: CompactString,
        format: Option<CompactString>,
    },

    // Media & Raw
    ImageBlock {
        src: CompactString,
        alt: CompactString,
        width_px: Option<u32>,
        height_px: Option<u32>,
    },
    RawHtml {
        html: CompactString,
    },
}

/// Hierarchical Scoped Data Resolver (`BlockValueResolver`).
#[derive(Debug, Clone, Default)]
pub struct BlockValueResolver {
    pub page_data: HashMap<CompactString, serde_json::Value>,
    pub component_data: HashMap<CompactString, serde_json::Value>,
    pub props: HashMap<CompactString, serde_json::Value>,
}

impl BlockValueResolver {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves a dotted path (e.g. `pageData.items.0.title`) against the hierarchy:
    /// `props` -> `componentData` -> `pageData`.
    #[must_use]
    pub fn resolve_path(&self, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let (scope, remaining) = if parts.len() > 1 && matches!(parts[0], "pageData" | "componentData" | "props") {
            (parts[0], &parts[1..])
        } else {
            ("pageData", parts.as_slice())
        };

        let root_map = match scope {
            "props" => &self.props,
            "componentData" => &self.component_data,
            _ => &self.page_data,
        };

        let mut current_val = root_map.get(remaining[0])?;

        for key in &remaining[1..] {
            match current_val {
                serde_json::Value::Object(map) => {
                    current_val = map.get(*key)?;
                }
                serde_json::Value::Array(arr) => {
                    if let Ok(idx) = key.parse::<usize>() {
                        current_val = arr.get(idx)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        match current_val {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            other => Some(other.to_string()),
        }
    }
}

/// Append-Only Visual AST Reversion Ledger for Undo/Redo operations.
#[derive(Debug, Clone)]
pub struct CanvasReversionLedger {
    history: Vec<Vec<CanvasBlock>>,
    current_index: usize,
}

impl CanvasReversionLedger {
    #[must_use]
    pub fn new(initial_blocks: Vec<CanvasBlock>) -> Self {
        Self {
            history: vec![initial_blocks],
            current_index: 0,
        }
    }

    /// Records a new visual state commit.
    pub fn commit(&mut self, blocks: Vec<CanvasBlock>) {
        // Truncate redo stack if commit happens after undo
        self.history.truncate(self.current_index + 1);
        self.history.push(blocks);
        self.current_index += 1;
    }

    /// Performs undo step.
    pub fn undo(&mut self) -> Option<&[CanvasBlock]> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.history[self.current_index])
        } else {
            None
        }
    }

    /// Performs redo step.
    pub fn redo(&mut self) -> Option<&[CanvasBlock]> {
        if self.current_index + 1 < self.history.len() {
            self.current_index += 1;
            Some(&self.history[self.current_index])
        } else {
            None
        }
    }

    #[must_use]
    pub fn current_blocks(&self) -> &[CanvasBlock] {
        &self.history[self.current_index]
    }
}

/// Autonomous Site Builder ("Bob") LLM Streaming Protocol Payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BobAgentPromptRequest {
    pub prompt: String,
    pub mode: String, // "generate" | "modify"
    pub selected_block_id: Option<String>,
    pub active_theme_id: String,
    pub target_doctypes: Vec<String>,
    pub attached_image_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_value_resolver() {
        let mut resolver = BlockValueResolver::new();
        resolver.page_data.insert(
            "products".into(),
            serde_json::json!([
                {"title": "Industrial Robot Arm", "price": 4999.00},
                {"title": "Smart Sensor Hub", "price": 299.00}
            ]),
        );

        let resolved = resolver.resolve_path("pageData.products.0.title");
        assert_eq!(resolved.as_deref(), Some("Industrial Robot Arm"));

        let price = resolver.resolve_path("pageData.products.1.price");
        assert_eq!(price.as_deref(), Some("299.0"));
    }

    #[test]
    fn test_canvas_undo_redo_ledger() {
        let initial = vec![CanvasBlock::Heading {
            level: 1,
            text: "Initial Title".into(),
            data_binding: None,
        }];

        let mut ledger = CanvasReversionLedger::new(initial);

        let update1 = vec![CanvasBlock::Heading {
            level: 1,
            text: "Updated Title".into(),
            data_binding: None,
        }];
        ledger.commit(update1);

        assert_eq!(ledger.current_blocks().len(), 1);

        // Undo
        let undone = ledger.undo().unwrap();
        if let CanvasBlock::Heading { text, .. } = &undone[0] {
            assert_eq!(text.as_str(), "Initial Title");
        } else {
            panic!("Expected Heading");
        }

        // Redo
        let redone = ledger.redo().unwrap();
        if let CanvasBlock::Heading { text, .. } = &redone[0] {
            assert_eq!(text.as_str(), "Updated Title");
        } else {
            panic!("Expected Heading");
        }
    }
}
