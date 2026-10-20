use frappe_meta::{DocTypeSchema, FieldType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Client-side UI widget type mapped from server DocField AST.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FormFieldWidget {
    TextInput {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    NumberInput {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    CurrencyInput {
        fieldname: String,
        label: String,
        reqd: bool,
    },
    Checkbox {
        fieldname: String,
        label: String,
        default_checked: bool,
    },
    LinkDropdown {
        fieldname: String,
        label: String,
        target_doctype: String,
        endpoint: String,
        reqd: bool,
    },
    SelectDropdown {
        fieldname: String,
        label: String,
        options: Vec<String>,
        reqd: bool,
    },
    TableSubgrid {
        fieldname: String,
        label: String,
        child_doctype: String,
    },
}

/// Dynamic Form Component Model compiled from DocTypeSchema AST (Milestone 5.6).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DynamicFormModel {
    pub doctype_name: String,
    pub is_submittable: bool,
    pub widgets: Vec<FormFieldWidget>,
}

impl DynamicFormModel {
    /// Compiles a server DocTypeSchema AST into a client-side DynamicFormModel.
    #[must_use]
    pub fn from_schema(schema: &DocTypeSchema) -> Self {
        let mut widgets = Vec::new();

        for field in &schema.fields {
            if field.hidden {
                continue;
            }

            let widget = match &field.fieldtype {
                FieldType::Data
                | FieldType::DataWithPreset { .. }
                | FieldType::Text
                | FieldType::LongText
                | FieldType::SmallText
                | FieldType::ReadOnly
                | FieldType::Code { .. }
                | FieldType::TextEditor
                | FieldType::HtmlEditor
                | FieldType::MarkdownEditor
                | FieldType::Password => FormFieldWidget::TextInput {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::Int | FieldType::Float | FieldType::Percent | FieldType::Rating => {
                    FormFieldWidget::NumberInput {
                        fieldname: field.fieldname.clone(),
                        label: field.label.clone(),
                        reqd: field.reqd,
                    }
                }
                FieldType::Currency => FormFieldWidget::CurrencyInput {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
                FieldType::Check => FormFieldWidget::Checkbox {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    default_checked: field
                        .default_value
                        .as_ref()
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false),
                },
                FieldType::Link { target_doctype } => FormFieldWidget::LinkDropdown {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    target_doctype: target_doctype.clone(),
                    endpoint: format!(
                        "/api/v1/resource/{}",
                        target_doctype.to_lowercase().replace(' ', "_")
                    ),
                    reqd: field.reqd,
                },
                FieldType::Select { options } => FormFieldWidget::SelectDropdown {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    options: options.clone(),
                    reqd: field.reqd,
                },
                FieldType::Table { child_doctype } => FormFieldWidget::TableSubgrid {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    child_doctype: child_doctype.clone(),
                },
                _ => FormFieldWidget::TextInput {
                    fieldname: field.fieldname.clone(),
                    label: field.label.clone(),
                    reqd: field.reqd,
                },
            };

            widgets.push(widget);
        }

        Self {
            doctype_name: schema.name.clone(),
            is_submittable: schema.is_submittable,
            widgets,
        }
    }
}

/// Evaluates Frappe dynamic form `depends_on` visibility conditions.
#[must_use]
pub fn eval_depends_on(expr: &str, form_values: &HashMap<String, String>) -> bool {
    if expr.trim().is_empty() {
        return true;
    }

    expr.split("&&").all(|condition| {
        let trimmed = condition.trim();
        if let Some((k, v)) = trimmed.split_once("==") {
            let key = k.trim();
            let val = v.trim().trim_matches('"');
            form_values.get(key).map(|x| x == val).unwrap_or(false)
        } else if let Some((k, v)) = trimmed.split_once("!=") {
            let key = k.trim();
            let val = v.trim().trim_matches('"');
            form_values.get(key).map(|x| x != val).unwrap_or(true)
        } else {
            false
        }
    })
}

use dioxus::prelude::*;

/// Dioxus v7 Dynamic DocType Form Component with validation and dirty-state tracking.
#[component]
pub fn DocTypeForm(model: DynamicFormModel, initial_values: HashMap<String, String>) -> Element {
    let mut values = use_signal(move || initial_values);
    let mut is_dirty = use_signal(|| false);
    let mut errors = use_signal(HashMap::<String, String>::new);

    let dirty = *is_dirty.read();
    let error_map = errors.read().clone();

    rsx! {
        form { class: "desk-form-container max-w-4xl mx-auto p-6 bg-zinc-950 border border-zinc-800 rounded-xl shadow-xl text-zinc-100",
            div { class: "desk-form-header flex items-center justify-between pb-4 mb-6 border-b border-zinc-800",
                div { class: "flex items-center gap-3",
                    h2 { class: "text-xl font-bold tracking-tight text-white", "{model.doctype_name}" }
                    span {
                        class: if dirty { "text-xs px-2 py-0.5 rounded-full bg-amber-500/20 text-amber-300 border border-amber-500/30" } else { "text-xs px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30" },
                        if dirty { "● Unsaved changes" } else { "✓ Saved" }
                    }
                }
                div { class: "flex items-center gap-2",
                    button {
                        r#type: "button",
                        class: "px-3 py-1.5 text-xs font-medium rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 cursor-pointer",
                        onclick: move |_| {
                            is_dirty.set(false);
                            errors.write().clear();
                        },
                        "Discard"
                    }
                    if model.is_submittable {
                        button {
                            r#type: "button",
                            class: "btn-submit px-4 py-1.5 text-xs font-semibold rounded bg-blue-600 hover:bg-blue-500 text-white cursor-pointer disabled:opacity-50",
                            disabled: !error_map.is_empty(),
                            onclick: move |_| {
                                is_dirty.set(false);
                            },
                            "Submit"
                        }
                    }
                }
            }

            div { class: "desk-form-grid grid grid-cols-1 md:grid-cols-2 gap-4",
                for widget in model.widgets {
                    match widget {
                        FormFieldWidget::TextInput { fieldname, label, reqd } => {
                            let fn_clone = fieldname.clone();
                            let fn_err = fieldname.clone();
                            let err_msg = error_map.get(&fn_err).cloned();
                            rsx! {
                                div { class: "form-group flex flex-col gap-1.5",
                                    label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                        "{label}"
                                        if reqd { span { class: "text-red-400", "*" } }
                                    }
                                    input {
                                        r#type: "text",
                                        required: reqd,
                                        class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 placeholder-zinc-500 focus:border-blue-500 outline-none",
                                        value: values.read().get(&fieldname).cloned().unwrap_or_default(),
                                        oninput: move |evt| {
                                            let v = evt.value();
                                            if reqd && v.trim().is_empty() {
                                                errors.write().insert(fn_clone.clone(), "This field is required".into());
                                            } else {
                                                errors.write().remove(&fn_clone);
                                            }
                                            values.write().insert(fn_clone.clone(), v);
                                            is_dirty.set(true);
                                        }
                                    }
                                    if let Some(err) = err_msg {
                                        span { class: "text-[11px] text-red-400", "{err}" }
                                    }
                                }
                            }
                        },
                        FormFieldWidget::NumberInput { fieldname, label, reqd } => {
                            let fn_clone = fieldname.clone();
                            let fn_err = fieldname.clone();
                            let err_msg = error_map.get(&fn_err).cloned();
                            rsx! {
                                div { class: "form-group flex flex-col gap-1.5",
                                    label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                        "{label}"
                                        if reqd { span { class: "text-red-400", "*" } }
                                    }
                                    input {
                                        r#type: "number",
                                        required: reqd,
                                        class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 placeholder-zinc-500 focus:border-blue-500 outline-none",
                                        value: values.read().get(&fieldname).cloned().unwrap_or_default(),
                                        oninput: move |evt| {
                                            let v = evt.value();
                                            if reqd && v.trim().is_empty() {
                                                errors.write().insert(fn_clone.clone(), "This field is required".into());
                                            } else {
                                                errors.write().remove(&fn_clone);
                                            }
                                            values.write().insert(fn_clone.clone(), v);
                                            is_dirty.set(true);
                                        }
                                    }
                                    if let Some(err) = err_msg {
                                        span { class: "text-[11px] text-red-400", "{err}" }
                                    }
                                }
                            }
                        },
                        FormFieldWidget::CurrencyInput { fieldname, label, reqd } => {
                            let fn_clone = fieldname.clone();
                            let fn_err = fieldname.clone();
                            let err_msg = error_map.get(&fn_err).cloned();
                            rsx! {
                                div { class: "form-group flex flex-col gap-1.5",
                                    label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                        "{label}"
                                        if reqd { span { class: "text-red-400", "*" } }
                                    }
                                    input {
                                        r#type: "number",
                                        step: "0.01",
                                        required: reqd,
                                        class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 placeholder-zinc-500 focus:border-blue-500 outline-none font-mono",
                                        value: values.read().get(&fieldname).cloned().unwrap_or_default(),
                                        oninput: move |evt| {
                                            let v = evt.value();
                                            if reqd && v.trim().is_empty() {
                                                errors.write().insert(fn_clone.clone(), "This field is required".into());
                                            } else {
                                                errors.write().remove(&fn_clone);
                                            }
                                            values.write().insert(fn_clone.clone(), v);
                                            is_dirty.set(true);
                                        }
                                    }
                                    if let Some(err) = err_msg {
                                        span { class: "text-[11px] text-red-400", "{err}" }
                                    }
                                }
                            }
                        },
                        FormFieldWidget::Checkbox { fieldname, label, default_checked } => {
                            let fn_clone = fieldname.clone();
                            rsx! {
                                div { class: "form-group flex items-center gap-2 pt-5",
                                    input {
                                        r#type: "checkbox",
                                        class: "w-4 h-4 rounded bg-zinc-900 border-zinc-700 text-blue-600 focus:ring-0",
                                        checked: values.read().get(&fieldname).map(|v| v == "true").unwrap_or(default_checked),
                                        onchange: move |evt| {
                                            let v = if evt.checked() { "true" } else { "false" };
                                            values.write().insert(fn_clone.clone(), v.into());
                                            is_dirty.set(true);
                                        }
                                    }
                                    label { class: "text-xs font-medium text-zinc-300", "{label}" }
                                }
                            }
                        },
                        FormFieldWidget::SelectDropdown { fieldname, label, options, reqd } => {
                            let fn_clone = fieldname.clone();
                            rsx! {
                                div { class: "form-group flex flex-col gap-1.5",
                                    label { class: "text-xs font-medium text-zinc-300 flex items-center gap-1",
                                        "{label}"
                                        if reqd { span { class: "text-red-400", "*" } }
                                    }
                                    select {
                                        required: reqd,
                                        class: "px-3 py-2 text-sm bg-zinc-900 border border-zinc-700 rounded-md text-zinc-100 focus:border-blue-500 outline-none",
                                        onchange: move |evt| {
                                            let v = evt.value();
                                            values.write().insert(fn_clone.clone(), v);
                                            is_dirty.set(true);
                                        },
                                        for opt in options {
                                            option { value: "{opt}", "{opt}" }
                                        }
                                    }
                                }
                            }
                        },
                        _ => rsx! { div {} }
                    }
                }
            }
        }
    }
}

/// Helper rendering DocTypeForm into an HTML string using Dioxus 0.7 SSR.
pub fn render_doctype_form_component(
    model: &DynamicFormModel,
    initial_values: &HashMap<String, String>,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        DocTypeForm,
        DocTypeFormProps {
            model: model.clone(),
            initial_values: initial_values.clone(),
        },
    );
    vdom.rebuild_in_place();
    dioxus::ssr::render(&vdom)
}
