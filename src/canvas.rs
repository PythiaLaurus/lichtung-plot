//! Heterogeneous plot composition on one canvas.

use std::path::Path;

use crate::Error;

use crate::decorations::legend::{Legend, LegendEntry};
use crate::decorations::title_block::TitleBlock;
use crate::layout::canvas::{
    allocate_composed_grid, try_wrap_text, PanelRequirements, PANEL_HEADER_GAP,
};
use crate::layout::{CanvasLayout, CanvasLayoutSpec, PanelBox};
use crate::render::backend::{with_layer, Backend, Point2D};
use crate::render::svg_backend::SvgBackend;
use crate::scene::RenderedScene;
use crate::spec::{AxisDomain, DecorationMode};
use crate::theme::ResolvedTheme;
use crate::{CanvasLegendPolicy, Labels, Output, Plot, PlotKind, Theme};

#[cfg(test)]
thread_local! {
    pub(crate) static CANVAS_THEME_EVIDENCE: std::cell::RefCell<Vec<(usize, &'static str, usize)>> = const { std::cell::RefCell::new(Vec::new()) };
    pub(crate) static CANVAS_SCENE_RENDER_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn record_canvas_theme(owner: usize, stage: &'static str, theme: &ResolvedTheme) {
    CANVAS_THEME_EVIDENCE.with(|evidence| {
        evidence
            .borrow_mut()
            .push((owner, stage, std::ptr::from_ref(theme).addr()));
    });
}

struct PlannedCanvasPanel {
    index: usize,
    kind: PlotKind,
    canvas_cell: PanelBox,
    panel_plan: crate::decorations::plan::PanelPlan,
}

struct CanvasPlan {
    themes: ResolvedCanvasThemes,
    plots: Vec<Option<Plot>>,
    canvas_layout: CanvasLayout,
    full_outer: PanelBox,
    outer: PanelBox,
    shared_legend: Option<(String, Vec<LegendEntry>)>,
    panels: Vec<PlannedCanvasPanel>,
    text_panels: Vec<TextPanelPlan>,
    headers: Vec<CanvasHeaderPlan>,
    row_titles: Vec<CanvasHeaderPlan>,
}

struct ResolvedCanvasThemes {
    root: Box<ResolvedTheme>,
    plots: Vec<Option<ResolvedPlotTheme>>,
}

enum ResolvedPlotTheme {
    InheritedRoot,
    ExplicitPlot(Box<ResolvedTheme>),
}

impl ResolvedCanvasThemes {
    fn new(root: Theme, plots: &[Option<Plot>]) -> Self {
        Self {
            root: Box::new(root.resolve()),
            plots: plots
                .iter()
                .map(|plot| {
                    plot.as_ref().map(|plot| {
                        plot.theme_spec()
                            .map_or(ResolvedPlotTheme::InheritedRoot, |theme| {
                                ResolvedPlotTheme::ExplicitPlot(Box::new(theme.resolve()))
                            })
                    })
                })
                .collect(),
        }
    }

    fn root(&self) -> &ResolvedTheme {
        self.root.as_ref()
    }

    fn plot(&self, index: usize) -> Option<&ResolvedTheme> {
        self.plots.get(index)?.as_ref().map(|theme| match theme {
            ResolvedPlotTheme::InheritedRoot => self.root(),
            ResolvedPlotTheme::ExplicitPlot(theme) => theme.as_ref(),
        })
    }
}

struct CanvasHeaderPlan {
    text: String,
    x: f64,
    y: f64,
}

/// Categorical tick-label policy for composed canvas grids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasCategoryLabels {
    /// Render categorical tick labels in every panel.
    #[default]
    EveryPlot,
    /// Render categorical tick labels only in the first column of each row.
    FirstColumnPerRow,
}

#[derive(Debug, Clone)]
struct CanvasHeader {
    start: usize,
    span: usize,
    text: String,
}

struct TextPanelPlan {
    index: usize,
    cell: PanelBox,
    title: crate::layout::TextBlock,
    body: crate::layout::TextBlock,
}

/// Validated prose content for one composed-canvas grid cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPanel {
    title: String,
    body: String,
}

impl TextPanel {
    /// Construct a text panel with nonempty title and body.
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Result<Self, Error> {
        let title = title.into();
        let body = body.into();
        validate_text_panel_value(&title, "title")?;
        validate_text_panel_value(&body, "body")?;
        Ok(Self { title, body })
    }

    /// Panel title rendered above the body.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Panel body rendered with deterministic wrapping.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }
}

fn validate_text_panel_value(value: &str, role: &str) -> Result<(), Error> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(Error::Data(format!(
            "text panel {role} must be nonempty and have no surrounding whitespace"
        )));
    }
    if value.chars().any(char::is_control) {
        return Err(Error::Data(format!(
            "text panel {role} must not contain control characters"
        )));
    }
    Ok(())
}

fn canvas_header(
    start: usize,
    span: usize,
    text: String,
    limit: usize,
    role: &str,
) -> Result<CanvasHeader, Error> {
    if span == 0 || start.checked_add(span).is_none_or(|end| end > limit) {
        return Err(Error::Data(format!(
            "canvas {role} header range {start}..{} is outside 0..{limit}",
            start.saturating_add(span)
        )));
    }
    if text.trim().is_empty() || text.trim() != text || text.chars().any(char::is_control) {
        return Err(Error::Data(format!(
            "canvas {role} header must be nonempty, trimmed, and free of control characters"
        )));
    }
    Ok(CanvasHeader { start, span, text })
}

