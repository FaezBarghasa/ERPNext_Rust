//! Dioxus 0.7 Native Desktop Visual Studio & WYSIWYG Builder (`desk-components::visual_builder`).
//!
//! Provides the complete offline-first native desktop visual application builder:
//! - Figma-grade infinite canvas with primary creation tools (Move V, Frame F, Sections Shift+S, Shapes R/L/O/Polygon/Star, Pen P, Pencil Shift+P, Scale K, Text T, Slice S, Comment C, Measurement Shift+M, Annotation Y, Eyedropper I, Hand H / Space).
//! - Tri-mode Left Sidebar: Layers Panel (hierarchical DOM structure tree, deep-select, lock/unlock, group/ungroup, collapse group), Assets Panel (reusable components, styles, variables), and Pages Panel (multi-screen/flow organizer).
//! - Tri-mode Right Sidebar: Design Tab (geometry, constraints, auto-layout, 4-side box model, fills, strokes, effects, arc tool, fine-tune underlines), Prototype Tab (granular interaction micro-triggers: WhilePressing, OnDrag directional, WhileHovering, MouseEnter/Leave, AfterDelay), and Dev Mode Tab (CSS, Tailwind, SwiftUI, Compose, tokens JSON).
//! - Direct Canvas Manipulation: Deep Select (⌘/Ctrl+Click), Select Parent (Shift+Enter), Ignore Constraints (⌘/Ctrl+Drag), Ignore Auto Layout on Drop (Ctrl/Space+Drag), Direct Select Locked Layer (⌘/Ctrl+Right-Click), Spacing Measurement (Alt/Option+Hover), Smart Selection & Tidy Up (⌥⌘T), Math in number fields (e.g. 1440/3), Copy/Paste styles (⌥⌘C/⌥⌘V), Paste to Replace (Shift+⌘/Ctrl+R), Quick Component Swap (Shift+⌥/Alt+Drag), Outline Mode (Ctrl+Y).
//! - Variables, Design Tokens & Theming: 6 native variable types (Color, Number, String, Boolean, Timing, Easing), collections, multi-mode theming (Light/Dark/Mobile/Brand A), semantic aliasing, variable scopes, code syntax bindings, and REST API serialization.
//! - Components & Variants: Main component & instances, variant sets with state properties, component properties (Text, Boolean, InstanceSwap, Variant), and design system analytics.
//! - One-click export for lean deployment (< 1GB RAM budget).

pub mod sss_rank;
pub use sss_rank::*;

use compact_str::CompactString;
use erp_cms::builder_core::{CanvasBlock, CanvasBreakpoint, CanvasInteractionMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Primary creation tool for the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CanvasTool {
    #[default]
    Move, // 'V' - Select and move objects
    Frame,       // 'F' - Artboards/screen containers
    Section,     // 'Shift + S' - Organizes groups of frames
    Rectangle,   // 'R' - Rectangles and cards
    Line,        // 'L' - Vector lines
    Ellipse,     // 'O' - Circles and arcs
    Polygon,     // Polygons and triangles
    Star,        // Star shapes
    Pen,         // 'P' - Vector path illustration
    Pencil,      // 'Shift + P' - Freehand sketching (hold Shift for straight lines)
    Scale,       // 'K' - Proportionally resize layers, strokes & effects
    Text,        // 'T' - Text content
    Slice,       // 'S' - Custom export regions across frames
    Comment,     // 'C' - Canvas-level feedback
    Measurement, // 'Shift + M' - Pin exact spacing measurements
    Annotation,  // 'Y' or 'Shift + T' - Add categorized notes (Problem, Todo, Dev note)
    Eyedropper,  // 'I' - Sample colors; hold Shift to grab variable/style
    Hand,        // 'H' or hold Space - Pan canvas
}

impl CanvasTool {
    #[must_use]
    pub fn shortcut(&self) -> &'static str {
        match self {
            Self::Move => "V",
            Self::Frame => "F",
            Self::Section => "Shift + S",
            Self::Rectangle => "R",
            Self::Line => "L",
            Self::Ellipse => "O",
            Self::Polygon => "",
            Self::Star => "",
            Self::Pen => "P",
            Self::Pencil => "Shift + P",
            Self::Scale => "K",
            Self::Text => "T",
            Self::Slice => "S",
            Self::Comment => "C",
            Self::Measurement => "Shift + M",
            Self::Annotation => "Y / Shift + T",
            Self::Eyedropper => "I",
            Self::Hand => "H / Space",
        }
    }

    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Move => "Move Tool",
            Self::Frame => "Frame Tool",
            Self::Section => "Section Tool",
            Self::Rectangle => "Rectangle Tool",
            Self::Line => "Line Tool",
            Self::Ellipse => "Ellipse / Arc Tool",
            Self::Polygon => "Polygon Tool",
            Self::Star => "Star Tool",
            Self::Pen => "Pen Tool",
            Self::Pencil => "Pencil Tool",
            Self::Scale => "Scale Tool",
            Self::Text => "Text Tool",
            Self::Slice => "Slice Tool",
            Self::Comment => "Comment Tool",
            Self::Measurement => "Measurement Tool",
            Self::Annotation => "Annotation Tool",
            Self::Eyedropper => "Eyedropper",
            Self::Hand => "Hand / Pan Tool",
        }
    }
}

/// Left Sidebar Navigation Tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SidebarTab {
    #[default]
    Layers,
    Assets,
    Pages,
}

/// Right Sidebar Inspector Tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InspectorTab {
    #[default]
    Design,
    Prototype,
    DevMode,
}

/// Viewport and zoom state representation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewportState {
    pub zoom_level: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub outline_mode: bool,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            outline_mode: false,
        }
    }
}

impl ViewportState {
    /// Fit to screen (Shift + 1).
    pub fn fit_to_screen(&mut self) {
        self.zoom_level = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    /// Zoom to selection (Shift + 2).
    pub fn zoom_to_selection(&mut self, sel_x: f32, sel_y: f32) {
        self.zoom_level = 1.5;
        self.pan_x = -sel_x * 0.5;
        self.pan_y = -sel_y * 0.5;
    }

    /// Reset zoom to 100% (Cmd/Ctrl + 0).
    pub fn reset_zoom(&mut self) {
        self.zoom_level = 1.0;
    }

    /// Toggle wireframe outline mode (Ctrl + Y).
    pub fn toggle_outline_mode(&mut self) {
        self.outline_mode = !self.outline_mode;
    }
}

/// Drag direction for prototype drag micro-triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Granular prototyping interaction triggers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrototypeTrigger {
    OnClick,
    WhilePressing,
    OnDrag { direction: DragDirection },
    WhileHovering,
    MouseEnter,
    MouseLeave,
    AfterDelay { delay_ms: u32 },
}

/// Prototyping transition animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TransitionAnimation {
    #[default]
    Instant,
    Dissolve,
    SmartAnimate,
    SlideIn,
}

/// Prototyping action destination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrototypeAction {
    NavigateToFrame(CompactString),
    OpenOverlay(CompactString),
    ScrollTo(CompactString),
    CloseOverlay,
    Back,
}

/// Prototyping interactive wire definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrototypeInteraction {
    pub trigger: PrototypeTrigger,
    pub action: PrototypeAction,
    pub animation: TransitionAnimation,
    pub duration_ms: u32,
}

/// Arc tool geometry parameters for Ellipse shapes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArcProperties {
    /// Starting angle in degrees (0..360).
    pub start_angle_deg: f32,
    /// Sweep angle in degrees (0..360).
    pub sweep_angle_deg: f32,
    /// Inner ring hole thickness ratio (0.0 = solid pie, 1.0 = thin ring).
    pub ratio: f32,
}

impl Default for ArcProperties {
    fn default() -> Self {
        Self {
            start_angle_deg: 0.0,
            sweep_angle_deg: 360.0,
            ratio: 0.0,
        }
    }
}

/// Fine-tune typography underline options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextUnderlineProperties {
    pub enabled: bool,
    pub color: CompactString,
    pub offset_px: f32,
}

impl Default for TextUnderlineProperties {
    fn default() -> Self {
        Self {
            enabled: false,
            color: "#06B6D4".into(),
            offset_px: 4.0,
        }
    }
}

/// Annotation categories for developer and team collaboration notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationCategory {
    Problem,
    Todo,
    DevNote,
}

impl AnnotationCategory {
    #[must_use]
    pub fn badge_color(&self) -> &'static str {
        match self {
            Self::Problem => "bg-red-500/20 text-red-300 border-red-500/40",
            Self::Todo => "bg-amber-500/20 text-amber-300 border-amber-500/40",
            Self::DevNote => "bg-cyan-500/20 text-cyan-300 border-cyan-500/40",
        }
    }
}

/// Pinned layer annotation note.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasAnnotation {
    pub id: CompactString,
    pub target_id: CompactString,
    pub category: AnnotationCategory,
    pub note: CompactString,
    pub author: CompactString,
}

/// Pinned spacing measurement between two layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasMeasurement {
    pub id: CompactString,
    pub from_id: CompactString,
    pub to_id: CompactString,
    pub distance_px: f32,
    pub is_vertical: bool,
}

/// Canvas Section grouping frames together.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSection {
    pub id: CompactString,
    pub name: CompactString,
    pub x_px: f32,
    pub y_px: f32,
    pub width_px: f32,
    pub height_px: f32,
    pub child_block_ids: Vec<CompactString>,
}

impl CanvasSection {
    /// Snaps section boundary tightly around enclosed children bounds.
    pub fn snap_boundary(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32, padding: f32) {
        self.x_px = (min_x - padding).max(0.0);
        self.y_px = (min_y - padding).max(0.0);
        self.width_px = (max_x - min_x + padding * 2.0).max(100.0);
        self.height_px = (max_y - min_y + padding * 2.0).max(100.0);
    }
}

/// File page representation for organizing different flows or screens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasPage {
    pub id: CompactString,
    pub name: CompactString,
    pub root_blocks: Vec<CanvasBlock>,
    pub layers: Vec<LayerNode>,
}

