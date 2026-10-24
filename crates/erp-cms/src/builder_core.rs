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

/// Definition of a named component slot for template composition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlotDefinition {
    pub name: CompactString,
    pub description: CompactString,
    pub allowed_components: Vec<CompactString>,
    pub required: bool,
}

impl SlotDefinition {
    #[must_use]
    pub fn new(name: impl Into<CompactString>, description: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            allowed_components: Vec::new(),
            required: false,
        }
    }
}

/// Breakpoint-indexed sparse CSS properties with desktop-first inheritance rules.
pub type ResponsiveStyleMap = ResponsiveStyleCascade;

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
    SlotContainer {
        name: CompactString,
        allowed_types: Vec<CompactString>,
        children: Vec<CanvasBlock>,
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
    #[serde(untagged)]
    Custom {
        block_type: CompactString,
        #[serde(flatten)]
        fields: HashMap<CompactString, serde_json::Value>,
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

    /// Resolves a dotted path (e.g. `pageData.items.0.title`, `{{ product.title }}`) against the hierarchy:
    /// `componentData` -> `props` -> `pageData`.
    #[must_use]
    pub fn resolve_path(&self, raw_path: &str) -> Option<String> {
        let clean = raw_path.trim();
        let path = if clean.starts_with("{{") && clean.ends_with("}}") {
            clean[2..clean.len() - 2].trim()
        } else {
            clean
        };

        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let (scope, remaining) =
            if parts.len() > 1 && matches!(parts[0], "pageData" | "componentData" | "props") {
                (parts[0], &parts[1..])
            } else if self.component_data.contains_key(parts[0]) {
                ("componentData", parts.as_slice())
            } else if self.props.contains_key(parts[0]) {
                ("props", parts.as_slice())
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

    /// Interpolates multiple `{{ ... }}` placeholders inside arbitrary strings.
    #[must_use]
    pub fn interpolate_string(&self, text: &str) -> String {
        if !text.contains("{{") {
            return text.to_string();
        }
        let mut result = text.to_string();
        while let Some(start) = result.find("{{") {
            if let Some(end) = result[start..].find("}}") {
                let full_end = start + end + 2;
                let expr = result[start + 2..start + end].trim();
                let replacement = self.resolve_path(expr).unwrap_or_default();
                result.replace_range(start..full_end, &replacement);
            } else {
                break;
            }
        }
        result
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

/// Multi-breakpoint device frame switcher for Desktop, Tablet, and Mobile simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceFrameSwitcher {
    pub breakpoint: CanvasBreakpoint,
    pub is_landscape: bool,
}

impl DeviceFrameSwitcher {
    pub const DESKTOP_WIDTH_PX: u32 = 1280;
    pub const TABLET_WIDTH_PX: u32 = 768;
    pub const MOBILE_WIDTH_PX: u32 = 375;

    pub const DESKTOP_HEIGHT_PX: u32 = 800;
    pub const TABLET_HEIGHT_PX: u32 = 1024;
    pub const MOBILE_HEIGHT_PX: u32 = 667;

    #[must_use]
    pub fn new(breakpoint: CanvasBreakpoint) -> Self {
        Self {
            breakpoint,
            is_landscape: false,
        }
    }

    #[must_use]
    pub fn desktop() -> Self {
        Self::new(CanvasBreakpoint::Desktop)
    }

    #[must_use]
    pub fn tablet() -> Self {
        Self::new(CanvasBreakpoint::Tablet)
    }

    #[must_use]
    pub fn mobile() -> Self {
        Self::new(CanvasBreakpoint::Mobile)
    }

    pub fn set_breakpoint(&mut self, breakpoint: CanvasBreakpoint) {
        self.breakpoint = breakpoint;
    }

    pub fn toggle_orientation(&mut self) {
        self.is_landscape = !self.is_landscape;
    }

    #[must_use]
    pub fn dimensions_px(&self) -> (u32, u32) {
        let (w, h) = match self.breakpoint {
            CanvasBreakpoint::Desktop => (Self::DESKTOP_WIDTH_PX, Self::DESKTOP_HEIGHT_PX),
            CanvasBreakpoint::Tablet => (Self::TABLET_WIDTH_PX, Self::TABLET_HEIGHT_PX),
            CanvasBreakpoint::Mobile => (Self::MOBILE_WIDTH_PX, Self::MOBILE_HEIGHT_PX),
        };
        if self.is_landscape { (h, w) } else { (w, h) }
    }

    #[must_use]
    pub fn frame_css_style(&self) -> String {
        let (w, h) = self.dimensions_px();
        format!(
            "width: {w}px; max-width: 100%; height: {h}px; margin: 0 auto; transition: all 0.25s ease-in-out; border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 8px; overflow: auto; background: #ffffff;"
        )
    }
}

/// Visual inspector for block styling attributes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BlockCssInspector {
    // Typography
    pub font_family: Option<CompactString>,
    pub font_size_px: Option<u32>,
    pub font_weight: Option<u16>,
    pub line_height: Option<f32>,
    pub text_align: Option<CompactString>,
    pub color: Option<CompactString>,

    // Spacing
    pub margin_top_px: Option<i32>,
    pub margin_right_px: Option<i32>,
    pub margin_bottom_px: Option<i32>,
    pub margin_left_px: Option<i32>,
    pub padding_top_px: Option<u32>,
    pub padding_right_px: Option<u32>,
    pub padding_bottom_px: Option<u32>,
    pub padding_left_px: Option<u32>,

    // Layout
    pub display: Option<CompactString>,
    pub flex_direction: Option<CompactString>,
    pub justify_content: Option<CompactString>,
    pub align_items: Option<CompactString>,
    pub gap_px: Option<u32>,
    pub grid_columns: Option<CompactString>,

    // Colors & Borders
    pub background_color: Option<CompactString>,
    pub border_width_px: Option<u32>,
    pub border_color: Option<CompactString>,
    pub border_radius_px: Option<u32>,
    pub box_shadow: Option<CompactString>,
}

impl BlockCssInspector {
    #[must_use]
    pub fn to_inline_css(&self) -> String {
        let mut props = Vec::new();
        if let Some(ff) = &self.font_family {
            props.push(format!("font-family: {ff};"));
        }
        if let Some(fs) = self.font_size_px {
            props.push(format!("font-size: {fs}px;"));
        }
        if let Some(fw) = self.font_weight {
            props.push(format!("font-weight: {fw};"));
        }
        if let Some(lh) = self.line_height {
            props.push(format!("line-height: {lh};"));
        }
        if let Some(ta) = &self.text_align {
            props.push(format!("text-align: {ta};"));
        }
        if let Some(c) = &self.color {
            props.push(format!("color: {c};"));
        }

        if let Some(mt) = self.margin_top_px {
            props.push(format!("margin-top: {mt}px;"));
        }
        if let Some(mr) = self.margin_right_px {
            props.push(format!("margin-right: {mr}px;"));
        }
        if let Some(mb) = self.margin_bottom_px {
            props.push(format!("margin-bottom: {mb}px;"));
        }
        if let Some(ml) = self.margin_left_px {
            props.push(format!("margin-left: {ml}px;"));
        }

        if let Some(pt) = self.padding_top_px {
            props.push(format!("padding-top: {pt}px;"));
        }
        if let Some(pr) = self.padding_right_px {
            props.push(format!("padding-right: {pr}px;"));
        }
        if let Some(pb) = self.padding_bottom_px {
            props.push(format!("padding-bottom: {pb}px;"));
        }
        if let Some(pl) = self.padding_left_px {
            props.push(format!("padding-left: {pl}px;"));
        }

        if let Some(d) = &self.display {
            props.push(format!("display: {d};"));
        }
        if let Some(fd) = &self.flex_direction {
            props.push(format!("flex-direction: {fd};"));
        }
        if let Some(jc) = &self.justify_content {
            props.push(format!("justify-content: {jc};"));
        }
        if let Some(ai) = &self.align_items {
            props.push(format!("align-items: {ai};"));
        }
        if let Some(gap) = self.gap_px {
            props.push(format!("gap: {gap}px;"));
        }
        if let Some(gc) = &self.grid_columns {
            props.push(format!("grid-template-columns: {gc};"));
        }

        if let Some(bg) = &self.background_color {
            props.push(format!("background-color: {bg};"));
        }
        if let Some(bw) = self.border_width_px {
            props.push(format!("border-width: {bw}px; border-style: solid;"));
        }
        if let Some(bc) = &self.border_color {
            props.push(format!("border-color: {bc};"));
        }
        if let Some(br) = self.border_radius_px {
            props.push(format!("border-radius: {br}px;"));
        }
        if let Some(bs) = &self.box_shadow {
            props.push(format!("box-shadow: {bs};"));
        }

        props.join(" ")
    }

    /// Merges overrides from another inspector if the corresponding field is `Some`.
    pub fn merge_override(&mut self, other: &Self) {
        if other.font_family.is_some() {
            self.font_family = other.font_family.clone();
        }
        if other.font_size_px.is_some() {
            self.font_size_px = other.font_size_px;
        }
        if other.font_weight.is_some() {
            self.font_weight = other.font_weight;
        }
        if other.line_height.is_some() {
            self.line_height = other.line_height;
        }
        if other.text_align.is_some() {
            self.text_align = other.text_align.clone();
        }
        if other.color.is_some() {
            self.color = other.color.clone();
        }

        if other.margin_top_px.is_some() {
            self.margin_top_px = other.margin_top_px;
        }
        if other.margin_right_px.is_some() {
            self.margin_right_px = other.margin_right_px;
        }
        if other.margin_bottom_px.is_some() {
            self.margin_bottom_px = other.margin_bottom_px;
        }
        if other.margin_left_px.is_some() {
            self.margin_left_px = other.margin_left_px;
        }

        if other.padding_top_px.is_some() {
            self.padding_top_px = other.padding_top_px;
        }
        if other.padding_right_px.is_some() {
            self.padding_right_px = other.padding_right_px;
        }
        if other.padding_bottom_px.is_some() {
            self.padding_bottom_px = other.padding_bottom_px;
        }
        if other.padding_left_px.is_some() {
            self.padding_left_px = other.padding_left_px;
        }

        if other.display.is_some() {
            self.display = other.display.clone();
        }
        if other.flex_direction.is_some() {
            self.flex_direction = other.flex_direction.clone();
        }
        if other.justify_content.is_some() {
            self.justify_content = other.justify_content.clone();
        }
        if other.align_items.is_some() {
            self.align_items = other.align_items.clone();
        }
        if other.gap_px.is_some() {
            self.gap_px = other.gap_px;
        }
        if other.grid_columns.is_some() {
            self.grid_columns = other.grid_columns.clone();
        }

        if other.background_color.is_some() {
            self.background_color = other.background_color.clone();
        }
        if other.border_width_px.is_some() {
            self.border_width_px = other.border_width_px;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color.clone();
        }
        if other.border_radius_px.is_some() {
            self.border_radius_px = other.border_radius_px;
        }
        if other.box_shadow.is_some() {
            self.box_shadow = other.box_shadow.clone();
        }
    }
}

/// Multi-breakpoint responsive style cascade with desktop-first inheritance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResponsiveStyleCascade {
    pub desktop: BlockCssInspector,
    pub tablet: Option<BlockCssInspector>,
    pub mobile: Option<BlockCssInspector>,
}

impl ResponsiveStyleCascade {
    #[must_use]
    pub fn new(desktop: BlockCssInspector) -> Self {
        Self {
            desktop,
            tablet: None,
            mobile: None,
        }
    }

    /// Resolves effective style by cascading desktop styles down to tablet and mobile.
    #[must_use]
    pub fn resolve_for(&self, breakpoint: CanvasBreakpoint) -> BlockCssInspector {
        match breakpoint {
            CanvasBreakpoint::Desktop => self.desktop.clone(),
            CanvasBreakpoint::Tablet => {
                let mut base = self.desktop.clone();
                if let Some(t) = &self.tablet {
                    base.merge_override(t);
                }
                base
            }
            CanvasBreakpoint::Mobile => {
                let mut base = self.desktop.clone();
                if let Some(t) = &self.tablet {
                    base.merge_override(t);
                }
                if let Some(m) = &self.mobile {
                    base.merge_override(m);
                }
                base
            }
        }
    }

    /// Compiles CSS rules with media queries for this specific selector.
    #[must_use]
    pub fn compile_element_css(&self, selector: &str) -> String {
        let mut css = String::new();
        let base_css = self.desktop.to_inline_css();
        if !base_css.is_empty() {
            css.push_str(&format!("{selector} {{ {base_css} }}\n"));
        }

        if let Some(tablet) = &self.tablet {
            let t_css = tablet.to_inline_css();
            if !t_css.is_empty() {
                css.push_str(&format!(
                    "@media (max-width: 768px) {{\n  {selector} {{ {t_css} }}\n}}\n"
                ));
            }
        }

        if let Some(mobile) = &self.mobile {
            let m_css = mobile.to_inline_css();
            if !m_css.is_empty() {
                css.push_str(&format!(
                    "@media (max-width: 480px) {{\n  {selector} {{ {m_css} }}\n}}\n"
                ));
            }
        }
        css
    }
}

/// Filter condition for live DocType SurrealQL query binding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocTypeBindingFilter {
    pub field: CompactString,
    pub operator: CompactString, // "=", "!=", ">", "<", "CONTAINS", "IN"
    pub value: serde_json::Value,
}

/// Live DocType data binding definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocTypeDataBinding {
    pub doctype: CompactString,
    pub fields: Vec<CompactString>,
    pub filters: Vec<DocTypeBindingFilter>,
    pub order_by: Option<CompactString>,
    pub limit: Option<u32>,
}

