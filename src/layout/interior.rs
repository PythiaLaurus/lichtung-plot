//! Closed interior boundary. All 25 families consume prepared semantics and
//! selected payloads. Parent composition adapters remain separately classified.
//! Native coordinate axes and frames share one orientation-aware producer.

use crate::layout::kernel::{FeedbackKind, FeedbackRequestId, FeedbackRound};
use crate::layout::ComponentAddress;
use crate::spec::PlotKind;
use crate::{Error, Plot};

pub(crate) mod coordinate;
pub(crate) mod guide;

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

/// Parent classification is independent of the child's interior classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParentCompositionAdapter {
    Facet,
    Canvas,
}

impl ParentCompositionAdapter {
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
        if InteriorRoute::classify(self.kind()).is_native() {
            return Ok(coordinate::Intrinsics::prepared(self, theme)?.minimum(self));
        }
        let mut minimum = DeferredLegacyPanelMinimum::SIZE;
        if self.kind() == PlotKind::Heatmap {
            let heatmap = crate::plots::heatmap::intrinsic_body_minimum(self, theme)?;
            minimum.0 = minimum.0.max(heatmap.0);
            minimum.1 = minimum.1.max(heatmap.1);
        }
        Ok(minimum)
    }
}

/// The sole named parity floor, pending T5c's exact mark/annotation intrinsics.
pub(crate) struct DeferredLegacyPanelMinimum;

impl DeferredLegacyPanelMinimum {
    pub(crate) const SIZE: (f64, f64) = (80.0, 60.0);
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

/// Selected mark consequences remain deferred to T5c, independently of axes.
#[derive(Debug)]
pub(crate) struct DeferredMarkPlan {
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

impl DeferredMarkPlan {
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

/// Current ordinary guide and panel-text placements, not future T5c semantics.
#[derive(Debug)]
pub(crate) struct DeferredAnnotationPlan {
    pub(crate) significance_styles: Option<Vec<crate::render::backend::BracketStyle>>,
    pub(crate) guides: crate::orchestrator::layers::GuideSnapshot,
    pub(crate) panel_layer: Option<String>,
    pub(crate) panel_clip: crate::layout::PanelBox,
    pub(crate) text: Vec<crate::render::backend::PlacedText>,
    pub(crate) geometry: crate::render::geometry::GeometrySnapshot,
}

impl DeferredAnnotationPlan {
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
pub(crate) mod distributions;

#[cfg(test)]
pub(crate) mod estimates;

#[cfg(test)]
pub(crate) mod bars;

#[cfg(test)]
pub(crate) mod categorical_distributions;