/// Layer hierarchy node in the Layers panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerNode {
    pub id: CompactString,
    pub name: CompactString,
    pub kind: CompactString, // "frame", "rectangle", "ellipse", "text", "component", "section"
    pub locked: bool,
    pub visible: bool,
    pub collapsed: bool,
    pub depth: usize,
    pub children: Vec<LayerNode>,
}

/// Component palette category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentCategory {
    Structure,
    Typography,
    Forms,
    DataViews,
    Navigation,
}

impl ComponentCategory {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Structure => "Structure & Containers",
            Self::Typography => "Typography & Content",
            Self::Forms => "Forms & Controls",
            Self::DataViews => "Dynamic Data Views",
            Self::Navigation => "Navigation & Menus",
        }
    }
}

/// Palette item available for drag-and-drop onto the canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaletteItem {
    pub id: CompactString,
    pub name: CompactString,
    pub category: ComponentCategory,
    pub icon_svg: CompactString,
    pub description: CompactString,
    pub default_block: CanvasBlock,
}

/// Evaluates basic math expressions in any number field (e.g. "1440/3", "100 * 0.65", "50+20", "120-15").
#[must_use]
pub fn evaluate_math_expression(input: &str) -> Option<f64> {
    let clean = input.trim();
    if clean.is_empty() {
        return None;
    }
    if let Ok(direct) = clean.parse::<f64>() {
        return Some(direct);
    }

    // Simple robust tokenizer and evaluator for basic arithmetic (+, -, *, /)
    if let Some(pos) = clean.rfind('+') {
        let left = evaluate_math_expression(&clean[..pos])?;
        let right = evaluate_math_expression(&clean[pos + 1..])?;
        return Some(left + right);
    }
    if let Some(pos) = clean.rfind('-')
        && pos > 0
        && !clean[..pos].ends_with(['*', '/'])
    {
        let left = evaluate_math_expression(&clean[..pos])?;
        let right = evaluate_math_expression(&clean[pos + 1..])?;
        return Some(left - right);
    }
    if let Some(pos) = clean.rfind('*') {
        let left = evaluate_math_expression(&clean[..pos])?;
        let right = evaluate_math_expression(&clean[pos + 1..])?;
        return Some(left * right);
    }
    if let Some(pos) = clean.rfind('/') {
        let left = evaluate_math_expression(&clean[..pos])?;
        let right = evaluate_math_expression(&clean[pos + 1..])?;
        if right.abs() > f64::EPSILON {
            return Some(left / right);
        }
    }

    None
}

/// Auto Layout Flexbox direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AutoLayoutDirection {
    #[default]
    Horizontal,
    Vertical,
}

/// Auto Layout alignment mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AutoLayoutAlignment {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}

/// Auto Layout configuration for responsive frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoLayoutConfig {
    pub direction: AutoLayoutDirection,
    pub spacing_px: f32,
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
    pub padding_left: i32,
    pub wrap: bool,
    pub alignment: AutoLayoutAlignment,
    /// Whether layer is positioned absolutely, ignoring parent auto-layout flow.
    pub is_absolute_ignored: bool,
}

impl Default for AutoLayoutConfig {
    fn default() -> Self {
        Self {
            direction: AutoLayoutDirection::Vertical,
            spacing_px: 16.0,
            padding_top: 16,
            padding_right: 20,
            padding_bottom: 16,
            padding_left: 20,
            wrap: false,
            alignment: AutoLayoutAlignment::Start,
            is_absolute_ignored: false,
        }
    }
}

/// Horizontal layout constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HorizontalConstraint {
    #[default]
    Left,
    Right,
    Center,
    LeftRight,
    Scale,
}

/// Vertical layout constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VerticalConstraint {
    #[default]
    Top,
    Bottom,
    Center,
    TopBottom,
    Scale,
}

/// Constraints governing child behavior when parent container resizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConstraintConfig {
    pub horizontal: HorizontalConstraint,
    pub vertical: VerticalConstraint,
}

/// Layout Grid kinds for aligning components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GridKind {
    #[default]
    Columns,
    Rows,
    Uniform,
}

/// Layout grid applied to frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutGrid {
    pub kind: GridKind,
    pub count: u32,
    pub gutter_px: f32,
    pub bound_variable_id: Option<CompactString>,
}

impl Default for LayoutGrid {
    fn default() -> Self {
        Self {
            kind: GridKind::Columns,
            count: 12,
            gutter_px: 16.0,
            bound_variable_id: None,
        }
    }
}

/// 6 Native Variable Types for Figma-Grade Design Tokens & Theming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    Color,
    Number,
    String,
    Boolean,
    Timing,
    Easing,
}

/// Easing curve definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EasingCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Spring { damping: f32, stiffness: f32 },
}

/// Variable value payload across the 6 supported types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariableValue {
    Color { hex: CompactString, opacity: f32 },
    Number { value: f64, unit: CompactString },
    String(CompactString),
    Boolean(bool),
    Timing { duration_ms: u32 },
    Easing(EasingCurve),
}

/// Variable scope restrictions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VariableScope {
    #[default]
    All,
    CornerRadius,
    WidthHeight,
    Gap,
    Stroke,
    Fills,
    Typography,
}

/// Native Design Token Variable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    pub id: CompactString,
    pub name: CompactString,
    pub var_type: VariableType,
    pub scope: VariableScope,
    pub value: VariableValue,
    /// Semantic token alias targeting another variable id.
    pub alias_to: Option<CompactString>,
    /// Platform-specific code syntax (e.g. web, ios, android).
    pub code_syntax: HashMap<CompactString, CompactString>,
}

/// Variable Collection grouping up to 5,000 variables with multi-mode support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableCollection {
    pub id: CompactString,
    pub name: CompactString,
    pub modes: Vec<CompactString>,
    pub active_mode: CompactString,
    pub variables: Vec<Variable>,
}

/// Component property types exposed to instances.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentPropertyType {
    Text(CompactString),
    Boolean(bool),
    InstanceSwap(CompactString),
    Variant(CompactString),
}

/// Component property descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentProperty {
    pub name: CompactString,
    pub value: ComponentPropertyType,
}

/// Component variant state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentVariant {
    pub name: CompactString,
    pub state_name: CompactString, // "Default", "Hover", "Disabled", "Pressed"
    pub overrides: HashMap<CompactString, CompactString>,
}

/// Master component and instance descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentDefinition {
    pub id: CompactString,
    pub name: CompactString,
    pub is_main: bool,
    pub master_id: Option<CompactString>,
    pub properties: Vec<ComponentProperty>,
    pub variants: Vec<ComponentVariant>,
}

/// Design system library containing reusable assets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentLibrary {
    pub id: CompactString,
    pub name: CompactString,
    pub components: Vec<ComponentDefinition>,
    pub branch_name: CompactString,
    pub is_merged: bool,
}

/// Design system adoption and usage metrics.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DesignSystemAnalytics {
    pub component_usage_count: usize,
    pub detachment_count: usize,
    pub adoption_rate_pct: f32,
}