impl DocTypeDataBinding {
    #[must_use]
    pub fn compile_surrealql(&self) -> String {
        let field_projection = if self.fields.is_empty() {
            "*".to_string()
        } else {
            self.fields.join(", ")
        };

        let mut query = format!(
            "SELECT {field_projection} FROM {}",
            self.doctype.to_ascii_lowercase()
        );

        if !self.filters.is_empty() {
            let clauses: Vec<String> = self
                .filters
                .iter()
                .map(|f| {
                    let formatted_val = match &f.value {
                        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "\\'")),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Array(arr) => {
                            let items: Vec<String> = arr
                                .iter()
                                .map(|item| match item {
                                    serde_json::Value::String(s) => {
                                        format!("'{}'", s.replace('\'', "\\'"))
                                    }
                                    other => other.to_string(),
                                })
                                .collect();
                            format!("[{}]", items.join(", "))
                        }
                        other => other.to_string(),
                    };
                    format!("{} {} {}", f.field, f.operator, formatted_val)
                })
                .collect();
            query.push_str(" WHERE ");
            query.push_str(&clauses.join(" AND "));
        }

        if let Some(order) = &self.order_by {
            query.push_str(&format!(" ORDER BY {order}"));
        }

        if let Some(lim) = self.limit {
            query.push_str(&format!(" LIMIT {lim}"));
        }

        query.push(';');
        query
    }
}

