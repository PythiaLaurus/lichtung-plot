//! Closed interior boundary. All 25 families consume prepared semantics and
//! selected payloads. Parent composition adapters remain separately classified.
//! Native coordinate axes and frames share one orientation-aware producer.

use crate::layout::kernel::{FeedbackKind, FeedbackRequestId, FeedbackRound};
use crate::layout::ComponentAddress;
use crate::spec::PlotKind;
use crate::{Error, Plot};

pub(crate) mod coordinate;
pub(crate) mod guide;
pub(crate) mod preference;

macro_rules! interior_routes {
    (native { $($native:ident { $($native_kind:ident),+ $(,)? }),+ $(,)? }
     legacy { $($class:ident { $($kind:ident),+ $(,)? }),+ $(,)? }) => {
        $(#[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum $native { $($native_kind),+ })+

        $(#[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum $class { $($kind),+ })+

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum InteriorRoute { $($native($native),)+ $($class($class)),+ }

        impl InteriorRoute {
            pub(crate) const fn classify(kind: PlotKind) -> Self {
                match kind {
                    $($(PlotKind::$native_kind => Self::$native($native::$native_kind),)+)+
                    $($(PlotKind::$kind => Self::$class($class::$kind),)+)+
                }
            }

            pub(crate) const fn kind(self) -> PlotKind {
                match self {
                    $($(Self::$native($native::$native_kind) => PlotKind::$native_kind,)+)+
                    $($(Self::$class($class::$kind) => PlotKind::$kind,)+)+
                }
            }
        }
    };
}

interior_routes! {
    native {
        StandardCoordinate { Scatter, Line, Trajectory, LinePath, Histogram, Density, Qq, Regression },
        OneCategoricalAxis { PointInterval, Coefficient, Forest, Dumbbell, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin, Raincloud, Interaction },
    }
    legacy {
        SpecializedLegacy {
            Heatmap, Marginal, Matrix, Beeswarm, Waterfall,
        },
    }
}

impl InteriorRoute {
    pub(crate) const fn is_native(self) -> bool {
        match self {
            Self::StandardCoordinate(_) | Self::OneCategoricalAxis(_) => true,
            Self::SpecializedLegacy(_) => false,
        }
    }

    pub(crate) fn validate(self, plot: &Plot) -> Result<(), Error> {
        if self.kind() != plot.kind() {
            return Err(Error::Data(format!(
                "interior route {:?} does not match prepared plot {:?}",
                self.kind(),
                plot.kind()
            )));
        }
        Ok(())
    }
}

/// Composition support is independent of the child's interior route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompositionChildPolicy {
    Facet,
    Canvas,
}

impl CompositionChildPolicy {
    pub(crate) fn validate_child(self, plot: &Plot) -> Result<(), Error> {
        match self {
            Self::Canvas if plot.kind() == PlotKind::Matrix => Err(Error::Render(
                "Matrix manages its own canvas and cannot be nested in a composed canvas".into(),
            )),
            Self::Canvas if plot.has_facets() => Err(Error::Render(
                "a faceted chart cannot be nested inside a composed canvas".into(),
            )),
            Self::Facet if matches!(plot.kind(), PlotKind::Matrix | PlotKind::Marginal) => {
                Err(Error::Render(format!(
                    "{:?} manages its own panels and cannot be nested in generic faceting",
                    plot.kind()
                )))
            }
            Self::Canvas | Self::Facet => Ok(()),
        }
    }
}

/// Root-owned scientific axes plus extracted mark semantics. Attachment shares
/// storage; no prepared Plot clone or alternate preparation route is retained.
/// Trajectory retains its condition order.
pub(crate) struct PreparedInteriorInputs {
    pub(crate) waterfall: Option<std::sync::Arc<crate::plots::waterfall::PreparedWaterfallInputs>>,
    pub(crate) beeswarm: Option<std::sync::Arc<crate::plots::beeswarm::PreparedBeeswarmInputs>>,
    pub(crate) marginal: Option<std::sync::Arc<crate::plots::marginal::PreparedMarginal>>,
    pub(crate) heatmap: Option<std::sync::Arc<crate::plots::heatmap::PreparedHeatmapInputs>>,
    pub(crate) interaction:
        Option<std::sync::Arc<crate::plots::interaction::PreparedInteractionInputs>>,
    pub(crate) distributions:
        Option<std::sync::Arc<crate::plots::categorical_distribution::PreparedDistributionInputs>>,
    pub(crate) bars: Option<std::sync::Arc<crate::plots::bar_data::PreparedBarInputs>>,
    pub(crate) estimate:
        Option<std::sync::Arc<crate::plots::interval_estimate::PreparedEstimateInputs>>,
    pub(crate) dumbbell: Option<std::sync::Arc<crate::plots::dumbbell::PreparedDumbbellInputs>>,
    pub(crate) route: InteriorRoute,
    pub(crate) axes: std::sync::Arc<Vec<(Option<String>, crate::plots::shared_x::PreparedAxes)>>,
    pub(crate) matrix: Option<std::sync::Arc<crate::plots::matrix::PreparedMatrixAxes>>,
    pub(crate) scatter: Option<std::sync::Arc<crate::plots::scatter::PreparedScatterInputs>>,
    pub(crate) line: Option<std::sync::Arc<crate::plots::line::PreparedLineInputs>>,
    pub(crate) qq: Option<std::sync::Arc<crate::plots::qq::PreparedQqInputs>>,
    pub(crate) regression:
        Option<std::sync::Arc<crate::plots::regression::PreparedRegressionInputs>>,
    pub(crate) histogram: Option<std::sync::Arc<crate::plots::histogram::PreparedHistogramInputs>>,
    pub(crate) density: Option<std::sync::Arc<crate::plots::density::PreparedDensityInputs>>,
}

impl PreparedInteriorInputs {
    /// Semantic attachment does not discover fonts. Each attached panel obtains
    /// its post-font summary before the parent starts width allocation.
    pub(crate) fn prepare_intrinsics(
        &self,
        plot: &Plot,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<(), Error> {
        self.route.validate(plot)?;
        for (index, (scope, axes)) in self.axes.iter().enumerate() {
            if self.route.is_native() {
                coordinate::Intrinsics::prepare(axes, theme, scope.as_deref())?;
            } else if let Some(heatmap) = &self.heatmap {
                let prepared = heatmap
                    .panels
                    .get(index)
                    .filter(|(owner, _)| owner == scope)
                    .ok_or_else(|| {
                        Error::Render("prepared heatmap intrinsic scope missing".into())
                    })?;
                crate::plots::heatmap::prepare_intrinsics(
                    &prepared.1,
                    axes,
                    plot.data(),
                    theme,
                    scope.as_deref(),
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn attach(&self, mut plot: Plot) -> Result<Plot, Error> {
        self.route.validate(&plot)?;
        plot.prepared_axes = self.axes.clone();
        plot.prepared_waterfall.clone_from(&self.waterfall);
        plot.prepared_beeswarm.clone_from(&self.beeswarm);
        plot.prepared_heatmap.clone_from(&self.heatmap);
        plot.prepared_marginal.clone_from(&self.marginal);
        plot.prepared_interaction.clone_from(&self.interaction);
        plot.prepared_matrix_axes.clone_from(&self.matrix);
        plot.prepared_scatter.clone_from(&self.scatter);
        plot.prepared_line.clone_from(&self.line);
        plot.prepared_histogram.clone_from(&self.histogram);
        plot.prepared_density.clone_from(&self.density);
        plot.prepared_qq.clone_from(&self.qq);
        plot.prepared_dumbbell.clone_from(&self.dumbbell);
        plot.prepared_estimate.clone_from(&self.estimate);
        plot.prepared_bars.clone_from(&self.bars);
        plot.prepared_distributions.clone_from(&self.distributions);
        plot.prepared_regression.clone_from(&self.regression);
        Ok(plot)
    }
}

impl Plot {
    pub(crate) fn intrinsic_body_minimum(
        &self,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<(f64, f64), Error> {
        let mut minimum = if InteriorRoute::classify(self.kind()).is_native() {
            coordinate::Intrinsics::prepared(self, theme)?.minimum(self)
        } else {
            (
                crate::layout::canvas::MINIMUM_PANEL_SPAN,
                crate::layout::canvas::MINIMUM_PANEL_SPAN,
            )
        };
        let mark = match self.kind() {
            PlotKind::Scatter => Some(crate::plots::scatter::intrinsic_body_minimum(self, theme)?),
            PlotKind::Qq => Some(crate::plots::qq::intrinsic_body_minimum(self, theme)?),
            PlotKind::Regression => Some(crate::plots::regression::intrinsic_body_minimum(
                self, theme,
            )?),
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                Some(crate::plots::line::intrinsic_body_minimum(self, theme)?)
            }
            PlotKind::Density => Some(crate::plots::density::intrinsic_body_minimum(self, theme)?),
            PlotKind::Histogram => Some(crate::plots::histogram::intrinsic_body_minimum(
                self, theme,
            )?),
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
                let prepared = self
                    .prepared_estimate
                    .as_ref()
                    .and_then(|inputs| {
                        inputs
                            .panels
                            .iter()
                            .find(|(scope, _)| scope.as_deref() == self.axis_scope())
                    })
                    .map(|(_, marks)| marks)
                    .ok_or_else(|| {
                        Error::Render("estimate intrinsic requires root-prepared rows".into())
                    })?;
                Some(prepared.mark_intrinsic(self, theme)?)
            }
            PlotKind::Dumbbell => {
                Some(crate::plots::dumbbell::intrinsic_body_minimum(self, theme)?)
            }
            PlotKind::Bar
            | PlotKind::StackedBar
            | PlotKind::HorizontalBar
            | PlotKind::StackedHorizontalBar => {
                Some(crate::plots::bar_data::intrinsic_body_minimum(self, theme)?)
            }
            PlotKind::Boxplot => Some(crate::plots::boxplot::intrinsic_body_minimum(self, theme)?),
            PlotKind::Raincloud if self.raincloud_geometry_input().is_some() => Some(
                crate::plots::violin::precomputed::intrinsic_body_minimum(self, theme)?,
            ),
            PlotKind::Violin | PlotKind::Raincloud => {
                Some(crate::plots::violin::intrinsic_body_minimum(self, theme)?)
            }
            PlotKind::Interaction => Some(crate::plots::interaction::intrinsic_body_minimum(
                self, theme,
            )?),
            PlotKind::Heatmap => Some(crate::plots::heatmap::intrinsic_body_minimum(self, theme)?),
            PlotKind::Beeswarm => {
                Some(crate::plots::beeswarm::intrinsic_body_minimum(self, theme)?)
            }
            PlotKind::Waterfall => Some(crate::plots::waterfall::intrinsic_body_minimum(
                self, theme,
            )?),
            PlotKind::Marginal => {
                Some(crate::plots::marginal::intrinsic_body_minimum(self, theme)?)
            }
            PlotKind::Matrix => Some(crate::plots::matrix::intrinsic_body_minimum(self, theme)?),
        };
        if let Some(mark) = mark {
            minimum.0 = minimum.0.max(mark.0);
            minimum.1 = minimum.1.max(mark.1);
        }
        Ok(minimum)
    }
}

/// Shared domain consequence, submitted by its actual producer to the parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClearanceAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DomainExpansion {
    pub(crate) axis: ClearanceAxis,
    pub(crate) limits: (f64, f64),
}

impl DomainExpansion {
    pub(crate) fn submit(
        self,
        round: &mut FeedbackRound,
        source: &ComponentAddress,
        target: &ComponentAddress,
    ) -> Result<Option<FeedbackRequestId>, Error> {
        let kind = match self.axis {
            ClearanceAxis::X => FeedbackKind::AutomaticDomainX,
            ClearanceAxis::Y => FeedbackKind::AutomaticDomainY,
        };
        round.request_domain(source, target, kind, self.limits)
    }
}

/// One automatic data-axis request for the canonical painted extent of an
/// extreme source mark. Exact data windows do not call this producer.
pub(crate) fn automatic_mark_ink_domain(
    axis: ClearanceAxis,
    source: (f64, f64),
    current: (f64, f64),
    pixels: (f64, f64),
    projected: (f64, f64),
    ink: (f64, f64),
    scope: &crate::layout::fit::FitScope,
) -> Result<Option<DomainExpansion>, crate::layout::fit::PlanError> {
    use crate::layout::fit::FitKind;
    if !source.0.is_finite()
        || !source.1.is_finite()
        || !projected.0.is_finite()
        || !projected.1.is_finite()
    {
        return Err(
            Error::Render("automatic mark ink source/projection must be finite".into()).into(),
        );
    }
    let quantum = 1.0 / 1024.0;
    let left = (-ink.0).max(0.0) + 2.0 * quantum;
    let right = ink.1.max(0.0) + 2.0 * quantum;
    if projected.0 + ink.0 >= pixels.0 + quantum && projected.1 + ink.1 <= pixels.1 - quantum {
        return Ok(None);
    }
    let available = pixels.1 - pixels.0 - left - right;
    if !available.is_finite() || available <= 0.0 {
        return Err(scope
            .failure(
                FitKind::TrackOrResidual,
                "automatic mark ink leaves no representable data span in its assigned panel",
            )
            .into());
    }
    let scale = source
        .0
        .abs()
        .max(source.1.abs())
        .max(current.0.abs())
        .max(current.1.abs())
        .max(1.0);
    // A facet can own only a narrow subset of a shared numeric domain. Use
    // the whole assigned domain span when it is wider than local source data;
    // padding against the local span alone could never move an edge already
    // extended by a sibling.
    let span =
        ((source.1 / scale) - (source.0 / scale)).max((current.1 / scale) - (current.0 / scale));
    if !span.is_finite() || span <= 0.0 {
        return Err(scope
            .failure(
                FitKind::SpecializedGeometry,
                "automatic mark ink source span is unrepresentable",
            )
            .into());
    }
    let lower = ((source.0 / scale) - span * left / available) * scale;
    let upper = ((source.1 / scale) + span * right / available) * scale;
    if !lower.is_finite() || !upper.is_finite() {
        return Err(scope
            .failure(
                FitKind::SpecializedGeometry,
                "automatic mark ink cannot obtain a finite data domain",
            )
            .into());
    }
    let limits = (current.0.min(lower), current.1.max(upper));
    if limits == current {
        return Err(scope
            .failure(
                FitKind::SpecializedGeometry,
                "automatic mark ink cannot extend this representable data domain",
            )
            .into());
    }
    Ok(Some(DomainExpansion { axis, limits }))
}

/// Use a single admitted extrema projection for both automatically owned
/// numeric axes. A categorical axis is excluded by the caller's semantic mask.
pub(crate) fn automatic_mark_ink_expansions(
    source: [(f64, f64); 2],
    projection: &crate::layout::Projection,
    bounds: crate::render::geometry::GeometryBox,
    automatic: [bool; 2],
    data: crate::data::PlotData<'_>,
    scope: &crate::layout::fit::FitScope,
) -> Result<[Option<DomainExpansion>; 2], crate::layout::fit::PlanError> {
    let mut expansions = [None; 2];
    if !automatic.into_iter().any(|axis| axis) {
        return Ok(expansions);
    }
    data.claim_source_visits(2)?;
    let low = projection.project(source[0].0, source[1].0);
    let high = projection.project(source[0].1, source[1].1);
    let panel = projection.panel_box();
    for (index, (axis, current, pixels, projected, painted)) in [
        (
            ClearanceAxis::X,
            projection.x_range(),
            (panel.x0, panel.x1),
            (low.0, high.0),
            (bounds.left, bounds.right),
        ),
        (
            ClearanceAxis::Y,
            projection.y_range(),
            (panel.y0, panel.y1),
            (low.1, high.1),
            (bounds.bottom, bounds.top),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if automatic[index] {
            expansions[index] = automatic_mark_ink_domain(
                axis,
                source[index],
                current,
                pixels,
                projected,
                painted,
                scope,
            )?;
        }
    }
    Ok(expansions)
}

/// An already selected polygon/line has exact canonical *absolute* ink. Relate
/// that ink to the retained scientific extrema before requesting a new domain;
/// clipping the candidate first would erase the missing outward region.
pub(crate) fn automatic_absolute_ink_expansions(
    source: [(f64, f64); 2],
    projection: &crate::layout::Projection,
    bounds: crate::render::geometry::GeometryBox,
    automatic: [bool; 2],
    data: crate::data::PlotData<'_>,
    scope: &crate::layout::fit::FitScope,
) -> Result<[Option<DomainExpansion>; 2], crate::layout::fit::PlanError> {
    let mut expansions = [None; 2];
    if !automatic.into_iter().any(|axis| axis) {
        return Ok(expansions);
    }
    data.claim_source_visits(2)?;
    let low = projection.project(source[0].0, source[1].0);
    let high = projection.project(source[0].1, source[1].1);
    let panel = projection.panel_box();
    for (index, (axis, current, pixels, projected, painted)) in [
        (
            ClearanceAxis::X,
            projection.x_range(),
            (panel.x0, panel.x1),
            (low.0, high.0),
            (bounds.left - low.0, bounds.right - high.0),
        ),
        (
            ClearanceAxis::Y,
            projection.y_range(),
            (panel.y0, panel.y1),
            (low.1, high.1),
            (bounds.bottom - low.1, bounds.top - high.1),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if automatic[index] {
            expansions[index] = automatic_mark_ink_domain(
                axis,
                source[index],
                current,
                pixels,
                projected,
                painted,
                scope,
            )?;
        }
    }
    Ok(expansions)
}

pub(crate) fn merge_mark_domain_expansions(
    mut existing: [Option<DomainExpansion>; 2],
    additional: [Option<DomainExpansion>; 2],
) -> [Option<DomainExpansion>; 2] {
    for (target, candidate) in existing.iter_mut().zip(additional) {
        if let Some(candidate) = candidate {
            *target = Some(target.map_or(candidate, |current| DomainExpansion {
                axis: current.axis,
                limits: (
                    current.limits.0.min(candidate.limits.0),
                    current.limits.1.max(candidate.limits.1),
                ),
            }));
        }
    }
    existing
}

/// Insets use the existing spatial quantization and operation-owned feedback.
/// No adapter owns a budget, retry loop, or root alternative. Requests replace
/// stale effects through FeedbackRound; rejected work remains charged there.
pub(crate) enum InteriorReserveRequest {
    AxisGutter([f64; 4]),
    OwnedInk([f64; 4]),
}

impl InteriorReserveRequest {
    pub(crate) fn submit(
        self,
        round: &mut FeedbackRound,
        source: &ComponentAddress,
        target: &ComponentAddress,
    ) -> Result<Option<FeedbackRequestId>, Error> {
        let (kind, insets) = match self {
            Self::AxisGutter(insets) => (FeedbackKind::AxisGutter, insets),
            Self::OwnedInk(insets) => (FeedbackKind::OwnedInk, insets),
        };
        round.request_insets(source, target, kind, insets)
    }
}

/// One selected immutable mark scene and its canonical painted geometry.
#[derive(Debug)]
pub(crate) struct SelectedMarkPlan {
    pub(crate) payload: MarkPayload,
    pub(crate) geometry: crate::render::geometry::GeometrySnapshot,
}

#[derive(Debug)]
pub(crate) enum MarkPayload {
    Waterfall(crate::plots::waterfall::WaterfallPayload),
    Beeswarm(crate::plots::beeswarm::BeeswarmPayload),
    Matrix(crate::plots::matrix::MatrixPayload),
    Marginal(crate::plots::marginal::MarginalPayload),
    Heatmap(crate::plots::heatmap::HeatmapPayload),
    Interaction(crate::plots::interaction::InteractionPayload),
    Distribution(crate::plots::categorical_distribution::DistributionPayload),
    Bars(crate::plots::bar_data::BarPayload),
    Estimate(crate::plots::interval_estimate::EstimatePayload),
    Dumbbell(crate::plots::dumbbell::DumbbellPayload),
    Scatter(crate::plots::scatter::ScatterPayload),
    Line(crate::plots::line::LinePayload),
    Histogram(crate::plots::histogram::HistogramPayload),
    Density(crate::plots::density::DensityPayload),
    Qq(crate::plots::qq::QqPayload),
    Regression(crate::plots::regression::RegressionPayload),
}

impl SelectedMarkPlan {
    pub(crate) fn draw(
        &self,
        backend: &mut dyn crate::render::backend::Backend,
    ) -> Result<(), Error> {
        match &self.payload {
            MarkPayload::Waterfall(payload) => payload.draw(backend),
            MarkPayload::Beeswarm(payload) => payload.draw(backend),
            MarkPayload::Matrix(payload) => payload.draw(backend),
            MarkPayload::Marginal(payload) => payload.draw(backend),
            MarkPayload::Heatmap(payload) => payload.draw(backend),
            MarkPayload::Interaction(payload) => payload.draw(backend),
            MarkPayload::Distribution(payload) => payload.draw(backend),
            MarkPayload::Bars(payload) => payload.draw(backend),
            MarkPayload::Estimate(payload) => payload.draw(backend),
            MarkPayload::Dumbbell(payload) => payload.draw(backend),
            MarkPayload::Scatter(payload) => payload.draw(backend),
            MarkPayload::Line(payload) => payload.draw(backend),
            MarkPayload::Histogram(payload) => payload.draw(backend),
            MarkPayload::Density(payload) => payload.draw(backend),
            MarkPayload::Qq(payload) => payload.draw(backend),
            MarkPayload::Regression(payload) => payload.draw(backend),
        }
    }
}

/// Selected guide, panel-text, and significance styles with frozen geometry.
#[derive(Debug)]
pub(crate) struct SelectedAnnotationPlan {
    pub(crate) significance_styles: Option<Vec<crate::render::backend::BracketStyle>>,
    pub(crate) guides: crate::orchestrator::layers::GuideSnapshot,
    pub(crate) panel_layer: Option<String>,
    pub(crate) panel_clip: crate::layout::PanelBox,
    pub(crate) text: Vec<crate::render::backend::PlacedText>,
    pub(crate) geometry: crate::render::geometry::GeometrySnapshot,
}

impl SelectedAnnotationPlan {
    pub(crate) fn draw(
        &self,
        backend: &mut dyn crate::render::backend::Backend,
    ) -> Result<(), Error> {
        self.guides.draw(backend)?;
        if let Some(layer) = &self.panel_layer {
            crate::render::backend::with_layer(backend, layer, |backend| {
                crate::render::backend::with_clip(backend, self.panel_clip, |backend| {
                    for text in &self.text {
                        text.draw(backend)?;
                    }
                    Ok(())
                })
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod automatic_ink_tests {
    use super::{automatic_mark_ink_domain, ClearanceAxis};
    use crate::layout::fit::{FitKind, PlanError, ROOT_SCOPE};

    #[test]
    fn painted_extremes_request_one_finite_monotonic_domain_per_automatic_axis() {
        let request = automatic_mark_ink_domain(
            ClearanceAxis::X,
            (0.0, 10.0),
            (-0.5, 10.5),
            (0.0, 100.0),
            (100.0 / 22.0, 100.0 * 21.0 / 22.0),
            (-20.0, 20.0),
            &ROOT_SCOPE,
        )
        .unwrap()
        .unwrap();
        assert!(request.limits.0 < -0.5 && request.limits.1 > 10.5);
        let project =
            |value: f64| (value - request.limits.0) / (request.limits.1 - request.limits.0) * 100.0;
        assert!(project(0.0) - 20.0 > 0.0);
        assert!(project(10.0) + 20.0 < 100.0);
        assert!(
            automatic_mark_ink_domain(
                ClearanceAxis::X,
                (0.0, 10.0),
                request.limits,
                (0.0, 100.0),
                (project(0.0), project(10.0)),
                (-20.0, 20.0),
                &ROOT_SCOPE,
            )
            .unwrap()
            .is_none(),
            "requested={:?} painted=[{}, {}]",
            request.limits,
            project(0.0) - 20.0,
            project(10.0) + 20.0
        );
        let miss = automatic_mark_ink_domain(
            ClearanceAxis::Y,
            (0.0, 10.0),
            (-0.5, 10.5),
            (0.0, 30.0),
            (0.0, 30.0),
            (-20.0, 20.0),
            &ROOT_SCOPE,
        )
        .unwrap_err();
        assert!(
            matches!(miss, PlanError::Fit(failure) if failure.reason.kind == FitKind::TrackOrResidual)
        );
        let extreme = automatic_mark_ink_domain(
            ClearanceAxis::X,
            (-f64::MAX, f64::MAX),
            (-f64::MAX, f64::MAX),
            (0.0, 100.0),
            (0.0, 100.0),
            (-4.0, 4.0),
            &ROOT_SCOPE,
        )
        .unwrap_err();
        assert!(
            matches!(extreme, PlanError::Fit(failure) if failure.reason.kind == FitKind::SpecializedGeometry)
        );
    }
}

#[cfg(test)]
pub(crate) mod distributions;

#[cfg(test)]
pub(crate) mod estimates;

#[cfg(test)]
pub(crate) mod bars;

#[cfg(test)]
pub(crate) mod categorical_distributions;