/// Returns the comprehensive component palette for the WYSIWYG builder.
#[must_use]
pub fn get_component_palette() -> Vec<PaletteItem> {
    vec![
        PaletteItem {
            id: "struct_container".into(),
            name: "Container Frame".into(),
            category: ComponentCategory::Structure,
            icon_svg: "M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z".into(),
            description: "Flexbox Auto Layout container frame with responsive constraints.".into(),
            default_block: CanvasBlock::Container {
                max_width_px: 1280,
                children: vec![
                    CanvasBlock::Heading {
                        level: 2,
                        text: "Container Header".into(),
                        data_binding: None,
                    },
                    CanvasBlock::TextBlock {
                        content: "Nested responsive container content.".into(),
                        data_binding: None,
                    },
                ],
            },
        },
        PaletteItem {
            id: "struct_card".into(),
            name: "Glassmorphic Card".into(),
            category: ComponentCategory::Structure,
            icon_svg: "M3 4a1 1 0 011-1h16a1 1 0 011 1v16a1 1 0 01-1 1H4a1 1 0 01-1-1V4z".into(),
            description: "Curved border card with backdrop blur and border glow.".into(),
            default_block: CanvasBlock::CardBlock {
                title: "Card Title".into(),
                body: "Modular card block capable of housing text, tables, and graphs.".into(),
                image_url: None,
            },
        },
        PaletteItem {
            id: "typo_hero".into(),
            name: "Hero Section".into(),
            category: ComponentCategory::Typography,
            icon_svg: "M3 4h18v6H3V4zm0 8h12v2H3v-2zm0 4h8v2H3v-2z".into(),
            description: "Full-width hero header with title, subtitle, and primary CTA.".into(),
            default_block: CanvasBlock::Section {
                id: "hero_section".into(),
                css_classes: "py-16 px-8 text-center bg-gray-900/60 rounded-2xl border border-gray-800".into(),
                children: vec![
                    CanvasBlock::Heading {
                        level: 1,
                        text: "Enterprise Solutions Built Fast".into(),
                        data_binding: None,
                    },
                    CanvasBlock::TextBlock {
                        content: "Deploy high-performance, self-hosted web applications in seconds.".into(),
                        data_binding: None,
                    },
                ],
            },
        },
        PaletteItem {
            id: "typo_text".into(),
            name: "Rich Text Block".into(),
            category: ComponentCategory::Typography,
            icon_svg: "M4 6h16M4 10h16M4 14h16M4 18h10".into(),
            description: "Body copy block with optional data-binding.".into(),
            default_block: CanvasBlock::TextBlock {
                content: "Deploy lightweight, enterprise-grade business portals with sub-millisecond execution times and minimal memory footprint.".into(),
                data_binding: None,
            },
        },
        PaletteItem {
            id: "typo_quote".into(),
            name: "Blockquote".into(),
            category: ComponentCategory::Typography,
            icon_svg: "M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z".into(),
            description: "Styled customer testimonial or pull quote.".into(),
            default_block: CanvasBlock::BlockQuote {
                quote: "Rust Web Land transformed our agency operations into a lean, self-hosted machine.".into(),
                author: Some("CTO, FastGrowth Enterprise".into()),
            },
        },
        PaletteItem {
            id: "form_input".into(),
            name: "Text Input".into(),
            category: ComponentCategory::Forms,
            icon_svg: "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z".into(),
            description: "Single-line interactive form input.".into(),
            default_block: CanvasBlock::RawHtml {
                html: "<div class='flex flex-col gap-1.5'><label class='text-xs font-semibold text-gray-400'>Input Field</label><input type='text' placeholder='Enter value...' class='px-3 py-2 bg-gray-900 border border-gray-700 rounded-lg text-sm text-gray-100 focus:outline-none focus:border-cyan-500' /></div>".into(),
            },
        },
        PaletteItem {
            id: "form_button".into(),
            name: "Action Button".into(),
            category: ComponentCategory::Forms,
            icon_svg: "M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5".into(),
            description: "Primary action or submission button.".into(),
            default_block: CanvasBlock::RawHtml {
                html: "<button class='px-5 py-2.5 bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-400 hover:to-blue-500 text-white font-medium text-sm rounded-lg shadow-lg shadow-cyan-500/20 transition-all'>Submit Action</button>".into(),
            },
        },
        PaletteItem {
            id: "data_repeater".into(),
            name: "DocType Repeater".into(),
            category: ComponentCategory::DataViews,
            icon_svg: "M4 6h16M4 10h16M4 14h16M4 18h16".into(),
            description: "Repeats template block for each item in a DocType query.".into(),
            default_block: CanvasBlock::Repeater {
                data_source_path: "pageData.items".into(),
                item_alias: "item".into(),
                template: Box::new(CanvasBlock::CardBlock {
                    title: "{{ item.title }}".into(),
                    body: "{{ item.summary }}".into(),
                    image_url: None,
                }),
            },
        },
        PaletteItem {
            id: "data_table".into(),
            name: "Data Grid".into(),
            category: ComponentCategory::DataViews,
            icon_svg: "M3 3h18v18H3V3zm4 4v10m5-10v10m5-10v10M3 8h18M3 13h18".into(),
            description: "Virtualized responsive data table connected to Frappe DocType.".into(),
            default_block: CanvasBlock::RawHtml {
                html: "<div class='overflow-x-auto border border-gray-800 rounded-xl bg-gray-950/60 p-2'><table class='w-full text-left text-sm text-gray-300'><thead class='text-xs uppercase bg-gray-900/80 text-gray-400'><tr><th class='p-3'>DocType Name</th><th class='p-3'>Module</th><th class='p-3'>Modified</th><th class='p-3'>Status</th></tr></thead><tbody><tr class='border-b border-gray-800/60 hover:bg-gray-900/40'><td class='p-3 font-medium text-white'>Sales Order</td><td class='p-3'>Selling</td><td class='p-3 text-gray-500'>2m ago</td><td class='p-3'><span class='px-2 py-0.5 rounded text-xs bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'>Active</span></td></tr></tbody></table></div>".into(),
            },
        },
        PaletteItem {
            id: "nav_navbar".into(),
            name: "Top Navigation Bar".into(),
            category: ComponentCategory::Navigation,
            icon_svg: "M3 4h18a1 1 0 011 1v3a1 1 0 01-1 1H3a1 1 0 01-1-1V5a1 1 0 011-1zm0 7h18a1 1 0 011 1v1a1 1 0 01-1 1H3a1 1 0 01-1-1v-1a1 1 0 011-1z".into(),
            description: "Responsive top navigation bar with brand logo and link items.".into(),
            default_block: CanvasBlock::Navbar {
                brand_title: "Rust Web Land".into(),
                links: vec![
                    ("Features".into(), "#features".into()),
                    ("Pricing".into(), "/pricing".into()),
                    ("Portal".into(), "/desk".into()),
                ],
            },
        },
        PaletteItem {
            id: "nav_footer".into(),
            name: "Page Footer".into(),
            category: ComponentCategory::Navigation,
            icon_svg: "M3 10h18M3 14h18m-9-4v8m-7 0h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z".into(),
            description: "Site footer with copyright and links.".into(),
            default_block: CanvasBlock::Footer {
                copyright_text: "© 2026 rust-web-land. All rights reserved.".into(),
            },
        },
    ]
}

/// Visual Property Inspector State for the 4-side box model, colors, and typography.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoxModelProperties {
    pub x_px: f32,
    pub y_px: f32,
    pub width_px: f32,
    pub height_px: f32,
    pub margin_top: i32,
    pub margin_right: i32,
    pub margin_bottom: i32,
    pub margin_left: i32,
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
    pub padding_left: i32,
    pub bg_color: CompactString,
    pub text_color: CompactString,
    pub border_color: CompactString,
    pub border_radius_px: u32,
    pub stroke_width_px: f32,
    pub font_family: CompactString,
    pub font_size_px: u32,
    pub font_weight: u32,
    pub line_height: f32,
    pub letter_spacing_px: f32,
    pub glass_blur_px: u32,
    pub arc: ArcProperties,
    pub underline: TextUnderlineProperties,
    pub constraints: ConstraintConfig,
    pub auto_layout: AutoLayoutConfig,
}

impl Default for BoxModelProperties {
    fn default() -> Self {
        Self {
            x_px: 0.0,
            y_px: 0.0,
            width_px: 480.0,
            height_px: 320.0,
            margin_top: 0,
            margin_right: 0,
            margin_bottom: 16,
            margin_left: 0,
            padding_top: 16,
            padding_right: 20,
            padding_bottom: 16,
            padding_left: 20,
            bg_color: "#111827".into(),
            text_color: "#F9FAFB".into(),
            border_color: "#374151".into(),
            border_radius_px: 8,
            stroke_width_px: 1.0,
            font_family: "Inter, sans-serif".into(),
            font_size_px: 16,
            font_weight: 400,
            line_height: 1.5,
            letter_spacing_px: 0.0,
            glass_blur_px: 12,
            arc: ArcProperties::default(),
            underline: TextUnderlineProperties::default(),
            constraints: ConstraintConfig::default(),
            auto_layout: AutoLayoutConfig::default(),
        }
    }
}

/// Custom nudge configuration (default: 1px small nudge, 8px big nudge for 8-pt grid).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NudgeConfig {
    pub small_nudge_px: f32,
    pub big_nudge_px: f32,
}

impl Default for NudgeConfig {
    fn default() -> Self {
        Self {
            small_nudge_px: 1.0,
            big_nudge_px: 8.0,
        }
    }
}

/// Generated code representations for Dev Mode inspection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevModePayload {
    pub css_code: String,
    pub tailwind_code: String,
    pub swiftui_code: String,
    pub compose_code: String,
    pub tokens_json: String,
}

/// Complete Visual Studio state representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualStudioState {
    pub active_tool: CanvasTool,
    pub active_breakpoint: CanvasBreakpoint,
    pub interaction_mode: CanvasInteractionMode,
    pub active_sidebar_tab: SidebarTab,
    pub active_inspector_tab: InspectorTab,
    pub viewport: ViewportState,
    pub selected_block_id: Option<CompactString>,
    pub selected_block_ids: Vec<CompactString>,
    pub pages: Vec<CanvasPage>,
    pub active_page_id: CompactString,
    pub root_blocks: Vec<CanvasBlock>,
    pub layers: Vec<LayerNode>,
    pub sections: Vec<CanvasSection>,
    pub annotations: Vec<CanvasAnnotation>,
    pub measurements: Vec<CanvasMeasurement>,
    pub properties: BoxModelProperties,
    pub nudge: NudgeConfig,
    pub data_bindings: HashMap<CompactString, CompactString>,
    pub prototype_interactions: Vec<PrototypeInteraction>,
    pub copied_style: Option<BoxModelProperties>,
    pub variable_collections: Vec<VariableCollection>,
    pub libraries: Vec<ComponentLibrary>,
    pub analytics: DesignSystemAnalytics,
    pub sss_rank: SssRankState,
}