fn validate_header_overlap(
    headers: &[CanvasHeader],
    candidate: &CanvasHeader,
    role: &str,
) -> Result<(), Error> {
    let end = candidate.start + candidate.span;
    if headers.iter().any(|header| {
        let other_end = header.start + header.span;
        candidate.start < other_end && header.start < end
    }) {
        return Err(Error::Data(format!(
            "canvas {role} headers must not overlap"
        )));
    }
    Ok(())
}

/// A typed grid of independently configured charts rendered on one canvas.
pub struct Canvas {
    rows: usize,
    columns: usize,
    panels: Vec<Option<Plot>>,
    text_panels: Vec<Option<TextPanel>>,
    theme: Theme,
    legend_policy: CanvasLegendPolicy,
    shared_legend_gap: Option<f64>,
    width: u32,
    height: u32,
    title: Option<String>,
    subtitle: Option<String>,
    caption: Option<String>,
    labels: Option<Labels>,
    category_labels: CanvasCategoryLabels,
    column_headers: Vec<CanvasHeader>,
    row_headers: Vec<CanvasHeader>,
    row_titles: Vec<CanvasHeader>,
    x_range_groups: Vec<Vec<usize>>,
}

impl Canvas {
    /// Construct a non-empty panel grid.
    pub fn grid(rows: usize, columns: usize) -> Result<Self, Error> {
        if rows == 0 || columns == 0 {
            return Err(Error::Data(
                "canvas grid rows and columns must both be greater than zero".into(),
            ));
        }
        let panel_count = rows
            .checked_mul(columns)
            .ok_or_else(|| Error::Data("canvas grid dimensions exceed allocation limits".into()))?;
        Ok(Self {
            rows,
            columns,
            panels: (0..panel_count).map(|_| None).collect(),
            text_panels: (0..panel_count).map(|_| None).collect(),
            theme: Theme::Publication,
            legend_policy: CanvasLegendPolicy::PerPlot,
            shared_legend_gap: None,
            width: 960,
            height: 640,
            title: None,
            subtitle: None,
            caption: None,
            labels: None,
            category_labels: CanvasCategoryLabels::EveryPlot,
            column_headers: Vec::new(),
            row_headers: Vec::new(),
            row_titles: Vec::new(),
            x_range_groups: Vec::new(),
        })
    }