/// Individual node in the CanvasBlockTree hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanvasBlockNode {
    pub id: CompactString,
    pub block: CanvasBlock,
    pub parent_id: Option<CompactString>,
    pub child_ids: Vec<CompactString>,
    pub css_styles: Option<BlockCssInspector>,
    #[serde(default)]
    pub responsive_styles: Option<ResponsiveStyleCascade>,
    #[serde(default)]
    pub slot: Option<CompactString>,
    #[serde(default)]
    pub slot_definitions: Vec<SlotDefinition>,
}

/// Hierarchical Block Canvas Tree supporting drag-and-drop insertion, removal, and reordering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CanvasBlockTree {
    pub nodes: HashMap<CompactString, CanvasBlockNode>,
    pub root_ids: Vec<CompactString>,
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub updated_by: Option<CompactString>,
    next_id: usize,
}

impl CanvasBlockTree {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a block into the canvas tree at a specified parent and child index.
    pub fn insert_block(
        &mut self,
        parent_id: Option<&str>,
        block: CanvasBlock,
        index: usize,
    ) -> CompactString {
        self.next_id += 1;
        self.version += 1;
        let id: CompactString = format!("blk_{}", self.next_id).into();
        let parent = parent_id.map(CompactString::new);

        if let Some(pid) = &parent {
            if let Some(parent_node) = self.nodes.get_mut(pid) {
                let idx = index.min(parent_node.child_ids.len());
                parent_node.child_ids.insert(idx, id.clone());
            } else {
                let idx = index.min(self.root_ids.len());
                self.root_ids.insert(idx, id.clone());
            }
        } else {
            let idx = index.min(self.root_ids.len());
            self.root_ids.insert(idx, id.clone());
        }

        let node = CanvasBlockNode {
            id: id.clone(),
            block,
            parent_id: parent,
            child_ids: Vec::new(),
            css_styles: None,
            responsive_styles: None,
            slot: None,
            slot_definitions: Vec::new(),
        };

        self.nodes.insert(id.clone(), node);
        id
    }