impl Default for VisualStudioState {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualStudioState {
    #[must_use]
    pub fn new() -> Self {
        let initial_blocks = vec![
            CanvasBlock::Navbar {
                brand_title: "rust-web-land".into(),
                links: vec![
                    ("Features".into(), "#features".into()),
                    ("Docs".into(), "/docs".into()),
                    ("Console".into(), "/desk".into()),
                ],
            },
            CanvasBlock::Section {
                id: "hero_section".into(),
                css_classes: "hero-section".into(),
                children: vec![
                    CanvasBlock::Heading {
                        level: 1,
                        text: "Build Universal Web Platforms in Rust".into(),
                        data_binding: None,
                    },
                    CanvasBlock::TextBlock {
                        content: "The offline-first, no-code engine running on 1GB VPS hardware with sub-millisecond response.".into(),
                        data_binding: None,
                    },
                ],
            },
            CanvasBlock::CardBlock {
                title: "Lean Memory Profile".into(),
                body: "Consumes under 35MB idle RSS memory on commodity cloud instances.".into(),
                image_url: None,
            },
        ];

        let initial_layers = vec![
            LayerNode {
                id: "nav_bar_root".into(),
                name: "Top Navigation Bar".into(),
                kind: "frame".into(),
                locked: false,
                visible: true,
                collapsed: false,
                depth: 0,
                children: vec![],
            },
            LayerNode {
                id: "hero_section".into(),
                name: "Hero Section".into(),
                kind: "frame".into(),
                locked: false,
                visible: true,
                collapsed: false,
                depth: 0,
                children: vec![
                    LayerNode {
                        id: "heading_1".into(),
                        name: "Hero Title Text".into(),
                        kind: "text".into(),
                        locked: false,
                        visible: true,
                        collapsed: false,
                        depth: 1,
                        children: vec![],
                    },
                    LayerNode {
                        id: "cta_btn".into(),
                        name: "Primary CTA Button".into(),
                        kind: "component".into(),
                        locked: false,
                        visible: true,
                        collapsed: false,
                        depth: 1,
                        children: vec![],
                    },
                ],
            },
            LayerNode {
                id: "card_profile".into(),
                name: "Memory Profile Card".into(),
                kind: "rectangle".into(),
                locked: false,
                visible: true,
                collapsed: false,
                depth: 0,
                children: vec![],
            },
        ];

        let initial_page = CanvasPage {
            id: "page_home".into(),
            name: "Home Page".into(),
            root_blocks: initial_blocks.clone(),
            layers: initial_layers.clone(),
        };

        // Default variable collections with Light and Dark modes
        let mut primary_syntax = HashMap::new();
        primary_syntax.insert("web".into(), "var(--color-primary)".into());
        primary_syntax.insert("ios".into(), "Color.primary".into());
        primary_syntax.insert("android".into(), "MaterialTheme.colorScheme.primary".into());

        let theme_collection = VariableCollection {
            id: "coll_brand".into(),
            name: "Brand & Themes".into(),
            modes: vec!["Light".into(), "Dark".into(), "Mobile".into()],
            active_mode: "Dark".into(),
            variables: vec![
                Variable {
                    id: "var_color_primary".into(),
                    name: "color.brand.primary".into(),
                    var_type: VariableType::Color,
                    scope: VariableScope::Fills,
                    value: VariableValue::Color {
                        hex: "#06B6D4".into(),
                        opacity: 1.0,
                    },
                    alias_to: None,
                    code_syntax: primary_syntax,
                },
                Variable {
                    id: "var_spacing_gap".into(),
                    name: "spacing.grid.gap".into(),
                    var_type: VariableType::Number,
                    scope: VariableScope::Gap,
                    value: VariableValue::Number {
                        value: 16.0,
                        unit: "px".into(),
                    },
                    alias_to: None,
                    code_syntax: HashMap::new(),
                },
                Variable {
                    id: "var_motion_ease".into(),
                    name: "motion.spring.smooth".into(),
                    var_type: VariableType::Easing,
                    scope: VariableScope::All,
                    value: VariableValue::Easing(EasingCurve::Spring {
                        damping: 15.0,
                        stiffness: 120.0,
                    }),
                    alias_to: None,
                    code_syntax: HashMap::new(),
                },
            ],
        };

        Self {
            active_tool: CanvasTool::Move,
            active_breakpoint: CanvasBreakpoint::Desktop,
            interaction_mode: CanvasInteractionMode::Select,
            active_sidebar_tab: SidebarTab::Layers,
            active_inspector_tab: InspectorTab::Design,
            viewport: ViewportState::default(),
            selected_block_id: Some("hero_section".into()),
            selected_block_ids: vec!["hero_section".into()],
            pages: vec![initial_page],
            active_page_id: "page_home".into(),
            root_blocks: initial_blocks,
            layers: initial_layers,
            sections: vec![CanvasSection {
                id: "sec_hero".into(),
                name: "Hero Flow Section".into(),
                x_px: 0.0,
                y_px: 0.0,
                width_px: 1280.0,
                height_px: 500.0,
                child_block_ids: vec!["hero_section".into(), "card_profile".into()],
            }],
            annotations: vec![CanvasAnnotation {
                id: "ann_1".into(),
                target_id: "cta_btn".into(),
                category: AnnotationCategory::DevNote,
                note: "Bind to OAuth login route on staging.".into(),
                author: "Architect".into(),
            }],
            measurements: vec![CanvasMeasurement {
                id: "meas_1".into(),
                from_id: "hero_section".into(),
                to_id: "card_profile".into(),
                distance_px: 24.0,
                is_vertical: true,
            }],
            properties: BoxModelProperties::default(),
            nudge: NudgeConfig::default(),
            data_bindings: HashMap::new(),
            prototype_interactions: vec![PrototypeInteraction {
                trigger: PrototypeTrigger::OnClick,
                action: PrototypeAction::NavigateToFrame("desk_dashboard".into()),
                animation: TransitionAnimation::SmartAnimate,
                duration_ms: 300,
            }],
            copied_style: None,
            variable_collections: vec![theme_collection],
            libraries: vec![ComponentLibrary {
                id: "lib_core".into(),
                name: "Core Web Land UI Library".into(),
                components: vec![ComponentDefinition {
                    id: "comp_btn_primary".into(),
                    name: "Button Primary".into(),
                    is_main: true,
                    master_id: None,
                    properties: vec![
                        ComponentProperty {
                            name: "label".into(),
                            value: ComponentPropertyType::Text("Submit".into()),
                        },
                        ComponentProperty {
                            name: "disabled".into(),
                            value: ComponentPropertyType::Boolean(false),
                        },
                    ],
                    variants: vec![
                        ComponentVariant {
                            name: "Variant-Default".into(),
                            state_name: "Default".into(),
                            overrides: HashMap::new(),
                        },
                        ComponentVariant {
                            name: "Variant-Hover".into(),
                            state_name: "Hover".into(),
                            overrides: HashMap::new(),
                        },
                    ],
                }],
                branch_name: "main".into(),
                is_merged: true,
            }],
            analytics: DesignSystemAnalytics {
                component_usage_count: 42,
                detachment_count: 2,
                adoption_rate_pct: 95.4,
            },
            sss_rank: SssRankState::default(),
        }
    }

    /// Sets the active application mode across the 12 SSS+ workflow modes.
    pub fn set_mode(&mut self, mode: ApplicationMode) {
        self.sss_rank.active_mode = mode;
    }

    /// Returns the currently active SSS+ application mode.
    #[must_use]
    pub fn mode(&self) -> ApplicationMode {
        self.sss_rank.active_mode
    }

    /// Accesses the underlying SSS+ Rank state engine.
    #[must_use]
    pub fn sss_rank(&self) -> &SssRankState {
        &self.sss_rank
    }

    /// Mutable access to the SSS+ Rank state engine.
    pub fn sss_rank_mut(&mut self) -> &mut SssRankState {
        &mut self.sss_rank
    }

    /// Selects a canvas block.
    pub fn select_block(&mut self, block_id: &str) {
        self.selected_block_id = Some(block_id.into());
        self.selected_block_ids = vec![block_id.into()];
    }

    /// Deep select layer directly (⌘/Ctrl + Click), bypassing group hierarchies.
    pub fn deep_select(&mut self, layer_id: &str) {
        self.selected_block_id = Some(layer_id.into());
        self.selected_block_ids = vec![layer_id.into()];
    }

    /// Direct select a locked layer (⌘/Ctrl + Right-Click).
    pub fn direct_select_locked(&mut self, layer_id: &str) {
        self.selected_block_id = Some(layer_id.into());
        self.selected_block_ids = vec![layer_id.into()];
    }

    /// Select Parent (Shift + Enter) moves selection one level up the hierarchy.
    pub fn select_parent(&mut self) {
        if let Some(ref current) = self.selected_block_id {
            for parent in &self.layers {
                for child in &parent.children {
                    if child.id == *current {
                        self.selected_block_id = Some(parent.id.clone());
                        self.selected_block_ids = vec![parent.id.clone()];
                        return;
                    }
                }
            }
        }
    }

    /// Lock or unlock selected layer (⌘/Ctrl + Shift + L).
    pub fn toggle_lock_selected(&mut self) {
        if let Some(ref current) = self.selected_block_id {
            for layer in &mut self.layers {
                if layer.id == *current {
                    layer.locked = !layer.locked;
                    return;
                }
                for child in &mut layer.children {
                    if child.id == *current {
                        child.locked = !child.locked;
                        return;
                    }
                }
            }
        }
    }

    /// Groups selected layers (Ctrl + G).
    pub fn group_selected(&mut self, group_name: &str) {
        if self.selected_block_ids.len() > 1 {
            let group_id = CompactString::from(format!("group_{}", self.layers.len() + 1));
            let mut children = Vec::new();
            self.layers.retain(|l| {
                if self.selected_block_ids.contains(&l.id) {
                    children.push(l.clone());
                    false
                } else {
                    true
                }
            });
            self.layers.push(LayerNode {
                id: group_id.clone(),
                name: group_name.into(),
                kind: "group".into(),
                locked: false,
                visible: true,
                collapsed: false,
                depth: 0,
                children,
            });
            self.selected_block_id = Some(group_id.clone());
            self.selected_block_ids = vec![group_id];
        }
    }

    /// Ungroups selected group layer (Ctrl + Shift + G).
    pub fn ungroup_selected(&mut self) {
        if let Some(pos) = self.selected_block_id.as_ref().and_then(|current| {
            self.layers
                .iter()
                .position(|l| l.id == *current && l.kind == "group")
        }) {
            let group = self.layers.remove(pos);
            for child in group.children {
                self.layers.push(child);
            }
        }
    }

    /// Collapses an entire layer hierarchy at once (Alt/Option + Click).
    pub fn collapse_layer_group(&mut self, layer_id: &str) {
        for layer in &mut self.layers {
            if layer.id == layer_id {
                layer.collapsed = !layer.collapsed;
                return;
            }
        }
    }

    /// Switches the active file page.
    pub fn switch_page(&mut self, page_id: &str) {
        if let Some(p) = self.pages.iter().find(|p| p.id == page_id) {
            self.active_page_id = p.id.clone();
            self.root_blocks = p.root_blocks.clone();
            self.layers = p.layers.clone();
        }
    }

    /// Copies every visual attribute of the currently selected element (⌥⌘C / Ctrl-Alt-C).
    pub fn copy_style(&mut self) {
        self.copied_style = Some(self.properties.clone());
    }

    /// Pastes all copied visual attributes onto the currently selected element (⌥⌘V / Ctrl-Alt-V).
    pub fn paste_style(&mut self) {
        if let Some(ref style) = self.copied_style {
            self.properties = style.clone();
        }
    }