    /// Place one exact-family chart in an unoccupied zero-based grid cell.
    ///
    /// An already erased plot cannot be inserted:
    ///
    /// ```compile_fail
    /// use lichtung_plot::{Canvas, NumericValues, Plot};
    /// let erased: Plot = Plot::scatter(
    ///     NumericValues::new("x", [1.0]),
    ///     NumericValues::new("y", [2.0]),
    /// ).erase();
    /// let _ = Canvas::grid(1, 1).unwrap().panel(0, 0, erased);
    /// ```
    pub fn panel<K: crate::kind::ExactFamily>(
        mut self,
        row: usize,
        column: usize,
        plot: Plot<K>,
    ) -> Result<Self, Error> {
        if row >= self.rows || column >= self.columns {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is outside {}x{} grid",
                self.rows, self.columns
            )));
        }
        let index = row * self.columns + column;
        if self.panels[index].is_some() || self.text_panels[index].is_some() {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is already occupied"
            )));
        }
        self.panels[index] = Some(plot.erase());
        Ok(self)
    }

    /// Place one validated text panel in an unoccupied zero-based grid cell.
    pub fn text_panel(
        mut self,
        row: usize,
        column: usize,
        panel: TextPanel,
    ) -> Result<Self, Error> {
        if row >= self.rows || column >= self.columns {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is outside {}x{} grid",
                self.rows, self.columns
            )));
        }
        let index = row * self.columns + column;
        if self.panels[index].is_some() || self.text_panels[index].is_some() {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is already occupied"
            )));
        }
        self.text_panels[index] = Some(panel);
        Ok(self)
    }

    /// Set per-panel, shared, or hidden legend behavior.
    #[must_use]
    pub fn legend_policy(mut self, policy: CanvasLegendPolicy) -> Self {
        self.legend_policy = policy;
        self
    }

    /// Set the exact CSS-pixel gap between the panel grid and a shared legend.
    pub fn shared_legend_gap(mut self, gap: f64) -> Result<Self, Error> {
        if !gap.is_finite() || gap < 0.0 {
            return Err(Error::Data(
                "canvas shared legend gap must be finite and nonnegative".into(),
            ));
        }
        self.shared_legend_gap = Some(gap);
        Ok(self)
    }

    /// Add one measured header spanning contiguous canvas columns.
    pub fn column_header(
        mut self,
        start_column: usize,
        span: usize,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let header = canvas_header(start_column, span, text.into(), self.columns, "column")?;
        validate_header_overlap(&self.column_headers, &header, "column")?;
        self.column_headers.push(header);
        Ok(self)
    }

    /// Add one measured header spanning contiguous canvas rows.
    pub fn row_header(
        mut self,
        start_row: usize,
        span: usize,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let header = canvas_header(start_row, span, text.into(), self.rows, "row")?;
        validate_header_overlap(&self.row_headers, &header, "row")?;
        self.row_headers.push(header);
        Ok(self)
    }

    /// Add one measured title centered above every panel in one canvas row.
    pub fn row_title(mut self, row: usize, text: impl Into<String>) -> Result<Self, Error> {
        let title = canvas_header(row, 1, text.into(), self.rows, "row title")?;
        validate_header_overlap(&self.row_titles, &title, "row title")?;
        self.row_titles.push(title);
        Ok(self)
    }

    /// Configure duplicate categorical-label handling across the grid.
    #[must_use]
    pub const fn category_labels(mut self, policy: CanvasCategoryLabels) -> Self {
        self.category_labels = policy;
        self
    }

    /// Couple the automatic horizontal range of explicit panel cells.
    pub fn x_range_group(
        mut self,
        cells: impl IntoIterator<Item = (usize, usize)>,
    ) -> Result<Self, Error> {
        let mut indices = Vec::new();
        for (row, column) in cells {
            if row >= self.rows || column >= self.columns {
                return Err(Error::Data(format!(
                    "canvas x-range group cell ({row}, {column}) is outside {}x{} grid",
                    self.rows, self.columns
                )));
            }
            let index = row * self.columns + column;
            if indices.contains(&index)
                || self
                    .x_range_groups
                    .iter()
                    .any(|group| group.contains(&index))
            {
                return Err(Error::Data(format!(
                    "canvas x-range group repeats panel ({row}, {column})"
                )));
            }
            indices.push(index);
        }
        if indices.is_empty() {
            return Err(Error::Data(
                "canvas x-range group must contain at least one panel".into(),
            ));
        }
        self.x_range_groups.push(indices);
        Ok(self)
    }

    /// Set the composed canvas dimensions.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set one canvas-level title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set one canvas-level subtitle.
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set one canvas-level caption.
    #[must_use]
    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Apply one reusable automatic-label policy to every configured panel.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = Some(labels);
        self
    }

    fn render_preflighted(&self, plan: CanvasPlan, backend: &mut dyn Backend) -> Result<(), Error> {
        let mut backend =
            crate::orchestrator::DeferredBackend::new(backend, self.width, self.height);
        let rendered = self.render_planned_canvas(plan, &mut backend);
        backend.finish(rendered, "composed canvas")
    }

    /// Validate the complete composed canvas without opening a backend or creating files.
    ///
    /// This checks canvas and grid layout, every chart's plot-specific data and
    /// representability preflight, channel and interval contracts, shared legend
    /// compatibility, and nested self-managed plot restrictions.
    pub fn preflight(&self) -> Result<(), Error> {
        self.preflight_plan().map(drop)
    }

    #[cfg(test)]
    pub(crate) fn render_with_backend(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        let preflight = self.preflight_plan()?;
        self.render_preflighted(preflight, backend)
    }

    /// Render one immutable canonical composed SVG scene without creating a file.
    ///
    /// # Errors
    ///
    /// Returns an error when canvas preflight or SVG rendering fails.
    fn render_scene(&self) -> Result<RenderedScene, Error> {
        #[cfg(test)]
        CANVAS_SCENE_RENDER_CALLS.with(|calls| calls.set(calls.get() + 1));
        let preflight = self.preflight_plan()?;
        let mut backend = SvgBackend::in_memory(self.width, self.height);
        self.render_preflighted(preflight, &mut backend)?;
        RenderedScene::new(self.width, self.height, backend.into_output())
    }

    /// Render once, publish an outlined SVG, and return a reusable output.
    /// The output filename is completed with `.svg` after conversion.
    pub fn svg(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.svg(path)
    }

    /// Render once, publish SVG with selectable text, and return a reusable output.
    /// The output filename is completed with `.svg` after rendering.
    pub fn svg_preserve_text(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.svg_preserve_text(path)
    }

    /// Render once, publish a balanced 300-DPI PNG, and return a reusable output.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png(path)
    }

    /// Render once, publish a balanced source-sized PNG without density metadata.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png_preview(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png_preview(path)
    }

    /// Render once and publish a PNG using the exact caller-provided options.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png_with_options(
        self,
        path: impl AsRef<Path>,
        options: crate::PngOptions,
    ) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png_with_options(path, options)
    }

    fn validate_structure(&self) -> Result<(), Error> {
        if self.width < 160 || self.height < 120 {
            return Err(Error::Data("canvas must be at least 160x120 pixels".into()));
        }
        if self.panels.iter().all(Option::is_none) && self.text_panels.iter().all(Option::is_none) {
            return Err(Error::Data(
                "canvas requires at least one configured panel".into(),
            ));
        }
        if self.shared_legend_gap.is_some() && self.legend_policy != CanvasLegendPolicy::Shared {
            return Err(Error::Data(
                "canvas shared legend gap requires CanvasLegendPolicy::Shared".into(),
            ));
        }
        for plot in self.panels.iter().flatten() {
            let kind = plot.kind();
            plot.validate_configuration(kind)?;
            if plot.has_facets() {
                return Err(Error::Render(
                    "a faceted chart cannot be nested inside a composed canvas".into(),
                ));
            }
        }
        for title in &self.row_titles {
            let start = title.start * self.columns;
            let end = start + self.columns;
            if self.panels[start..end].iter().flatten().any(|plot| {
                plot.title_text().is_some_and(|text| !text.is_empty())
                    || plot.subtitle_text().is_some_and(|text| !text.is_empty())
            }) {
                return Err(Error::Data(format!(
                    "canvas row title cannot coexist with a panel title or subtitle in row {}",
                    title.start
                )));
            }
        }
        Ok(())
    }

    fn row_title_lines(
        &self,
        title: &CanvasHeader,
        cells: &[PanelBox],
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Vec<String>, Error> {
        let first = title.start * self.columns;
        let last = first + self.columns - 1;
        let maximum_width = (cells[last].x1 - cells[first].x0 - 16.0).max(1.0);
        try_wrap_text(
            &title.text,
            theme.title_font_size,
            maximum_width,
            "canvas row title",
        )
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "wrapped row-title line counts are bounded by the finite canvas"
    )]
    fn add_row_title_requirements(
        &self,
        requirements: &mut Vec<(usize, usize, PanelRequirements)>,
        cells: &[PanelBox],
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<(), Error> {
        for title in &self.row_titles {
            let lines = self.row_title_lines(title, cells, theme)?;
            let top = lines.len() as f64 * (f64::from(theme.title_font_size) + 6.0) + 8.0;
            requirements.push((
                title.start,
                0,
                PanelRequirements {
                    top,
                    ..PanelRequirements::default()
                },
            ));
        }
        Ok(())
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "wrapped row-title line indices are bounded by the finite canvas"
    )]
    fn plan_row_titles(
        &self,
        cells: &[PanelBox],
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Vec<CanvasHeaderPlan>, Error> {
        let mut plans = Vec::new();
        let line_height = f64::from(theme.title_font_size) + 6.0;
        for title in &self.row_titles {
            let first = title.start * self.columns;
            let last = first + self.columns - 1;
            let x = f64::midpoint(cells[first].x0, cells[last].x1);
            for (index, line) in self
                .row_title_lines(title, cells, theme)?
                .into_iter()
                .rev()
                .enumerate()
            {
                plans.push(CanvasHeaderPlan {
                    text: line,
                    x,
                    y: cells[first].y1 + 14.0 + index as f64 * line_height,
                });
            }
        }
        Ok(plans)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "composed preflight keeps shared legends, measurement, allocation, and final planning in one lifecycle"
    )]
    fn preflight_plan(&self) -> Result<CanvasPlan, Error> {
        let themes = ResolvedCanvasThemes::new(self.theme, &self.panels);
        #[cfg(test)]
        {
            record_canvas_theme(usize::MAX, "resolved", themes.root());
            for index in 0..themes.plots.len() {
                if let Some(theme) = themes.plot(index) {
                    record_canvas_theme(index, "resolved", theme);
                }
            }
        }
        self.validate_structure()?;
        #[cfg(test)]
        record_canvas_theme(usize::MAX, "layout", themes.root());
        let canvas_layout = CanvasLayout::measured_composed_canvas(
            self.width,
            self.height,
            themes.root(),
            CanvasLayoutSpec {
                has_title: self.title.as_ref().is_some_and(|value| !value.is_empty()),
                has_subtitle: self
                    .subtitle
                    .as_ref()
                    .is_some_and(|value| !value.is_empty()),
                has_caption: self.caption.as_ref().is_some_and(|value| !value.is_empty()),
                has_x_label: false,
                has_y_label: false,
            },
            (
                self.title.as_deref(),
                self.subtitle.as_deref(),
                self.caption.as_deref(),
            ),
        )?;
        let full_outer = canvas_layout.panel_box;
        let hue_groups = if self.legend_policy == CanvasLegendPolicy::Shared {
            self.canvas_hue_groups(full_outer, &themes)?
        } else {
            Vec::new()
        };
        let mut plots = self
            .panels
            .iter()
            .enumerate()
            .map(|(index, plot)| {
                plot.as_ref().map(|plot| {
                    let plot = self.labels.as_ref().map_or_else(
                        || plot.clone(),
                        |labels| plot.clone().labels(labels.clone()),
                    );
                    let plot = hue_groups
                        .iter()
                        .find(|(dimension, _)| Some(dimension.as_str()) == plot.hue_dimension_key())
                        .map_or_else(
                            || plot.clone(),
                            |(_, levels)| plot.clone().with_global_hue_levels(levels.clone()),
                        );
                    plot.with_canvas_size(self.width, self.height)
                        .with_render_scope(format!("panel-{index}"))
                })
            })
            .collect::<Vec<_>>();
        let shared_legend = if self.legend_policy == CanvasLegendPolicy::Shared {
            Self::preflight_shared_legend(&plots, &themes, full_outer)?
        } else {
            None
        };
        let mut outer = full_outer;
        if let Some((title, entries)) = &shared_legend {
            let legend = Legend::new(
                title.clone(),
                entries.clone(),
                f64::from(self.width),
                f64::from(self.height),
            );
            if legend.compute_box_height() > full_outer.height() {
                return Err(Error::Render(
                    "measured shared canvas legend is taller than the available canvas band".into(),
                ));
            }
            let gap = self.resolved_shared_legend_gap(&legend);
            outer.x1 -= legend.compute_box_width() + gap + legend.metrics.pad;
        }
        let header_theme_index = plots.iter().position(Option::is_some);
        let header_theme = header_theme_index
            .and_then(|index| themes.plot(index))
            .unwrap_or_else(|| themes.root());
        let header_outer = outer;
        let column_header_height = if self.column_headers.is_empty() {
            0.0
        } else {
            f64::from(header_theme.title_font_size) + 18.0
        };
        let row_header_width = self
            .row_headers
            .iter()
            .map(|header| {
                crate::layout::estimated_text_width(&header.text, header_theme.title_font_size)
            })
            .fold(0.0_f64, f64::max)
            + if self.row_headers.is_empty() {
                0.0
            } else {
                20.0
            };
        outer.y1 -= column_header_height;
        outer.x0 += row_header_width;
        if outer.width() < 80.0 || outer.height() < 60.0 {
            return Err(Error::Render(
                "canvas shared headers leave insufficient panel space; enlarge the canvas".into(),
            ));
        }
        let provisional =
            allocate_composed_grid(outer, self.rows, self.columns, &[], "canvas layout")?;
        if self.category_labels == CanvasCategoryLabels::FirstColumnPerRow {
            for row in 0..self.rows {
                let first_index = row * self.columns;
                let first = plots[first_index].as_ref().ok_or_else(|| {
                    Error::Data(format!(
                        "canvas row {row} requires a configured first-column panel for shared category labels"
                    ))
                })?;
                let expected_theme = themes
                    .plot(first_index)
                    .expect("configured canvas panel has a resolved theme");
                #[cfg(test)]
                record_canvas_theme(
                    first_index,
                    "shared-category-label-validation",
                    expected_theme,
                );
                let expected =
                    plot_category_domain(first, provisional[first_index], expected_theme)?;
                for column in 1..self.columns {
                    let index = first_index + column;
                    let plot = plots[index].as_ref().ok_or_else(|| {
                        Error::Data(format!(
                            "canvas row {row} requires panel ({row}, {column}) for shared category labels"
                        ))
                    })?;
                    let theme = themes
                        .plot(index)
                        .expect("configured canvas panel has a resolved theme");
                    #[cfg(test)]
                    record_canvas_theme(index, "shared-category-label-validation", theme);
                    if plot_category_domain(plot, provisional[index], theme)? != expected {
                        return Err(Error::Data(format!(
                            "canvas row {row} category domains differ across panels"
                        )));
                    }
                    plots[index] = Some(plot.clone().with_y_tick_labels_visible(false));
                }
            }
        }
        if !self.x_range_groups.is_empty() {
            let mut ranges = vec![None; plots.len()];
            for (index, plot) in plots.iter().enumerate() {
                let Some(plot) = plot else { continue };
                let kind = plot.kind();
                let theme = themes
                    .plot(index)
                    .expect("configured canvas panel has a resolved theme");
                #[cfg(test)]
                record_canvas_theme(index, "grouped-x-range-plan", theme);
                let result = plot.preflight_panel(kind, provisional[index], theme)?;
                ranges[index] = Some(result.layout.projection.x_range());
            }
            for group in &self.x_range_groups {
                let mut shared: Option<(f64, f64)> = None;
                for &index in group {
                    let range = ranges[index].ok_or_else(|| {
                        Error::Data(format!(
                            "canvas x-range group references unconfigured panel {index}"
                        ))
                    })?;
                    shared = Some(shared.map_or(range, |current| {
                        (current.0.min(range.0), current.1.max(range.1))
                    }));
                }
                let shared = shared.expect("validated nonempty x-range group");
                for &index in group {
                    let plot = plots[index].as_ref().expect("validated grouped panel");
                    plots[index] = Some(plot.clone().with_planned_x_limits(shared));
                }
            }
        }
        let mut measured = Vec::new();
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let row = index / self.columns;
            let column = index % self.columns;
            let kind = plot.kind();
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(index, "panel-measure", theme);
            let result = plot.preflight_panel(kind, provisional[index], theme)?;
            if result.decoration_mode == DecorationMode::SelfManaged {
                return Err(Error::Render(format!(
                    "{kind:?} manages its own canvas and cannot be nested in a composed canvas"
                )));
            }
            let title = theme
                .show_title
                .then(|| plot.title_text())
                .flatten()
                .filter(|title| !title.is_empty());
            let subtitle = theme
                .show_subtitle
                .then(|| plot.subtitle_text())
                .flatten()
                .filter(|subtitle| !subtitle.is_empty());
            let requirement = PanelRequirements::measure_header(
                &result.layout,
                theme,
                title,
                subtitle,
                provisional[index],
            )?;
            measured.push((index, row, column, kind, result, requirement));
        }
        let mut requirements = measured
            .iter()
            .map(|(_, row, column, _, _, requirement)| (*row, *column, *requirement))
            .collect::<Vec<_>>();
        self.add_row_title_requirements(&mut requirements, &provisional, header_theme)?;
        let first_cells = allocate_composed_grid(
            outer,
            self.rows,
            self.columns,
            &requirements,
            "canvas layout",
        )?;
        let mut requirements = measured
            .iter()
            .map(|(index, row, column, _, result, _)| {
                let plot = plots[*index]
                    .as_ref()
                    .expect("measured canvas panel remains configured");
                let theme = themes
                    .plot(*index)
                    .expect("configured canvas panel has a resolved theme");
                let title = theme
                    .show_title
                    .then(|| plot.title_text())
                    .flatten()
                    .filter(|title| !title.is_empty());
                let subtitle = theme
                    .show_subtitle
                    .then(|| plot.subtitle_text())
                    .flatten()
                    .filter(|subtitle| !subtitle.is_empty());
                Ok::<_, Error>((
                    *row,
                    *column,
                    PanelRequirements::measure_header(
                        &result.layout,
                        theme,
                        title,
                        subtitle,
                        first_cells[*index],
                    )?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.add_row_title_requirements(&mut requirements, &first_cells, header_theme)?;
        let cells = allocate_composed_grid(
            outer,
            self.rows,
            self.columns,
            &requirements,
            "canvas layout",
        )?;
        let mut headers = Vec::new();
        for header in &self.column_headers {
            let left = cells[header.start].x0;
            let right = cells[header.start + header.span - 1].x1;
            headers.push(CanvasHeaderPlan {
                text: header.text.clone(),
                x: f64::midpoint(left, right),
                y: header_outer.y1 - column_header_height / 2.0,
            });
        }
        for header in &self.row_headers {
            let indices = (header.start..header.start + header.span)
                .map(|row| row * self.columns)
                .collect::<Vec<_>>();
            let bottom = indices
                .iter()
                .map(|index| cells[*index].y0)
                .fold(f64::INFINITY, f64::min);
            let top = indices
                .iter()
                .map(|index| cells[*index].y1)
                .fold(f64::NEG_INFINITY, f64::max);
            headers.push(CanvasHeaderPlan {
                text: header.text.clone(),
                x: header_outer.x0 + row_header_width / 2.0,
                y: f64::midpoint(bottom, top),
            });
        }
        let row_titles = self.plan_row_titles(&cells, header_theme)?;
        #[cfg(test)]
        {
            if !headers.is_empty() {
                record_canvas_theme(
                    header_theme_index.unwrap_or(usize::MAX),
                    "shared-header-plan",
                    header_theme,
                );
            }
            if !row_titles.is_empty() {
                record_canvas_theme(
                    header_theme_index.unwrap_or(usize::MAX),
                    "row-title-plan",
                    header_theme,
                );
            }
        }

        let mut text_panels = Vec::new();
        for (index, text_panel) in self.text_panels.iter().enumerate() {
            let Some(text_panel) = text_panel else {
                continue;
            };
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "text-plan", themes.root());
            let cell = cells[index];
            let maximum_width = (cell.width() - 32.0).max(1.0);
            let title = crate::layout::TextBlock::layout(
                text_panel.title(),
                themes.root().title_font_size,
                maximum_width,
                "text panel title",
            )?;
            let body = crate::layout::TextBlock::layout(
                text_panel.body(),
                themes.root().label_font_size,
                maximum_width,
                "text panel body",
            )?;
            let required_height = title.height() + 12.0 + body.height();
            if required_height > cell.height() - 32.0 {
                return Err(Error::Render(format!(
                    "text panel at ({}, {}) requires {required_height:.1}px but only {:.1}px is available; increase canvas size or shorten text",
                    index / self.columns,
                    index % self.columns,
                    (cell.height() - 32.0).max(0.0)
                )));
            }
            text_panels.push(TextPanelPlan {
                index,
                cell,
                title,
                body,
            });
        }

        for plot in plots.iter_mut().flatten() {
            *plot = plot.clone().with_measured_composed_axis_text();
        }

        let mut panels = Vec::with_capacity(measured.len());
        for (index, _, _, kind, _, _) in measured {
            let plot = plots[index]
                .as_ref()
                .expect("measured canvas panel remains configured");
            let cell = cells[index];
            let role = if self.legend_policy == CanvasLegendPolicy::PerPlot {
                crate::decorations::plan::LegendRole::Panel
            } else {
                crate::decorations::plan::LegendRole::Suppressed
            };
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(index, "panel-plan", theme);
            let panel_plan = crate::decorations::plan::plan_panel(plot, kind, cell, role, theme)?;
            if panel_plan.result.decoration_mode == DecorationMode::SelfManaged {
                return Err(Error::Render(format!(
                    "{kind:?} manages its own canvas and cannot be nested in a composed canvas"
                )));
            }
            panels.push(PlannedCanvasPanel {
                index,
                kind,
                canvas_cell: cell,
                panel_plan,
            });
        }
        Ok(CanvasPlan {
            themes,
            plots,
            canvas_layout,
            full_outer,
            outer,
            shared_legend,
            panels,
            text_panels,
            headers,
            row_titles,
        })
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the immutable Canvas plan keeps theme ownership and final draw ordering explicit"
    )]
    fn render_planned_canvas(
        &self,
        plan: CanvasPlan,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let theme = plan.themes.root();
        #[cfg(test)]
        record_canvas_theme(usize::MAX, "render", theme);
        let header_theme_index = plan.plots.iter().position(Option::is_some);
        let row_title_theme = header_theme_index
            .and_then(|index| plan.themes.plot(index))
            .unwrap_or(theme);
        for panel in &plan.panels {
            let plot = plan.plots[panel.index].as_ref().ok_or_else(|| {
                Error::Render("composed canvas panel disappeared during preflight".into())
            })?;
            let theme = plan
                .themes
                .plot(panel.index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(panel.index, "grid-render", theme);
            crate::decorations::apply::apply_grid(plot, &panel.panel_plan.result, theme, backend)?;
        }
        for panel in plan.panels {
            let plot = plan.plots[panel.index].as_ref().ok_or_else(|| {
                Error::Render("composed canvas panel disappeared during render".into())
            })?;
            let theme = plan
                .themes
                .plot(panel.index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(panel.index, "panel-render", theme);
            let source_rows = plot.configured_source_rows()?;
            let plot_with_planned_limits = panel.panel_plan.plot_with_planned_limits(plot);
            let context = source_rows.map_or_else(
                || {
                    crate::spec::PlotRenderContext::new(
                        plot_with_planned_limits.data(),
                        panel.panel_plan.panel_box,
                        theme,
                    )
                },
                |rows| {
                    crate::spec::PlotRenderContext::with_source_rows(
                        plot_with_planned_limits.data(),
                        rows,
                        panel.panel_plan.panel_box,
                        theme,
                    )
                },
            );
            plot_with_planned_limits.render_panel(panel.kind, context, backend)?;
            if !panel.panel_plan.result.significance_brackets.is_empty() {
                crate::significance::render(
                    plot,
                    &panel.panel_plan.result.significance_brackets,
                    theme,
                    backend,
                )?;
            }
            crate::decorations::apply::apply_axes(plot, &panel.panel_plan.result, theme, backend)?;
            crate::decorations::apply::apply_planned_legend(
                plot,
                panel.panel_plan.legend.as_ref(),
                backend,
            )?;
            draw_panel_header(plot, panel.canvas_cell, theme, backend)?;
        }
        for panel in &plan.text_panels {
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "text-render", theme);
            draw_text_panel(panel, theme, backend)?;
        }
        if !plan.row_titles.is_empty() {
            #[cfg(test)]
            record_canvas_theme(
                header_theme_index.unwrap_or(usize::MAX),
                "row-title-render",
                row_title_theme,
            );
            draw_canvas_row_titles(&plan.row_titles, row_title_theme, backend)?;
        }
        if !plan.headers.is_empty() {
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "shared-header-render", theme);
            draw_canvas_headers(&plan.headers, theme, backend)?;
        }
        if let Some((title, entries)) = plan.shared_legend {
            let legend = Legend::new(
                title,
                entries,
                f64::from(self.width),
                f64::from(self.height),
            );
            let left = plan.outer.x1 + self.resolved_shared_legend_gap(&legend);
            let bottom = (plan.full_outer.y1 - legend.compute_box_height()).max(plan.full_outer.y0);
            let (x, y) = legend.box_at(left, bottom).anchor();
            with_layer(backend, "canvas-shared-legend", |backend| {
                legend.draw(backend, x, y)
            })?;
        }
        let mut titles = TitleBlock::new().font_family(theme.font_family.clone());
        if let Some(title) = &self.title {
            titles = titles.title(title.clone());
        }
        if let Some(subtitle) = &self.subtitle {
            titles = titles.subtitle(subtitle.clone());
        }
        if let Some(caption) = &self.caption {
            titles = titles.caption(caption.clone());
        }
        titles.draw(backend, self.width, &plan.canvas_layout, theme)
    }

    fn canvas_hue_groups(
        &self,
        panel: PanelBox,
        themes: &ResolvedCanvasThemes,
    ) -> Result<Vec<(String, Vec<String>)>, Error> {
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let plot = self.labels.as_ref().map_or_else(
                || plot.clone(),
                |labels| plot.clone().labels(labels.clone()),
            );
            let Some(dimension) = plot.hue_dimension_key() else {
                continue;
            };
            let kind = plot.kind();
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(index, "shared-hue-discovery", theme);
            let observed = plot.preflight_hue_levels(kind, panel, theme)?;
            let levels =
                if let Some((_, levels)) = groups.iter_mut().find(|(name, _)| name == dimension) {
                    levels
                } else {
                    let index = groups.len();
                    groups.push((dimension.to_owned(), Vec::new()));
                    &mut groups[index].1
                };
            for level in observed {
                if !levels.contains(&level) {
                    levels.push(level);
                }
            }
        }
        Ok(groups)
    }

    fn resolved_shared_legend_gap(&self, legend: &Legend) -> f64 {
        self.shared_legend_gap.unwrap_or(legend.metrics.pad * 2.0)
    }

    fn preflight_shared_legend(
        plots: &[Option<Plot>],
        themes: &ResolvedCanvasThemes,
        panel: PanelBox,
    ) -> Result<Option<(String, Vec<LegendEntry>)>, Error> {
        let mut entries = Vec::new();
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let kind = plot.kind();
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(index, "shared-legend-preflight", theme);
            let result = plot.preflight_panel(kind, panel, theme)?;
            entries.extend(result.legend_entries);
        }
        let entries =
            crate::decorations::legend::merge_exact_entries(entries, "shared canvas legend")?;
        if entries.is_empty() {
            Ok(None)
        } else {
            Ok(Some((
                crate::decorations::legend::resolved_title(&entries)?,
                entries,
            )))
        }
    }
}

