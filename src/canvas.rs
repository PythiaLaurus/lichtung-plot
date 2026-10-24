//! Heterogeneous plot composition on one canvas.

use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use crate::Error;

use crate::decorations::legend::discrete::scoped::{
    GuideAllocation, GuideConsumption, SUPPRESSED_GUIDES,
};
use crate::decorations::legend::Legend;
use crate::decorations::plan::children::MeasurementPhase;
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{FitFailure, FitScope};
use crate::layout::kernel::{FeedbackKind, FitKind};
use crate::layout::{PanelBox, RootFramePlan};
use crate::render::backend::{with_layer, Backend, SurfaceRole};
use crate::scene::RenderedScene;
use crate::spec::AxisDomain;
use crate::theme::ResolvedTheme;
use crate::{CanvasLegendPolicy, Labels, LegendComposition, Output, Plot, PlotKind, Theme};

mod dependencies;
mod plot_text;
mod sizing;
mod text_panel;
mod tracks;

#[cfg(test)]
pub(crate) mod lifecycle_tests;

#[cfg(test)]
thread_local! {
    pub(crate) static CANVAS_SCENE_RENDER_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static CANVAS_ASSIGNED_PANEL_EVIDENCE: std::cell::RefCell<Vec<(PanelBox, PanelBox)>> = const { std::cell::RefCell::new(Vec::new()) };
}

struct PlannedCanvasPanel {
    index: usize,
    kind: PlotKind,
    canvas_cell: PanelBox,
    panel_plan: Arc<crate::decorations::plan::PanelPlan>,
    local_text: plot_text::PlotLocalTextPlan,
    header_layer: String,
    legend: Option<crate::decorations::legend::LegendDrawPlan>,
    layers: [String; 3],
}

type PreparedColorDomains = Vec<(String, Vec<String>)>;
use crate::decorations::legend::discrete::PreparedGuide;

struct CanvasPlan {
    dimensions: (u32, u32),
    themes: Arc<ResolvedCanvasThemes>,
    plots: Vec<Option<Plot>>,
    canvas_layout: RootFramePlan,
    shared_legend_draw: Option<crate::decorations::legend::LegendDrawPlan>,
    panels: Vec<PlannedCanvasPanel>,
    text_panels: Vec<TextPanelPlan>,
    headers: Vec<CanvasHeaderPlan>,
    row_titles: Vec<CanvasHeaderPlan>,
}

struct PreparedCanvasInputs {
    themes: Arc<ResolvedCanvasThemes>,
    text_census: crate::layout::PreparedTextCensus,
    root_text: crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
    shared_x: Vec<Option<(f64, f64)>>,
    color_domains: Option<Arc<PreparedColorDomains>>,
    shared_guide: Option<Arc<PreparedGuide>>,
    interiors: Vec<Option<crate::layout::interior::PreparedInteriorInputs>>,
    text_panel_intrinsics: Vec<Option<text_panel::TextPanelIntrinsics>>,
    plot_intrinsics: Vec<Option<crate::decorations::title_block::plan::RootTitleBlockIntrinsics>>,
}