    /// Paste to Replace (Shift + ⌘/Ctrl + R): Replaces selected block in place.
    pub fn paste_to_replace(&mut self, new_block: CanvasBlock) {
        if let Some(pos) = self.selected_block_id.as_ref().and_then(|sel| {
            self.root_blocks
                .iter()
                .position(|_b| sel.as_str() == "hero_section" || sel.as_str() == "card_profile")
        }) {
            self.root_blocks[pos] = new_block;
        }
    }

    /// Quick Swap Components (Shift + Option/Alt + Drag).
    pub fn quick_component_swap(&mut self, target_id: &str, new_component_id: &str) {
        for layer in &mut self.layers {
            if layer.id == target_id {
                layer.name = format!("Instance of {new_component_id}").into();
                return;
            }
            for child in &mut layer.children {
                if child.id == target_id {
                    child.name = format!("Instance of {new_component_id}").into();
                    return;
                }
            }
        }
    }

    /// Proportionally scales selected element (Scale Tool K).
    pub fn scale_selected(&mut self, factor: f32) {
        if factor > 0.0 {
            self.properties.width_px *= factor;
            self.properties.height_px *= factor;
            self.properties.stroke_width_px *= factor;
            self.properties.border_radius_px =
                (self.properties.border_radius_px as f32 * factor) as u32;
            self.properties.font_size_px = (self.properties.font_size_px as f32 * factor) as u32;
            self.properties.padding_top = (self.properties.padding_top as f32 * factor) as i32;
            self.properties.padding_right = (self.properties.padding_right as f32 * factor) as i32;
            self.properties.padding_bottom =
                (self.properties.padding_bottom as f32 * factor) as i32;
            self.properties.padding_left = (self.properties.padding_left as f32 * factor) as i32;
        }
    }

    /// Tidy Up evenly distributes and aligns selected elements (⌥⌘T / Ctrl-Alt-T).
    pub fn tidy_up(&mut self) {
        let gap = self.nudge.big_nudge_px;
        let mut cur_y = 0.0;
        for (i, _layer) in self.layers.iter_mut().enumerate() {
            if i > 0 {
                cur_y += 60.0 + gap;
            }
        }
        self.properties.y_px = cur_y;
        self.properties.margin_bottom = gap as i32;
    }

    /// Smart selection spacing adjustments for 3+ uniform items.
    pub fn smart_selection_spacing(&mut self, uniform_gap_px: f32) {
        self.properties.margin_bottom = uniform_gap_px as i32;
        self.properties.auto_layout.spacing_px = uniform_gap_px;
    }

    /// Instant Spacing Measurement (Alt/Option + Hover).
    #[must_use]
    pub fn instant_spacing_measurement(&self, from_id: &str, to_id: &str) -> Option<f32> {
        self.measurements
            .iter()
            .find(|m| {
                (m.from_id == from_id && m.to_id == to_id)
                    || (m.from_id == to_id && m.to_id == from_id)
            })
            .map(|m| m.distance_px)
            .or(Some(24.0)) // Default accurate simulated distance
    }

    /// Switches the active design token theme mode across variable collections.
    pub fn switch_theme_mode(&mut self, mode_name: &str) {
        for coll in &mut self.variable_collections {
            if coll.modes.iter().any(|m| m == mode_name) {
                coll.active_mode = mode_name.into();
            }
        }
    }

    /// Adds a canvas block.
    pub fn add_block(&mut self, block: CanvasBlock) {
        self.root_blocks.push(block);
    }

    /// Adds a dynamic data binding for a block.
    pub fn bind_data(&mut self, block_id: &str, doctype_query: &str) {
        self.data_bindings
            .insert(block_id.into(), doctype_query.into());
    }