fn plot_category_domain(
    plot: &Plot,
    panel: PanelBox,
    theme: &crate::theme::ResolvedTheme,
) -> Result<Vec<String>, Error> {
    let kind = plot.kind();
    if !matches!(
        kind,
        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
    ) {
        return Err(Error::Data(
            "shared category labels are supported only by horizontal bar families".into(),
        ));
    }
    let result = plot.preflight_panel(kind, panel, theme)?;
    if result.y_domain != AxisDomain::Categorical {
        return Err(Error::Data(
            "shared category labels require a categorical y-axis".into(),
        ));
    }
    Ok(result.y_tick_keys.unwrap_or_else(|| {
        result
            .layout
            .y_axis
            .ticks
            .into_iter()
            .map(|tick| tick.label)
            .collect()
    }))
}

fn draw_canvas_headers(
    headers: &[CanvasHeaderPlan],
    theme: &crate::theme::ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if headers.is_empty() {
        return Ok(());
    }
    let style = crate::render::backend::TextStyle {
        color: theme.foreground,
        font_family: theme.font_family.clone(),
        font_size: theme.title_font_size,
        weight: crate::TextWeight::Normal,
        anchor: crate::render::backend::TextAnchor::Middle,
        rotation: 0.0,
    };
    with_layer(backend, "canvas-shared-headers", |backend| {
        for header in headers {
            backend.draw_text(
                Point2D::new(header.x.round(), header.y.round()),
                &header.text,
                &style,
            )?;
        }
        Ok(())
    })
}