struct PreparedCanvasRound<'a> {
    content_sizing: bool,
    measurements: Option<&'a sizing::HeightMeasurements>,
    interiors: &'a [Option<crate::layout::interior::PreparedInteriorInputs>],
    themes: Arc<ResolvedCanvasThemes>,
    canvas_layout: RootFramePlan,
    plots: Vec<Option<Plot>>,
    shared_legend: Option<Arc<PreparedGuide>>,
    body_probes: Vec<Option<Arc<crate::spec::PlotRenderResult>>>,
    shared_x: &'a [Option<(f64, f64)>],
    text_panel_intrinsics: &'a [Option<text_panel::TextPanelIntrinsics>],
    plot_intrinsics:
        &'a [Option<crate::decorations::title_block::plan::RootTitleBlockIntrinsics>],
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
    fn new(root: &crate::theme::Theme, plots: &[Option<Plot>]) -> Result<Self, Error> {
        let root = Box::new(root.resolve()?);
        let plots = plots
            .iter()
            .map(|plot| {
                plot.as_ref()
                    .map(|plot| match plot.theme_spec() {
                        None => Ok(ResolvedPlotTheme::InheritedRoot),
                        Some(theme) => {
                            let mut theme = Box::new(theme.resolve()?);
                            theme.canvas_surface_override = Some(root.surfaces.canvas_root);
                            theme.share_font_environment_from(&root);
                            Ok(ResolvedPlotTheme::ExplicitPlot(theme))
                        }
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self { root, plots })
    }

    fn prepare_font_environment(
        &self,
        census: &mut crate::layout::PreparedTextCensus,
        presence: Vec<u64>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<(), Error> {
        self.root
            .prepare_font_environment_from_census(census, presence, prepared_stage)
    }

    fn root(&self) -> &ResolvedTheme {
        self.root.as_ref()
    }

    fn plot(&self, index: usize) -> Option<&ResolvedTheme> {
        self.plot_selection(index).map(|theme| match theme {
            ResolvedPlotTheme::InheritedRoot => self.root(),
            ResolvedPlotTheme::ExplicitPlot(theme) => theme.as_ref(),
        })
    }

    fn plot_selection(&self, index: usize) -> Option<&ResolvedPlotTheme> {
        self.plots.get(index)?.as_ref()
    }
}

struct CanvasHeaderPlan {
    request: crate::Text,
    text: crate::render::backend::PlacedText,
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
    text: crate::Text,
}

struct TextPanelPlan {
    layer: String,
    cell: PanelBox,
    title: crate::layout::TextBlock,
    body: crate::layout::TextBlock,
    title_placement: crate::render::backend::PlannedTextPlacement,
    body_placement: crate::render::backend::PlannedTextPlacement,
}

/// Validated prose content for one composed-canvas grid cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPanel {
    title: crate::Text,
    body: crate::Text,
}

impl TextPanel {
    /// Construct a text panel, retaining verbatim title/body and mandatory breaks.
    pub fn new(title: impl Into<crate::Text>, body: impl Into<crate::Text>) -> Result<Self, Error> {
        let title = title.into();
        let body = body.into();
        crate::text::validate_for(title.as_str(), "text panel title")?;
        crate::text::validate_for(body.as_str(), "text panel body")?;
        Ok(Self { title, body })
    }

    /// Panel title rendered above the body.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Panel body rendered with deterministic wrapping.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }
}

fn canvas_header(
    start: usize,
    span: usize,
    text: crate::Text,
    limit: usize,
    role: &str,
) -> Result<CanvasHeader, Error> {
    if span == 0 || start.checked_add(span).is_none_or(|end| end > limit) {
        return Err(Error::Data(format!(
            "canvas {role} header range {start}..{} is outside 0..{limit}",
            start.saturating_add(span)
        )));
    }
    crate::text::validate_for(text.as_str(), &format!("canvas {role} header"))?;
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
///
/// Canvas owns one root theme, which defaults to [`Theme::publication`]. Plots
/// without an explicit theme inherit that root. Dimensions are derived from
/// content unless [`Self::width`] or [`Self::size`] replaces the sizing request.
pub struct Canvas {
    rows: usize,
    columns: usize,
    panels: Vec<Option<Plot>>,
    text_panels: Vec<Option<TextPanel>>,
    theme: Arc<crate::theme::Theme>,
    legend_policy: CanvasLegendPolicy,
    legend_composition: LegendComposition,
    root_size: crate::layout::RootSize,
    title: crate::text::TextSlot,
    subtitle: crate::text::TextSlot,
    // Parent defaults apply only to embedded Plot prose, never Canvas root headings.
    pub(crate) panel_title_alignment: Option<crate::TextAlignment>,
    pub(crate) panel_subtitle_alignment: Option<crate::TextAlignment>,
    caption: crate::text::TextSlot,
    labels: Option<Labels>,
    category_labels: CanvasCategoryLabels,
    column_headers: Vec<CanvasHeader>,
    row_headers: Vec<CanvasHeader>,
    row_titles: Vec<CanvasHeader>,
    x_range_groups: Vec<Vec<usize>>,
}

fn push_text_requirement(
    requirements: &mut Vec<crate::layout::PreparedTextRequirement>,
    address: crate::layout::ComponentAddress,
    role: crate::layout::PreparedTextRole,
    local_index: u32,
    source: &str,
    appearance: &crate::theme::TextTheme,
    prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
) -> Result<(), Error> {
    crate::layout::PreparedTextRequirement::push_into(
        requirements,
        address,
        role,
        local_index,
        source,
        &[],
        appearance,
        Some(prepared_stage),
    )
    .map_err(|error| Error::Render(format!("prepared canvas text census: {error}")))
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
            theme: Arc::new(crate::theme::Theme::publication()),
            legend_policy: CanvasLegendPolicy::PerPlot,
            legend_composition: LegendComposition::Combined,
            root_size: crate::layout::RootSize::natural(),
            title: crate::text::TextSlot::EMPTY,
            subtitle: crate::text::TextSlot::EMPTY,
            panel_title_alignment: None,
            panel_subtitle_alignment: None,
            caption: crate::text::TextSlot::EMPTY,
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
        let mut plot = plot.erase();
        plot.numeric_cell = Some(crate::data::source::source_arc((row, column))?);
        self.panels[index] = Some(plot);
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

    /// Replace the semantic composition of a Shared discrete guide.
    ///
    /// Combined is the default. Under Shared this selection owns every
    /// contribution; stored child selections remain unchanged. PerPlot, Hidden
    /// and all-continuous guides leave this value inactive. Theme, placement and
    /// legend-policy call order are independent.
    /// ```
    /// use lichtung_plot::{Canvas, CanvasLegendPolicy, LegendComposition};
    /// let canvas = Canvas::grid(1, 2)?
    ///     .legend_composition(LegendComposition::Separate)
    ///     .legend_policy(CanvasLegendPolicy::Shared);
    /// # let _ = canvas;
    /// # Ok::<(), lichtung_plot::Error>(())
    /// ```
    #[must_use]
    pub const fn legend_composition(mut self, composition: LegendComposition) -> Self {
        self.legend_composition = composition;
        self
    }

    /// Set the title alignment default for embedded Plots, independently of Canvas prose.
    ///
    /// An explicit Plot title instruction wins; otherwise this wins over that
    /// Plot's theme default. Start and End mean the physical left and right edges
    /// of each Plot's inset content span. Headers and TextPanel text are unaffected.
    /// ```
    /// use lichtung_plot::{Canvas, NumericValues, Plot, Theme, TextAlignment};
    /// let canvas = Canvas::grid(1, 1)?
    ///     .panel_title_alignment(TextAlignment::Start)
    ///     .panel_subtitle_alignment(TextAlignment::End)
    ///     .theme(Theme::nature())
    ///     .panel(0, 0, Plot::scatter(
    ///         NumericValues::new("x", [1.0, 2.0]),
    ///         NumericValues::new("y", [2.0, 1.0]),
    ///     ).title("Panel title").subtitle("Panel subtitle"))?;
    /// # let _ = canvas;
    /// # Ok::<(), lichtung_plot::Error>(())
    /// ```
    #[must_use]
    pub const fn panel_title_alignment(mut self, alignment: crate::TextAlignment) -> Self {
        self.panel_title_alignment = Some(alignment);
        self
    }

    /// Set the subtitle alignment default for embedded Plots only.
    ///
    /// An explicit Plot subtitle instruction wins; otherwise this wins over the
    /// optional Plot-theme subtitle default and final effective title alignment.
    /// Theme and alignment attachment order is independent. Canvas-owned prose
    /// and captions use their own Text instructions over their complete theme roles.
    #[must_use]
    pub const fn panel_subtitle_alignment(mut self, alignment: crate::TextAlignment) -> Self {
        self.panel_subtitle_alignment = Some(alignment);
        self
    }

    /// Add one measured header spanning contiguous canvas columns.
    /// Text can replace the complete Canvas header style and align its lines;
    /// the block remains centered over the declared span.
    pub fn column_header(
        mut self,
        start_column: usize,
        span: usize,
        text: impl Into<crate::Text>,
    ) -> Result<Self, Error> {
        let header = canvas_header(start_column, span, text.into(), self.columns, "column")?;
        validate_header_overlap(&self.column_headers, &header, "column")?;
        self.column_headers.push(header);
        Ok(self)
    }

    /// Add one measured header spanning contiguous canvas rows.
    /// Local Text style owns its font-relative padding. Line alignment defaults
    /// to Center inside the block positioned beside the declared rows.
    pub fn row_header(
        mut self,
        start_row: usize,
        span: usize,
        text: impl Into<crate::Text>,
    ) -> Result<Self, Error> {
        let header = canvas_header(start_row, span, text.into(), self.rows, "row")?;
        validate_header_overlap(&self.row_headers, &header, "row")?;
        self.row_headers.push(header);
        Ok(self)
    }

    /// Add one measured title centered above every panel in one canvas row.
    /// Text retains an independent complete style and line alignment, defaulting
    /// to the Canvas header role and Center.
    pub fn row_title(mut self, row: usize, text: impl Into<crate::Text>) -> Result<Self, Error> {
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

    /// Couple the automatic continuous horizontal range of explicit panel cells.
    ///
    /// Grouped cells must contain configured plots without explicit x limits or
    /// categorical x domains.
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

    /// Allocate exactly this width in CSS pixels and derive height from content.
    ///
    /// This replaces the complete request, clearing any height set by
    /// [`Self::size`]. Without a sizing call, both dimensions derive from content.
    /// Zero is rejected during preflight, without a setter panic.
    #[must_use]
    pub const fn width(mut self, width: u32) -> Self {
        self.root_size = crate::layout::RootSize::width(width);
        self
    }

    /// Allocate exactly this width and height in CSS pixels.
    ///
    /// This replaces the complete sizing request. All content must fit without
    /// changing either dimension; zero dimensions reject during preflight.
    /// Embedded Plots do not contribute standalone sizing requests.
    #[must_use]
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.root_size = crate::layout::RootSize::new(width, height);
        self
    }

    /// Set one canvas-level title.
    ///
    /// A plain string replaces content while retaining earlier local style and
    /// layout. Present Text instructions replace the complete style or layout;
    /// an explicit default layout restores owner defaults.
    #[must_use]
    pub fn title(mut self, title: impl Into<crate::Text>) -> Self {
        self.title.replace(title.into());
        self
    }

    /// Set one canvas-level subtitle.
    /// Content and instruction replacement follow [`Self::title`].
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<crate::Text>) -> Self {
        self.subtitle.replace(subtitle.into());
        self
    }

    /// Set one canvas-level caption.
    /// Content and instruction replacement follow [`Self::title`].
    #[must_use]
    pub fn caption(mut self, caption: impl Into<crate::Text>) -> Self {
        self.caption.replace(caption.into());
        self
    }

    fn prose(&self) -> [Option<&crate::Text>; 3] {
        [&self.title, &self.subtitle, &self.caption].map(crate::text::TextSlot::content)
    }

    fn prose_alignments(&self) -> [Option<crate::TextAlignment>; 3] {
        [&self.title, &self.subtitle, &self.caption].map(crate::text::TextSlot::alignment_request)
    }

    /// Apply one reusable automatic-label policy to every configured panel.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Set the Canvas root theme.
    ///
    /// Plots without an explicit theme inherit this theme. A theme selected with
    /// [`Plot::theme`] completely replaces it for that Plot. If this method is
    /// omitted, the Canvas root uses [`Theme::publication`].
    #[must_use]
    ///
    /// Shared legend spacing is configured only in the complete theme:
    /// ```compile_fail
    /// let _ = lichtung_plot::Canvas::grid(1, 1).unwrap().shared_legend_gap(8.0);
    /// ```
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Arc::new(theme);
        self
    }

    fn render_preflighted(
        root: &crate::layout::FrozenRoot<crate::orchestrator::TextReplayPlan, PreparedCanvasInputs>,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let _ = &root.prepared().text_census;
        let (width, height) = root.dimensions();
        root.plan()
            .replay(width, height, backend, "prepared canvas scene replay")
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
        Self::render_preflighted(&preflight, backend)
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
        let (width, height) = preflight.dimensions();
        let mut backend = preflight.plan().take_svg_backend()?;
        Self::render_preflighted(&preflight, &mut backend)?;
        RenderedScene::new(width, height, backend.into_output())
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
        for source in [
            self.title.source(),
            self.subtitle.source(),
            self.caption.source(),
        ]
        .into_iter()
        .flatten()
        {
            crate::text::validate_for(source, "canvas prose")?;
        }
        if self.panels.iter().all(Option::is_none) && self.text_panels.iter().all(Option::is_none) {
            return Err(Error::Data(
                "canvas requires at least one configured panel".into(),
            ));
        }
        for plot in self.panels.iter().flatten() {
            let kind = plot.kind();
            plot.validate_configuration(kind)?;
            crate::layout::interior::CompositionChildPolicy::Canvas.validate_child(plot)?;
            if self.legend_policy == CanvasLegendPolicy::Shared
                && matches!(
                    plot.legend_policy_spec(),
                    crate::LegendPolicy::Visible(position) if position != crate::LegendPosition::Auto
                )
            {
                return Err(Error::Data(
                    "Canvas shared legend cannot honor explicit per-Plot placement".into(),
                ));
            }
        }
        for group in &self.x_range_groups {
            for &index in group {
                let plot = self.panels[index].as_ref().ok_or_else(|| {
                    Error::Data(format!(
                        "canvas x-range group references unconfigured panel {index}"
                    ))
                })?;
                if plot.user_x_limits().is_some() {
                    return Err(Error::Data(format!(
                        "canvas x-range group panel {index} has explicit x limits"
                    )));
                }
                if plot
                    .automatic_axis_category_values(plot.kind(), true)
                    .is_some()
                {
                    return Err(Error::Data(format!(
                        "canvas x-range group panel {index} has a categorical x axis"
                    )));
                }
            }
        }
        for title in &self.row_titles {
            let start = title.start * self.columns;
            let end = start + self.columns;
            if self.panels[start..end]
                .iter()
                .flatten()
                .any(|plot| plot.title_text().is_some() || plot.subtitle_text().is_some())
            {
                return Err(Error::Data(format!(
                    "canvas row title cannot coexist with a panel title or subtitle in row {}",
                    title.start
                )));
            }
        }
        Ok(())
    }

    fn prepare_continuous_shared_guide(
        &self,
        themes: &ResolvedCanvasThemes,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
    ) -> Result<Option<crate::decorations::legend::descriptor::ContinuousDescriptor>, Error> {
        if self.legend_policy != CanvasLegendPolicy::Shared {
            return Ok(None);
        }
        let Some(_) = self.panels.iter().flatten().next() else {
            return Ok(None);
        };

        let mut contributors = Vec::new();
        crate::data::source::reserve_source(&mut contributors, self.panels.len())?;
        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };
            if !matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm) {
                continue;
            }
            let inputs = interiors
                .get(index)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    Error::Render("shared continuous guide has no prepared interior".into())
                })?;

            let theme = themes.plot(index).ok_or_else(|| {
                Error::Render("shared continuous guide has no owning Plot theme".into())
            })?;
            let contributor =
                crate::decorations::legend::descriptor::ContinuousContributor::for_plot(
                    plot, inputs, theme,
                )?
                .ok_or_else(|| {
                    Error::Render("shared continuous guide has no color identity".into())
                })?;
            contributors.push(contributor);
        }
        crate::decorations::legend::descriptor::ContinuousContributor::merge(contributors)
    }

    fn install_shared_continuous_domain(
        &self,
        interiors: &mut [Option<crate::layout::interior::PreparedInteriorInputs>],
        descriptor: &crate::decorations::legend::descriptor::ContinuousDescriptor,
    ) -> Result<(), Error> {
        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let Some(input) = interiors.get_mut(index).and_then(Option::as_mut) else {
                continue;
            };
            match plot.kind() {
                PlotKind::Heatmap => {
                    let panels = Arc::get_mut(input.heatmap.as_mut().ok_or_else(|| {
                        Error::Render("shared Heatmap has no prepared inputs".into())
                    })?)
                    .ok_or_else(|| Error::Render("shared Heatmap inputs are aliased".into()))?;
                    for (_, marks) in &mut panels.panels {
                        marks.install_shared_domain(descriptor)?;
                    }
                }
                PlotKind::Beeswarm => {
                    let panels = Arc::get_mut(input.beeswarm.as_mut().ok_or_else(|| {
                        Error::Render("shared Beeswarm has no prepared inputs".into())
                    })?)
                    .ok_or_else(|| Error::Render("shared Beeswarm inputs are aliased".into()))?;
                    for (_, marks) in &mut panels.panels {
                        marks.install_shared_domain(descriptor)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Coordinate scientific categorical color identities before fonts,
    /// independently of guide visibility and section contribution.
    fn prepare_color_domains(
        &self,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
    ) -> Result<Option<Arc<PreparedColorDomains>>, Error> {
        if self.legend_policy != CanvasLegendPolicy::Shared {
            return Ok(None);
        }
        let mut groups = Vec::new();
        crate::data::source::reserve_source(&mut groups, self.panels.len())?;
        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let Some(dimension) = plot.categorical_color_field() else {
                continue;
            };

            let inputs = interiors
                .get(index)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    Error::Render("Canvas color domain is missing prepared semantics".into())
                })?;
            let group_index =
                crate::data::source::source_position(&groups, |(field, _)| field == dimension);
            let levels = if let Some(group_index) = group_index {
                &mut groups[group_index].1
            } else {
                groups.push((
                    crate::data::source::copy_source_string(dimension)?,
                    Vec::new(),
                ));
                &mut groups.last_mut().expect("new guide field").1
            };
            for level in inputs.observed_color_levels(plot)? {
                if crate::data::source::source_position(levels, |existing| existing == &level)
                    .is_none()
                {
                    crate::data::source::reserve_source(levels, 1)?;
                    levels.push(level);
                }
            }
        }
        crate::data::source::source_arc(groups).map(Some)
    }

    /// Prepare every local contribution or explicit absence, then retain the
    /// one Shared union when selected. No body or font work determines presence.
    fn prepare_discrete_guides(
        &self,
        themes: &ResolvedCanvasThemes,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
        hue_groups: Option<&[(String, Vec<String>)]>,
        resources: &crate::font::canonical::OperationContext,
    ) -> Result<Option<Arc<PreparedGuide>>, Error> {
        let mut contributions = Vec::new();
        crate::data::source::reserve_source(&mut contributions, self.panels.len())?;
        for (index, plot) in self.panels.iter().enumerate() {
            let Some(source) = plot else { continue };
            let mut plot = self.cell_visibility(source.clone(), index);
            if let Some(labels) = &self.labels {
                plot = plot.labels(labels.clone());
            }
            if let Some((_, levels)) = hue_groups.and_then(|groups| {
                groups
                    .iter()
                    .find(|(field, _)| Some(field.as_str()) == plot.categorical_color_field())
            }) {
                let levels = crate::data::source::collect_source(
                    levels
                        .iter()
                        .map(|level| crate::data::source::copy_source_string(level)),
                )?;
                plot = plot.with_global_color_levels(levels);
            }
            let theme = themes
                .plot(index)
                .expect("configured Canvas panel has a resolved theme");
            let inputs = interiors
                .get(index)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    Error::Render("Canvas point/path guide is missing prepared semantics".into())
                })?;
            let owner = crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index).map_err(|_| {
                    Error::Render("Canvas guide contributor address overflow".into())
                })?,
            )?;
            inputs.prepare_discrete_guides(&plot, theme, resources, &owner, false)?;
            if self.legend_policy == CanvasLegendPolicy::Shared {
                if let Some(descriptor) = &inputs
                    .discrete_guides
                    .get()
                    .expect("prepared Canvas guide")
                    .root
                {
                    contributions.push(Arc::clone(descriptor));
                }
            }
        }
        PreparedGuide::merge(&contributions, &[], "shared canvas legend")
    }

    fn prepare_shared_x(
        &self,
        stage: &mut crate::layout::kernel::PreparedStage<'_>,
        scatter: &[Option<crate::plots::scatter::PreparedScatter>],
        line: &[Option<crate::plots::line::PreparedLine>],
        reference: &[Option<crate::plots::shared_x::PreparedReference>],
    ) -> Result<Vec<Option<(f64, f64)>>, Error> {
        let mut ranges = Vec::new();
        if self.x_range_groups.is_empty() {
            return Ok(ranges);
        }

        stage.reserve_prepared(&mut ranges, self.panels.len(), "shared x domains")?;
        ranges.resize(self.panels.len(), None);
        for group in &self.x_range_groups {
            let mut shared: Option<(f64, f64)> = None;
            for &index in group {
                let plot = self.panels[index]
                    .as_ref()
                    .expect("validated shared x member");
                let range = if let Some(marks) = &reference[index] {
                    marks.x_range(plot)?
                } else if plot.kind() == PlotKind::Scatter {
                    crate::layout::continuous_auto_range(
                        scatter[index]
                            .as_ref()
                            .ok_or_else(|| {
                                Error::Render(
                                    "shared Scatter range requires root-prepared inputs".into(),
                                )
                            })?
                            .ranges[0],
                        plot.x_axis_spec(),
                    )?
                } else if matches!(
                    plot.kind(),
                    PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
                ) {
                    let marks = line[index].as_ref().ok_or_else(|| {
                        Error::Render("shared line range requires root-prepared inputs".into())
                    })?;
                    if marks.conditions.is_some() {
                        return Err(Error::Data(
                            "canvas x-range group has a categorical x axis".into(),
                        ));
                    }
                    marks.ranges[0]
                } else {
                    crate::plots::shared_x::domain(plot, &stage.operation())?
                };
                shared = Some(
                    shared.map_or(range, |prior| (prior.0.min(range.0), prior.1.max(range.1))),
                );
            }
            let shared = shared.expect("validated nonempty shared x group");
            for &index in group {
                let plot = self.panels[index]
                    .as_ref()
                    .expect("validated shared x member");
                if let Some(ticks) = plot.explicit_x_ticks() {
                    for tick in ticks {
                        let position = if plot.kind() == PlotKind::Forest
                            && plot.axis_scale_spec() == crate::AxisScale::Log10
                        {
                            crate::numeric::log10(tick.position)?
                        } else {
                            tick.position
                        };
                        if !position.is_finite() || position < shared.0 || position > shared.1 {
                            return Err(Error::Data(format!("x-axis explicit tick position {} is outside resolved shared range {}..{}", tick.position, shared.0, shared.1)));
                        }
                    }
                }
                ranges[index] = Some(shared);
            }
        }
        Ok(ranges)
    }

    #[expect(
        clippy::type_complexity,
        clippy::too_many_lines,
        reason = "root component inputs and their coordinated x domains are prepared together before fonts"
    )]
    fn prepare_interiors(
        &self,
        stage: &mut crate::layout::kernel::PreparedStage<'_>,
        themes: Option<&ResolvedCanvasThemes>,
    ) -> Result<
        (
            Vec<Option<crate::layout::interior::PreparedInteriorInputs>>,
            Vec<Option<(f64, f64)>>,
        ),
        Error,
    > {
        let mut interiors = Vec::new();
        stage.reserve_prepared(&mut interiors, self.panels.len(), "Canvas interior inputs")?;
        let resources = stage.operation();
        let mut scatter = Vec::new();
        stage.reserve_prepared(&mut scatter, self.panels.len(), "Canvas Scatter semantics")?;
        let mut line = Vec::new();
        stage.reserve_prepared(&mut line, self.panels.len(), "Canvas line semantics")?;
        let mut reference = Vec::new();
        stage.reserve_prepared(
            &mut reference,
            self.panels.len(),
            "Canvas distribution/reference semantics",
        )?;
        for (index, plot) in self.panels.iter().enumerate() {
            let suppressed = plot
                .as_ref()
                .filter(|plot| {
                    matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm)
                        && matches!(
                            self.legend_policy,
                            CanvasLegendPolicy::Shared | CanvasLegendPolicy::Hidden
                        )
                })
                .map(|plot| plot.clone().with_suppressed_continuous_guide());
            let plot = if suppressed.is_some() {
                &suppressed
            } else {
                plot
            };
            reference.push(
                match plot.as_ref().filter(|plot| {
                    matches!(
                        plot.kind(),
                        PlotKind::Histogram
                            | PlotKind::Density
                            | PlotKind::Qq
                            | PlotKind::Regression
                            | PlotKind::Dumbbell
                            | PlotKind::PointInterval
                            | PlotKind::Coefficient
                            | PlotKind::Forest
                            | PlotKind::Bar
                            | PlotKind::HorizontalBar
                            | PlotKind::StackedBar
                            | PlotKind::StackedHorizontalBar
                            | PlotKind::Boxplot
                            | PlotKind::Violin
                            | PlotKind::Raincloud
                            | PlotKind::Interaction
                            | PlotKind::Heatmap
                            | PlotKind::Beeswarm
                            | PlotKind::Waterfall
                            | PlotKind::Marginal
                    )
                }) {
                    Some(plot) => {
                        let data = plot
                            .configured_source_rows()?
                            .map_or_else(|| plot.data(), |rows| plot.data().map_source_rows(rows))
                            .with_operation(&resources);
                        Some(crate::plots::shared_x::PreparedReference::new(
                            plot,
                            data,
                            themes
                                .and_then(|themes| themes.plot(index))
                                .map(|theme| &**theme),
                        )?)
                    }
                    None => None,
                },
            );
            line.push(
                match plot.as_ref().filter(|plot| {
                    matches!(
                        plot.kind(),
                        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
                    )
                }) {
                    Some(plot) => {
                        let data = plot
                            .configured_source_rows()?
                            .map_or_else(|| plot.data(), |rows| plot.data().map_source_rows(rows))
                            .with_operation(&resources);
                        Some(crate::plots::line::PreparedLine::new(plot, data, None)?)
                    }
                    None => None,
                },
            );
            scatter.push(
                match plot
                    .as_ref()
                    .filter(|plot| plot.kind() == PlotKind::Scatter)
                {
                    Some(plot) => {
                        let data = plot
                            .configured_source_rows()?
                            .map_or_else(|| plot.data(), |rows| plot.data().map_source_rows(rows))
                            .with_operation(&resources);
                        Some(crate::plots::scatter::PreparedScatter::new(plot, data)?)
                    }
                    None => None,
                },
            );
        }
        let shared_x = self.prepare_shared_x(stage, &scatter, &line, &reference)?;
        for (index, plot) in self.panels.iter().enumerate() {
            interiors.push(
                match plot.as_ref().filter(|plot| {
                    matches!(
                        plot.kind(),
                        PlotKind::Scatter
                            | PlotKind::Histogram
                            | PlotKind::Density
                            | PlotKind::Qq
                            | PlotKind::Regression
                            | PlotKind::Dumbbell
                            | PlotKind::PointInterval
                            | PlotKind::Coefficient
                            | PlotKind::Forest
                            | PlotKind::Bar
                            | PlotKind::HorizontalBar
                            | PlotKind::StackedBar
                            | PlotKind::StackedHorizontalBar
                            | PlotKind::Boxplot
                            | PlotKind::Violin
                            | PlotKind::Raincloud
                            | PlotKind::Interaction
                            | PlotKind::Heatmap
                            | PlotKind::Beeswarm
                            | PlotKind::Waterfall
                            | PlotKind::Marginal
                            | PlotKind::Line
                            | PlotKind::Trajectory
                            | PlotKind::LinePath
                    )
                }) {
                    Some(plot) if reference[index].is_some() => {
                        let scope_bytes = 6 + index
                            .checked_ilog10()
                            .map_or(1, |digits| digits as usize + 1);
                        let mut scope = String::new();
                        crate::data::source::reserve_source_string(&mut scope, scope_bytes)?;
                        std::fmt::Write::write_fmt(&mut scope, format_args!("panel-{index}"))
                            .map_err(|error| Error::Render(format!("Canvas scope: {error}")))?;
                        Some(crate::plots::shared_x::prepare_reference_plot(
                            plot,
                            reference[index]
                                .take()
                                .expect("prepared reference component"),
                            Some(&scope),
                            shared_x.get(index).copied().flatten(),
                            self.labels.as_ref(),
                            &resources,
                        )?)
                    }
                    Some(plot) => {
                        let mut scope = String::new();
                        crate::data::source::reserve_source_string(&mut scope, 32)?;
                        write!(scope, "panel-{index}").expect("admitted scope");
                        let mut plot = plot.clone();
                        if let Some(labels) = &self.labels {
                            plot = plot.labels(labels.clone());
                        }
                        let mut plot = plot.with_render_scope(scope);
                        if let Some(Some(range)) = shared_x.get(index) {
                            plot = plot.with_planned_x_limits(*range);
                        }
                        Some(if plot.kind() == PlotKind::Scatter {
                            crate::plots::shared_x::prepare_scatter_plot(
                                &plot,
                                scatter[index].take().expect("prepared Scatter component"),
                                &resources,
                            )?
                        } else {
                            crate::plots::shared_x::prepare_line_plot(
                                &plot,
                                line[index].take().expect("prepared line component"),
                                &resources,
                            )?
                        })
                    }
                    None => None,
                },
            );
        }
        for group in &self.x_range_groups {
            if !group.iter().any(|&index| {
                self.panels[index]
                    .as_ref()
                    .is_some_and(|plot| plot.user_x_limits().is_none())
            }) {
                continue;
            }
            for &index in group {
                let Some(inputs) = interiors[index].as_mut() else {
                    continue;
                };
                let Some(waterfall) = inputs.waterfall.as_mut() else {
                    continue;
                };
                let waterfall = Arc::get_mut(waterfall).ok_or_else(|| {
                    Error::Render("Waterfall totals were aliased before window preparation".into())
                })?;
                for (scope, marks) in &mut waterfall.panels {
                    let axes = inputs
                        .axes
                        .iter()
                        .find(|(candidate, _)| candidate == scope)
                        .map(|(_, axes)| axes)
                        .ok_or_else(|| {
                            Error::Render("Waterfall total window scope missing".into())
                        })?;
                    marks.prepare_totals(axes, true)?;
                }
            }
        }
        Ok((interiors, shared_x))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one ordered semantic, font, intrinsic, allocation, and capture lifecycle"
    )]
    fn preflight_plan(
        &self,
    ) -> Result<
        crate::layout::FrozenRoot<crate::orchestrator::TextReplayPlan, PreparedCanvasInputs>,
        Error,
    > {
        let sizing_request = self.root_size.requested_dimensions();
        let mut children = crate::decorations::plan::children::ChildPlans::canvas(self);
        let measurements = sizing::HeightMeasurements::default();
        let root = crate::layout::plan_exact_root(
            sizing_request.0,
            sizing_request.1,
            |prepared_stage| {
                let themes = Arc::new(ResolvedCanvasThemes::new(&self.theme, &self.panels)?);
                self.validate_structure()?;
                let (mut interiors, shared_x) =
                    self.prepare_interiors(prepared_stage, Some(&themes))?;
                let color_domains = self.prepare_color_domains(&interiors)?;
                for (index, plot) in self.panels.iter().enumerate() {
                    if let (Some(plot), Some(inputs)) = (plot, interiors[index].as_ref()) {
                        let global_hues = color_domains.as_ref().and_then(|groups| {
                            groups
                                .iter()
                                .find(|(field, _)| {
                                    Some(field.as_str()) == plot.categorical_color_field()
                                })
                                .map(|(_, levels)| levels.as_slice())
                        });
                        inputs.validate_categorical_scale(
                            plot,
                            themes.plot(index).expect("configured Canvas theme"),
                            global_hues,
                        )?;
                    }
                }
                let continuous_shared_guide =
                    self.prepare_continuous_shared_guide(&themes, &interiors)?;
                if let Some(descriptor) = &continuous_shared_guide {
                    self.install_shared_continuous_domain(&mut interiors, descriptor)?;
                }
                let shared_guide = self.prepare_discrete_guides(
                    &themes,
                    &interiors,
                    color_domains.as_deref().map(Vec::as_slice),
                    &prepared_stage.operation(),
                )?;
                let (mut census, scalar_presence) = self.foundational_text_census(
                    &themes,
                    &interiors,
                    shared_guide.as_ref(),
                    prepared_stage,
                )?;
                themes.prepare_font_environment(&mut census, scalar_presence, prepared_stage)?;
                let (root_text, text_panel_intrinsics, plot_intrinsics) =
                    self.prepare_intrinsics(&themes, &interiors)?;
                Ok(PreparedCanvasInputs {
                    themes,
                    text_census: census,
                    root_text,
                    shared_x,
                    color_domains,
                    shared_guide,
                    interiors,
                    text_panel_intrinsics,
                    plot_intrinsics,
                })
            },
            |prepared, assigned, round| {
                let assigned = sizing::measure(self, prepared, assigned, round, &measurements)?;
                if let Some(guide) = &prepared.shared_guide {
                    round.observe_guide(guide)?;
                }
                prepared
                    .root_text
                    .check_width(
                        f64::from(assigned.dimensions()?.0) - 32.0,
                        &FitScope::root(),
                    )
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                #[cfg(test)]
                let backend_begins =
                    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get);
                let plan = self.build_canvas_plan(
                    Arc::clone(&prepared.themes),
                    assigned,
                    round,
                    &mut children,
                    &prepared.shared_x,
                    prepared.color_domains.as_deref().map(Vec::as_slice),
                    prepared.shared_guide.as_ref(),
                    &prepared.text_panel_intrinsics,
                    &prepared.plot_intrinsics,
                    &prepared.interiors,
                    sizing_request.1.is_none(),
                    Some(&measurements),
                )?;
                #[cfg(test)]
                assert_eq!(
                    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
                    backend_begins,
                    "Canvas measurement must not open a backend"
                );
                children.finish_round(round);
                let assigned = crate::layout::RootSize::new(plan.dimensions.0, plan.dimensions.1);
                Ok(crate::layout::MeasuredLayout::new(assigned, plan))
            },
            |_, assigned, plan, final_stage| {
                let (width, height) = assigned.dimensions()?;
                #[cfg(test)]
                let before = lifecycle_tests::capture_counters();
                #[cfg(test)]
                let panel_measurements =
                    crate::orchestrator::RENDER_PANEL_CALLS.with(std::cell::Cell::get);
                let replay = crate::orchestrator::capture_text_replay_plan(
                    width,
                    height,
                    "prepared canvas scene capture",
                    plan.themes.root(),
                    final_stage,
                    |backend| Self::render_planned_canvas(&plan, width, height, backend),
                )?;
                #[cfg(test)]
                assert_eq!(
                    lifecycle_tests::capture_counters(),
                    before,
                    "Canvas first capture must not read, prepare, project, shape, or measure"
                );
                #[cfg(test)]
                assert_eq!(
                    crate::orchestrator::RENDER_PANEL_CALLS.with(std::cell::Cell::get),
                    panel_measurements,
                    "Canvas capture must not enter the panel measurement path"
                );
                Ok(replay)
            },
        )?;
        let (width, height) = root.dimensions();
        crate::orchestrator::validate_text_replay_plan(
            width,
            height,
            root.plan(),
            "prepared canvas Null replay",
        )?;
        Ok(root)
    }

    #[expect(
        clippy::type_complexity,
        reason = "one prepared root and both cell-intrinsic inventories are retained before parent allocation"
    )]
    fn prepare_intrinsics(
        &self,
        themes: &ResolvedCanvasThemes,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
    ) -> Result<
        (
            crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
            Vec<Option<text_panel::TextPanelIntrinsics>>,
            Vec<Option<crate::decorations::title_block::plan::RootTitleBlockIntrinsics>>,
        ),
        Error,
    > {
        // All summaries use the prepared labels and the actual owner theme,
        // after the font census and before any parent allocation or child probe.
        for (index, inputs) in interiors.iter().enumerate() {
            if let Some(inputs) = inputs {
                let source = self.panels[index].as_ref().expect("prepared Canvas panel");
                let plot = self.cell_visibility(source.clone(), index);
                inputs.prepare_intrinsics(
                    &plot,
                    themes.plot(index).expect("prepared Canvas panel theme"),
                )?;
            }
        }
        let mut text_panels = Vec::new();
        crate::layout::fit::reserve(
            &mut text_panels,
            self.text_panels.len(),
            "TextPanel intrinsic summaries",
        )?;
        for panel in &self.text_panels {
            text_panels.push(
                panel
                    .as_ref()
                    .map(|panel| text_panel::TextPanelIntrinsics::prepare(panel, themes.root()))
                    .transpose()?,
            );
        }
        let mut plot_titles = Vec::new();
        crate::layout::fit::reserve(
            &mut plot_titles,
            self.panels.len(),
            "Canvas Plot text intrinsics",
        )?;

        for (index, plot) in self.panels.iter().enumerate() {
            let summary = plot
                .as_ref()
                .map(|plot| -> Result<_, Error> {
                    let mut scope = String::new();
                    crate::data::source::reserve_source_string(&mut scope, 32)?;
                    write!(scope, "panel-{index}").expect("reserved Canvas scope");
                    crate::decorations::title_block::plan::RootTitleBlockPlan::intrinsic_widths(
                        themes.plot(index).expect("configured Canvas Plot theme"),
                        plot.prose(),
                        Some(&scope),
                    )
                })
                .transpose()?;
            plot_titles.push(summary);
        }
        let root = crate::decorations::title_block::plan::RootTitleBlockPlan::intrinsic_widths(
            themes.root(),
            self.prose(),
            None,
        )?;
        Ok((root, text_panels, plot_titles))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the Canvas census keeps canonical root, composition, and panel preorder explicit"
    )]
    fn foundational_text_census(
        &self,
        themes: &ResolvedCanvasThemes,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
        shared_guide: Option<&Arc<PreparedGuide>>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<(crate::layout::PreparedTextCensus, Vec<u64>), Error> {
        let root = crate::layout::ComponentAddress::root();
        let mut requirements = Vec::new();
        for (source, role, address_role, appearance) in [
            (
                self.title.content(),
                crate::layout::PreparedTextRole::Title,
                crate::layout::AddressRole::RootTitle,
                &themes.root().typography.root.title,
            ),
            (
                self.subtitle.content(),
                crate::layout::PreparedTextRole::Subtitle,
                crate::layout::AddressRole::RootSubtitle,
                &themes.root().typography.root.subtitle,
            ),
            (
                self.caption.content(),
                crate::layout::PreparedTextRole::Caption,
                crate::layout::AddressRole::RootCaption,
                &themes.root().typography.root.caption,
            ),
        ] {
            if let Some(source) = source {
                let address = root.child(address_role, 0)?;
                crate::layout::PreparedTextRequirement::push_authored(
                    &mut requirements,
                    address,
                    role,
                    Some(source),
                    appearance,
                    true,
                    Some(prepared_stage),
                )?;
            }
        }

        if let Some(guide) = shared_guide {
            guide.append_requirements(
                &mut requirements,
                themes.root(),
                &root.child(crate::layout::AddressRole::CanvasSharedLegend, 0)?,
                Some(prepared_stage),
            )?;
        }
        let composition_title = &themes.root().typography.composition.composition_header;
        for (index, header) in self.column_headers.iter().enumerate() {
            let address = root.child(
                crate::layout::AddressRole::CanvasHeader,
                u32::try_from(index).map_err(|_| {
                    Error::Render("canvas column-header census index overflowed".into())
                })?,
            )?;
            crate::layout::PreparedTextRequirement::push_authored(
                &mut requirements,
                address,
                crate::layout::PreparedTextRole::Header,
                Some(&header.text),
                composition_title,
                false,
                Some(prepared_stage),
            )?;
        }
        for (index, header) in self.row_headers.iter().enumerate() {
            let address = root.child(
                crate::layout::AddressRole::CanvasHeader,
                u32::try_from(self.column_headers.len() + index).map_err(|_| {
                    Error::Render("canvas row-header census index overflowed".into())
                })?,
            )?;
            crate::layout::PreparedTextRequirement::push_authored(
                &mut requirements,
                address,
                crate::layout::PreparedTextRole::Header,
                Some(&header.text),
                composition_title,
                false,
                Some(prepared_stage),
            )?;
        }
        for (index, title) in self.row_titles.iter().enumerate() {
            let address = root.child(
                crate::layout::AddressRole::CanvasRowTitle,
                u32::try_from(index).map_err(|_| {
                    Error::Render("canvas row-title census index overflowed".into())
                })?,
            )?;
            crate::layout::PreparedTextRequirement::push_authored(
                &mut requirements,
                address,
                crate::layout::PreparedTextRole::Header,
                Some(&title.text),
                composition_title,
                false,
                Some(prepared_stage),
            )?;
        }
        for (index, panel) in self.text_panels.iter().enumerate() {
            let Some(panel) = panel else { continue };
            let address = root.child(
                crate::layout::AddressRole::TextPanelContent,
                u32::try_from(index)
                    .map_err(|_| Error::Render("text-panel census index overflowed".into()))?,
            )?;
            let [title_style, body_style] = panel.appearances(themes.root());
            for (slot, role, source, appearance) in [
                (
                    0,
                    crate::layout::PreparedTextRole::Header,
                    panel.title(),
                    title_style.theme,
                ),
                (
                    1,
                    crate::layout::PreparedTextRole::Body,
                    panel.body(),
                    body_style.theme,
                ),
            ] {
                let requirement_address = address.clone();
                push_text_requirement(
                    &mut requirements,
                    requirement_address,
                    role,
                    0,
                    source,
                    appearance,
                    prepared_stage,
                )?;
                push_text_requirement(
                    &mut requirements,
                    address.child(crate::layout::AddressRole::PreferenceMetric, slot)?,
                    crate::layout::PreparedTextRole::PreferenceMetric,
                    0,
                    "0",
                    appearance,
                    prepared_stage,
                )?;
            }
        }

        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };

            let mut plot = plot.clone();
            if let Some(labels) = &self.labels {
                plot = plot.labels(labels.clone());
            }
            let mut plot = self.cell_visibility(plot, index);
            if self.legend_policy == CanvasLegendPolicy::Shared {
                plot = plot.with_suppressed_legend();
            }
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            let cell = root.child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas text census index overflowed".into()))?,
            )?;
            let filter_point_label_x_to_limits = !self
                .x_range_groups
                .iter()
                .any(|group| group.contains(&index));
            crate::layout::PreparedTextRequirement::append_plot(
                interiors.get(index).and_then(Option::as_ref),
                &mut requirements,
                &plot,
                plot.kind(),
                theme,
                &cell,
                true,
                filter_point_label_x_to_limits,
                true,
                Some(prepared_stage),
            )?;
        }
        crate::layout::PreparedTextRequirement::append_preference_metrics(
            &mut requirements,
            themes.root(),
            &root,
            prepared_stage,
        )?;
        for (index, plot) in self.panels.iter().enumerate() {
            if plot.is_some() {
                let owner = root.child(
                    crate::layout::AddressRole::CanvasCell,
                    u32::try_from(index).map_err(|_| {
                        Error::Render("Canvas metric census owner overflowed".into())
                    })?,
                )?;
                crate::layout::PreparedTextRequirement::append_preference_metrics(
                    &mut requirements,
                    themes
                        .plot(index)
                        .expect("configured Canvas Plot retains theme"),
                    &owner,
                    prepared_stage,
                )?;
            }
        }
        let text_topology =
            crate::layout::PreparedTextCensus::structural_address_count(&requirements)
                .map_err(|error| Error::Render(format!("prepared canvas text census: {error}")))?;
        let mut scalar_presence = prepared_stage.reserve_scalar_presence_map()?;
        let census = crate::layout::PreparedTextCensus::materialize(
            requirements,
            Some(prepared_stage),
            Some(&mut scalar_presence),
        )
        .map_err(|error| Error::Render(format!("prepared canvas text census: {error}")))?;
        if census.counts.structural_addresses != text_topology {
            return Err(Error::Render(format!(
                "prepared canvas text topology counted {text_topology} addresses but materialized {}",
                census.counts.structural_addresses
            )));
        }
        Ok((census, scalar_presence))
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "one Canvas round consumes separate retained domains, hue identities, and child inputs"
    )]
    fn build_canvas_plan(
        &self,
        themes: Arc<ResolvedCanvasThemes>,
        assigned: crate::layout::RootSize,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans,
        shared_x: &[Option<(f64, f64)>],
        color_domains: Option<&[(String, Vec<String>)]>,
        prepared_guide: Option<&Arc<PreparedGuide>>,
        text_panel_intrinsics: &[Option<text_panel::TextPanelIntrinsics>],
        plot_intrinsics: &[Option<
            crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
        >],
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
        content_sizing: bool,
        measurements: Option<&sizing::HeightMeasurements>,
    ) -> Result<CanvasPlan, Error> {
        use crate::layout::fit::FitOutcome;
        use crate::layout::kernel::TrialOutcome;
        let prepared = self.prepare_canvas_round(
            themes,
            assigned,
            round,
            children,
            shared_x,
            color_domains,
            prepared_guide,
            text_panel_intrinsics,
            plot_intrinsics,
            interiors,
            content_sizing,
            measurements,
        )?;
        let mut compact_changes = true;
        if self.legend_policy != CanvasLegendPolicy::Shared || prepared.shared_legend.is_none() {
            return match self.build_canvas_attempt(
                &prepared,
                assigned,
                round,
                None,
                &mut compact_changes,
                children,
            )? {
                FitOutcome::Measured(plan) => Ok(plan),
                FitOutcome::Miss(reason) => Err(reason.into_error()),
            };
        }
        let owner = crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::CanvasSharedLegend, 0)?;
        let mut first_miss = None;
        for (edge_index, edge) in [
            crate::LegendPosition::OutsideRight,
            crate::LegendPosition::OutsideTop,
            crate::LegendPosition::OutsideBottom,
            crate::LegendPosition::OutsideLeft,
        ]
        .into_iter()
        .enumerate()
        {
            compact_changes = true;
            for compact in [false, true] {
                if compact && !compact_changes {
                    continue;
                }
                round.claim_guide_trial(
                    &owner,
                    8,
                    u32::try_from(edge_index * 2 + usize::from(compact))
                        .map_err(|_| Error::Render("Canvas guide ordinal overflowed".into()))?,
                )?;
                let checkpoint = round.begin_trial()?;
                #[cfg(test)]
                let guide_trial = prepared.shared_legend.is_some().then(|| {
                    crate::orchestrator::measurements::GuideTrial::new(&owner, edge, compact)
                });
                round.observe_count(edge as usize)?;
                round.observe_count(usize::from(compact))?;
                let outcome = self.build_canvas_attempt(
                    &prepared,
                    assigned,
                    round,
                    Some((edge, compact)),
                    &mut compact_changes,
                    children,
                )?;
                match outcome {
                    FitOutcome::Measured(plan) => match round.finish_trial(checkpoint) {
                        _outcome @ (TrialOutcome::Fit | TrialOutcome::Provisional) => {
                            #[cfg(test)]
                            #[expect(
                                clippy::used_underscore_binding,
                                reason = "outcome is observed only in tests"
                            )]
                            if let Some(trial) = &guide_trial {
                                trial.complete(&_outcome);
                            }
                            return Ok(plan);
                        }
                        TrialOutcome::Miss(reason) => {
                            #[cfg(test)]
                            if let Some(trial) = &guide_trial {
                                trial.reject();
                            }
                            FitFailure::retain_least(&mut first_miss, reason);
                        }
                    },
                    FitOutcome::Miss(reason) => {
                        #[cfg(test)]
                        if let Some(trial) = &guide_trial {
                            trial.reject();
                        }
                        round.record_failure(reason)?;
                        if let TrialOutcome::Miss(reason) = round.finish_trial(checkpoint) {
                            FitFailure::retain_least(&mut first_miss, reason);
                        }
                    }
                }
            }
        }
        Err(first_miss
            .unwrap_or_else(|| {
                FitScope::new(&owner, 0).failure(
                    FitKind::LegendOrGuide,
                    "shared guide cannot fit its containing Canvas frame",
                )
            })
            .into_error())
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "admitted scoped configuration and shared-guide preparation form one parent round"
    )]
    fn prepare_canvas_round<'a>(
        &self,
        themes: Arc<ResolvedCanvasThemes>,
        assigned: crate::layout::RootSize,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans<'_>,
        shared_x: &'a [Option<(f64, f64)>],
        color_domains: Option<&[(String, Vec<String>)]>,
        prepared_guide: Option<&'a Arc<PreparedGuide>>,
        text_panel_intrinsics: &'a [Option<text_panel::TextPanelIntrinsics>],
        plot_intrinsics: &'a [Option<
            crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
        >],
        interiors: &'a [Option<crate::layout::interior::PreparedInteriorInputs>],
        content_sizing: bool,
        measurements: Option<&'a sizing::HeightMeasurements>,
    ) -> Result<PreparedCanvasRound<'a>, Error> {
        let (width, height) = assigned.dimensions()?;

        let canvas_layout = RootFramePlan::measure(
            (width, height),
            themes.root(),
            self.prose(),
            crate::decorations::title_block::plan::RootAlignments::resolve(
                themes.root(),
                self.prose_alignments(),
                [None; 2],
            ),
            (
                crate::layout::canvas::MINIMUM_PANEL_SPAN,
                crate::layout::canvas::MINIMUM_PANEL_SPAN,
            ),
        )
        .map_err(crate::layout::fit::PlanError::into_error)?;
        let full_outer = canvas_layout.panel_box;
        round.observe_canvas(&canvas_layout)?;

        let mut plots = Vec::new();
        crate::layout::fit::reserve(
            &mut plots,
            self.panels.len(),
            "Canvas scoped configurations",
        )?;
        for (index, source) in self.panels.iter().enumerate() {
            let Some(source) = source else {
                plots.push(None);
                continue;
            };

            let mut scope = String::new();
            crate::data::source::reserve_source_string(&mut scope, 32)?;
            write!(scope, "panel-{index}").expect("admitted scope");
            let mut plot = source.clone();
            if let Some(labels) = &self.labels {
                plot = plot.labels(labels.clone());
            }
            plot = plot.with_render_scope(scope);
            if let Some(inputs) = interiors.get(index).and_then(Option::as_ref) {
                plot = inputs.attach(plot)?;
            }
            if let Some(Some(range)) = shared_x.get(index) {
                let kind = plot.kind();
                plot = plot.with_resolved_facet_ranges(kind, Some(*range), None);
            }
            plot = self.cell_visibility(plot, index);
            plots.push(Some(plot));
        }
        let hue_groups: &[(String, Vec<String>)] =
            if self.legend_policy != CanvasLegendPolicy::Shared {
                &[]
            } else if let Some(hues) = color_domains {
                hues
            } else {
                return Err(Error::Render(
                    "Canvas shared color domains were not prepared".into(),
                ));
            };
        for slot in &mut plots {
            if let Some(plot) = slot.take() {
                let levels = hue_groups.iter().find(|(dimension, _)| {
                    Some(dimension.as_str()) == plot.categorical_color_field()
                });
                let plot = if let Some((_, levels)) = levels {
                    let levels = crate::data::source::collect_source(
                        levels
                            .iter()
                            .map(|level| crate::data::source::copy_source_string(level)),
                    )?;
                    plot.with_global_color_levels(levels)
                } else {
                    plot
                };
                *slot = Some(
                    plot.with_canvas_size(width, height)
                        .with_parent_axis_gutters(),
                );
            }
        }
        let (shared_legend, body_probes) = if self.legend_policy == CanvasLegendPolicy::Shared {
            Self::preflight_shared_legend(
                &plots,
                &themes,
                full_outer,
                children,
                round,
                prepared_guide,
            )?
        } else {
            (None, Vec::new())
        };

        Ok(PreparedCanvasRound {
            content_sizing,
            measurements,
            interiors,
            themes,
            canvas_layout,
            plots,
            shared_legend,
            body_probes,
            shared_x,
            text_panel_intrinsics,
            plot_intrinsics,
        })
    }

    fn cell_visibility(&self, mut plot: Plot, index: usize) -> Plot {
        if self.legend_policy == CanvasLegendPolicy::Shared {
            plot = plot.with_parent_legend_composition(self.legend_composition);
        }
        if self.legend_policy == CanvasLegendPolicy::Hidden {
            plot = plot
                .with_suppressed_legend()
                .with_suppressed_continuous_guide();
        } else if self.legend_policy == CanvasLegendPolicy::Shared
            && matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm)
        {
            plot = plot.with_suppressed_continuous_guide();
        }
        if self.category_labels == CanvasCategoryLabels::FirstColumnPerRow
            && !index.is_multiple_of(self.columns)
        {
            plot = plot.with_y_tick_labels_visible(false);
        }
        plot
    }

    /// Install the same coordinated domain and visibility inputs before every
    /// width-dependent height pass and the final assigned child measurements.
    fn coordinate_plots(
        &self,
        plots: &mut [Option<Plot>],
        shared_x: &[Option<(f64, f64)>],
        round: &crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        for group in &self.x_range_groups {
            let mut shared: Option<(f64, f64)> = None;
            for &index in group {
                let base = shared_x[index].expect("prepared shared x member");
                let address = crate::layout::ComponentAddress::root().child(
                    crate::layout::AddressRole::CanvasCell,
                    u32::try_from(index).map_err(|_| {
                        Error::Render("Canvas coordinated address overflowed".into())
                    })?,
                )?;
                let range = round
                    .feedback_domain(&address, FeedbackKind::AutomaticDomainX)
                    .map_or(base, |range| (base.0.min(range.0), base.1.max(range.1)));
                shared = Some(shared.map_or(range, |current| {
                    (current.0.min(range.0), current.1.max(range.1))
                }));
            }
            let shared = shared.expect("validated nonempty x-range group");
            if !shared.0.is_finite() || !shared.1.is_finite() || shared.0 >= shared.1 {
                return Err(Error::Render(
                    "Canvas shared x domain must be finite and ascending".into(),
                ));
            }
            for &index in group {
                let plot = plots[index].take().expect("validated grouped panel");
                let kind = plot.kind();
                plots[index] = Some(plot.with_resolved_facet_ranges(kind, Some(shared), None));
            }
        }
        for (index, slot) in plots.iter_mut().enumerate() {
            let Some(mut plot) = slot.take() else {
                continue;
            };
            let address = crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("Canvas coordinated address overflowed".into()))?,
            )?;
            let mut grouped = false;
            for group in &self.x_range_groups {
                for &member in group {
                    if member == index {
                        grouped = true;
                        break;
                    }
                }
                if grouped {
                    break;
                }
            }
            if !grouped {
                if let Some(range) = round.feedback_domain(&address, FeedbackKind::AutomaticDomainX)
                {
                    plot = plot.with_planned_x_limits(range);
                }
            }
            if let Some(range) = round.feedback_domain(&address, FeedbackKind::AutomaticDomainY) {
                plot = plot.with_planned_y_limits(range);
            }
            *slot = Some(self.cell_visibility(plot, index));
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "gutter probes consume assigned cells and the selected Shared residual allocation"
    )]
    fn complete_cell_gutters(
        &self,
        plots: &[Option<Plot>],
        cells: &[PanelBox],
        themes: &ResolvedCanvasThemes,
        children: &mut crate::decorations::plan::children::ChildPlans,
        round: &crate::layout::kernel::FeedbackRound,
        requirements: &mut [(usize, usize, PanelRequirements)],
        shared_allocation: Option<GuideAllocation>,
    ) -> Result<(), Error> {
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else {
                continue;
            };
            let theme = themes.plot(index).expect("configured Canvas plot theme");
            let dependencies = self.consumed_guide_dependencies(
                index,
                cells[index].width(),
                shared_allocation,
                round,
            )?;
            let guides = GuideConsumption {
                dependencies: &dependencies,
                ..SUPPRESSED_GUIDES
            };
            let probe = children.canvas_probe(
                plot,
                crate::spec::PlotRenderContext::new(plot.data(), cells[index], theme)
                    .with_guides(&guides),
                index,
                MeasurementPhase::Probe,
                round,
            )?;
            let measured = PanelRequirements::measure_header(
                probe
                    .axes
                    .as_ref()
                    .ok_or_else(|| Error::Render("Canvas probe is missing measured axes".into()))?,
                theme,
            )
            .map_err(crate::layout::fit::PlanError::into_error)?;
            let target = &mut requirements[index].2;
            target.left = target.left.max(measured.left);
            target.right = target.right.max(measured.right);
            target.bottom = target.bottom.max(measured.bottom);
            target.top = target.top.max(measured.top);
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        clippy::cast_precision_loss,
        reason = "one Canvas trial allocates declared tracks and measures every affected child"
    )]
    fn build_canvas_attempt(
        &self,
        prepared: &PreparedCanvasRound,
        assigned: crate::layout::RootSize,
        round: &mut crate::layout::kernel::FeedbackRound,
        guide_trial: Option<(crate::LegendPosition, bool)>,
        compact_changes: &mut bool,
        children: &mut crate::decorations::plan::children::ChildPlans,
    ) -> Result<crate::layout::fit::FitOutcome<CanvasPlan>, Error> {
        use crate::layout::fit::{self, FitOutcome};
        let root_address = crate::layout::ComponentAddress::root();
        let guide_owner = root_address.child(crate::layout::AddressRole::CanvasSharedLegend, 0)?;
        let scope = FitScope::new(&root_address, round.attempt_ordinal(&guide_owner));
        let guide_scope = FitScope::new(&guide_owner, scope.ordinal);
        let themes = Arc::clone(&prepared.themes);
        let mut canvas_layout = prepared.canvas_layout.clone();
        let mut full_outer = canvas_layout.panel_box;

        let mut plots = Vec::new();
        fit::reserve(
            &mut plots,
            prepared.plots.len(),
            "Canvas candidate configurations",
        )?;
        for plot in &prepared.plots {
            plots.push(plot.clone());
        }
        self.coordinate_plots(&mut plots, prepared.shared_x, round)?;
        let natural_grid_height = |width: f64,
                                   shared: bool,
                                   round: &crate::layout::kernel::FeedbackRound|
         -> Result<FitOutcome<f64>, Error> {
            // Both height composition and final allocation consume the same
            // root-owned gutters. A probe at a provisional height can choose
            // different ticks and must not replace those scoped inputs locally.
            let gutters = sizing::grid_gutters(self, round)?;
            let grid = tracks::allocate(
                PanelBox::new(0.0, width, 0.0, 0.0),
                (self.rows, self.columns),
                &plots,
                prepared.text_panel_intrinsics,
                prepared.plot_intrinsics,
                &self.column_headers,
                &self.row_headers,
                &self.row_titles,
                &gutters,
                &themes,
                &scope,
                Some(&tracks::ContentSizing {
                    source: self,
                    interiors: prepared.interiors,
                    round,
                    measurements: prepared.measurements,
                    shared_allocation: shared.then(|| GuideAllocation::width(width)).transpose()?,
                    natural_width: false,
                    natural_height: true,
                }),
            )?;
            Ok(match grid {
                FitOutcome::Measured(grid) => FitOutcome::Measured(grid.size.1),
                FitOutcome::Miss(reason) => FitOutcome::Miss(reason),
            })
        };
        let mut outer = full_outer;
        let shared_legend = if let Some(descriptor) = &prepared.shared_legend {
            let (edge, compact) = guide_trial.expect("shared guide has an owned trial");
            let horizontal = matches!(
                edge,
                crate::LegendPosition::OutsideTop | crate::LegendPosition::OutsideBottom
            );
            let gap = themes.root().legend_shared_gap()?;
            let grid_gap = themes.root().composition_grid_gap()?;
            let positive_span = crate::layout::canvas::MINIMUM_PANEL_SPAN;
            let mut columns = Vec::new();
            let mut rows = Vec::new();
            fit::reserve(&mut columns, self.columns, "Canvas guide column minima")?;
            fit::reserve(&mut rows, self.rows, "Canvas guide row minima")?;
            columns.resize(self.columns, positive_span);
            rows.resize(self.rows, positive_span);

            for (index, plot) in plots.iter().enumerate() {
                let hard = if let Some(plot) = plot {
                    plot.intrinsic_body_minimum(
                        themes.plot(index).expect("Plot retains its theme"),
                    )?
                } else if let Some(text) = prepared.text_panel_intrinsics[index] {
                    text.minimum
                } else {
                    continue;
                };
                columns[index % self.columns] = columns[index % self.columns].max(hard.0);
                rows[index / self.columns] = rows[index / self.columns].max(hard.1);
            }
            let minimum_width =
                columns.iter().sum::<f64>() + self.columns.saturating_sub(1) as f64 * grid_gap;
            let minimum_height =
                rows.iter().sum::<f64>() + self.rows.saturating_sub(1) as f64 * grid_gap;
            if !minimum_width.is_finite() || !minimum_height.is_finite() {
                return Err(Error::Render(
                    "Canvas shared-guide child tracks overflowed".into(),
                ));
            }
            let frame = crate::decorations::plan::AssignedGuideFrame::new(
                edge,
                (minimum_width, minimum_height),
                gap,
            );
            let available = frame.available(full_outer, [0.0; 4]);
            let measured = match Legend::measure_assigned(
                Arc::clone(descriptor),
                (
                    available.0,
                    (!prepared.content_sizing).then_some(available.1),
                ),
                horizontal,
                compact,
                themes.root(),
                compact_changes,
                &guide_scope,
            )? {
                FitOutcome::Measured(legend) => legend,
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            };
            let (w, h) = (measured.compute_box_width(), measured.compute_box_height());
            if prepared.content_sizing {
                let grid_width = full_outer.width() - if horizontal { 0.0 } else { w + gap };
                let height = match natural_grid_height(grid_width, true, round)? {
                    FitOutcome::Measured(height) => height,
                    FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
                };
                let height = if horizontal {
                    height + h + gap
                } else {
                    height.max(h)
                };
                canvas_layout =
                    sizing::content_frame(self, themes.root(), assigned.dimensions()?.0, height)?;
                full_outer = canvas_layout.panel_box;
            }
            #[cfg(test)]
            crate::orchestrator::measurements::guide_body();
            outer = match frame.assign_fit(full_outer, Some((w, h)), [0.0; 4], &scope)? {
                FitOutcome::Measured(panel) => panel,
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            };
            let placed = frame.place(outer, (w, h));
            let (left, bottom) = (placed.x0, placed.y0);
            Some(crate::decorations::plan::ResolvedLegend {
                bounds: measured.box_at(left, bottom),
                legend: measured,
            })
        } else {
            if prepared.content_sizing {
                let height = match natural_grid_height(full_outer.width(), false, round)? {
                    FitOutcome::Measured(height) => height,
                    FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
                };
                canvas_layout =
                    sizing::content_frame(self, themes.root(), assigned.dimensions()?.0, height)?;
                full_outer = canvas_layout.panel_box;
                outer = full_outer;
            }
            None
        };
        let shared_allocation = shared_legend
            .as_ref()
            .map(|_| GuideAllocation::bounds(outer))
            .transpose()?;
        let shared_width = shared_legend
            .as_ref()
            .map(|_| GuideAllocation::width(outer.width()))
            .transpose()?;
        let current_height = crate::layout::interior::sizing::integral(
            canvas_layout.panel_box.y1 + canvas_layout.insets.top,
        )?;
        for slot in &mut plots {
            if let Some(plot) = slot.take() {
                *slot = Some(plot.with_canvas_size(assigned.dimensions()?.0, current_height));
            }
        }
        let provisional = match tracks::allocate(
            outer,
            (self.rows, self.columns),
            &plots,
            prepared.text_panel_intrinsics,
            prepared.plot_intrinsics,
            &self.column_headers,
            &self.row_headers,
            &self.row_titles,
            &[],
            &themes,
            &scope.child(crate::layout::AddressRole::CanvasGrid, 0)?,
            prepared
                .content_sizing
                .then_some(tracks::ContentSizing {
                    source: self,
                    interiors: prepared.interiors,
                    round,
                    measurements: prepared.measurements,
                    shared_allocation: shared_width,
                    natural_width: false,
                    natural_height: false,
                })
                .as_ref(),
        )? {
            FitOutcome::Measured(grid) => grid.cells,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
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
                let dependencies = self.consumed_guide_dependencies(
                    first_index,
                    provisional[first_index].width(),
                    shared_allocation,
                    round,
                )?;
                let guides = GuideConsumption {
                    dependencies: &dependencies,
                    ..SUPPRESSED_GUIDES
                };
                let expected = plot_category_domain(
                    first,
                    crate::spec::PlotRenderContext::new(
                        first.data(),
                        provisional[first_index],
                        expected_theme,
                    )
                    .with_guides(&guides),
                    children,
                    first_index,
                    round,
                )?;
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
                    let dependencies = self.consumed_guide_dependencies(
                        index,
                        provisional[index].width(),
                        shared_allocation,
                        round,
                    )?;
                    let guides = GuideConsumption {
                        dependencies: &dependencies,
                        ..SUPPRESSED_GUIDES
                    };
                    if plot_category_domain(
                        plot,
                        crate::spec::PlotRenderContext::new(plot.data(), provisional[index], theme)
                            .with_guides(&guides),
                        children,
                        index,
                        round,
                    )? != expected
                    {
                        return Err(Error::Data(format!(
                            "canvas row {row} category domains differ across panels"
                        )));
                    }
                    plots[index] = plots[index]
                        .take()
                        .map(|plot| plot.with_y_tick_labels_visible(false));
                }
            }
        }
        let grid_address = crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::CanvasGrid, 0)?;
        round.begin_owner(&grid_address)?;
        round.observe_box(outer)?;
        let mut requirements = Vec::new();
        fit::reserve(
            &mut requirements,
            self.panels.len(),
            "Canvas gutter requirements",
        )?;
        for index in 0..self.panels.len() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas grid address overflowed".into()))?,
            )?;
            let [left, right, bottom, top] = round.insets(&address, FeedbackKind::AxisGutter);
            let owned = round.insets(&address, FeedbackKind::OwnedInk);
            let top = top + owned[3];
            requirements.push((
                index / self.columns,
                index % self.columns,
                PanelRequirements {
                    left,
                    right,
                    bottom,
                    top,
                },
            ));
        }
        for (group_index, group) in self.x_range_groups.iter().enumerate() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasRangeGroup,
                u32::try_from(group_index)
                    .map_err(|_| Error::Render("canvas group address overflowed".into()))?,
            )?;
            let alignment = round.insets(&address, FeedbackKind::SharedAlignment);
            for &index in group {
                requirements[index].2.left = requirements[index].2.left.max(alignment[0]);
                requirements[index].2.right = requirements[index].2.right.max(alignment[1]);
            }
        }
        if !prepared.content_sizing {
            self.complete_cell_gutters(
                &plots,
                &provisional,
                &themes,
                children,
                round,
                &mut requirements,
                shared_allocation,
            )?;
        }
        let grid = match tracks::allocate(
            outer,
            (self.rows, self.columns),
            &plots,
            prepared.text_panel_intrinsics,
            prepared.plot_intrinsics,
            &self.column_headers,
            &self.row_headers,
            &self.row_titles,
            &requirements,
            &themes,
            &scope.child(crate::layout::AddressRole::CanvasGrid, 0)?,
            prepared
                .content_sizing
                .then_some(tracks::ContentSizing {
                    source: self,
                    interiors: prepared.interiors,
                    round,
                    measurements: prepared.measurements,
                    shared_allocation: shared_width,
                    natural_width: false,
                    natural_height: false,
                })
                .as_ref(),
        )? {
            FitOutcome::Measured(grid) => grid,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let cells = grid.cells;
        let content_insets = grid.content_insets;
        let headers = grid.headers;
        let row_titles = grid.row_titles;
        let mut measured = Vec::new();
        let mut requirements = Vec::new();
        fit::reserve(&mut measured, plots.len(), "Canvas measured children")?;
        fit::reserve(
            &mut requirements,
            plots.len(),
            "Canvas measured requirements",
        )?;
        let mut child_miss = None;
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let row = index / self.columns;
            let column = index % self.columns;
            let kind = plot.kind();
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            let child_scope = scope.child(crate::layout::AddressRole::CanvasCell, index)?;
            let local = match plot_text::PlotLocalTextPlan::measure(
                plot,
                cells[index],
                theme,
                [self.panel_title_alignment, self.panel_subtitle_alignment],
                prepared.plot_intrinsics[index].expect("configured Canvas Plot text intrinsics"),
                &child_scope,
            )? {
                FitOutcome::Measured(local) => local,
                FitOutcome::Miss(reason) => {
                    FitFailure::retain_least(&mut child_miss, reason);
                    continue;
                }
            };
            if shared_legend.is_some() {
                let base = prepared.body_probes[index]
                    .as_ref()
                    .expect("shared guide body probe");
                if let FitOutcome::Miss(reason) = crate::decorations::plan::check_assigned_body(
                    plot,
                    base,
                    full_outer,
                    local.body,
                    theme,
                    &scope.child(crate::layout::AddressRole::CanvasCell, index)?,
                )? {
                    FitFailure::retain_least(&mut child_miss, reason);
                }
            }
            let owner = crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas address overflowed".into()))?,
            )?;
            let source_rows = plot.configured_source_rows()?;
            let dependencies = self.consumed_guide_dependencies(
                index,
                cells[index].width(),
                shared_allocation,
                round,
            )?;
            let guides = GuideConsumption {
                dependencies: &dependencies,
                ..SUPPRESSED_GUIDES
            };
            let mut context = source_rows
                .map_or_else(
                    || crate::spec::PlotRenderContext::new(plot.data(), local.body, theme),
                    |rows| {
                        crate::spec::PlotRenderContext::with_source_rows(
                            plot.data(),
                            rows,
                            local.body,
                            theme,
                        )
                    },
                )
                .with_fit_scope(&child_scope)
                .with_guides(&guides);
            if prepared.content_sizing
                && self.legend_policy == CanvasLegendPolicy::PerPlot
                && !matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm)
            {
                context.content_height = true;
                context.fill_content_height = true;
            }
            let role = if self.legend_policy == CanvasLegendPolicy::PerPlot {
                crate::decorations::plan::LegendRole::Panel
            } else {
                crate::decorations::plan::LegendRole::Suppressed
            };
            let guide = if role == crate::decorations::plan::LegendRole::Panel {
                plot.prepared_discrete_guide()?
                    .map(|descriptor| {
                        crate::decorations::legend::discrete::GuideSlot::new(
                            descriptor,
                            &owner,
                            plot.legend_position_spec(),
                        )
                    })
                    .transpose()?
            } else {
                None
            };
            let panel_plan =
                match children.plan(plot, role, context, guide.as_ref(), round, &owner)? {
                    FitOutcome::Measured(result) => result,
                    FitOutcome::Miss(failure) => {
                        FitFailure::retain_least(&mut child_miss, failure);
                        continue;
                    }
                };
            if context.fill_content_height {
                let target = grid_address.child(
                    crate::layout::AddressRole::CanvasCell,
                    u32::try_from(index).map_err(|_| {
                        Error::Render("Canvas content height address overflowed".into())
                    })?,
                )?;
                let preferred = sizing::measured_preference(
                    plot,
                    &panel_plan,
                    cells[index].height() - local.body.height(),
                    round,
                    &owner,
                    theme,
                )?;
                if let Some(measurements) = prepared.measurements {
                    measurements.record(
                        plot,
                        cells[index].width(),
                        preferred,
                        content_insets[index],
                        round,
                        &target,
                        context.guides.dependencies,
                    )?;
                }
                let request = round.request_insets(
                    &owner,
                    &target,
                    FeedbackKind::ContentHeight,
                    [cells[index].width(), 0.0, preferred, 0.0],
                )?;
                if request.is_some() || preferred > cells[index].height() + 1.0 / 1024.0 {
                    round.miss(
                        &target,
                        crate::layout::kernel::FitKind::TrackOrResidual,
                        &[request],
                        "selected Plot guide and prose require their measured content row",
                    )?;
                }
            }
            #[cfg(test)]
            CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| {
                evidence
                    .borrow_mut()
                    .push((local.body, panel_plan.result.layout.projection.panel_box()));
            });
            let requirement = match fit::outcome(PanelRequirements::measure_header(
                panel_plan.result.axes.as_ref().ok_or_else(|| {
                    Error::Render("composed panel is missing its measured axes".into())
                })?,
                theme,
            ))? {
                FitOutcome::Measured(requirement) => requirement,
                FitOutcome::Miss(failure) => {
                    FitFailure::retain_least(&mut child_miss, failure);
                    continue;
                }
            };
            measured.push((index, kind, panel_plan, local));
            requirements.push((row, column, requirement));
        }
        if let Some(failure) = child_miss {
            return Ok(FitOutcome::Miss(failure));
        }
        for (group_index, group) in self.x_range_groups.iter().enumerate() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasRangeGroup,
                u32::try_from(group_index)
                    .map_err(|_| Error::Render("canvas group address overflowed".into()))?,
            )?;
            let mut alignment = [0.0_f64; 4];
            for &index in group {
                if let Some((_, _, requirement)) = requirements
                    .iter()
                    .find(|(row, column, _)| row * self.columns + column == index)
                {
                    alignment[0] = alignment[0].max(requirement.left);
                    alignment[1] = alignment[1].max(requirement.right);
                }
            }
            if let Some(request) = round.request_insets(
                &address,
                &address,
                FeedbackKind::SharedAlignment,
                alignment,
            )? {
                round.miss(
                    &address,
                    FitKind::TrackOrResidual,
                    &[Some(request)],
                    "shared range group requires aligned panel edges",
                )?;
            }
        }
        for (row, column, requirement) in requirements {
            let index = row * self.columns + column;
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas grid address overflowed".into()))?,
            )?;
            round.observe_box(cells[index])?;
            if let Some(request) = round.request_insets(
                &address,
                &address,
                crate::layout::kernel::FeedbackKind::AxisGutter,
                [
                    requirement.left,
                    requirement.right,
                    requirement.bottom,
                    requirement.top,
                ],
            )? {
                round.miss(
                    &address,
                    crate::layout::kernel::FitKind::Tick,
                    &[Some(request)],
                    "canvas cell requires measured decoration gutters",
                )?;
            }
        }

        let mut text_panels = Vec::new();
        crate::render::backend::reserve_text_records(
            &mut text_panels,
            self.text_panels.iter().flatten().count(),
        )?;
        for (index, text_panel) in self.text_panels.iter().enumerate() {
            let Some(text_panel) = text_panel else {
                continue;
            };
            let intrinsic = prepared.text_panel_intrinsics[index]
                .expect("configured TextPanel retains prepared intrinsics");
            match TextPanelPlan::measure(
                text_panel,
                index,
                cells[index],
                themes.root(),
                intrinsic,
                &scope,
            )? {
                FitOutcome::Measured(plan) => text_panels.push(plan),
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            }
        }

        for text_panel in &text_panels {
            round.observe_box(text_panel.cell)?;
            round.observe_text_block(&text_panel.title)?;
            round.observe_text_block(&text_panel.body)?;
        }
        for header in headers.iter().chain(&row_titles) {
            round.observe_authored_text(&header.request)?;
            round.observe_text(&header.text.plan)?;
            round.observe_number(header.text.placement.position.x)?;
            round.observe_number(header.text.placement.position.y)?;
        }
        round.observe_count(usize::from(shared_legend.is_some()))?;
        if let Some(legend) = &shared_legend {
            legend.observe_feedback(round)?;
        }

        let mut panels = Vec::new();
        fit::reserve(&mut panels, measured.len(), "Canvas selected panels")?;
        for (index, kind, panel_plan, local) in measured {
            let cell = cells[index];
            let mut layers = [String::new(), String::new(), String::new(), String::new()];
            for (layer, suffix) in layers.iter_mut().zip(["grid", "axes", "legend", "title"]) {
                crate::data::source::reserve_source_string(layer, 40)?;
                write!(layer, "panel-{index}-{suffix}").expect("admitted Canvas layer name");
            }
            let [grid, axes, legend, header_layer] = layers;
            panels.push(PlannedCanvasPanel {
                index,
                kind,
                canvas_cell: cell,
                legend: panel_plan
                    .legend
                    .as_ref()
                    .map(|legend| {
                        let (x, y) = legend.bounds.anchor();
                        legend.legend.select(x, y)
                    })
                    .transpose()?,
                panel_plan,
                local_text: local,
                header_layer,
                layers: [grid, axes, legend],
            });
        }
        let plan = CanvasPlan {
            dimensions: (
                assigned.dimensions()?.0,
                crate::layout::interior::sizing::integral(
                    canvas_layout.panel_box.y1 + canvas_layout.insets.top,
                )?,
            ),
            themes,
            canvas_layout,
            shared_legend_draw: shared_legend
                .as_ref()
                .map(|legend| {
                    let (x, y) = legend.bounds.anchor();
                    legend.legend.select(x, y)
                })
                .transpose()?,
            plots,
            panels,
            text_panels,
            headers,
            row_titles,
        };
        Self::observe_arranged(&plan, round)?;
        Ok(FitOutcome::Measured(plan))
    }

    fn observe_arranged(
        plan: &CanvasPlan,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        for panel in &plan.text_panels {
            geometry.text(panel.title_placement, panel.title.plan(), None)?;
            geometry.text(panel.body_placement, panel.body.plan(), None)?;
        }
        for header in plan.row_titles.iter().chain(&plan.headers) {
            geometry.text(header.text.placement, &header.text.plan, None)?;
        }
        if let Some(legend) = &plan.shared_legend_draw {
            legend.geometry(&mut geometry)?;
        }
        geometry.observe_feedback(round)?;
        for panel in &plan.panels {
            round.observe_box(panel.canvas_cell)?;
            round.observe_box(panel.local_text.body)?;
            panel.local_text.text.observe(round)?;

            let mut geometry = crate::render::geometry::GeometrySnapshot::new();
            panel.local_text.text.geometry(&mut geometry)?;
            if let Some(legend) = &panel.legend {
                legend.geometry(&mut geometry)?;
            }
            geometry.observe_feedback(round)?;
        }
        Ok(())
    }

    fn render_planned_canvas(
        plan: &CanvasPlan,
        width: u32,
        height: u32,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let theme = plan.themes.root();
        backend.draw_surface(
            PanelBox::new(0.0, f64::from(width), 0.0, f64::from(height)),
            theme.surfaces.canvas_root,
            SurfaceRole::CanvasRoot,
        )?;
        for panel in &plan.panels {
            let theme = plan
                .themes
                .plot(panel.index)
                .expect("configured canvas panel has a resolved theme");
            backend.draw_surface(
                panel.panel_plan.panel_box,
                theme.surfaces.plot_panel,
                SurfaceRole::PlotPanel,
            )?;
        }
        for panel in &plan.panels {
            crate::render::backend::with_layer_role(
                backend,
                &panel.layers[0],
                crate::render::backend::LayerRole::Grid,
                |backend| {
                    panel
                        .panel_plan
                        .result
                        .axes
                        .as_ref()
                        .ok_or_else(|| {
                            Error::Render("Canvas plan is missing immutable axes".into())
                        })?
                        .draw_grid(backend)
                },
            )?;
        }
        for panel in &plan.panels {
            let plot = plan.plots[panel.index].as_ref().ok_or_else(|| {
                Error::Render("composed canvas panel disappeared during render".into())
            })?;
            let theme = plan
                .themes
                .plot(panel.index)
                .expect("configured canvas panel has a resolved theme");
            let source_rows = plot.configured_source_rows()?;
            let context = source_rows
                .map_or_else(
                    || {
                        crate::spec::PlotRenderContext::new(
                            plot.data(),
                            panel.panel_plan.panel_box,
                            theme,
                        )
                    },
                    |rows| {
                        crate::spec::PlotRenderContext::with_source_rows(
                            plot.data(),
                            rows,
                            panel.panel_plan.panel_box,
                            theme,
                        )
                    },
                )
                .with_retained_body(&panel.panel_plan.result);
            plot.render_panel(panel.kind, context, backend)?;
            crate::significance::render_selected(&panel.panel_plan.result, backend)?;
            with_layer(backend, &panel.layers[1], |backend| {
                panel
                    .panel_plan
                    .result
                    .axes
                    .as_ref()
                    .ok_or_else(|| Error::Render("Canvas plan is missing immutable axes".into()))?
                    .draw(backend)
            })?;
            if let Some(legend) = &panel.legend {
                with_layer(backend, &panel.layers[2], |backend| legend.draw(backend))?;
            }
            with_layer(backend, &panel.header_layer, |backend| {
                panel.local_text.text.draw_retained(backend)
            })?;
        }
        for panel in &plan.text_panels {
            draw_text_panel(panel, backend)?;
        }
        if !plan.row_titles.is_empty() {
            draw_canvas_row_titles(&plan.row_titles, backend)?;
        }
        if !plan.headers.is_empty() {
            draw_canvas_headers(&plan.headers, backend)?;
        }
        if let Some(legend) = &plan.shared_legend_draw {
            with_layer(backend, "canvas-shared-legend", |backend| {
                legend.draw(backend)
            })?;
        }
        plan.canvas_layout.root_text.draw_retained(backend)
    }

    #[expect(
        clippy::type_complexity,
        reason = "shared descriptor preparation also retains the exact full-body probes used by frame attempts"
    )]
    fn preflight_shared_legend(
        plots: &[Option<Plot>],
        themes: &ResolvedCanvasThemes,
        panel: PanelBox,
        children: &mut crate::decorations::plan::children::ChildPlans<'_>,
        round: &crate::layout::kernel::FeedbackRound,
        prepared_discrete_guide: Option<&Arc<PreparedGuide>>,
    ) -> Result<
        (
            Option<Arc<PreparedGuide>>,
            Vec<Option<Arc<crate::spec::PlotRenderResult>>>,
        ),
        Error,
    > {
        let mut probes = Vec::new();

        crate::layout::fit::reserve(&mut probes, plots.len(), "shared guide body probes")?;
        probes.resize_with(plots.len(), || None);
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let selection = themes
                .plot_selection(index)
                .expect("configured canvas panel has a resolved theme");
            let theme = match selection {
                ResolvedPlotTheme::InheritedRoot => themes.root(),
                ResolvedPlotTheme::ExplicitPlot(theme) => theme.as_ref(),
            };
            let result = children.canvas_probe(
                plot,
                crate::spec::PlotRenderContext::new(plot.data(), panel, theme),
                index,
                MeasurementPhase::Legend,
                round,
            )?;
            probes[index] = Some(result);
        }
        Ok((prepared_discrete_guide.cloned(), probes))
    }
}