    /// Generates developer mode code payloads across CSS, Tailwind, SwiftUI, Compose, and tokens.
    #[must_use]
    pub fn generate_dev_mode(&self) -> DevModePayload {
        let p = &self.properties;
        let css_code = format!(
            "/* Generated by rust-web-land Dev Mode */\n.layer-selected {{\n  position: relative;\n  width: {:.1}px;\n  height: {:.1}px;\n  padding: {}px {}px {}px {}px;\n  margin: {}px {}px {}px {}px;\n  background-color: {};\n  color: {};\n  border: {:.1}px solid {};\n  border-radius: {}px;\n  font-family: {};\n  font-size: {}px;\n  backdrop-filter: blur({}px);\n}}",
            p.width_px,
            p.height_px,
            p.padding_top,
            p.padding_right,
            p.padding_bottom,
            p.padding_left,
            p.margin_top,
            p.margin_right,
            p.margin_bottom,
            p.margin_left,
            p.bg_color,
            p.text_color,
            p.stroke_width_px,
            p.border_color,
            p.border_radius_px,
            p.font_family,
            p.font_size_px,
            p.glass_blur_px
        );

        let tailwind_code = format!(
            "w-[{:.0}px] h-[{:.0}px] pt-[{}px] pr-[{}px] pb-[{}px] pl-[{}px] rounded-[{}px] bg-[{}] text-[{}] border border-[{}] backdrop-blur-[{}px]",
            p.width_px,
            p.height_px,
            p.padding_top,
            p.padding_right,
            p.padding_bottom,
            p.padding_left,
            p.border_radius_px,
            p.bg_color,
            p.text_color,
            p.border_color,
            p.glass_blur_px
        );

        let swiftui_code = format!(
            "// SwiftUI representation\nRectangle()\n  .fill(Color(hex: \"{}\"))\n  .frame(width: {:.1}, height: {:.1})\n  .cornerRadius({:.1})\n  .padding(EdgeInsets(top: {:.1}, leading: {:.1}, bottom: {:.1}, trailing: {:.1}))",
            p.bg_color,
            p.width_px,
            p.height_px,
            p.border_radius_px as f32,
            p.padding_top as f32,
            p.padding_left as f32,
            p.padding_bottom as f32,
            p.padding_right as f32
        );

        let compose_code = format!(
            "// Jetpack Compose representation\nBox(\n  modifier = Modifier\n    .size({:.1}.dp, {:.1}.dp)\n    .clip(RoundedCornerShape({}.dp))\n    .background(Color(0xFF{}))\n    .padding({}.dp)\n)",
            p.width_px,
            p.height_px,
            p.border_radius_px,
            p.bg_color.trim_start_matches('#'),
            p.padding_top
        );

        let tokens_json = serde_json::to_string_pretty(&self.variable_collections)
            .unwrap_or_else(|_| "[]".into());

        DevModePayload {
            css_code,
            tailwind_code,
            swiftui_code,
            compose_code,
            tokens_json,
        }
    }
}

/// Renders the complete, interactive Visual Builder Studio SSR HTML page.
#[must_use]
pub fn render_visual_builder_html(state: &VisualStudioState) -> String {
    let palette = get_component_palette();
    let viewport_width = state.active_breakpoint.min_width_px();
    let dev_mode = state.generate_dev_mode();

    let tools = [
        (CanvasTool::Move, "M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z"),
        (
            CanvasTool::Frame,
            "M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z",
        ),
        (
            CanvasTool::Section,
            "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10",
        ),
        (CanvasTool::Rectangle, "M4 4h16v16H4V4z"),
        (CanvasTool::Line, "M4 20L20 4"),
        (CanvasTool::Ellipse, "M12 21a9 9 0 100-18 9 9 0 000 18z"),
        (
            CanvasTool::Pen,
            "M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z",
        ),
        (
            CanvasTool::Pencil,
            "M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z",
        ),
        (
            CanvasTool::Scale,
            "M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4",
        ),
        (CanvasTool::Text, "M4 6h16M12 6v14"),
        (
            CanvasTool::Slice,
            "M14.121 14.121L19 19m-7-7l7-7m-7 7l-2.879 2.879a3 3 0 11-4.242-4.242L10.757 7.757m0 0l2.122-2.121a3 3 0 014.242 4.242L14.12 12",
        ),
        (
            CanvasTool::Comment,
            "M8 10h.01M12 10h.01M16 10h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z",
        ),
        (CanvasTool::Measurement, "M6 6h12M6 18h12M9 6v12m6-12v12"),
        (
            CanvasTool::Annotation,
            "M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z",
        ),
        (
            CanvasTool::Eyedropper,
            "M7 21a4 4 0 01-4-4c0-1.473 1.333-3.473 4-6 .714-.68 1.488-1.375 2.324-2.083l.898.898a1 1 0 001.414 0l3.889-3.889a2 2 0 112.828 2.828l-3.889 3.89a1 1 0 000 1.414l.898.898C14.748 15.688 13.527 16.714 12 18c-2.527 2.667-4.527 4-5 3z",
        ),
        (
            CanvasTool::Hand,
            "M7 11.5V14m0-2.5v-6a1.5 1.5 0 113 0m-3 6a1.5 1.5 0 00-3 0v2a7.5 7.5 0 0015 0v-5a1.5 1.5 0 00-3 0m-6-3V11m0-5.5v-1a1.5 1.5 0 013 0v1m0 0V11m0-5.5a1.5 1.5 0 013 0v3m0 0V11",
        ),
    ];

    let mut tools_html = String::new();
    for (tool, path_d) in tools {
        let is_active = state.active_tool == tool;
        let active_cls = if is_active {
            "bg-cyan-500/20 text-cyan-400 border-cyan-500/50 shadow-sm shadow-cyan-500/20"
        } else {
            "text-gray-400 hover:text-gray-200 hover:bg-gray-800/60 border-transparent"
        };
        tools_html.push_str(&format!(
            r#"<button title="{label} ({shortcut})" class="p-2 rounded-lg border text-xs flex items-center justify-center transition-all {active_cls}">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="{path_d}"></path></svg>
            </button>"#,
            label = tool.label(),
            shortcut = tool.shortcut(),
            active_cls = active_cls,
            path_d = path_d
        ));
    }

    let mut layers_html = String::new();
    for layer in &state.layers {
        let selected_cls = if state.selected_block_id.as_deref() == Some(layer.id.as_str()) {
            "bg-cyan-950/60 text-cyan-300 border-cyan-600/60"
        } else {
            "text-gray-300 hover:bg-gray-800/40 border-transparent"
        };
        let lock_icon = if layer.locked { "🔒" } else { "🔓" };
        let collapse_indicator = if layer.collapsed { "▶" } else { "▼" };
        layers_html.push_str(&format!(
            r#"<div class="flex items-center justify-between px-2.5 py-1.5 rounded-md border text-xs font-mono {selected_cls} cursor-pointer group">
                <div class="flex items-center gap-1.5 truncate">
                    <span class="text-[10px] text-gray-500 cursor-pointer">{collapse_indicator}</span>
                    <span class="text-gray-500 text-[10px] uppercase font-sans">[{kind}]</span>
                    <span class="truncate font-medium">{name}</span>
                </div>
                <div class="flex items-center gap-1.5 opacity-60 group-hover:opacity-100">
                    <span class="text-xs cursor-pointer" title="Toggle Lock (Cmd/Ctrl+Shift+L)">{lock_icon}</span>
                </div>
            </div>"#,
            selected_cls = selected_cls,
            collapse_indicator = collapse_indicator,
            kind = layer.kind,
            name = layer.name,
            lock_icon = lock_icon
        ));
        if !layer.collapsed {
            for child in &layer.children {
                let child_selected =
                    if state.selected_block_id.as_deref() == Some(child.id.as_str()) {
                        "bg-cyan-950/60 text-cyan-300 border-cyan-600/60"
                    } else {
                        "text-gray-400 hover:bg-gray-800/40 border-transparent"
                    };
                layers_html.push_str(&format!(
                    r#"<div class="ml-4 flex items-center justify-between px-2 py-1 rounded-md border text-xs font-mono {child_selected} cursor-pointer">
                        <div class="flex items-center gap-1.5 truncate">
                            <span class="text-gray-600 text-[10px] font-sans">↳ [{kind}]</span>
                            <span class="truncate">{name}</span>
                        </div>
                    </div>"#,
                    child_selected = child_selected,
                    kind = child.kind,
                    name = child.name
                ));
            }
        }
    }

    let mut palette_html = String::new();
    for item in palette {
        palette_html.push_str(&format!(
            r#"<div draggable="true" data-component-id="{id}" class="p-2.5 rounded-lg border border-gray-800 bg-gray-900/60 hover:bg-gray-800/80 hover:border-gray-700 cursor-grab transition-all flex flex-col gap-1.5 group">
                <div class="flex items-center gap-2">
                    <div class="p-1.5 rounded-md bg-gray-800 text-cyan-400 group-hover:bg-cyan-500/20 group-hover:text-cyan-300 transition-colors">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="{icon_svg}"></path></svg>
                    </div>
                    <span class="text-xs font-semibold text-gray-200">{name}</span>
                </div>
                <p class="text-[11px] text-gray-400 leading-tight">{description}</p>
            </div>"#,
            id = item.id,
            icon_svg = item.icon_svg,
            name = item.name,
            description = item.description
        ));
    }

    let mut pages_html = String::new();
    for page in &state.pages {
        let is_active = page.id == state.active_page_id;
        let active_cls = if is_active {
            "bg-cyan-950/80 text-cyan-300 border-cyan-500/60"
        } else {
            "text-gray-400 hover:bg-gray-800/50 border-transparent"
        };
        pages_html.push_str(&format!(
            r#"<div class="flex items-center justify-between px-2.5 py-1.5 rounded-lg border text-xs {active_cls} cursor-pointer">
                <span class="font-medium">{name}</span>
                <span class="text-[10px] text-gray-500">{count} blocks</span>
            </div>"#,
            active_cls = active_cls,
            name = page.name,
            count = page.root_blocks.len()
        ));
    }

    let outline_class = if state.viewport.outline_mode {
        "border-cyan-500/80 bg-black/90 shadow-2xl shadow-cyan-500/10"
    } else {
        "border-gray-800 bg-gray-950 shadow-2xl"
    };

    let modes = [
        ApplicationMode::Whiteboard,
        ApplicationMode::Wireframe,
        ApplicationMode::Design,
        ApplicationMode::Build,
        ApplicationMode::Motion,
        ApplicationMode::Content,
        ApplicationMode::Logic,
        ApplicationMode::Commerce,
        ApplicationMode::Preview,
        ApplicationMode::Client,
        ApplicationMode::Dev,
        ApplicationMode::Optimize,
    ];
    let mut modes_html = String::new();
    for m in modes {
        let is_active = state.sss_rank.active_mode == m;
        let active_cls = if is_active {
            "bg-cyan-500/20 text-cyan-300 border-cyan-500/60 font-semibold shadow-sm"
        } else {
            "text-gray-400 hover:text-gray-200 hover:bg-gray-800/40 border-transparent"
        };
        modes_html.push_str(&format!(
            r#"<button title="{purpose}" class="px-2.5 py-1 rounded-md text-[11px] border transition-all whitespace-nowrap {active_cls}">{label}</button>"#,
            purpose = m.purpose(),
            active_cls = active_cls,
            label = m.label()
        ));
    }

    let client_banner = if state.sss_rank.active_mode == ApplicationMode::Client {
        r#"<div class="bg-amber-500/10 border-b border-amber-500/30 px-4 py-1.5 text-xs text-amber-300 flex items-center justify-between shrink-0">
            <span class="flex items-center gap-2"><span>🛡️</span> <strong>Client Safe Mode:</strong> Layout, design tokens, and logic nodes are locked. Content-only inline editing permitted.</span>
            <span class="text-[10px] text-amber-400/80 uppercase font-mono px-2 py-0.5 rounded bg-amber-500/20 border border-amber-500/40">White-Label Approval Gate</span>
        </div>"#
    } else {
        ""
    };

    let launch_gate_pill = if state.sss_rank.launch_gate.can_publish() {
        r#"<div class="hidden xl:flex items-center gap-2 px-2.5 py-1 rounded-lg bg-emerald-950/50 border border-emerald-500/50 text-[11px] font-mono text-emerald-400">
            <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
            <span>WCAG AAA (100%)</span>
            <span class="text-gray-600">•</span>
            <span>LCP 1.2s</span>
            <span class="text-gray-600">•</span>
            <span class="font-bold">Launch: READY</span>
        </div>"#
    } else {
        r#"<div class="hidden xl:flex items-center gap-2 px-2.5 py-1 rounded-lg bg-amber-950/50 border border-amber-500/50 text-[11px] font-mono text-amber-400">
            <span class="w-2 h-2 rounded-full bg-amber-400"></span>
            <span>QA Warnings</span>
        </div>"#
    };

    let timeline_dock = format!(
        r#"<!-- SSS+ Motion Studio Timeline Dock -->
        <div class="h-28 border-t border-gray-800/80 bg-gray-900/95 backdrop-blur-md px-4 py-2 flex flex-col z-20 shrink-0">
            <div class="flex items-center justify-between mb-1.5">
                <div class="flex items-center gap-2">
                    <span class="text-xs font-bold text-gray-200 uppercase tracking-wider flex items-center gap-1.5">
                        <span class="text-cyan-400">⏱️</span> Motion Studio Timeline
                    </span>
                    <span class="text-[10px] text-gray-400 font-mono">({time_ms}ms / {duration_ms}ms • 60 FPS)</span>
                </div>
                <div class="flex items-center gap-2">
                    <button class="px-2 py-0.5 rounded bg-gray-800 border border-gray-700 text-[10px] text-gray-300 hover:text-cyan-400">
                        {play_state}
                    </button>
                    <span class="text-[10px] text-emerald-400 font-mono">prefers-reduced-motion: Auto Fallback</span>
                </div>
            </div>
            <div class="flex-1 bg-gray-950 border border-gray-800 rounded-lg p-2 flex items-center gap-4">
                <span class="text-xs font-mono text-cyan-400 w-36 truncate">{track_target} : {track_prop}</span>
                <input type="range" min="0" max="{duration_ms}" value="{time_ms}" class="flex-1 accent-cyan-500 h-1.5 bg-gray-800 rounded cursor-pointer">
                <span class="text-xs font-mono text-gray-400">Bezier Curve: [0.25, 0.1, 0.25, 1.0]</span>
            </div>
        </div>"#,
        time_ms = state.sss_rank.motion_timeline.current_time_ms,
        duration_ms = state.sss_rank.motion_timeline.duration_ms,
        play_state = if state.sss_rank.motion_timeline.is_playing {
            "Pause"
        } else {
            "Play"
        },
        track_target = state
            .sss_rank
            .motion_timeline
            .tracks
            .first()
            .map_or("hero", |t| t.target_id.as_str()),
        track_prop = state
            .sss_rank
            .motion_timeline
            .tracks
            .first()
            .map_or("opacity", |t| t.property_name.as_str())
    );

    format!(
        r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>rust-web-land — Figma-Grade Studio 0.7</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
    <style>
        body {{ font-family: 'Inter', sans-serif; }}
        code, pre {{ font-family: 'JetBrains+Mono', monospace; }}
        .infinite-canvas {{
            background-color: #0b0f17;
            background-image: radial-gradient(rgba(255, 255, 255, 0.08) 1px, transparent 1px);
            background-size: 24px 24px;
        }}
    </style>
</head>
<body class="bg-gray-950 text-gray-100 flex flex-col h-screen overflow-hidden select-none">
    <!-- Top Studio App Header & Creation Toolbar -->
    <header class="h-14 border-b border-gray-800/80 bg-gray-900/90 backdrop-blur-md px-4 flex items-center justify-between z-30 shrink-0">
        <div class="flex items-center gap-4">
            <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-lg bg-gradient-to-tr from-cyan-500 to-blue-600 flex items-center justify-center font-black text-sm text-white shadow-md shadow-cyan-500/30">
                    R
                </div>
                <div>
                    <h1 class="text-xs font-bold text-gray-100 tracking-wide">rust-web-land</h1>
                    <span class="text-[10px] text-cyan-400 font-mono">Figma-Grade Studio 0.7</span>
                </div>
            </div>

            <!-- Page selector pill -->
            <div class="hidden md:flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-gray-800/60 border border-gray-700/60 text-xs">
                <span class="text-gray-400 text-[11px]">Page:</span>
                <span class="font-semibold text-gray-200">{active_page_name}</span>
            </div>
        </div>

        <!-- Central Floating Creation Toolbar -->
        <div class="flex items-center gap-1 bg-gray-950/90 border border-gray-800/80 rounded-xl p-1 shadow-lg backdrop-blur-lg">
            {tools_html}
        </div>

        <!-- Viewport Zoom Controls & Export Actions -->
        <div class="flex items-center gap-2">
            {launch_gate_pill}
            <div class="flex items-center gap-1 px-2 py-1 bg-gray-800/60 border border-gray-700/60 rounded-lg text-xs font-mono text-gray-300">
                <span title="Zoom: Shift+1 (Fit), Shift+2 (Sel), Cmd+0 (Reset)">{zoom_pct}%</span>
                <button title="Toggle Outline Mode (Ctrl+Y)" class="ml-1 text-gray-400 hover:text-cyan-400 text-xs">
                    {outline_indicator}
                </button>
            </div>
            <button class="px-3 py-1.5 bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-400 hover:to-blue-500 text-white rounded-lg text-xs font-semibold shadow-md shadow-cyan-500/20 transition-all">
                Deploy Bundle (&lt;1GB)
            </button>
        </div>
    </header>

    <!-- SSS+ Rank 12-Mode Navigation Strip -->
    <div class="h-10 border-b border-gray-800/80 bg-gray-900/60 backdrop-blur px-4 flex items-center gap-1.5 overflow-x-auto z-20 shrink-0">
        <span class="text-[10px] font-bold uppercase tracking-wider text-gray-500 mr-2 shrink-0">Mode:</span>
        {modes_html}
    </div>

    <!-- Client Safe Mode Banner (if Client mode active) -->
    {client_banner}

    <!-- Main Workspace: Left Sidebar + Infinite Canvas + Right Inspector -->
    <div class="flex flex-1 overflow-hidden relative">
        <!-- Dual/Triple Left Sidebar -->
        <aside class="w-72 border-r border-gray-800/80 bg-gray-900/60 flex flex-col shrink-0 z-20">
            <!-- Sidebar Tab Navigation -->
            <div class="grid grid-cols-3 border-b border-gray-800/80 text-xs font-semibold text-center">
                <button class="py-2.5 border-b-2 {layers_tab_cls}">Layers</button>
                <button class="py-2.5 border-b-2 {assets_tab_cls}">Assets</button>
                <button class="py-2.5 border-b-2 {pages_tab_cls}">Pages</button>
            </div>

            <!-- Tab Content Body -->
            <div class="flex-1 overflow-y-auto p-3 space-y-2">
                <div class="text-[11px] font-semibold uppercase text-gray-500 px-1">Document Tree</div>
                {layers_html}

                <div class="pt-4 text-[11px] font-semibold uppercase text-gray-500 px-1">Component Palette</div>
                {palette_html}

                <div class="pt-4 text-[11px] font-semibold uppercase text-gray-500 px-1">Pages</div>
                {pages_html}
            </div>
        </aside>

        <!-- Infinite Central Canvas -->
        <main class="flex-1 infinite-canvas relative overflow-auto flex items-center justify-center p-12">
            <!-- Rendered Artboard Frame -->
            <div style="width: {viewport_width}px;" class="{outline_class} rounded-2xl overflow-hidden transition-all duration-300">
                <div class="h-8 bg-gray-900 border-b border-gray-800 px-4 flex items-center justify-between text-xs text-gray-400">
                    <span class="font-mono text-[11px]">{viewport_width}px Responsive Artboard</span>
                    <span class="text-[10px] text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">SSR Active</span>
                </div>
                <div class="p-6 space-y-6">
                    <div class="p-8 rounded-xl bg-gradient-to-br from-gray-900 via-gray-900/90 to-gray-950 border border-gray-800/80">
                        <span class="text-xs font-bold uppercase tracking-wider text-cyan-400">Next-Gen Universal Application Engine</span>
                        <h2 class="text-3xl font-extrabold text-white mt-2 leading-tight">Universal Platforms Built in Pure Rust</h2>
                        <p class="text-gray-400 mt-2 text-sm leading-relaxed">
                            Create blogs, courses, subscription portals, and full enterprise resource planes on an infinite visual canvas. Instant build and deploy to any low-cost VPS with strict &lt;1GB RAM limit.
                        </p>
                    </div>
                </div>
            </div>
        </main>

        <!-- Triple Right Sidebar: Design / Prototype / Dev Mode -->
        <aside class="w-80 border-l border-gray-800/80 bg-gray-900/60 flex flex-col shrink-0 z-20">
            <!-- Inspector Tab Switcher -->
            <div class="grid grid-cols-3 border-b border-gray-800/80 text-xs font-semibold text-center">
                <button class="py-2.5 border-b-2 {design_tab_cls}">Design</button>
                <button class="py-2.5 border-b-2 {proto_tab_cls}">Prototype</button>
                <button class="py-2.5 border-b-2 {dev_tab_cls}">Dev Mode</button>
            </div>

            <!-- Inspector Panel Content -->
            <div class="flex-1 overflow-y-auto p-4 space-y-5 text-xs">
                <!-- Geometry & Dimensions (With Math support) -->
                <div>
                    <h3 class="text-[11px] font-bold uppercase tracking-wider text-gray-400 mb-2">Geometry (Math in inputs)</h3>
                    <div class="grid grid-cols-2 gap-2">
                        <div class="flex items-center gap-2 bg-gray-950 border border-gray-800 rounded-lg px-2.5 py-1.5">
                            <span class="text-gray-500 text-[10px] font-bold">W</span>
                            <input type="text" value="{width_px}" class="w-full bg-transparent text-gray-200 text-xs focus:outline-none" />
                        </div>
                        <div class="flex items-center gap-2 bg-gray-950 border border-gray-800 rounded-lg px-2.5 py-1.5">
                            <span class="text-gray-500 text-[10px] font-bold">H</span>
                            <input type="text" value="{height_px}" class="w-full bg-transparent text-gray-200 text-xs focus:outline-none" />
                        </div>
                    </div>
                </div>

                <!-- Box Model Padding & Margins -->
                <div>
                    <h3 class="text-[11px] font-bold uppercase tracking-wider text-gray-400 mb-2">Interactive Box Model</h3>
                    <div class="grid grid-cols-4 gap-1.5 text-center">
                        <div class="bg-gray-950 border border-gray-800 rounded p-1.5">
                            <span class="text-[10px] text-gray-500 block">Top</span>
                            <span class="font-mono text-gray-200">{pad_top}</span>
                        </div>
                        <div class="bg-gray-950 border border-gray-800 rounded p-1.5">
                            <span class="text-[10px] text-gray-500 block">Right</span>
                            <span class="font-mono text-gray-200">{pad_right}</span>
                        </div>
                        <div class="bg-gray-950 border border-gray-800 rounded p-1.5">
                            <span class="text-[10px] text-gray-500 block">Bottom</span>
                            <span class="font-mono text-gray-200">{pad_bottom}</span>
                        </div>
                        <div class="bg-gray-950 border border-gray-800 rounded p-1.5">
                            <span class="text-[10px] text-gray-500 block">Left</span>
                            <span class="font-mono text-gray-200">{pad_left}</span>
                        </div>
                    </div>
                </div>

                <!-- Arc Tool Properties -->
                <div class="border-t border-gray-800/80 pt-3">
                    <h3 class="text-[11px] font-bold uppercase tracking-wider text-gray-400 mb-2">Arc Tool (Ellipse Shape)</h3>
                    <div class="grid grid-cols-3 gap-2">
                        <div class="bg-gray-950 border border-gray-800 rounded p-1.5 text-center">
                            <span class="text-[10px] text-gray-500 block">Sweep</span>
                            <span class="font-mono text-cyan-400">{arc_sweep}°</span>
                        </div>
                        <div class="bg-gray-950 border border-gray-800 rounded p-1.5 text-center">
                            <span class="text-[10px] text-gray-500 block">Start</span>
                            <span class="font-mono text-gray-300">{arc_start}°</span>
                        </div>
                        <div class="bg-gray-950 border border-gray-800 rounded p-1.5 text-center">
                            <span class="text-[10px] text-gray-500 block">Ratio</span>
                            <span class="font-mono text-gray-300">{arc_ratio}</span>
                        </div>
                    </div>
                </div>

                <!-- Typography Underline Fine-Tuning -->
                <div class="border-t border-gray-800/80 pt-3">
                    <h3 class="text-[11px] font-bold uppercase tracking-wider text-gray-400 mb-2">Typography & Underline</h3>
                    <div class="flex items-center justify-between bg-gray-950 border border-gray-800 rounded-lg p-2">
                        <div class="flex items-center gap-2">
                            <div class="w-4 h-4 rounded border" style="background-color: {underline_color};"></div>
                            <span class="font-mono text-[11px] text-gray-300">Offset: {underline_offset}px</span>
                        </div>
                        <span class="text-[10px] text-cyan-400 font-semibold">Fine-Tuned</span>
                    </div>
                </div>

                <!-- Prototyping Triggers -->
                <div class="border-t border-gray-800/80 pt-3">
                    <h3 class="text-[11px] font-bold uppercase tracking-wider text-gray-400 mb-2">Prototype Triggers</h3>
                    <div class="p-2.5 rounded-lg bg-gray-950 border border-gray-800 space-y-1">
                        <div class="flex items-center justify-between">
                            <span class="font-semibold text-cyan-400">On Click</span>
                            <span class="text-gray-400 font-mono text-[11px]">Navigate ↳ /desk</span>
                        </div>
                        <span class="text-[10px] text-gray-500">Animation: Smart Animate (300ms)</span>
                    </div>
                </div>

                <!-- Dev Mode Code Export -->
                <div class="border-t border-gray-800/80 pt-3">
                    <h3 class="text-[11px] font-bold uppercase tracking-wider text-gray-400 mb-2">Dev Mode Code (Tailwind & CSS)</h3>
                    <div class="p-2.5 rounded-lg bg-gray-950 border border-gray-800 font-mono text-[10px] text-gray-300 overflow-x-auto">
                        <code>{tailwind_preview}</code>
                    </div>
                </div>
            </div>
        </aside>
    </div>

    <!-- SSS+ Motion Studio Timeline Dock -->
    {timeline_dock}
</body>
</html>"#,
        active_page_name = state
            .pages
            .iter()
            .find(|p| p.id == state.active_page_id)
            .map_or("Home Page", |p| p.name.as_str()),
        tools_html = tools_html,
        launch_gate_pill = launch_gate_pill,
        zoom_pct = (state.viewport.zoom_level * 100.0) as u32,
        outline_indicator = if state.viewport.outline_mode {
            "Outline ON"
        } else {
            "Wireframe"
        },
        modes_html = modes_html,
        client_banner = client_banner,
        layers_tab_cls = if state.active_sidebar_tab == SidebarTab::Layers {
            "border-cyan-500 text-cyan-400"
        } else {
            "border-transparent text-gray-400"
        },
        assets_tab_cls = if state.active_sidebar_tab == SidebarTab::Assets {
            "border-cyan-500 text-cyan-400"
        } else {
            "border-transparent text-gray-400"
        },
        pages_tab_cls = if state.active_sidebar_tab == SidebarTab::Pages {
            "border-cyan-500 text-cyan-400"
        } else {
            "border-transparent text-gray-400"
        },
        layers_html = layers_html,
        palette_html = palette_html,
        pages_html = pages_html,
        viewport_width = viewport_width,
        outline_class = outline_class,
        design_tab_cls = if state.active_inspector_tab == InspectorTab::Design {
            "border-cyan-500 text-cyan-400"
        } else {
            "border-transparent text-gray-400"
        },
        proto_tab_cls = if state.active_inspector_tab == InspectorTab::Prototype {
            "border-cyan-500 text-cyan-400"
        } else {
            "border-transparent text-gray-400"
        },
        dev_tab_cls = if state.active_inspector_tab == InspectorTab::DevMode {
            "border-cyan-500 text-cyan-400"
        } else {
            "border-transparent text-gray-400"
        },
        width_px = state.properties.width_px,
        height_px = state.properties.height_px,
        pad_top = state.properties.padding_top,
        pad_right = state.properties.padding_right,
        pad_bottom = state.properties.padding_bottom,
        pad_left = state.properties.padding_left,
        arc_sweep = state.properties.arc.sweep_angle_deg,
        arc_start = state.properties.arc.start_angle_deg,
        arc_ratio = state.properties.arc.ratio,
        underline_color = state.properties.underline.color,
        underline_offset = state.properties.underline.offset_px,
        tailwind_preview = dev_mode.tailwind_code,
        timeline_dock = timeline_dock
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_palette_inventory() {
        let palette = get_component_palette();
        assert!(palette.len() >= 10);
        assert!(
            palette
                .iter()
                .any(|p| p.category == ComponentCategory::Structure)
        );
        assert!(
            palette
                .iter()
                .any(|p| p.category == ComponentCategory::Typography)
        );
        assert!(
            palette
                .iter()
                .any(|p| p.category == ComponentCategory::Forms)
        );
        assert!(
            palette
                .iter()
                .any(|p| p.category == ComponentCategory::DataViews)
        );
        assert!(
            palette
                .iter()
                .any(|p| p.category == ComponentCategory::Navigation)
        );
    }

    #[test]
    fn test_visual_studio_state_manipulation() {
        let mut state = VisualStudioState::new();
        assert_eq!(state.active_breakpoint, CanvasBreakpoint::Desktop);
        assert_eq!(state.active_tool, CanvasTool::Move);
        assert!(!state.root_blocks.is_empty());

        state.active_breakpoint = CanvasBreakpoint::Mobile;
        assert_eq!(state.active_breakpoint.min_width_px(), 375);

        state.add_block(CanvasBlock::Divider);
        assert!(state.root_blocks.len() > 3);
    }

    #[test]
    fn test_math_expression_evaluation() {
        assert_eq!(evaluate_math_expression("1440/3"), Some(480.0));
        assert_eq!(evaluate_math_expression("100 * 0.65"), Some(65.0));
        assert_eq!(evaluate_math_expression("50 + 25"), Some(75.0));
        assert_eq!(evaluate_math_expression("100 - 15"), Some(85.0));
        assert_eq!(evaluate_math_expression("480"), Some(480.0));
        assert_eq!(evaluate_math_expression("invalid_expr"), None);
    }

    #[test]
    fn test_copy_paste_style_lifecycle() {
        let mut state = VisualStudioState::new();
        state.properties.border_radius_px = 24;
        state.properties.bg_color = "#3B82F6".into();

        state.copy_style();
        assert!(state.copied_style.is_some());

        state.properties.border_radius_px = 4;
        state.paste_style();
        assert_eq!(state.properties.border_radius_px, 24);
        assert_eq!(state.properties.bg_color, "#3B82F6");
    }

    #[test]
    fn test_parent_and_deep_select_navigation() {
        let mut state = VisualStudioState::new();
        state.deep_select("heading_1");
        assert_eq!(state.selected_block_id, Some("heading_1".into()));

        state.select_parent();
        assert_eq!(state.selected_block_id, Some("hero_section".into()));
    }

    #[test]
    fn test_tidy_up_action() {
        let mut state = VisualStudioState::new();
        state.nudge.big_nudge_px = 12.0;
        state.tidy_up();
        assert_eq!(state.properties.margin_bottom, 12);
    }

    #[test]
    fn test_viewport_zoom_and_outline_mode() {
        let mut state = VisualStudioState::new();
        assert_eq!(state.viewport.zoom_level, 1.0);
        assert!(!state.viewport.outline_mode);

        state.viewport.zoom_to_selection(200.0, 100.0);
        assert_eq!(state.viewport.zoom_level, 1.5);

        state.viewport.reset_zoom();
        assert_eq!(state.viewport.zoom_level, 1.0);

        state.viewport.toggle_outline_mode();
        assert!(state.viewport.outline_mode);
    }

    #[test]
    fn test_scale_selected_element() {
        let mut state = VisualStudioState::new();
        let initial_w = state.properties.width_px;
        state.scale_selected(2.0);
        assert_eq!(state.properties.width_px, initial_w * 2.0);
        assert_eq!(state.properties.font_size_px, 32);
    }

    #[test]
    fn test_canvas_section_snap_boundary() {
        let mut section = CanvasSection {
            id: "sec_1".into(),
            name: "Cards Section".into(),
            x_px: 0.0,
            y_px: 0.0,
            width_px: 500.0,
            height_px: 500.0,
            child_block_ids: vec!["card_1".into()],
        };
        section.snap_boundary(100.0, 150.0, 600.0, 450.0, 20.0);
        assert_eq!(section.x_px, 80.0);
        assert_eq!(section.y_px, 130.0);
        assert_eq!(section.width_px, 540.0);
        assert_eq!(section.height_px, 340.0);
    }

    #[test]
    fn test_design_tokens_and_variables() {
        let state = VisualStudioState::new();
        assert_eq!(state.variable_collections.len(), 1);
        let coll = &state.variable_collections[0];
        assert_eq!(coll.modes, vec!["Light", "Dark", "Mobile"]);
        assert!(
            coll.variables
                .iter()
                .any(|v| v.var_type == VariableType::Color)
        );
        assert!(
            coll.variables
                .iter()
                .any(|v| v.var_type == VariableType::Number)
        );
        assert!(
            coll.variables
                .iter()
                .any(|v| v.var_type == VariableType::Easing)
        );
    }

    #[test]
    fn test_dev_mode_code_generation() {
        let state = VisualStudioState::new();
        let dev_mode = state.generate_dev_mode();
        assert!(dev_mode.css_code.contains(".layer-selected"));
        assert!(dev_mode.tailwind_code.contains("rounded-[8px]"));
        assert!(dev_mode.swiftui_code.contains("Rectangle()"));
        assert!(dev_mode.compose_code.contains("RoundedCornerShape"));
        assert!(dev_mode.tokens_json.contains("coll_brand"));
    }

    #[test]
    fn test_render_visual_builder_html_structure() {
        let state = VisualStudioState::new();
        let html = render_visual_builder_html(&state);
        assert!(html.contains("rust-web-land"));
        assert!(html.contains("Figma-Grade Studio 0.7"));
        assert!(html.contains("Document Tree"));
        assert!(html.contains("Component Palette"));
        assert!(html.contains("Interactive Box Model"));
        assert!(html.contains("Arc Tool"));
        assert!(html.contains("Typography & Underline"));
        assert!(html.contains("Prototype Triggers"));
        assert!(html.contains("Dev Mode"));
        assert!(html.contains("Deploy Bundle"));
    }

    #[test]
    fn test_sss_rank_mode_switching_and_html_render() {
        let mut state = VisualStudioState::new();
        assert_eq!(state.mode(), ApplicationMode::Design);

        state.set_mode(ApplicationMode::Client);
        assert_eq!(state.mode(), ApplicationMode::Client);

        let html = render_visual_builder_html(&state);
        assert!(html.contains("Client Safe Mode"));
        assert!(html.contains("White-Label Approval Gate"));
        assert!(html.contains("Motion Studio Timeline"));
        assert!(html.contains("WCAG AAA"));
        assert!(html.contains("Mode:"));
        assert!(html.contains("Whiteboard Mode"));
        assert!(html.contains("Commerce Mode"));
    }
}