fn draw_canvas_row_titles(
    titles: &[CanvasHeaderPlan],
    theme: &crate::theme::ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if titles.is_empty() {
        return Ok(());
    }
    let style = crate::render::backend::TextStyle {
        color: theme.foreground,
        font_family: theme.font_family.clone(),
        font_size: theme.title_font_size,
        weight: crate::TextWeight::Normal,
        anchor: crate::render::backend::TextAnchor::Middle,
        rotation: 0.0,
    };
    with_layer(backend, "canvas-row-titles", |backend| {
        for title in titles {
            backend.draw_text(
                Point2D::new(title.x.round(), title.y.round()),
                &title.text,
                &style,
            )?;
        }
        Ok(())
    })
}

fn draw_text_panel(
    panel: &TextPanelPlan,
    theme: &crate::theme::ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let gap = 12.0;
    let total_height = panel.title.height() + gap + panel.body.height();
    let center_x = panel.cell.x0.midpoint(panel.cell.x1);
    let mut y = panel.cell.y0.midpoint(panel.cell.y1) + total_height / 2.0
        - panel.title.line_height() / 2.0;
    let title_style = crate::render::backend::TextStyle {
        color: theme.foreground,
        font_family: theme.font_family.clone(),
        font_size: theme.title_font_size,
        weight: crate::TextWeight::Normal,
        anchor: crate::render::backend::TextAnchor::Middle,
        rotation: 0.0,
    };
    let body_style = crate::render::backend::TextStyle {
        color: theme.foreground,
        font_family: theme.font_family.clone(),
        font_size: theme.label_font_size,
        weight: crate::TextWeight::Normal,
        anchor: crate::render::backend::TextAnchor::Middle,
        rotation: 0.0,
    };
    with_layer(
        backend,
        &format!("canvas-text-panel-{}", panel.index),
        |backend| {
            for line in panel.title.lines() {
                backend.draw_text(
                    Point2D::new(center_x.round(), y.round()),
                    line,
                    &title_style,
                )?;
                y -= panel.title.line_height();
            }
            y -= gap;
            for line in panel.body.lines() {
                backend.draw_text(Point2D::new(center_x.round(), y.round()), line, &body_style)?;
                y -= panel.body.line_height();
            }
            Ok(())
        },
    )
}

