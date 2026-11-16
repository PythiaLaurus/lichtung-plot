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
use crate::layout::canvas::EdgeSpaceRequirements;
use crate::layout::fit::{FitFailure, FitOutcome, FitScope};
use crate::layout::kernel::{FeedbackKind, FitKind};
use crate::layout::{PixelRect, RootFramePlan};
use crate::orchestrator::assignment::{
    GuideTrialAssignment, ParentInsets, PlotAllocationInputs, PlotMeasureAssignmentFactory,
    PlotMeasureDimensions, RootAssignment,
};
use crate::orchestrator::plan::{MeasuredOwnedPlan, MeasuredRootContent};
use crate::orchestrator::registration::{PlotOccurrenceId, PlotRegistration};
use crate::orchestrator::root::{
    CanvasAnchor, CanvasHeaderRole, CanvasHeaderSource, CompositionIntrinsics, GridSlot,
    PreparedCanvasTopology, PreparedComposition, Root, RootSource,
};
use crate::render::backend::{with_layer, Backend, SurfaceRole};
use crate::scene::RenderedScene;
use crate::spec::AxisDomain;
use crate::spec::PlotMeasureContext;
use crate::theme::ResolvedTheme;
use crate::{CanvasLegendPolicy, Labels, LegendComposition, Output, Plot, PlotKind, Theme};

mod dependencies;
mod placement;
mod plot_text;
mod sizing;
pub(crate) use sizing::HeightMeasurements;
mod text_clear;
pub(crate) mod text_panel;
mod tracks;

#[cfg(test)]
pub(crate) mod lifecycle_tests;

#[cfg(test)]
thread_local! {
    pub(crate) static CANVAS_SCENE_RENDER_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static CANVAS_ASSIGNED_PANEL_EVIDENCE: std::cell::RefCell<Vec<(PixelRect, PixelRect)>> = const { std::cell::RefCell::new(Vec::new()) };
}

struct OwnedCanvasPlot {
    anchor: crate::orchestrator::root::GridSlot,
    plot: Arc<crate::decorations::plan::MeasuredPlotPlan>,
    prose: crate::decorations::title_block::plan::RootTitleBlockPlan,
    prose_layer: String,
    legend: Option<(String, crate::decorations::legend::LegendDrawPlan)>,
}

use crate::decorations::legend::discrete::PreparedGuide;

pub(crate) struct OwnedCanvasPlan {
    grid: tracks::CanvasGridAssignment,
    plots: Vec<OwnedCanvasPlot>,
    text_panels: Vec<TextPanelPlan>,
    shared_guide: Option<crate::decorations::legend::LegendDrawPlan>,
}

impl Root<'_> {
    fn canvas_topology(&self) -> &PreparedCanvasTopology {
        let PreparedComposition::Canvas(topology) = &self.composition else {
            unreachable!("Canvas root topology")
        };
        topology
    }

    fn canvas_plot(&self, index: usize) -> Option<&PlotRegistration<'_>> {
        plot_at(&self.plots, &self.canvas_topology().slots, index)
    }

    fn canvas_guide(&self) -> Option<&Arc<PreparedGuide>> {
        self.canvas_topology()
            .shared_guide
            .as_ref()
            .map(|slot| &slot.descriptor)
    }

    fn canvas_text(&self) -> &[Option<text_panel::TextPanelIntrinsics>] {
        let CompositionIntrinsics::Canvas { text, .. } = &self.composition_intrinsics else {
            unreachable!("Canvas text intrinsics")
        };
        text
    }

    fn canvas_prose(
        &self,
        id: &PlotOccurrenceId,
    ) -> crate::decorations::title_block::plan::RootTitleBlockIntrinsics {
        let CompositionIntrinsics::Canvas { prose, .. } = &self.composition_intrinsics else {
            unreachable!("Canvas prose intrinsics")
        };
        prose
            .iter()
            .find(|(found, _)| found == id)
            .expect("configured Canvas Plot text intrinsics")
            .1
    }
}

fn plot_at<'v, 'src>(
    plots: &'v [crate::orchestrator::registration::PlotRegistration<'src>],
    slots: &[Option<crate::orchestrator::root::CanvasAnchor>],
    index: usize,
) -> Option<&'v crate::orchestrator::registration::PlotRegistration<'src>> {
    match slots[index].as_ref()? {
        crate::orchestrator::root::CanvasAnchor::Plot(id) => Some(&plots[id.slot]),
        crate::orchestrator::root::CanvasAnchor::Text => None,
    }
}