    /// Removes a block and its entire descendant subtree from the canvas tree.
    pub fn remove_block(&mut self, id: &str) -> Option<CanvasBlock> {
        let node = self.nodes.remove(id)?;

        // Remove from parent or root list
        if let Some(pid) = &node.parent_id {
            if let Some(parent_node) = self.nodes.get_mut(pid) {
                parent_node.child_ids.retain(|c| c.as_str() != id);
            }
        } else {
            self.root_ids.retain(|r| r.as_str() != id);
        }

        // Recursively remove descendants
        let mut to_remove = node.child_ids.clone();
        while let Some(child_id) = to_remove.pop() {
            if let Some(child_node) = self.nodes.remove(child_id.as_str()) {
                to_remove.extend(child_node.child_ids);
            }
        }

        self.version += 1;
        Some(node.block)
    }

    /// Moves a block to a new parent and index with cycle detection.
    pub fn move_block(
        &mut self,
        id: &str,
        new_parent_id: Option<&str>,
        new_index: usize,
    ) -> Result<(), &'static str> {
        if !self.nodes.contains_key(id) {
            return Err("Block not found");
        }

        // Cycle check: new_parent_id cannot be id or any descendant of id
        if let Some(npid) = new_parent_id {
            if npid == id {
                return Err("Cannot move block into itself");
            }
            let mut curr = Some(npid);
            while let Some(curr_id) = curr {
                if curr_id == id {
                    return Err("Cannot move block into its descendant");
                }
                curr = self.nodes.get(curr_id).and_then(|n| n.parent_id.as_deref());
            }
        }

        // Unlink from current parent or root
        let old_parent = self.nodes.get(id).and_then(|n| n.parent_id.clone());
        if let Some(opid) = old_parent {
            if let Some(old_p) = self.nodes.get_mut(opid.as_str()) {
                old_p.child_ids.retain(|c| c.as_str() != id);
            }
        } else {
            self.root_ids.retain(|r| r.as_str() != id);
        }

        let id_compact: CompactString = id.into();

        // Link into new parent or root
        if let Some(npid) = new_parent_id {
            if let Some(new_p) = self.nodes.get_mut(npid) {
                let idx = new_index.min(new_p.child_ids.len());
                new_p.child_ids.insert(idx, id_compact.clone());
                if let Some(target) = self.nodes.get_mut(id) {
                    target.parent_id = Some(npid.into());
                }
            } else {
                return Err("New parent block not found");
            }
        } else {
            let idx = new_index.min(self.root_ids.len());
            self.root_ids.insert(idx, id_compact.clone());
            if let Some(target) = self.nodes.get_mut(id) {
                target.parent_id = None;
            }
        }