fn plot_category_domain(
    plot: &Plot,
    context: crate::spec::PlotRenderContext<'_>,
    children: &mut crate::decorations::plan::children::ChildPlans<'_>,
    index: usize,
    round: &crate::layout::kernel::FeedbackRound,
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
    let result = children.canvas_probe(plot, context, index, MeasurementPhase::Category, round)?;
    if result.y_domain != AxisDomain::Categorical {
        return Err(Error::Data(
            "shared category labels require a categorical y-axis".into(),
        ));
    }

    match &result.y_tick_keys {
        Some(keys) => crate::data::source::collect_source(
            keys.iter()
                .map(|key| crate::data::source::copy_source_string(key)),
        ),
        None => crate::data::source::collect_source(
            result
                .layout
                .y_axis
                .ticks
                .iter()
                .map(|tick| crate::data::source::copy_source_string(&tick.label)),
        ),
    }
}

fn draw_canvas_headers(
    headers: &[CanvasHeaderPlan],
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if headers.is_empty() {
        return Ok(());
    }
    with_layer(backend, "canvas-shared-headers", |backend| {
        for header in headers {
            header.text.draw(backend)?;
        }
        Ok(())
    })
}

fn draw_canvas_row_titles(
    titles: &[CanvasHeaderPlan],
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if titles.is_empty() {
        return Ok(());
    }
    with_layer(backend, "canvas-row-titles", |backend| {
        for title in titles {
            title.text.draw(backend)?;
        }
        Ok(())
    })
}

fn draw_text_panel(panel: &TextPanelPlan, backend: &mut dyn Backend) -> Result<(), Error> {
    with_layer(backend, &panel.layer, |backend| {
        backend.draw_planned_text(panel.title_placement, panel.title.plan())?;
        backend.draw_planned_text(panel.body_placement, panel.body.plan())
    })
}