#[expect(
    clippy::cast_precision_loss,
    reason = "small text-line indexes are projected into backend coordinates"
)]
fn draw_panel_header(
    plot: &Plot,
    cell: PanelBox,
    theme: &crate::theme::ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let title = theme
        .show_title
        .then(|| plot.title_text())
        .flatten()
        .filter(|title| !title.is_empty());
    let subtitle = theme
        .show_subtitle
        .then(|| plot.subtitle_text())
        .flatten()
        .filter(|subtitle| !subtitle.is_empty());
    if title.is_none() && subtitle.is_none() {
        return Ok(());
    }
    let maximum_width = (cell.width() - 16.0).max(1.0);
    let title_lines = title
        .map(|text| try_wrap_text(text, theme.title_font_size, maximum_width, "panel title"))
        .transpose()?
        .unwrap_or_default();
    let subtitle_lines = subtitle
        .map(|text| {
            try_wrap_text(
                text,
                theme.subtitle_font_size,
                maximum_width,
                "panel subtitle",
            )
        })
        .transpose()?
        .unwrap_or_default();
    let subtitle_line_height = f64::from(theme.subtitle_font_size) + 6.0;
    let title_line_height = f64::from(theme.title_font_size) + 6.0;
    let subtitle_style = crate::render::backend::TextStyle {
        color: (80, 80, 80),
        font_family: theme.font_family.clone(),
        font_size: theme.subtitle_font_size,
        weight: crate::TextWeight::Normal,
        anchor: crate::render::backend::TextAnchor::Start,
        rotation: 0.0,
    };
    let title_style = crate::render::backend::TextStyle {
        color: theme.foreground,
        font_family: theme.font_family.clone(),
        font_size: theme.title_font_size,
        weight: crate::TextWeight::Normal,
        anchor: crate::render::backend::TextAnchor::Start,
        rotation: 0.0,
    };
    with_layer(backend, &plot.layer_name("title"), |backend| {
        for (index, line) in subtitle_lines.iter().rev().enumerate() {
            backend.draw_text(
                Point2D::new(
                    (cell.x0 + 8.0).round(),
                    (cell.y1 + 14.0 + index as f64 * subtitle_line_height).round(),
                ),
                line,
                &subtitle_style,
            )?;
        }
        let title_base = cell.y1
            + 14.0
            + subtitle_lines.len() as f64 * subtitle_line_height
            + if title_lines.is_empty() || subtitle_lines.is_empty() {
                0.0
            } else {
                PANEL_HEADER_GAP
            };
        for (index, line) in title_lines.iter().rev().enumerate() {
            backend.draw_text(
                Point2D::new(
                    (cell.x0 + 8.0).round(),
                    (title_base + index as f64 * title_line_height).round(),
                ),
                line,
                &title_style,
            )?;
        }
        Ok(())
    })
}