        self.version += 1;
        Ok(())
    }

    #[must_use]
    pub fn get_node(&self, id: &str) -> Option<&CanvasBlockNode> {
        self.nodes.get(id)
    }

    #[must_use]
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut CanvasBlockNode> {
        self.nodes.get_mut(id)
    }

    /// Compiles all responsive and custom CSS rules into a centralized stylesheet.
    #[must_use]
    pub fn compile_stylesheet(&self) -> String {
        let mut stylesheet = String::new();
        for (id, node) in &self.nodes {
            let selector = format!("#{id}");
            if let Some(resp) = &node.responsive_styles {
                stylesheet.push_str(&resp.compile_element_css(&selector));
            } else if let Some(css) = &node.css_styles {
                let inline = css.to_inline_css();
                if !inline.is_empty() {
                    stylesheet.push_str(&format!("{selector} {{ {inline} }}\n"));
                }
            }
        }
        stylesheet
    }

    /// Renders the complete block tree to HTML with scoped data resolution and responsive media queries.
    #[must_use]
    pub fn render_html(&self, resolver: &BlockValueResolver) -> String {
        let mut out = String::new();
        let stylesheet = self.compile_stylesheet();
        if !stylesheet.is_empty() {
            out.push_str("<style>\n");
            out.push_str(&stylesheet);
            out.push_str("</style>\n");
        }
        for root_id in &self.root_ids {
            self.render_node(root_id.as_str(), resolver, &mut out);
        }
        out
    }

    fn render_node(&self, id: &str, resolver: &BlockValueResolver, out: &mut String) {
        let Some(node) = self.nodes.get(id) else {
            return;
        };
        let style_attr = if let Some(s) = &node.css_styles {
            format!(r#" style="{}""#, s.to_inline_css())
        } else if let Some(resp) = &node.responsive_styles {
            format!(r#" style="{}""#, resp.desktop.to_inline_css())
        } else {
            String::new()
        };
        let slot_attr = node
            .slot
            .as_ref()
            .map(|s| format!(r#" data-slot="{s}""#))
            .unwrap_or_default();

        match &node.block {
            CanvasBlock::Heading {
                level,
                text,
                data_binding,
            } => {
                let resolved = data_binding
                    .as_deref()
                    .and_then(|p| resolver.resolve_path(p))
                    .unwrap_or_else(|| resolver.interpolate_string(text.as_str()));
                out.push_str(&format!(
                    "<h{level} id=\"{id}\"{slot_attr}{style_attr}>{resolved}</h{level}>\n"
                ));
            }
            CanvasBlock::TextBlock {
                content,
                data_binding,
            } => {
                let resolved = data_binding
                    .as_deref()
                    .and_then(|p| resolver.resolve_path(p))
                    .unwrap_or_else(|| resolver.interpolate_string(content.as_str()));
                out.push_str(&format!(
                    "<p id=\"{id}\"{slot_attr}{style_attr}>{resolved}</p>\n"
                ));
            }
            CanvasBlock::BlockQuote { quote, author } => {
                let q_res = resolver.interpolate_string(quote.as_str());
                out.push_str(&format!(
                    "<blockquote id=\"{id}\"{slot_attr}{style_attr}><p>{q_res}</p>"
                ));
                if let Some(a) = author {
                    let a_res = resolver.interpolate_string(a.as_str());
                    out.push_str(&format!("<footer>— <cite>{a_res}</cite></footer>"));
                }
                out.push_str("</blockquote>\n");
            }
            CanvasBlock::Section { css_classes, .. } => {
                out.push_str(&format!(
                    "<section id=\"{id}\" class=\"{css_classes}\"{slot_attr}{style_attr}>\n"
                ));
                for cid in &node.child_ids {
                    self.render_node(cid.as_str(), resolver, out);
                }
                out.push_str("</section>\n");
            }
            CanvasBlock::Div { css_classes, .. } => {
                out.push_str(&format!(
                    "<div id=\"{id}\" class=\"{css_classes}\"{slot_attr}{style_attr}>\n"
                ));
                for cid in &node.child_ids {
                    self.render_node(cid.as_str(), resolver, out);
                }
                out.push_str("</div>\n");
            }
            CanvasBlock::Container { max_width_px, .. } => {
                out.push_str(&format!(
                    "<div id=\"{id}\" class=\"container\"{slot_attr} style=\"max-width: {max_width_px}px; margin: 0 auto;{style_attr}\">\n"
                ));
                for cid in &node.child_ids {
                    self.render_node(cid.as_str(), resolver, out);
                }
                out.push_str("</div>\n");
            }
            CanvasBlock::Spacer { height_px } => {
                out.push_str(&format!(
                    "<div id=\"{id}\"{slot_attr} style=\"height: {height_px}px;{style_attr}\"></div>\n"
                ));
            }
            CanvasBlock::Divider => {
                out.push_str(&format!(
                    "<hr id=\"{id}\" class=\"builder-divider\"{slot_attr}{style_attr} />\n"
                ));
            }
            CanvasBlock::Navbar { brand_title, links } => {
                let brand = resolver.interpolate_string(brand_title.as_str());
                out.push_str(&format!(
                    "<nav id=\"{id}\" class=\"builder-navbar\"{slot_attr}{style_attr}><div class=\"nav-brand\">{brand}</div><ul class=\"nav-links\">"
                ));
                for (lbl, url) in links {
                    let l_res = resolver.interpolate_string(lbl.as_str());
                    let u_res = resolver.interpolate_string(url.as_str());
                    out.push_str(&format!("<li><a href=\"{u_res}\">{l_res}</a></li>"));
                }
                out.push_str("</ul></nav>\n");
            }
            CanvasBlock::Footer { copyright_text } => {
                let copy = resolver.interpolate_string(copyright_text.as_str());
                out.push_str(&format!(
                    "<footer id=\"{id}\" class=\"builder-footer\"{slot_attr}{style_attr}><p>{copy}</p></footer>\n"
                ));
            }
            CanvasBlock::SlotContainer { name, .. } => {
                out.push_str(&format!(
                    "<div id=\"{id}\" class=\"builder-slot-container\" data-slot=\"{name}\"{slot_attr}{style_attr}>\n"
                ));
                for cid in &node.child_ids {
                    self.render_node(cid.as_str(), resolver, out);
                }
                out.push_str("</div>\n");
            }
            CanvasBlock::CardBlock {
                title,
                body,
                image_url,
            } => {
                let title_res = resolver.interpolate_string(title.as_str());
                let body_res = resolver.interpolate_string(body.as_str());
                let img_res = image_url
                    .as_ref()
                    .map(|img| resolver.interpolate_string(img.as_str()));

                out.push_str(&format!(
                    "<div id=\"{id}\" class=\"card-block\"{slot_attr}{style_attr}>"
                ));
                if let Some(img) = img_res.filter(|img| !img.is_empty()) {
                    out.push_str(&format!(
                        "<img src=\"{img}\" alt=\"{title_res}\" class=\"card-img\" />"
                    ));
                }
                out.push_str(&format!("<h3>{title_res}</h3><p>{body_res}</p></div>\n"));
            }
            CanvasBlock::NumberCardBlock {
                label,
                value_data_path,
                format: _,
            } => {
                let val = resolver
                    .resolve_path(value_data_path.as_str())
                    .unwrap_or_else(|| "0".into());
                let lbl = resolver.interpolate_string(label.as_str());
                out.push_str(&format!(
                    "<div id=\"{id}\" class=\"number-card\"{slot_attr}{style_attr}><span class=\"number-label\">{lbl}</span><span class=\"number-value\">{val}</span></div>\n"
                ));
            }
            CanvasBlock::ImageBlock {
                src,
                alt,
                width_px,
                height_px,
            } => {
                let src_res = resolver.interpolate_string(src.as_str());
                let alt_res = resolver.interpolate_string(alt.as_str());
                let mut dims = String::new();
                if let Some(w) = width_px {
                    dims.push_str(&format!(" width=\"{w}\""));
                }
                if let Some(h) = height_px {
                    dims.push_str(&format!(" height=\"{h}\""));
                }
                out.push_str(&format!(
                    "<img id=\"{id}\" src=\"{src_res}\" alt=\"{alt_res}\"{dims}{slot_attr}{style_attr} />\n"
                ));
            }
            CanvasBlock::RawHtml { html } => {
                let html_res = resolver.interpolate_string(html.as_str());
                out.push_str(html_res.as_str());
                out.push('\n');
            }
            CanvasBlock::Repeater {
                data_source_path,
                item_alias,
                template,
            } => {
                out.push_str(&format!(
                    "<div id=\"{id}\" class=\"repeater\" data-source=\"{data_source_path}\"{slot_attr}{style_attr}>\n"
                ));
                let clean_path = data_source_path
                    .trim()
                    .trim_start_matches("{{")
                    .trim_end_matches("}}")
                    .trim();
                let parts: Vec<&str> = clean_path.split('.').collect();
                let key = if parts.len() > 1 && parts[0] == "pageData" {
                    parts[1]
                } else {
                    parts[0]
                };
                if let Some(serde_json::Value::Array(items)) = resolver.page_data.get(key) {
                    for (idx, item) in items.iter().enumerate() {
                        let mut item_resolver = resolver.clone();
                        item_resolver
                            .component_data
                            .insert(item_alias.clone(), item.clone());
                        let temp_id = format!("{id}_item_{idx}");
                        let temp_node = CanvasBlockNode {
                            id: temp_id.into(),
                            block: (**template).clone(),
                            parent_id: Some(id.into()),
                            child_ids: Vec::new(),
                            css_styles: None,
                            responsive_styles: None,
                            slot: None,
                            slot_definitions: Vec::new(),
                        };
                        let mut temp_tree = CanvasBlockTree::default();
                        temp_tree
                            .nodes
                            .insert(temp_node.id.clone(), temp_node.clone());
                        temp_tree.root_ids.push(temp_node.id.clone());
                        out.push_str(&temp_tree.render_html(&item_resolver));
                    }
                } else {
                    for cid in &node.child_ids {
                        self.render_node(cid.as_str(), resolver, out);
                    }
                }
                out.push_str("</div>\n");
            }
            CanvasBlock::Custom { block_type, .. } => {
                out.push_str(&format!(
                    "<div id=\"{id}\" class=\"custom-block\" data-block-type=\"{block_type}\"{slot_attr}{style_attr}>\n"
                ));
                for cid in &node.child_ids {
                    self.render_node(cid.as_str(), resolver, out);
                }
                out.push_str("</div>\n");
            }
        }
    }
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

    #[test]
    fn test_canvas_block_tree_operations() {
        let mut tree = CanvasBlockTree::new();

        // Insert root section
        let section_id = tree.insert_block(
            None,
            CanvasBlock::Section {
                id: "main_section".into(),
                children: Vec::new(),
                css_classes: "hero-section".into(),
            },
            0,
        );

        // Insert child heading into section
        let heading_id = tree.insert_block(
            Some(section_id.as_str()),
            CanvasBlock::Heading {
                level: 1,
                text: "Planetary ERP".into(),
                data_binding: None,
            },
            0,
        );

        // Insert paragraph after heading
        let para_id = tree.insert_block(
            Some(section_id.as_str()),
            CanvasBlock::TextBlock {
                content: "Sub-millisecond latency substrate.".into(),
                data_binding: None,
            },
            1,
        );

        assert_eq!(tree.root_ids.len(), 1);
        let section = tree.get_node(section_id.as_str()).unwrap();
        assert_eq!(section.child_ids.len(), 2);
        assert_eq!(section.child_ids[0], heading_id);
        assert_eq!(section.child_ids[1], para_id);

        // Move paragraph to root at index 0
        assert!(tree.move_block(para_id.as_str(), None, 0).is_ok());
        assert_eq!(tree.root_ids.len(), 2);
        assert_eq!(tree.root_ids[0], para_id);

        // Test cycle detection
        assert!(
            tree.move_block(section_id.as_str(), Some(heading_id.as_str()), 0)
                .is_err()
        );

        // Render HTML
        let resolver = BlockValueResolver::new();
        let html = tree.render_html(&resolver);
        assert!(html.contains("<h1 id=\"blk_2\">Planetary ERP</h1>"));
        assert!(html.contains("<p id=\"blk_3\">Sub-millisecond latency substrate.</p>"));

        // Remove block
        let removed = tree.remove_block(section_id.as_str());
        assert!(removed.is_some());
        assert_eq!(tree.get_node(section_id.as_str()), None);
        assert_eq!(tree.get_node(heading_id.as_str()), None);
    }

    #[test]
    fn test_device_frame_switcher() {
        let mut switcher = DeviceFrameSwitcher::desktop();
        assert_eq!(switcher.dimensions_px(), (1280, 800));

        switcher.set_breakpoint(CanvasBreakpoint::Tablet);
        assert_eq!(switcher.dimensions_px(), (768, 1024));

        switcher.toggle_orientation();
        assert_eq!(switcher.dimensions_px(), (1024, 768));

        switcher.set_breakpoint(CanvasBreakpoint::Mobile);
        switcher.toggle_orientation(); // back to portrait
        assert_eq!(switcher.dimensions_px(), (375, 667));

        let css = switcher.frame_css_style();
        assert!(css.contains("width: 375px;"));
        assert!(css.contains("height: 667px;"));
    }

    #[test]
    fn test_block_css_inspector() {
        let inspector = BlockCssInspector {
            font_size_px: Some(24),
            font_weight: Some(600),
            color: Some("#ffffff".into()),
            margin_bottom_px: Some(16),
            padding_top_px: Some(8),
            display: Some("flex".into()),
            gap_px: Some(12),
            background_color: Some("#121212".into()),
            border_radius_px: Some(4),
            ..Default::default()
        };

        let inline_css = inspector.to_inline_css();
        assert!(inline_css.contains("font-size: 24px;"));
        assert!(inline_css.contains("font-weight: 600;"));
        assert!(inline_css.contains("color: #ffffff;"));
        assert!(inline_css.contains("margin-bottom: 16px;"));
        assert!(inline_css.contains("padding-top: 8px;"));
        assert!(inline_css.contains("display: flex;"));
        assert!(inline_css.contains("gap: 12px;"));
        assert!(inline_css.contains("background-color: #121212;"));
        assert!(inline_css.contains("border-radius: 4px;"));
    }

    #[test]
    fn test_doctype_data_binding_surrealql() {
        let binding = DocTypeDataBinding {
            doctype: "Item".into(),
            fields: vec![
                "item_code".into(),
                "item_name".into(),
                "standard_rate".into(),
            ],
            filters: vec![
                DocTypeBindingFilter {
                    field: "is_sales_item".into(),
                    operator: "=".into(),
                    value: serde_json::json!(true),
                },
                DocTypeBindingFilter {
                    field: "disabled".into(),
                    operator: "=".into(),
                    value: serde_json::json!(0),
                },
            ],
            order_by: Some("item_code ASC".into()),
            limit: Some(25),
        };

        let query = binding.compile_surrealql();
        assert_eq!(
            query,
            "SELECT item_code, item_name, standard_rate FROM item WHERE is_sales_item = true AND disabled = 0 ORDER BY item_code ASC LIMIT 25;"
        );
    }

    #[test]
    fn test_canvas_repeater_rendering() {
        let mut tree = CanvasBlockTree::new();
        let _repeater_id = tree.insert_block(
            None,
            CanvasBlock::Repeater {
                data_source_path: "pageData.items".into(),
                item_alias: "item".into(),
                template: Box::new(CanvasBlock::TextBlock {
                    content: "Item: {{ item.name }} | Cost: {{ item.cost }}".into(),
                    data_binding: None,
                }),
            },
            0,
        );

        let mut resolver = BlockValueResolver::new();
        resolver.page_data.insert(
            "items".into(),
            serde_json::json!([
                {"name": "Titanium Gear", "cost": 150},
                {"name": "Ceramic Bearing", "cost": 45}
            ]),
        );

        let html = tree.render_html(&resolver);
        assert!(html.contains("class=\"repeater\""));
        assert!(html.contains("Item: Titanium Gear | Cost: 150"));
        assert!(html.contains("Item: Ceramic Bearing | Cost: 45"));
    }

    #[test]
    fn test_responsive_style_cascade() {
        let desktop = BlockCssInspector {
            font_size_px: Some(32),
            color: Some("#ffffff".into()),
            display: Some("flex".into()),
            ..Default::default()
        };

        let tablet_override = BlockCssInspector {
            font_size_px: Some(24),
            ..Default::default()
        };

        let mobile_override = BlockCssInspector {
            font_size_px: Some(18),
            flex_direction: Some("column".into()),
            ..Default::default()
        };

        let cascade = ResponsiveStyleCascade {
            desktop,
            tablet: Some(tablet_override),
            mobile: Some(mobile_override),
        };

        // Desktop resolution
        let res_d = cascade.resolve_for(CanvasBreakpoint::Desktop);
        assert_eq!(res_d.font_size_px, Some(32));
        assert_eq!(res_d.color.as_deref(), Some("#ffffff"));

        // Tablet resolution inherits desktop color
        let res_t = cascade.resolve_for(CanvasBreakpoint::Tablet);
        assert_eq!(res_t.font_size_px, Some(24));
        assert_eq!(res_t.color.as_deref(), Some("#ffffff"));

        // Mobile resolution inherits desktop color and overrides direction
        let res_m = cascade.resolve_for(CanvasBreakpoint::Mobile);
        assert_eq!(res_m.font_size_px, Some(18));
        assert_eq!(res_m.flex_direction.as_deref(), Some("column"));
        assert_eq!(res_m.color.as_deref(), Some("#ffffff"));

        // CSS Media Queries compilation
        let css = cascade.compile_element_css("#hero_heading");
        assert!(css.contains("#hero_heading { font-size: 32px; color: #ffffff; display: flex; }"));
        assert!(css.contains("@media (max-width: 768px)"));
        assert!(css.contains("#hero_heading { font-size: 24px; }"));
        assert!(css.contains("@media (max-width: 480px)"));
        assert!(css.contains("#hero_heading { font-size: 18px; flex-direction: column; }"));
    }

    #[test]
    fn test_canvas_slot_and_stylesheet_rendering() {
        let mut tree = CanvasBlockTree::new();
        let section_id = tree.insert_block(
            None,
            CanvasBlock::Section {
                id: "layout_sec".into(),
                children: Vec::new(),
                css_classes: "page-layout".into(),
            },
            0,
        );

        let heading_id = tree.insert_block(
            Some(section_id.as_str()),
            CanvasBlock::Heading {
                level: 1,
                text: "Header Content".into(),
                data_binding: None,
            },
            0,
        );

        // Assign named slot and responsive styles to heading
        if let Some(node) = tree.get_node_mut(heading_id.as_str()) {
            node.slot = Some("header".into());
            node.responsive_styles = Some(ResponsiveStyleCascade::new(BlockCssInspector {
                color: Some("#00ffcc".into()),
                ..Default::default()
            }));
        }

        let resolver = BlockValueResolver::new();
        let html = tree.render_html(&resolver);
        assert!(html.contains("<style>"));
        assert!(html.contains("#blk_2 { color: #00ffcc; }"));
        assert!(html.contains("data-slot=\"header\""));
        assert!(html.contains("Header Content"));
    }

    #[test]
    fn test_slot_definition_and_slot_container_rendering() {
        let slot_def = SlotDefinition {
            name: "main_content".into(),
            description: "Main article container".into(),
            allowed_components: vec!["Heading".into(), "TextBlock".into()],
            required: true,
        };

        let mut tree = CanvasBlockTree::new();
        let slot_container_id = tree.insert_block(
            None,
            CanvasBlock::SlotContainer {
                name: "main_content".into(),
                allowed_types: vec!["Heading".into(), "TextBlock".into()],
                children: Vec::new(),
            },
            0,
        );

        let p_id = tree.insert_block(
            Some(slot_container_id.as_str()),
            CanvasBlock::TextBlock {
                content: "Rendered within slot container.".into(),
                data_binding: None,
            },
            0,
        );

        if let Some(node) = tree.get_node_mut(slot_container_id.as_str()) {
            node.slot_definitions = vec![slot_def];
            let mut style_map = ResponsiveStyleMap::default();
            style_map.desktop.background_color = Some("#1a1a2e".into());
            style_map.mobile = Some(BlockCssInspector {
                background_color: Some("#000000".into()),
                ..Default::default()
            });
            node.responsive_styles = Some(style_map);
        }

        let resolver = BlockValueResolver::new();
        let html = tree.render_html(&resolver);
        assert!(html.contains("class=\"builder-slot-container\""));
        assert!(html.contains("data-slot=\"main_content\""));
        assert!(html.contains("Rendered within slot container."));
        assert!(html.contains("@media (max-width: 480px)"));
        assert!(html.contains("#blk_1 { background-color: #000000; }"));
        assert_eq!(
            tree.get_node(slot_container_id.as_str())
                .unwrap()
                .slot_definitions
                .len(),
            1
        );
        assert_eq!(
            tree.get_node(p_id.as_str()).unwrap().parent_id.as_deref(),
            Some(slot_container_id.as_str())
        );
    }
}
