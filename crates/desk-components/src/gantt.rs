//! Interactive Virtualized Gantt & CPM Critical Path Visualizer Model.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GanttTaskRow {
    pub task_id: usize,
    pub name: String,
    pub start_day: i64,
    pub duration_days: i64,
    pub total_float: i64,
    pub is_critical: bool,
    pub progress_percent: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GanttDependencyLink {
    pub from_task_id: usize,
    pub to_task_id: usize,
    pub lag_days: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GanttViewModel {
    pub project_name: String,
    pub total_duration_days: i64,
    pub tasks: Vec<GanttTaskRow>,
    pub dependencies: Vec<GanttDependencyLink>,
    pub buffer_penetration_percent: f64,
    pub buffer_zone: String, // "Green", "Yellow", "Red"
}

impl GanttViewModel {
    #[must_use]
    pub fn new(project_name: String, total_duration: i64) -> Self {
        Self {
            project_name,
            total_duration_days: total_duration,
            tasks: Vec::new(),
            dependencies: Vec::new(),
            buffer_penetration_percent: 0.0,
            buffer_zone: "Green".into(),
        }
    }

    pub fn add_task(&mut self, task: GanttTaskRow) {
        self.tasks.push(task);
    }

    pub fn add_link(&mut self, link: GanttDependencyLink) {
        self.dependencies.push(link);
    }

    /// Generates SVG markup for virtualized render.
    #[must_use]
    pub fn render_svg_preview(&self, row_height: f64, day_width: f64) -> String {
        let mut svg = String::new();
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"gantt-chart\">\n");

        for (idx, task) in self.tasks.iter().enumerate() {
            let y = idx as f64 * row_height;
            let x = task.start_day as f64 * day_width;
            let width = (task.duration_days as f64 * day_width).max(2.0);
            let color = if task.is_critical {
                "#ef4444"
            } else {
                "#3b82f6"
            };

            svg.push_str(&format!(
                "  <rect x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{}\" fill=\"{color}\" rx=\"4\" />\n",
                row_height - 6.0
            ));
        }

        svg.push_str("</svg>");
        svg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gantt_view_model_rendering() {
        let mut model = GanttViewModel::new("Aero Megaproject".into(), 45);
        model.add_task(GanttTaskRow {
            task_id: 1,
            name: "Structural Analysis".into(),
            start_day: 0,
            duration_days: 15,
            total_float: 0,
            is_critical: true,
            progress_percent: 100.0,
        });

        let svg = model.render_svg_preview(24.0, 10.0);
        assert!(svg.contains("fill=\"#ef4444\""));
    }
}