fn resolve_appearances(
    root: &crate::theme::Theme,
    placements: &placement::Placements,
    guide_layouts: &[Option<crate::TextLayout>; 3],
) -> Result<(ResolvedTheme, Vec<Option<ResolvedTheme>>), Error> {
    let mut root = root.resolve()?;
    root.text_layouts.guide.clone_from(guide_layouts);
    let plots = placements
        .plots()
        .map(|plot| {
            plot.map(|plot| {
                let mut theme = match plot.theme_spec() {
                    None => root.clone(),
                    Some(theme) => {
                        let mut theme = theme.resolve()?;
                        theme.canvas_surface_override = Some(root.surfaces.canvas_root);
                        theme
                    }
                };
                theme.text_layouts = plot.text_layouts().clone();
                theme.apply_text_defaults(plot.kind());
                Ok(theme)
            })
            .transpose()
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok((root, plots))
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
    cell: PixelRect,
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
        title.required_source("text panel title")?;
        body.required_source("text panel body")?;
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
    text.required_source(&format!("canvas {role} header"))?;
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
/// content unless [`Self::width`], [`Self::max_width`] or [`Self::size`] replaces
/// the sizing request. Clones share immutable placements until a placement edit
/// separates them. Terminal exports borrow the configuration and render a fresh
/// scene per call.
#[derive(Clone)]
pub struct Canvas {
    guide_text_layouts: [Option<crate::TextLayout>; 3],
    rows: usize,
    columns: usize,
    placements: Arc<placement::Placements>,
    theme: Arc<crate::theme::Theme>,
    legend_policy: CanvasLegendPolicy,
    legend_composition: LegendComposition,
    root_size: crate::layout::RootRequest,
    title: crate::Text,
    subtitle: crate::Text,
    // Parent defaults apply only to embedded Plot prose, never Canvas root headings.
    pub(crate) panel_title_alignment: Option<crate::TextAlignment>,
    pub(crate) panel_subtitle_alignment: Option<crate::TextAlignment>,
    caption: crate::Text,
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
    /// Replace one Canvas-owned Shared guide role's complete layout default.
    #[must_use]
    pub fn guide_text_layout(
        mut self,
        role: crate::GuideTextRole,
        layout: crate::TextLayout,
    ) -> Self {
        self.guide_text_layouts[role.index()] = Some(layout);
        self
    }
    /// Construct a non-empty panel grid.
    pub fn grid(rows: usize, columns: usize) -> Result<Self, Error> {
        if rows == 0 || columns == 0 {
            return Err(Error::Data(
                "canvas grid rows and columns must both be greater than zero".into(),
            ));
        }
        Ok(Self {
            rows,
            guide_text_layouts: [None, None, None],
            columns,
            placements: Arc::new(placement::Placements::new(rows, columns)?),
            theme: Arc::new(crate::theme::Theme::publication()),
            legend_policy: CanvasLegendPolicy::PerPlot,
            legend_composition: LegendComposition::Combined,
            root_size: crate::layout::RootRequest::Natural,
            title: crate::Text::UNSET,
            subtitle: crate::Text::UNSET,
            panel_title_alignment: None,
            panel_subtitle_alignment: None,
            caption: crate::Text::UNSET,
            labels: None,
            category_labels: CanvasCategoryLabels::EveryPlot,
            column_headers: Vec::new(),
            row_headers: Vec::new(),
            row_titles: Vec::new(),
            x_range_groups: Vec::new(),
        })
    }

    /// Whether this cell is the top-left anchor of a placed Plot.
    ///
    /// Covered cells, TextPanels, empty cells and out-of-range coordinates return false.
    #[must_use]
    pub fn contains_plot_anchor(&self, row: usize, column: usize) -> bool {
        if row >= self.rows || column >= self.columns {
            return false;
        }
        self.placements.plot(row * self.columns + column).is_some()
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
        self,
        row: usize,
        column: usize,
        plot: Plot<K>,
    ) -> Result<Self, Error> {
        self.panel_span(row, column, 1, 1, plot)
    }

    /// Place one Plot across a checked rectangle, including internal track gaps.
    /// Only the top-left anchor owns preparation, prose, guides and capture.
    /// ```
    /// use lichtung_plot::{Canvas, NumericValues, Plot};
    /// let plot = || Plot::scatter(NumericValues::new("x",[0.0,1.0]),NumericValues::new("y",[1.0,0.0]));
    /// let canvas = Canvas::grid(2,2)?
    ///     .panel(0,0,plot())?.panel(1,0,plot())?
    ///     .panel_span(0,1,2,1,plot())?;
    /// # let _ = canvas;
    /// # Ok::<(), lichtung_plot::Error>(())
    /// ```
    pub fn panel_span<K: crate::kind::ExactFamily>(
        mut self,
        row: usize,
        column: usize,
        row_span: usize,
        column_span: usize,
        plot: Plot<K>,
    ) -> Result<Self, Error> {
        self.place_panel(row, column, row_span, column_span, plot, false)?;
        Ok(self)
    }

    /// Place a chart in one cell, removing every intersecting placement in full.
    /// Cells outside this new footprint are freed if their old owner intersects it.
    /// Use [`Self::panel`] instead to reject all overlap.
    ///
    /// ```
    /// use lichtung_plot::{Canvas, NumericValues, Plot};
    /// let plot = || Plot::scatter(NumericValues::new("x", [0., 1.]), NumericValues::new("y", [1., 2.]));
    /// let canvas = Canvas::grid(2, 3)?.panel_span(0, 0, 2, 2, plot())?
    ///     .set_panel(1, 1, plot())?.panel(0, 0, plot())?;
    /// # let _ = canvas;
    /// # Ok::<(), lichtung_plot::Error>(())
    /// ```
    pub fn set_panel<K: crate::kind::ExactFamily>(
        self,
        row: usize,
        column: usize,
        plot: Plot<K>,
    ) -> Result<Self, Error> {
        self.set_panel_span(row, column, 1, 1, plot)
    }

    /// Replace an occupied region with one chart, removing whole intersecting
    /// Plot or TextPanel owners, even when their anchors lie outside the region.
    /// Edge-touching owners remain intact. Empty space permits ordinary insertion.
    ///
    /// Footprint validation and fallible placement resources precede any removal.
    /// Grid dimensions, headers and explicit range-group requests are unchanged;
    /// their normal preflight checks apply to the resulting topology.
    pub fn set_panel_span<K: crate::kind::ExactFamily>(
        mut self,
        row: usize,
        column: usize,
        row_span: usize,
        column_span: usize,
        plot: Plot<K>,
    ) -> Result<Self, Error> {
        self.place_panel(row, column, row_span, column_span, plot, true)?;
        Ok(self)
    }

    /// Place an owned exact-family Plot without consuming this Canvas on error.
    ///
    /// `overwrite=false` requires an entirely empty footprint; `true` removes
    /// complete intersecting owners as documented by [`Self::set_panel_span`].
    /// Validation/resources precede mutation. Existing Canvas clones are unchanged,
    /// and an unshared Canvas edits its placements without copying the owner set.
    /// Consuming placement builders delegate to this same operation.
    pub fn place_panel<K: crate::kind::ExactFamily>(
        &mut self,
        row: usize,
        column: usize,
        row_span: usize,
        column_span: usize,
        plot: Plot<K>,
        overwrite: bool,
    ) -> Result<(), Error> {
        let footprint = placement::Footprint {
            row,
            column,
            row_span,
            column_span,
        };
        let payload = placement::Payload::Plot(plot.into_description());
        let placements = Arc::make_mut(&mut self.placements);
        if overwrite {
            placements.replace(footprint, payload)
        } else {
            placements.insert(footprint, payload)
        }
    }

    pub(crate) fn owner_topology(&self, index: usize) -> Option<[usize; 6]> {
        let owner = self.placements.footprint(index)?;
        Some([
            self.rows,
            self.columns,
            owner.row,
            owner.column,
            owner.row_span,
            owner.column_span,
        ])
    }

    pub(crate) fn owner_topology_at(&self, (row, column): (usize, usize)) -> Option<[usize; 6]> {
        self.owner_topology(row * self.columns + column)
    }

    pub(crate) fn range_group_contains(&self, index: usize) -> bool {
        self.x_range_groups
            .iter()
            .any(|group| group.contains(&index))
    }

    fn category_bands(&self) -> Result<Vec<Vec<usize>>, Error> {
        let mut bands = Vec::new();
        let mut row = 0;
        while row < self.rows {
            let first = self
                .placements
                .anchor(row, 0)
                .filter(|&index| self.placements.plot(index).is_some())
                .ok_or_else(|| {
                    Error::Data("shared category band requires a first-column Plot owner".into())
                })?;
            let band = self.placements.footprint(first).expect("occupied anchor");
            if band.row != row {
                return Err(Error::Data(
                    "shared category row intervals do not agree".into(),
                ));
            }
            let mut members = Vec::new();
            let mut column = 0;
            while column < self.columns {
                let index = self
                    .placements
                    .anchor(row, column)
                    .filter(|&index| self.placements.plot(index).is_some())
                    .ok_or_else(|| {
                        Error::Data(
                            "shared category band requires Plot peers across every column".into(),
                        )
                    })?;
                let footprint = self.placements.footprint(index).expect("occupied anchor");
                if footprint.row != band.row || footprint.row_span != band.row_span {
                    return Err(Error::Data(
                        "shared category owners must cover the same row interval".into(),
                    ));
                }
                crate::data::source::reserve_source(&mut members, 1)?;
                members.push(index);
                column += footprint.column_span;
            }
            crate::data::source::reserve_source(&mut bands, 1)?;
            bands.push(members);
            row += band.row_span;
        }
        Ok(bands)
    }

    /// Place one validated text panel in an unoccupied zero-based grid cell.
    pub fn text_panel(
        mut self,
        row: usize,
        column: usize,
        panel: TextPanel,
    ) -> Result<Self, Error> {
        self.place_text_panel(row, column, panel)?;
        Ok(self)
    }

    /// Insert a strict single-cell TextPanel without consuming the Canvas on error.
    /// Other configuration copies and occupied placements remain unchanged on failure.
    pub fn place_text_panel(
        &mut self,
        row: usize,
        column: usize,
        panel: TextPanel,
    ) -> Result<(), Error> {
        Arc::make_mut(&mut self.placements).insert(
            placement::Footprint::cell(row, column),
            placement::Payload::Text(panel),
        )
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
        self.root_size = crate::layout::RootRequest::ExactWidth(width);
        self
    }

    /// Cap the natural complete Canvas width; height remains content-driven.
    ///
    /// This replaces any prior sizing request, retains declared topology and
    /// never fills a cap wider than natural content. Embedded Plot sizing requests
    /// remain ignored. Zero rejects during owning-root preflight.
    #[must_use]
    pub const fn max_width(mut self, maximum: u32) -> Self {
        self.root_size = crate::layout::RootRequest::MaxWidth(maximum);
        self
    }

    /// Allocate exactly this width and height in CSS pixels.
    ///
    /// This replaces the complete sizing request. All content must fit without
    /// changing either dimension; zero dimensions reject during preflight.
    /// Embedded Plots do not contribute standalone sizing requests.
    #[must_use]
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.root_size = crate::layout::RootRequest::ExactSize { width, height };
        self
    }

    /// Set one canvas-level title.
    ///
    /// A plain string replaces content while retaining earlier local style and
    /// layout. Present Text instructions replace the complete style or layout;
    /// an explicit default layout restores owner defaults.
    #[must_use]
    pub fn title(mut self, title: impl Into<crate::Text>) -> Self {
        self.title
            .replace_from(title.into())
            .expect("public Text always has a source");
        self
    }

    /// Set one canvas-level subtitle.
    /// Content and instruction replacement follow [`Self::title`].
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<crate::Text>) -> Self {
        self.subtitle
            .replace_from(subtitle.into())
            .expect("public Text always has a source");
        self
    }

    /// Set one canvas-level caption.
    /// Content and instruction replacement follow [`Self::title`].
    #[must_use]
    pub fn caption(mut self, caption: impl Into<crate::Text>) -> Self {
        self.caption
            .replace_from(caption.into())
            .expect("public Text always has a source");
        self
    }

    fn prose(&self) -> [Option<&crate::Text>; 3] {
        [&self.title, &self.subtitle, &self.caption].map(crate::Text::provided)
    }

    fn prose_alignments(&self) -> [Option<crate::TextAlignment>; 3] {
        [&self.title, &self.subtitle, &self.caption].map(crate::Text::alignment_request)
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
        root: &crate::layout::FrozenRoot<crate::orchestrator::TextReplayPlan>,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
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
    pub fn svg(&self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.svg(path)
    }

    /// Render once, publish SVG with selectable text, and return a reusable output.
    /// The output filename is completed with `.svg` after rendering.
    pub fn svg_preserve_text(&self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.svg_preserve_text(path)
    }

    /// Render once, publish a balanced 300-DPI PNG, and return a reusable output.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png(&self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png(path)
    }

    /// Render once, publish a balanced source-sized PNG without density metadata.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png_preview(&self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png_preview(path)
    }

    /// Render once and publish a PNG using the exact caller-provided options.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png_with_options(
        &self,
        path: impl AsRef<Path>,
        options: crate::PngOptions,
    ) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png_with_options(path, options)
    }

    fn validate_structure(&self) -> Result<Vec<Vec<usize>>, Error> {
        let bands = if self.category_labels == CanvasCategoryLabels::FirstColumnPerRow {
            self.category_bands()?
        } else {
            Vec::new()
        };
        for source in [
            self.title.provided().map(crate::Text::as_str),
            self.subtitle.provided().map(crate::Text::as_str),
            self.caption.provided().map(crate::Text::as_str),
        ]
        .into_iter()
        .flatten()
        {
            crate::text::validate_for(source, "canvas prose")?;
        }
        if self.placements.plots().all(|plot| plot.is_none())
            && self.placements.texts().all(|text| text.is_none())
        {
            return Err(Error::Data(
                "canvas requires at least one configured panel".into(),
            ));
        }
        for plot in self.placements.plots().flatten() {
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
                let plot = self.placements.plot(index).ok_or_else(|| {
                    Error::Data(format!(
                        "canvas x-range group references unconfigured panel {index}"
                    ))
                })?;
                if plot.x_axis_spec().window.is_some() {
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
            for (index, plot) in self.placements.plots().enumerate() {
                let Some(plot) = plot else {
                    continue;
                };
                let footprint = self.placements.footprint(index).expect("Plot owner");
                let end = footprint.row + footprint.row_span;
                if footprint.row < title.start && title.start < end {
                    return Err(Error::Data(
                        "Canvas row title would cut through a Plot footprint".into(),
                    ));
                }
                if footprint.row == title.start
                    && (plot.title_text().is_some() || plot.subtitle_text().is_some())
                {
                    return Err(Error::Data(format!(
                        "canvas row title cannot coexist with a panel title or subtitle in row {}",
                        title.start
                    )));
                }
            }
        }
        Ok(bands)
    }

    fn prepare_continuous_shared_guide(
        &self,
        registry: &crate::orchestrator::registration::PlotRegistryBuilder<'_>,
        slots: &[Option<crate::orchestrator::root::CanvasAnchor>],
        operation: &crate::font::canonical::OperationContext,
    ) -> Result<Option<crate::decorations::legend::descriptor::ContinuousDescriptor>, Error> {
        if self.legend_policy != CanvasLegendPolicy::Shared {
            return Ok(None);
        }
        let Some(_) = self.placements.plots().flatten().next() else {
            return Ok(None);
        };

        let mut contributors = Vec::new();
        crate::data::source::reserve_source(&mut contributors, self.placements.len())?;
        for (index, plot) in self.placements.plots().enumerate() {
            let Some(plot) = plot else { continue };
            if !matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm) {
                continue;
            }
            let Some(crate::orchestrator::root::CanvasAnchor::Plot(id)) = &slots[index] else {
                return Err(Error::Render(
                    "shared continuous guide has no prepared interior".into(),
                ));
            };
            let registered = &registry.panels[id.slot];
            let contributor =
                crate::decorations::legend::descriptor::ContinuousContributor::for_plot(
                    registered,
                    registered.scope.axis_numeric_binding(
                        registered.source,
                        &registered.row_source,
                        operation,
                    )?,
                )?
                .ok_or_else(|| {
                    Error::Render("shared continuous guide has no color identity".into())
                })?;
            contributors.push(contributor);
        }
        crate::decorations::legend::descriptor::merge_continuous(
            &operation.root_identity(),
            operation,
            contributors,
        )
    }

    fn install_shared_continuous_domain(
        &self,
        registry: &mut crate::orchestrator::registration::PlotRegistryBuilder<'_>,
        slots: &[Option<crate::orchestrator::root::CanvasAnchor>],
        descriptor: &crate::decorations::legend::descriptor::ContinuousDescriptor,
    ) -> Result<(), Error> {
        for (index, plot) in self.placements.plots().enumerate() {
            let Some(plot) = plot else { continue };
            let Some(crate::orchestrator::root::CanvasAnchor::Plot(id)) = &slots[index] else {
                continue;
            };
            let registered = &mut registry.panels[id.slot];
            match plot.kind() {
                PlotKind::Heatmap => {
                    if let Some(crate::orchestrator::registration::PreparedFamily::Heatmap(marks)) =
                        &mut registered.family
                    {
                        marks.install_shared_domain(descriptor)?;
                    }
                }
                PlotKind::Beeswarm => {
                    if let Some(crate::orchestrator::registration::PreparedFamily::Beeswarm(
                        marks,
                    )) = &mut registered.family
                    {
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
        registry: &crate::orchestrator::registration::PlotRegistryBuilder<'_>,
        slots: &[Option<crate::orchestrator::root::CanvasAnchor>],
    ) -> Result<Vec<(String, Arc<crate::data::category::CategoryDomain>)>, Error> {
        if self.legend_policy != CanvasLegendPolicy::Shared {
            return Ok(Vec::new());
        }
        let mut groups = Vec::new();
        crate::data::source::reserve_source(&mut groups, self.placements.len())?;
        for (index, plot) in self.placements.plots().enumerate() {
            let Some(plot) = plot else { continue };
            let Some(dimension) = plot.categorical_color_field() else {
                continue;
            };

            let Some(crate::orchestrator::root::CanvasAnchor::Plot(id)) = &slots[index] else {
                return Err(Error::Render(
                    "Canvas color domain is missing prepared semantics".into(),
                ));
            };
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
            for level in crate::layout::interior::guide::observed_color_levels(
                plot,
                std::slice::from_ref(&registry.panels[id.slot]),
            )? {
                if crate::data::source::source_position(levels, |existing| existing == &level)
                    .is_none()
                {
                    crate::data::source::reserve_source(levels, 1)?;
                    levels.push(level);
                }
            }
        }
        crate::data::source::collect_source(groups.into_iter().map(|(field, values)| {
            let domain = crate::data::category::CategoryDomain::from_values(
                Some(&crate::data::category::RawField::new(&field)?),
                values,
            )?;
            Ok((field, crate::data::source::source_arc(domain)?))
        }))
    }

    /// Prepare every local contribution or explicit absence, then retain the
    /// one Shared union when selected. No body or font work determines presence.
    fn prepare_discrete_guides(
        &self,
        registry: &mut crate::orchestrator::registration::PlotRegistryBuilder<'_>,
        slots: &[Option<crate::orchestrator::root::CanvasAnchor>],
        hue_groups: &[(String, Arc<crate::data::category::CategoryDomain>)],
        resources: &crate::font::canonical::OperationContext,
    ) -> Result<Option<Arc<PreparedGuide>>, Error> {
        let mut contributions = Vec::new();
        crate::data::source::reserve_source(&mut contributions, self.placements.len())?;
        for (index, plot) in self.placements.plots().enumerate() {
            let Some(source) = plot else {
                continue;
            };
            let Some(crate::orchestrator::root::CanvasAnchor::Plot(id)) = &slots[index] else {
                return Err(Error::Render(
                    "Canvas point/path guide is missing prepared semantics".into(),
                ));
            };
            let registered = &mut registry.panels[id.slot];
            if let Some((_, domain)) = hue_groups
                .iter()
                .find(|(field, _)| Some(field.as_str()) == source.categorical_color_field())
            {
                registered.set_parent_colors(Arc::clone(domain));
            }
            let root_guide = crate::layout::interior::guide::prepare_discrete_guides(
                source,
                std::slice::from_mut(registered),
                resources,
                false,
            )?;
            if self.legend_policy == CanvasLegendPolicy::Shared {
                if let Some(descriptor) = &root_guide {
                    contributions.push(Arc::clone(descriptor));
                }
            }
        }
        PreparedGuide::merge(&contributions, &[], "shared canvas legend")
    }

    fn prepare_shared_x(
        &self,
        stage: &mut crate::layout::kernel::PreparedStage<'_>,
        registry: &mut crate::orchestrator::registration::PlotRegistryBuilder<'_>,
        composition: &PreparedComposition,
    ) -> Result<Vec<Option<(f64, f64)>>, Error> {
        let mut ranges = Vec::new();
        if self.x_range_groups.is_empty() {
            return Ok(ranges);
        }

        stage.reserve_prepared(&mut ranges, self.placements.len(), "shared x domains")?;
        ranges.resize(self.placements.len(), None);
        crate::plots::shared_x::complete_shared_bases(&mut registry.panels, composition)?;
        let PreparedComposition::Canvas(topology) = composition else {
            unreachable!("Canvas shared bases")
        };
        for group in &topology.range_groups {
            for id in group {
                let panel = &registry.panels[id.slot];
                let index = usize::try_from(panel.scope.address.leaf_slot_index())
                    .expect("Canvas slot fits usize");
                // Some families complete y later. Only x exists at this point.
                ranges[index] = Some(
                    panel.axes.as_ref().expect("shared x axes").sources()[0]
                        .base_seed
                        .expect("shared x seed")
                        .pair(),
                );
            }
        }
        Ok(ranges)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "root component inputs and their coordinated x domains are prepared together before fonts"
    )]
    fn prepare_interiors(
        &self,
        stage: &mut crate::layout::kernel::PreparedStage<'_>,
        appearances: Vec<Option<ResolvedTheme>>,
    ) -> Result<
        (
            crate::orchestrator::registration::PlotRegistryBuilder<'_>,
            PreparedCanvasTopology,
        ),
        Error,
    > {
        let resources = stage.operation();
        let mut registry =
            crate::orchestrator::registration::PlotRegistryBuilder::new(resources.root_identity());
        stage.reserve_prepared(
            &mut registry.panels,
            self.placements.len(),
            "Canvas interior inputs",
        )?;
        let mut slots = Vec::new();
        stage.reserve_prepared(&mut slots, self.placements.len(), "Canvas plot occurrences")?;
        for (index, (plot, appearance)) in self.placements.plots().zip(appearances).enumerate() {
            let Some(source) = plot else {
                slots.push(
                    self.placements
                        .text(index)
                        .map(|_| crate::orchestrator::root::CanvasAnchor::Text),
                );
                continue;
            };
            let plot = plot.expect("configured Canvas source");
            let mut scope = crate::orchestrator::registration::Scope::new(
                plot,
                &resources,
                index,
                crate::layout::ComponentAddress::root().child(
                    crate::layout::AddressRole::CanvasCell,
                    u32::try_from(index)
                        .map_err(|_| Error::Render("canvas address overflowed".into()))?,
                )?,
                crate::orchestrator::registration::RowSelection::Full,
                None,
                None,
            )?;
            scope.numeric_owner.canvas = Some((index / self.columns, index % self.columns));
            scope.axis_numeric_owner.canvas = scope.numeric_owner.canvas;
            if plot.kind() == PlotKind::Matrix {
                slots.push(None);
                continue;
            }
            let policy =
                crate::orchestrator::registration::PlotScopePolicy::for_source(source, None);
            let id = registry.reserve(
                source,
                scope.address,
                scope.numeric_owner,
                scope.axis_numeric_owner,
                scope.rows,
                scope.render_name,
                policy,
                appearance.expect("configured Canvas panel has a resolved theme"),
                &resources,
            )?;
            let panel = &mut registry.panels[id.slot];
            if let Some(labels) = &self.labels {
                panel.policy.labels.clone_from(labels);
            }
            let guide_use = match self.legend_policy {
                CanvasLegendPolicy::Hidden => {
                    crate::decorations::legend::discrete::scoped::GuideUse::Hidden
                }
                CanvasLegendPolicy::Shared
                    if plot.legend_policy_spec() != crate::LegendPolicy::Hidden =>
                {
                    crate::decorations::legend::discrete::scoped::GuideUse::SharedSuppressed
                }
                CanvasLegendPolicy::PerPlot | CanvasLegendPolicy::Shared => {
                    panel.policy.discrete_guide_use
                }
            };
            panel.policy.discrete_guide_use = guide_use;
            panel.policy.continuous_guide_use = guide_use;
            panel.policy.gutter_owner = crate::orchestrator::registration::GutterOwner::Composition;
            panel.policy.axis_text.tick_labels[1] = self.category_labels
                != CanvasCategoryLabels::FirstColumnPerRow
                || index.is_multiple_of(self.columns);
            if self.legend_policy == CanvasLegendPolicy::Shared {
                panel.policy.guide_composition = self.legend_composition;
            }
            panel.policy.continuous_materialization = match self.legend_policy {
                CanvasLegendPolicy::Hidden => {
                    crate::orchestrator::registration::ContinuousMaterialization::Suppressed
                }
                CanvasLegendPolicy::Shared
                    if matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm) =>
                {
                    crate::orchestrator::registration::ContinuousMaterialization::SharedFinal
                }
                CanvasLegendPolicy::PerPlot | CanvasLegendPolicy::Shared => {
                    crate::orchestrator::registration::ContinuousMaterialization::Local
                }
            };
            let family = match plot.kind() {
                PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                    plot.configured_source_rows()?;
                    let (marks, ranges, conditions, hues) = crate::plots::line::PreparedLine::new(
                        plot,
                        panel.data(&resources)?,
                        None,
                        panel
                            .parent_orders
                            .collectives
                            .as_deref()
                            .map(Vec::as_slice),
                    )?;
                    panel
                        .categorical
                        .as_mut()
                        .expect("prepared color binding")
                        .set_local_keys(hues)?;
                    panel.axes = Some(crate::plots::shared_x::prepare_pair_axes(
                        panel,
                        ranges.map(Some),
                        [conditions, None],
                    )?);
                    crate::orchestrator::registration::PreparedFamily::Line {
                        kind: plot.kind(),
                        marks,
                    }
                }
                PlotKind::Scatter => {
                    plot.configured_source_rows()?;
                    let (marks, ranges) =
                        crate::plots::scatter::PreparedScatter::new(plot, panel.data(&resources)?)?;
                    panel.axes = Some(crate::plots::shared_x::prepare_pair_axes(
                        panel,
                        ranges.map(Some),
                        [None, None],
                    )?);
                    crate::orchestrator::registration::PreparedFamily::Scatter(marks)
                }
                _ => {
                    plot.configured_source_rows()?;
                    crate::plots::shared_x::prepare_plot_registration(panel, &resources)?;
                    panel.family.take().expect("prepared reference family")
                }
            };
            panel.family = Some(family);
            slots.push(Some(crate::orchestrator::root::CanvasAnchor::Plot(id)));
        }
        let producers = crate::data::source::collect_source(
            slots
                .iter()
                .enumerate()
                .filter(|(_, anchor)| anchor.is_some())
                .map(|(index, _)| Ok(GridSlot(index))),
        )?;
        let range_groups =
            crate::data::source::collect_source(self.x_range_groups.iter().map(|group| {
                crate::data::source::collect_source(group.iter().map(|&index| {
                    let Some(CanvasAnchor::Plot(id)) = &slots[index] else {
                        unreachable!("validated Canvas range group")
                    };
                    Ok(id.clone())
                }))
            }))?;
        let composition = PreparedComposition::Canvas(PreparedCanvasTopology {
            slots,
            producers,
            range_groups,
            headers: Vec::new(),
            category_bands: Vec::new(),
            shared_guide: None,
        });
        let shared_x = self.prepare_shared_x(stage, &mut registry, &composition)?;
        let PreparedComposition::Canvas(topology) = composition else {
            unreachable!("Canvas composition")
        };
        let slots = &topology.slots;
        for (index, plot) in self.placements.plots().enumerate() {
            let Some(crate::orchestrator::root::CanvasAnchor::Plot(id)) = &slots[index] else {
                continue;
            };
            let plot = plot.expect("Canvas producer source");
            let panel = &mut registry.panels[id.slot];
            if let Some(
                crate::orchestrator::registration::PreparedFamily::Scatter(_)
                | crate::orchestrator::registration::PreparedFamily::Line { .. },
            ) = &panel.family
            {
                let mut scope = String::new();
                crate::data::source::reserve_source_string(&mut scope, 32)?;
                write!(scope, "panel-{index}").expect("admitted scope");
                panel
                    .scope
                    .set_render_name(crate::data::source::copy_source_string(&scope)?)?;
                let shared_x_window = shared_x.get(index).copied().flatten();
                if plot.kind() == PlotKind::Scatter {
                    crate::plots::shared_x::prepare_scatter_plot(plot, panel, shared_x_window)?;
                } else {
                    crate::plots::shared_x::prepare_line_plot(plot, panel, shared_x_window)?;
                }
            } else {
                let scope_bytes = 6 + index
                    .checked_ilog10()
                    .map_or(1, |digits| digits as usize + 1);
                let mut scope = String::new();
                crate::data::source::reserve_source_string(&mut scope, scope_bytes)?;
                std::fmt::Write::write_fmt(&mut scope, format_args!("panel-{index}"))
                    .map_err(|error| Error::Render(format!("Canvas scope: {error}")))?;
                panel.scope.set_render_name(scope)?;
                crate::plots::shared_x::prepare_reference_plot(
                    plot,
                    panel,
                    shared_x.get(index).copied().flatten(),
                    self.labels.is_some(),
                    &resources,
                )?;
            }
        }
        for group in &self.x_range_groups {
            if !group.iter().any(|&index| {
                self.placements
                    .plot(index)
                    .is_some_and(|plot| plot.x_axis_spec().window.is_none())
            }) {
                continue;
            }
            for &index in group {
                let Some(crate::orchestrator::root::CanvasAnchor::Plot(id)) = &slots[index] else {
                    continue;
                };
                let registered = &mut registry.panels[id.slot];
                if let Some(crate::orchestrator::registration::PreparedFamily::Waterfall(marks)) =
                    &mut registered.family
                {
                    let numeric = registered.scope.numeric_binding(
                        registered.source,
                        &registered.row_source,
                        &resources,
                    )?;
                    let axes = registered.axes.as_ref().ok_or_else(|| {
                        Error::Render("Waterfall total window scope missing".into())
                    })?;
                    let source = registered.source;
                    marks.prepare_totals(
                        crate::plots::waterfall::WaterfallTotalsPrepare {
                            numeric,
                            axes: [axes.axis(source, 0), axes.axis(source, 1)],
                            format: source.total_format_spec(),
                        },
                        true,
                    )?;
                }
            }
        }
        Ok((registry, topology))
    }

    fn preflight_plan(
        &self,
    ) -> Result<crate::layout::FrozenRoot<crate::orchestrator::TextReplayPlan>, Error> {
        crate::orchestrator::plan_scene(RootSource::Canvas(self))
    }

    pub(crate) const fn sizing_request(&self) -> crate::layout::RootRequest {
        self.root_size
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one ordered semantic, font, intrinsic, allocation, and capture lifecycle"
    )]
    pub(crate) fn prepare_root<'src>(
        &'src self,
        stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<Root<'src>, Error> {
        let (root_appearance, appearances) =
            resolve_appearances(&self.theme, &self.placements, &self.guide_text_layouts)?;
        let text = crate::text::TextService::default();
        let request = crate::text::TextRequestContext::new(&root_appearance, &text);
        let bands = self.validate_structure()?;
        let (mut registry, mut topology) = self.prepare_interiors(stage, appearances)?;
        let slots = &topology.slots;
        let color_domains = self.prepare_color_domains(&registry, slots)?;
        for (index, plot) in self.placements.plots().enumerate() {
            if let (Some(plot), Some(CanvasAnchor::Plot(id))) = (plot, slots[index].as_ref()) {
                let panel = &registry.panels[id.slot];
                let global_hues = color_domains
                    .iter()
                    .find(|(field, _)| Some(field.as_str()) == plot.categorical_color_field())
                    .map(|(_, domain)| domain.as_ref());
                crate::layout::interior::guide::validate_categorical_scale(
                    plot,
                    std::slice::from_ref(panel),
                    &panel.appearance,
                    global_hues,
                )?;
            }
        }
        let operation = stage.operation();
        let continuous = self.prepare_continuous_shared_guide(&registry, slots, &operation)?;
        if let Some(descriptor) = &continuous {
            self.install_shared_continuous_domain(&mut registry, slots, descriptor)?;
        }
        let shared_guide =
            self.prepare_discrete_guides(&mut registry, slots, &color_domains, &operation)?;
        let plots = registry.finish()?;
        let plot_intrinsics = crate::data::source::collect_source(plots.iter().map(|plot| {
            Ok((
                plot.scope.id.clone(),
                crate::orchestrator::root::PlotIntrinsics::for_kind(plot.kind()),
            ))
        }))?;
        let (mut census, presence) =
            self.foundational_text_census(&request, &plots, slots, shared_guide.as_ref(), stage)?;
        text.prepare_font_environment_from_census(&mut census, presence, stage)?;
        let (root_text, text_panels, prose) =
            self.prepare_intrinsics(&request, &plots, slots, &plot_intrinsics, &operation)?;
        let headers = crate::data::source::collect_source(
            [
                (CanvasHeaderRole::Column, self.column_headers.len()),
                (CanvasHeaderRole::Row, self.row_headers.len()),
                (CanvasHeaderRole::RowTitle, self.row_titles.len()),
            ]
            .into_iter()
            .flat_map(|(role, count)| {
                (0..count).map(move |declaration| {
                    Ok(CanvasHeaderSource {
                        role: match role {
                            CanvasHeaderRole::Column => CanvasHeaderRole::Column,
                            CanvasHeaderRole::Row => CanvasHeaderRole::Row,
                            CanvasHeaderRole::RowTitle => CanvasHeaderRole::RowTitle,
                        },
                        declaration,
                    })
                })
            }),
        )?;
        let ids = |groups: &[Vec<usize>]| {
            crate::data::source::collect_source(groups.iter().map(|group| {
                crate::data::source::collect_source(group.iter().map(|&index| {
                    let Some(CanvasAnchor::Plot(id)) = &slots[index] else {
                        unreachable!("validated Canvas group member")
                    };
                    Ok(id.clone())
                }))
            }))
        };
        let category_bands = ids(&bands)?;
        let shared_guide = shared_guide
            .map(|descriptor| {
                Ok::<_, Error>(crate::decorations::legend::discrete::GuideSlot {
                    descriptor,
                    owner: crate::layout::ComponentAddress::root()
                        .child(crate::layout::AddressRole::CanvasSharedLegend, 0)?,
                })
            })
            .transpose()?;
        topology.headers = headers;
        topology.category_bands = category_bands;
        topology.shared_guide = shared_guide;
        Ok(Root {
            source: RootSource::Canvas(self),
            identity: operation.root_identity(),
            operation,
            next_candidate: std::cell::Cell::new(0),
            root_appearance,
            plots,
            composition: PreparedComposition::Canvas(topology),
            census,
            root_text,
            text,
            plot_intrinsics,
            composition_intrinsics: CompositionIntrinsics::Canvas {
                text: text_panels,
                prose,
            },
            caches: std::cell::RefCell::new(crate::orchestrator::root::RootLayoutCaches::new(
                crate::decorations::plan::children::ChildPlans::canvas(),
            )),
        })
    }

    pub(crate) fn measure_root(
        &self,
        root: &Root<'_>,
        requested: crate::layout::RootRequest,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans,
        heights: &mut sizing::HeightMeasurements,
    ) -> Result<MeasuredOwnedPlan, Error> {
        let assigned = sizing::measure(self, root, requested, round, heights)?;
        if let Some(guide) = root.canvas_guide() {
            round.observe_guide(guide)?;
        }
        root.root_text
            .check_width(
                f64::from(assigned.dimensions()?.0) - 32.0,
                &FitScope::root(),
            )
            .map_err(crate::layout::fit::PlanError::into_error)?;
        #[cfg(test)]
        let backend_begins =
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get);
        let measurement = root.assignment_factory().assign(assigned);
        let plan = self.build_canvas_plan(
            root,
            measurement,
            round,
            children,
            self.root_size.automatic_height(),
            Some(heights),
        )?;
        #[cfg(test)]
        assert_eq!(
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
            backend_begins,
            "Canvas measurement must not open a backend",
        );
        Ok(plan)
    }

    #[expect(
        clippy::type_complexity,
        reason = "one prepared root and both cell-intrinsic inventories are retained before parent allocation"
    )]
    fn prepare_intrinsics(
        &self,
        request: &crate::text::TextRequestContext<'_, '_>,
        plots: &[crate::orchestrator::registration::PlotRegistration<'_>],
        slots: &[Option<crate::orchestrator::root::CanvasAnchor>],
        intrinsics: &[(
            crate::orchestrator::registration::PlotOccurrenceId,
            crate::orchestrator::root::PlotIntrinsics,
        )],
        operation: &crate::font::canonical::OperationContext,
    ) -> Result<
        (
            crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
            Vec<Option<text_panel::TextPanelIntrinsics>>,
            Vec<(
                PlotOccurrenceId,
                crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
            )>,
        ),
        Error,
    > {
        // All summaries use the prepared labels and the actual owner theme,
        // after the font census and before any parent allocation or child probe.
        for index in 0..slots.len() {
            if let Some(registered) = plot_at(plots, slots, index) {
                crate::layout::interior::InteriorRoute::classify(registered.source.kind())
                    .validate(registered.source)?;
                if registered.axes.coordinate().is_none() {
                    continue;
                }
                crate::orchestrator::root::prepare_plot_intrinsics(
                    &intrinsics[registered.scope.id.slot].1,
                    &crate::orchestrator::root::PlotIntrinsicInputs {
                        registered,
                        request: crate::text::TextRequestContext::new(
                            &registered.appearance,
                            request.text,
                        ),
                        operation,
                        diagnostic_scope: registered.scope.render_name.as_deref(),
                        y_visible: registered.policy.axis_text.tick_labels[1],
                    },
                )?;
            }
        }
        let mut text_panels = Vec::new();
        crate::layout::fit::reserve(
            &mut text_panels,
            self.placements.len(),
            "TextPanel intrinsic summaries",
        )?;
        for panel in self.placements.texts() {
            text_panels.push(
                panel
                    .as_ref()
                    .map(|panel| text_panel::TextPanelIntrinsics::prepare(panel, request))
                    .transpose()?,
            );
        }
        let mut plot_titles = Vec::new();
        crate::layout::fit::reserve(
            &mut plot_titles,
            self.placements.len(),
            "Canvas Plot text intrinsics",
        )?;

        for (index, plot) in self.placements.plots().enumerate() {
            if let Some(plot) = plot {
                let mut scope = String::new();
                crate::data::source::reserve_source_string(&mut scope, 32)?;
                write!(scope, "panel-{index}").expect("reserved Canvas scope");
                let summary =
                    crate::decorations::title_block::plan::RootTitleBlockPlan::intrinsic_widths(
                        &crate::text::TextRequestContext::new(
                            &plot_at(plots, slots, index)
                                .expect("configured Canvas Plot theme")
                                .appearance,
                            request.text,
                        ),
                        plot.prose(),
                        Some(&scope),
                    )?;
                plot_titles.push((
                    plot_at(plots, slots, index)
                        .expect("configured Canvas prose")
                        .scope
                        .id
                        .clone(),
                    summary,
                ));
            }
        }
        let root = crate::decorations::title_block::plan::RootTitleBlockPlan::intrinsic_widths(
            request,
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
        request: &crate::text::TextRequestContext<'_, '_>,
        plots: &[crate::orchestrator::registration::PlotRegistration<'_>],
        slots: &[Option<crate::orchestrator::root::CanvasAnchor>],
        shared_guide: Option<&Arc<PreparedGuide>>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<(crate::layout::PreparedTextCensus, Vec<u64>), Error> {
        let root = crate::layout::ComponentAddress::root();
        let mut requirements = Vec::new();
        for (source, role, address_role, appearance) in [
            (
                self.title.provided(),
                crate::layout::PreparedTextRole::Title,
                crate::layout::AddressRole::RootTitle,
                &request.typography.root.title,
            ),
            (
                self.subtitle.provided(),
                crate::layout::PreparedTextRole::Subtitle,
                crate::layout::AddressRole::RootSubtitle,
                &request.typography.root.subtitle,
            ),
            (
                self.caption.provided(),
                crate::layout::PreparedTextRole::Caption,
                crate::layout::AddressRole::RootCaption,
                &request.typography.root.caption,
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
                request,
                &root.child(crate::layout::AddressRole::CanvasSharedLegend, 0)?,
                Some(prepared_stage),
            )?;
        }
        let composition_title = &request.typography.composition.composition_header;
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
        for (index, panel) in self.placements.texts().enumerate() {
            let Some(panel) = panel else { continue };
            let address = root.child(
                crate::layout::AddressRole::TextPanelContent,
                u32::try_from(index)
                    .map_err(|_| Error::Render("text-panel census index overflowed".into()))?,
            )?;
            let [title_style, body_style] = panel.appearances(request);
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

        for (index, plot) in self.placements.plots().enumerate() {
            let Some(plot) = plot else { continue };

            let registered =
                plot_at(plots, slots, index).expect("configured canvas panel has a resolved theme");
            let theme = &crate::text::TextRequestContext::new(&registered.appearance, request.text);
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
                Some(std::slice::from_ref(registered)),
                registered.guide.descriptor.as_ref(),
                &mut requirements,
                plot,
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
            request,
            &root,
            prepared_stage,
        )?;
        for (index, plot) in self.placements.plots().enumerate() {
            if plot.is_some() {
                let owner = root.child(
                    crate::layout::AddressRole::CanvasCell,
                    u32::try_from(index).map_err(|_| {
                        Error::Render("Canvas metric census owner overflowed".into())
                    })?,
                )?;
                crate::layout::PreparedTextRequirement::append_preference_metrics(
                    &mut requirements,
                    &crate::text::TextRequestContext::new(
                        &plot_at(plots, slots, index)
                            .expect("configured Canvas Plot retains theme")
                            .appearance,
                        request.text,
                    ),
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
        clippy::too_many_lines,
        reason = "one Canvas round consumes separate retained domains, hue identities, and child inputs"
    )]
    fn build_canvas_plan(
        &self,
        root: &Root<'_>,
        measurement: RootAssignment,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans,
        content_sizing: bool,
        mut measurements: Option<&mut sizing::HeightMeasurements>,
    ) -> Result<MeasuredOwnedPlan, Error> {
        use crate::layout::fit::FitOutcome;
        use crate::layout::kernel::TrialOutcome;
        let assigned = measurement.measurement_size;
        let canvas_layout = self.prepare_canvas_frame(root, assigned, round)?;
        let body_probes = if self.legend_policy == CanvasLegendPolicy::Shared {
            self.preflight_shared_legend(
                root,
                &measurement,
                canvas_layout.panel_box,
                children,
                round,
            )?
        } else {
            Vec::new()
        };
        let shared_legend = root.canvas_topology().shared_guide.as_ref();
        let mut compact_changes = true;
        if self.legend_policy != CanvasLegendPolicy::Shared || shared_legend.is_none() {
            return match self.build_canvas_attempt(
                root,
                measurement,
                &canvas_layout,
                &body_probes,
                content_sizing,
                round,
                None,
                &mut compact_changes,
                children,
                measurements.as_deref_mut(),
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
                round.observe_count(edge as usize)?;
                round.observe_count(usize::from(compact))?;
                let outcome = self.build_canvas_attempt(
                    root,
                    RootAssignment {
                        candidate: measurement.candidate.clone(),
                        request: measurement.request,
                        measurement_size: measurement.measurement_size,
                    },
                    &canvas_layout,
                    &body_probes,
                    content_sizing,
                    round,
                    Some((edge, compact)),
                    &mut compact_changes,
                    children,
                    measurements.as_deref_mut(),
                )?;
                match outcome {
                    FitOutcome::Measured(plan) => match round.finish_trial(checkpoint) {
                        _outcome @ (TrialOutcome::Fit | TrialOutcome::Provisional) => {
                            return Ok(plan);
                        }
                        TrialOutcome::Miss(reason) => {
                            FitFailure::retain_least(&mut first_miss, reason);
                        }
                    },
                    FitOutcome::Miss(reason) => {
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

    fn prepare_canvas_frame(
        &self,
        root: &Root<'_>,
        assigned: crate::layout::RootSize,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<RootFramePlan, Error> {
        let (width, height) = assigned.dimensions()?;
        let request = &root.request();

        let canvas_layout = RootFramePlan::measure(
            (width, height),
            request,
            self.prose(),
            crate::decorations::title_block::plan::RootAlignments::resolve(
                request,
                self.prose_alignments(),
                [None; 2],
            ),
            (
                crate::layout::canvas::MINIMUM_PANEL_SPAN,
                crate::layout::canvas::MINIMUM_PANEL_SPAN,
            ),
        )
        .map_err(crate::layout::fit::PlanError::into_error)?;
        round.observe_canvas(&canvas_layout)?;
        for (index, source) in self.placements.plots().enumerate() {
            let Some(source) = source else { continue };
            if let Some(registered) = root.canvas_plot(index) {
                crate::layout::interior::InteriorRoute::classify(registered.source.kind())
                    .validate(source)?;
            }
        }
        Ok(canvas_layout)
    }

    /// Retain the parent reduction and its finite-domain error at the original
    /// coordination boundary. Assignments consume these same real members.
    fn coordinate_domains(
        &self,
        root: &Root<'_>,
        round: &crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        for group in &root.canvas_topology().range_groups {
            let mut shared: Option<(f64, f64)> = None;
            for id in group {
                let registered = &root.plots[id.slot];
                let index = registered.scope.address.leaf_slot_index();
                let base = registered.axes.ranges()[0];
                let address = crate::layout::ComponentAddress::root()
                    .child(crate::layout::AddressRole::CanvasCell, index)?;
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
        }
        for (index, source) in self.placements.plots().enumerate() {
            if source.is_none() {
                continue;
            }
            let _ = crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("Canvas coordinated address overflowed".into()))?,
            )?;
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "gutter probes consume assigned cells and the selected Shared residual allocation"
    )]
    fn complete_cell_gutters(
        &self,
        root: &Root<'_>,
        measurement: &RootAssignment,
        dimensions: PlotMeasureDimensions,
        cells: &[Option<PixelRect>],
        request: &crate::text::TextRequestContext<'_, '_>,
        children: &mut crate::decorations::plan::children::ChildPlans,
        round: &crate::layout::kernel::FeedbackRound,
        requirements: &mut [(usize, usize, EdgeSpaceRequirements)],
        shared_allocation: Option<GuideAllocation>,
    ) -> Result<FitOutcome<()>, Error> {
        for GridSlot(index) in &root.canvas_topology().producers {
            let index = *index;
            let Some(registered) = root.canvas_plot(index) else {
                continue;
            };
            let theme = &crate::text::TextRequestContext::new(&registered.appearance, request.text);
            let dependencies = self.consumed_guide_dependencies(
                index,
                cells[index].expect("configured owner assignment").width(),
                shared_allocation,
                round,
            )?;
            let guides = GuideConsumption {
                dependencies: &dependencies,
                ..SUPPRESSED_GUIDES
            };
            let factory = root.plot_assignment_factory(measurement, &registered.scope.id)?;
            let entry = factory.assign(
                probe_allocation(
                    dimensions,
                    cells[index].expect("configured owner assignment"),
                    round,
                    &registered.scope.address,
                ),
                round,
                Some(MeasurementPhase::Probe),
            )?;
            let scope = FitScope::root();
            let context = factory.context(&entry, guides, &scope)?;
            let probe = match canvas_probe(
                &factory,
                &context,
                children,
                index,
                MeasurementPhase::Probe,
                round,
            )? {
                FitOutcome::Measured(probe) => probe,
                FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
            };
            let measured = EdgeSpaceRequirements::measure_header(
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
        Ok(FitOutcome::Measured(()))
    }

    #[expect(
        clippy::too_many_lines,
        clippy::too_many_arguments,
        reason = "one Canvas trial allocates declared tracks and measures every affected child"
    )]
    fn build_canvas_attempt(
        &self,
        root: &Root<'_>,
        measurement: RootAssignment,
        initial_frame: &RootFramePlan,
        body_probes: &[Option<Arc<crate::spec::PlotRenderResult>>],
        content_sizing: bool,
        round: &mut crate::layout::kernel::FeedbackRound,
        guide_trial: Option<(crate::LegendPosition, bool)>,
        compact_changes: &mut bool,
        children: &mut crate::decorations::plan::children::ChildPlans,
        mut measurements: Option<&mut sizing::HeightMeasurements>,
    ) -> Result<crate::layout::fit::FitOutcome<MeasuredOwnedPlan>, Error> {
        use crate::layout::fit;
        let root_address = crate::layout::ComponentAddress::root();
        let guide_owner = root_address.child(crate::layout::AddressRole::CanvasSharedLegend, 0)?;
        let scope = FitScope::new(&root_address, round.attempt_ordinal(&guide_owner));
        let guide_scope = FitScope::new(&guide_owner, scope.ordinal);
        let request = &root.request();
        let assigned = measurement.measurement_size;
        let mut canvas_layout = initial_frame.clone();
        let mut full_outer = canvas_layout.panel_box;
        self.coordinate_domains(root, round)?;
        let natural_grid_height = |width: f64,
                                   shared: bool,
                                   round: &crate::layout::kernel::FeedbackRound|
         -> Result<FitOutcome<f64>, Error> {
            // Both height composition and final allocation consume the same
            // root-owned gutters. A probe at a provisional height can choose
            // different ticks and must not replace those scoped inputs locally.
            let gutters = sizing::grid_gutters(self, root, round)?;
            let grid = tracks::allocate(
                PixelRect::new(0.0, width, 0.0, 0.0),
                &self.placements,
                root,
                Some(round),
                &self.column_headers,
                &self.row_headers,
                &self.row_titles,
                &gutters,
                request,
                &scope,
                Some(&tracks::ContentSizing {
                    source: self,
                    root,
                    output_dimensions: Some(assigned),
                    initial_sizing: false,
                    round,
                    measurements: measurements.as_deref(),
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
        let shared_legend = if let Some(slot) = &root.canvas_topology().shared_guide {
            let descriptor = &slot.descriptor;
            let (edge, compact) = guide_trial.expect("shared guide has an owned trial");
            let horizontal = matches!(
                edge,
                crate::LegendPosition::OutsideTop | crate::LegendPosition::OutsideBottom
            );
            let gap = request.legend_shared_gap()?;
            let minimum = match tracks::minimum(&self.placements, root, round, request, &scope)? {
                FitOutcome::Measured(minimum) => minimum,
                FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
            };
            let frame = crate::decorations::plan::AssignedGuideFrame::new(edge, minimum, gap);
            let available = frame.available(full_outer, [0.0; 4]);
            let measured = match Legend::measure_assigned(
                Arc::clone(descriptor),
                (available.0, (!content_sizing).then_some(available.1)),
                horizontal,
                compact,
                request,
                compact_changes,
                &guide_scope,
            )? {
                FitOutcome::Measured(legend) => legend,
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            };
            let (w, h) = (measured.compute_box_width(), measured.compute_box_height());
            if content_sizing {
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
                    sizing::content_frame(self, request, assigned.dimensions()?.0, height)?;
                full_outer = canvas_layout.panel_box;
            }
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
            if content_sizing {
                let height = match natural_grid_height(full_outer.width(), false, round)? {
                    FitOutcome::Measured(height) => height,
                    FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
                };
                canvas_layout =
                    sizing::content_frame(self, request, assigned.dimensions()?.0, height)?;
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
        let current_height = crate::layout::kernel::root_dimension(
            canvas_layout.panel_box.y1 + canvas_layout.insets.top,
        )?;
        let dimensions = PlotMeasureDimensions {
            canvas: crate::layout::RootSize::new(assigned.dimensions()?.0, current_height),
            output_override: Some(assigned),
        };
        let provisional = match tracks::allocate(
            outer,
            &self.placements,
            root,
            Some(round),
            &self.column_headers,
            &self.row_headers,
            &self.row_titles,
            &[],
            request,
            &scope.child(crate::layout::AddressRole::CanvasGrid, 0)?,
            content_sizing
                .then_some(tracks::ContentSizing {
                    source: self,
                    root,
                    output_dimensions: dimensions.output_override,
                    initial_sizing: false,
                    round,
                    measurements: measurements.as_deref(),
                    shared_allocation: shared_width,
                    natural_width: false,
                    natural_height: false,
                })
                .as_ref(),
        )? {
            FitOutcome::Measured(grid) => grid.owners,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        if self.category_labels == CanvasCategoryLabels::FirstColumnPerRow {
            let _ = self.category_bands()?;
            for band in &root.canvas_topology().category_bands {
                let first = root.plot(&band[0])?;
                let first_index = usize::try_from(first.scope.address.leaf_slot_index())
                    .expect("Canvas slot fits usize");
                let row = first_index / self.columns;
                let dependencies = self.consumed_guide_dependencies(
                    first_index,
                    provisional[first_index]
                        .expect("configured owner assignment")
                        .width(),
                    shared_allocation,
                    round,
                )?;
                let guides = GuideConsumption {
                    dependencies: &dependencies,
                    ..SUPPRESSED_GUIDES
                };
                let expected = category_probe(
                    root,
                    &measurement,
                    dimensions,
                    provisional[first_index].expect("configured owner assignment"),
                    guides,
                    children,
                    first_index,
                    round,
                )?;
                let expected_selection = first.source.y_axis_spec().category_labels.clone();
                for id in &band[1..] {
                    let registered = root.plot(id)?;
                    let index = usize::try_from(registered.scope.address.leaf_slot_index())
                        .expect("Canvas slot fits usize");
                    if expected_selection != registered.source.y_axis_spec().category_labels {
                        return Err(Error::Data(
                            "shared category axes require identical label selection policies"
                                .into(),
                        ));
                    }
                    let dependencies = self.consumed_guide_dependencies(
                        index,
                        provisional[index]
                            .expect("configured owner assignment")
                            .width(),
                        shared_allocation,
                        round,
                    )?;
                    let guides = GuideConsumption {
                        dependencies: &dependencies,
                        ..SUPPRESSED_GUIDES
                    };
                    if category_probe(
                        root,
                        &measurement,
                        dimensions,
                        provisional[index].expect("configured owner assignment"),
                        guides,
                        children,
                        index,
                        round,
                    )? != expected
                    {
                        return Err(Error::Data(format!(
                            "canvas row {row} category domains differ across panels"
                        )));
                    }
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
            self.placements.len(),
            "Canvas gutter requirements",
        )?;
        for index in 0..self.placements.len() {
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
                EdgeSpaceRequirements {
                    left,
                    right,
                    bottom,
                    top,
                },
            ));
        }
        for (group_index, group) in root.canvas_topology().range_groups.iter().enumerate() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasRangeGroup,
                u32::try_from(group_index)
                    .map_err(|_| Error::Render("canvas group address overflowed".into()))?,
            )?;
            let alignment = round.insets(&address, FeedbackKind::SharedAlignment);
            for id in group {
                let index = usize::try_from(root.plots[id.slot].scope.address.leaf_slot_index())
                    .expect("Canvas slot fits usize");
                requirements[index].2.left = requirements[index].2.left.max(alignment[0]);
                requirements[index].2.right = requirements[index].2.right.max(alignment[1]);
            }
        }
        if !content_sizing {
            if let FitOutcome::Miss(failure) = self.complete_cell_gutters(
                root,
                &measurement,
                dimensions,
                &provisional,
                request,
                children,
                round,
                &mut requirements,
                shared_allocation,
            )? {
                return Ok(FitOutcome::Miss(failure));
            }
        }
        let grid = match tracks::allocate(
            outer,
            &self.placements,
            root,
            Some(round),
            &self.column_headers,
            &self.row_headers,
            &self.row_titles,
            &requirements,
            request,
            &scope.child(crate::layout::AddressRole::CanvasGrid, 0)?,
            content_sizing
                .then_some(tracks::ContentSizing {
                    source: self,
                    root,
                    output_dimensions: dimensions.output_override,
                    initial_sizing: false,
                    round,
                    measurements: measurements.as_deref(),
                    shared_allocation: shared_width,
                    natural_width: false,
                    natural_height: false,
                })
                .as_ref(),
        )? {
            FitOutcome::Measured(grid) => grid,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        grid.tracks.observe(round)?;
        let cells = &grid.owners;
        let content_insets = &grid.content_insets;
        let headers = &grid.headers;
        let row_titles = &grid.row_titles;
        let mut measured = Vec::new();
        let mut requirements = Vec::new();
        fit::reserve(
            &mut measured,
            self.placements.len(),
            "Canvas measured children",
        )?;
        fit::reserve(
            &mut requirements,
            self.placements.len(),
            "Canvas measured requirements",
        )?;
        let mut child_miss = None;
        for GridSlot(index) in &root.canvas_topology().producers {
            let index = *index;
            let Some(registered) = root.canvas_plot(index) else {
                continue;
            };
            let plot = registered.source;
            let intrinsic = root.plot_intrinsic(&registered.scope.id)?;
            let row = index / self.columns;
            let column = index % self.columns;
            let kind = plot.kind();
            let theme = &crate::text::TextRequestContext::new(&registered.appearance, request.text);
            let child_scope = scope.child(crate::layout::AddressRole::CanvasCell, index)?;
            let cell = cells[index].expect("configured owner assignment");
            for value in self.owner_topology(index).expect("Plot topology") {
                round.observe_count(value)?;
            }
            let local = match plot_text::PlotLocalTextPlan::measure(
                registered,
                intrinsic,
                &root.operation,
                cell,
                theme,
                [self.panel_title_alignment, self.panel_subtitle_alignment],
                root.canvas_prose(&registered.scope.id),
                &child_scope,
                sizing::axis_inputs(root, registered, round, false).0,
            )? {
                FitOutcome::Measured(local) => local,
                FitOutcome::Miss(reason) => {
                    FitFailure::retain_least(&mut child_miss, reason);
                    continue;
                }
            };
            let owner = &registered.scope.address;
            let fill_content_height = content_sizing
                && self.legend_policy == CanvasLegendPolicy::PerPlot
                && !matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm);
            let factory = root.plot_assignment_factory(&measurement, &registered.scope.id)?;
            let mut allocation = probe_allocation(dimensions, local.body, round, owner);
            allocation.content_height = fill_content_height;
            allocation.fill_content_height = fill_content_height;
            allocation.guide.edge = plot.legend_position_spec();
            let entry = factory.assign(allocation, round, None)?;
            if shared_legend.is_some() {
                let base = body_probes[index]
                    .as_ref()
                    .expect("shared guide body probe");
                if let FitOutcome::Miss(reason) = crate::decorations::plan::check_assigned_body(
                    &factory.context(&entry, SUPPRESSED_GUIDES, &child_scope)?,
                    base,
                    full_outer,
                    local.body,
                    theme,
                    &scope.child(crate::layout::AddressRole::CanvasCell, index)?,
                )? {
                    FitFailure::retain_least(&mut child_miss, reason);
                }
            }
            plot.configured_source_rows()?;
            let dependencies =
                self.consumed_guide_dependencies(index, cell.width(), shared_allocation, round)?;
            let role = if self.legend_policy == CanvasLegendPolicy::PerPlot {
                crate::decorations::plan::LegendRole::Panel
            } else {
                crate::decorations::plan::LegendRole::Suppressed
            };
            let guide = if role == crate::decorations::plan::LegendRole::Panel {
                registered.local_guide_slot()?
            } else {
                None
            };
            let guides = GuideConsumption {
                dependencies: &dependencies,
                use_kind: if registered.source.legend_policy_spec() == crate::LegendPolicy::Hidden {
                    crate::decorations::legend::discrete::scoped::GuideUse::Hidden
                } else if role == crate::decorations::plan::LegendRole::Panel {
                    crate::decorations::legend::discrete::scoped::GuideUse::Local
                } else {
                    crate::decorations::legend::discrete::scoped::GuideUse::SharedSuppressed
                },
                local_guide: guide.as_ref(),
            };
            let panel_plan =
                match children.plan(&factory, entry, role, guides, &child_scope, round, owner)? {
                    FitOutcome::Measured(result) => result,
                    FitOutcome::Miss(failure) => {
                        FitFailure::retain_least(&mut child_miss, failure);
                        continue;
                    }
                };
            if fill_content_height {
                let target = grid_address.child(
                    crate::layout::AddressRole::CanvasCell,
                    u32::try_from(index).map_err(|_| {
                        Error::Render("Canvas content height address overflowed".into())
                    })?,
                )?;
                let preferred = sizing::measured_preference(
                    registered,
                    intrinsic,
                    &root.operation,
                    &panel_plan,
                    cell.height() - local.body.height(),
                    round,
                    owner,
                    theme,
                    sizing::axis_inputs(root, registered, round, false).0,
                )?;
                if let Some(measurements) = measurements.as_deref_mut() {
                    let (ranges, provenance, resolved_x) =
                        sizing::axis_inputs(root, registered, round, false);
                    measurements.record(
                        registered,
                        Some(self),
                        dimensions.output_override,
                        ranges,
                        provenance,
                        resolved_x,
                        cell.width(),
                        preferred,
                        content_insets[index],
                        round,
                        &target,
                        guides.dependencies,
                    )?;
                }
                let request = round.request_insets(
                    owner,
                    &target,
                    FeedbackKind::ContentHeight,
                    [cell.width(), 0.0, preferred, 0.0],
                )?;
                if request.is_some() || preferred > cell.height() + 1.0 / 1024.0 {
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
                assert_eq!(panel_plan.selected.cell, local.body);
                evidence
                    .borrow_mut()
                    .push((local.body, panel_plan.result.layout.projection.panel_box()));
            });
            let requirement = match fit::outcome(EdgeSpaceRequirements::measure_header(
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
            measured.push((index, panel_plan, local));
            requirements.push((row, column, requirement));
        }
        if let Some(failure) = child_miss {
            return Ok(FitOutcome::Miss(failure));
        }
        for (group_index, group) in root.canvas_topology().range_groups.iter().enumerate() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasRangeGroup,
                u32::try_from(group_index)
                    .map_err(|_| Error::Render("canvas group address overflowed".into()))?,
            )?;
            let mut alignment = [0.0_f64; 4];
            for id in group {
                let index = usize::try_from(root.plots[id.slot].scope.address.leaf_slot_index())
                    .expect("Canvas slot fits usize");
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
            round.observe_box(cells[index].expect("measured owner assignment"))?;
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
            self.placements.texts().flatten().count(),
        )?;
        for (index, text_panel) in self.placements.texts().enumerate() {
            let Some(text_panel) = text_panel else {
                continue;
            };
            let intrinsic = root.canvas_text()[index]
                .expect("configured TextPanel retains prepared intrinsics");
            match TextPanelPlan::measure(
                text_panel,
                index,
                cells[index].expect("configured TextPanel assignment"),
                request,
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
        for header in headers.iter().chain(row_titles) {
            round.observe_authored_text(&header.request)?;
            round.observe_text(&header.text.plan)?;
            round.observe_number(header.text.placement.position.x)?;
            round.observe_number(header.text.placement.position.y)?;
        }
        round.observe_count(usize::from(shared_legend.is_some()))?;
        if let Some(legend) = &shared_legend {
            legend.observe_feedback(round, legend.legend.available_width())?;
        }

        let mut panels = Vec::new();
        fit::reserve(&mut panels, measured.len(), "Canvas selected panels")?;
        for (index, panel_plan, local) in measured {
            let _ = cells[index].expect("measured owner assignment");
            let mut layers = [String::new(), String::new()];
            for (layer, suffix) in layers.iter_mut().zip(["legend", "title"]) {
                crate::data::source::reserve_source_string(layer, 40)?;
                write!(layer, "panel-{index}-{suffix}").expect("admitted Canvas layer name");
            }
            let [legend, header_layer] = layers;
            let legend = panel_plan
                .legend
                .as_ref()
                .map(|guide| {
                    let (x, y) = guide.bounds.anchor();
                    guide.legend.select(x, y)
                })
                .transpose()?
                .map(|draw| (legend, draw));
            panels.push(OwnedCanvasPlot {
                anchor: crate::orchestrator::root::GridSlot(index),
                plot: panel_plan,
                prose: local.text,
                prose_layer: header_layer,
                legend,
            });
        }
        let size = crate::layout::RootSize::new(
            assigned.dimensions()?.0,
            crate::layout::kernel::root_dimension(
                canvas_layout.panel_box.y1 + canvas_layout.insets.top,
            )?,
        );
        let content = OwnedCanvasPlan {
            grid,
            plots: panels,
            text_panels,
            shared_guide: shared_legend
                .as_ref()
                .map(|legend| {
                    let (x, y) = legend.bounds.anchor();
                    legend.legend.select(x, y)
                })
                .transpose()?,
        };
        let plan = MeasuredOwnedPlan {
            measurement,
            size,
            frame: canvas_layout.into_shared(),
            root_paint: request.surfaces.canvas_root,
            content: MeasuredRootContent::Canvas(content),
        };
        let MeasuredRootContent::Canvas(content) = &plan.content else {
            unreachable!("measured Canvas content")
        };
        Self::observe_arranged(content, round)?;
        Ok(FitOutcome::Measured(plan))
    }

    fn observe_arranged(
        content: &OwnedCanvasPlan,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        for panel in &content.text_panels {
            geometry.text(panel.title_placement, panel.title.plan(), None)?;
            geometry.text(panel.body_placement, panel.body.plan(), None)?;
        }
        for header in content.grid.row_titles.iter().chain(&content.grid.headers) {
            geometry.text(header.text.placement, &header.text.plan, None)?;
        }
        if let Some(legend) = &content.shared_guide {
            legend.geometry(&mut geometry)?;
        }
        geometry.observe_feedback(round)?;
        for panel in &content.plots {
            round.observe_box(
                content.grid.owners[panel.anchor.0].expect("measured owner assignment"),
            )?;
            round.observe_box(panel.plot.selected.cell)?;
            panel.prose.observe(round)?;

            let mut geometry = crate::render::geometry::GeometrySnapshot::new();
            panel.prose.geometry(&mut geometry)?;
            if let Some((_, legend)) = &panel.legend {
                legend.geometry(&mut geometry)?;
            }
            geometry.observe_feedback(round)?;
        }
        Ok(())
    }

    pub(crate) fn render_planned_canvas(
        &self,
        plan: &MeasuredOwnedPlan,
        width: u32,
        height: u32,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        backend.draw_surface(
            PixelRect::new(0.0, f64::from(width), 0.0, f64::from(height)),
            plan.root_paint,
            SurfaceRole::CanvasRoot,
        )?;
        let MeasuredRootContent::Canvas(content) = &plan.content else {
            unreachable!("measured Canvas content")
        };
        for panel in &content.plots {
            backend.draw_surface(
                panel.plot.panel_box,
                panel.plot.surface,
                SurfaceRole::PlotPanel,
            )?;
        }
        for panel in &content.plots {
            crate::render::backend::with_layer_role(
                backend,
                &panel.plot.layers.as_ref().expect("Canvas plot layers").grid,
                crate::render::backend::LayerRole::Grid,
                |backend| {
                    panel
                        .plot
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
        for panel in &content.plots {
            let plot = self.placements.plot(panel.anchor.0).ok_or_else(|| {
                Error::Render("composed canvas panel disappeared during render".into())
            })?;
            plot.configured_source_rows()?;
            crate::layout::interior::InteriorRoute::classify(plot.kind()).validate(plot)?;
            panel.plot.result.draw_body(backend)?;
            crate::significance::render_selected(&panel.plot.result, backend)?;
            with_layer(
                backend,
                &panel.plot.layers.as_ref().expect("Canvas plot layers").axes,
                |backend| {
                    panel
                        .plot
                        .result
                        .axes
                        .as_ref()
                        .ok_or_else(|| {
                            Error::Render("Canvas plan is missing immutable axes".into())
                        })?
                        .draw(backend)
                },
            )?;
            if let Some((layer, legend)) = &panel.legend {
                with_layer(backend, layer, |backend| legend.draw(backend))?;
            }
            with_layer(backend, &panel.prose_layer, |backend| {
                panel.prose.draw_retained(backend)
            })?;
        }
        for panel in &content.text_panels {
            draw_text_panel(panel, backend)?;
        }
        if !content.grid.row_titles.is_empty() {
            draw_canvas_row_titles(&content.grid.row_titles, backend)?;
        }
        if !content.grid.headers.is_empty() {
            draw_canvas_headers(&content.grid.headers, backend)?;
        }
        if let Some(legend) = &content.shared_guide {
            with_layer(backend, "canvas-shared-legend", |backend| {
                legend.draw(backend)
            })?;
        }
        plan.frame.root_text.draw_retained(backend)
    }

    fn preflight_shared_legend(
        &self,
        root: &Root<'_>,
        measurement: &RootAssignment,
        panel: PixelRect,
        children: &mut crate::decorations::plan::children::ChildPlans,
        round: &crate::layout::kernel::FeedbackRound,
    ) -> Result<Vec<Option<Arc<crate::spec::PlotRenderResult>>>, Error> {
        let mut probes = Vec::new();
        crate::layout::fit::reserve(
            &mut probes,
            self.placements.len(),
            "shared guide body probes",
        )?;
        probes.resize_with(self.placements.len(), || None);
        for GridSlot(index) in &root.canvas_topology().producers {
            let index = *index;
            let Some(registered) = root.canvas_plot(index) else {
                continue;
            };
            let factory = root.plot_assignment_factory(measurement, &registered.scope.id)?;
            let entry = factory.assign(
                probe_allocation(
                    PlotMeasureDimensions {
                        canvas: measurement.measurement_size,
                        output_override: Some(measurement.measurement_size),
                    },
                    panel,
                    round,
                    &registered.scope.address,
                ),
                round,
                Some(MeasurementPhase::Legend),
            )?;
            let scope = FitScope::root();
            let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
            let result = crate::layout::fit::measured(canvas_probe(
                &factory,
                &context,
                children,
                index,
                MeasurementPhase::Legend,
                round,
            ))
            .map_err(crate::layout::fit::PlanError::into_error)?;
            probes[index] = Some(result);
        }
        Ok(probes)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the category probe consumes one existing assigned cell and its exact parent guide operands"
)]
fn category_probe(
    root: &Root<'_>,
    measurement: &RootAssignment,
    dimensions: PlotMeasureDimensions,
    cell: PixelRect,
    guides: GuideConsumption<'_>,
    children: &mut crate::decorations::plan::children::ChildPlans,
    index: usize,
    round: &crate::layout::kernel::FeedbackRound,
) -> Result<Vec<String>, Error> {
    let registered = root.canvas_plot(index).expect("prepared Canvas plot");
    if !matches!(
        registered.kind(),
        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
    ) {
        return Err(Error::Data(
            "shared category labels are supported only by horizontal bar families".into(),
        ));
    }
    let factory = root.plot_assignment_factory(measurement, &registered.scope.id)?;
    let entry = factory.assign(
        probe_allocation(dimensions, cell, round, &registered.scope.address),
        round,
        Some(MeasurementPhase::Category),
    )?;
    let scope = FitScope::root();
    let context = factory.context(&entry, guides, &scope)?;
    plot_category_domain(&factory, &context, children, index, round)
}

fn probe_allocation(
    dimensions: PlotMeasureDimensions,
    cell: PixelRect,
    round: &crate::layout::kernel::FeedbackRound,
    owner: &crate::layout::ComponentAddress,
) -> PlotAllocationInputs {
    PlotAllocationInputs {
        dimensions,
        cell,
        frame: cell,
        body: cell,
        axis_label_width: cell.width(),
        content_height: false,
        fill_content_height: false,
        content_preference: None,
        insets: ParentInsets::consumed(round, owner),
        guide: GuideTrialAssignment {
            edge: crate::LegendPosition::Auto,
            compact: false,
            allocation: None,
        },
    }
}

fn canvas_probe(
    factory: &PlotMeasureAssignmentFactory<'_, '_>,
    context: &PlotMeasureContext<'_, '_>,
    children: &mut crate::decorations::plan::children::ChildPlans,
    index: usize,
    phase: MeasurementPhase,
    round: &crate::layout::kernel::FeedbackRound,
) -> Result<FitOutcome<Arc<crate::spec::PlotRenderResult>>, Error> {
    let owner = crate::layout::ComponentAddress::root().child(
        crate::layout::AddressRole::CanvasCell,
        u32::try_from(index).map_err(|_| Error::Render("canvas address overflowed".into()))?,
    )?;
    children.measure(context, &owner, phase, round, || {
        crate::layout::fit::outcome(
            crate::orchestrator::measure_probe(
                factory,
                context.assignment.local.clone(),
                context.guides,
                context.fit_scope,
            )
            .map(|(_, result)| result),
        )
    })
}

fn plot_category_domain(
    factory: &PlotMeasureAssignmentFactory<'_, '_>,
    context: &PlotMeasureContext<'_, '_>,
    children: &mut crate::decorations::plan::children::ChildPlans,
    index: usize,
    round: &crate::layout::kernel::FeedbackRound,
) -> Result<Vec<String>, Error> {
    let kind = context.registered.kind();
    if !matches!(
        kind,
        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
    ) {
        return Err(Error::Data(
            "shared category labels are supported only by horizontal bar families".into(),
        ));
    }
    let result = crate::layout::fit::measured(canvas_probe(
        factory,
        context,
        children,
        index,
        MeasurementPhase::Category,
        round,
    ))
    .map_err(crate::layout::fit::PlanError::into_error)?;
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
