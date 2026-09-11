//! Closed interior boundary. All current bodies are combined legacy adapters,
//! not native producers or extracted mark payloads. Extraction replaces variants
//! here rather than installing a fallback alongside the current route.

use crate::layout::kernel::{FeedbackKind, FeedbackRequestId, FeedbackRound};
use crate::layout::ComponentAddress;
use crate::spec::{PlotKind, PlotRenderContext, PlotRenderResult};
use crate::{Error, Plot};

macro_rules! interior_routes {
    ($($class:ident { $($kind:ident => $renderer:path),+ $(,)? }),+ $(,)?) => {
        $(#[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum $class { $($kind),+ })+

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[expect(clippy::enum_variant_names, reason = "every currently admitted route is explicitly legacy; native variants arrive only with extraction")]
        pub(crate) enum InteriorRoute { $($class($class)),+ }

        impl InteriorRoute {
            pub(crate) const fn classify(kind: PlotKind) -> Self {
                match kind { $($(PlotKind::$kind => Self::$class($class::$kind),)+)+ }
            }

            pub(crate) const fn kind(self) -> PlotKind {
                match self { $($(Self::$class($class::$kind) => PlotKind::$kind,)+)+ }
            }

            /// Transitional combined execution, deliberately requiring a backend.
            /// This is not the future backend-free prepared producer interface.
            pub(crate) fn combined(
                self,
                plot: &Plot,
                context: PlotRenderContext<'_>,
                backend: &mut dyn crate::render::backend::Backend,
            ) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
                self.validate(plot)?;
                match self { $($(Self::$class($class::$kind) => $renderer(plot, context, backend),)+)+ }
            }
        }
    };
}

interior_routes! {
    StandardLegacy {
        Scatter => crate::plots::scatter::render,
        Histogram => crate::plots::histogram::render,
        Density => crate::plots::density::render,
        Qq => crate::plots::qq::render,
        Regression => crate::plots::regression::render,
        Line => crate::plots::line::render,
        Trajectory => crate::plots::line::render_trajectory,
        LinePath => crate::plots::line::render_path,
    },
    CategoricalLegacy {
        Interaction => crate::plots::interaction::render,
        PointInterval => crate::plots::point_interval::render,
        Coefficient => crate::plots::coefficient::render,
        Forest => crate::plots::forest::render,
        Dumbbell => crate::plots::dumbbell::render,
        Bar => crate::plots::bar::render,
        StackedBar => crate::plots::bar::render,
        HorizontalBar => crate::plots::horizontal_bar::render,
        StackedHorizontalBar => crate::plots::horizontal_bar::render,
        Boxplot => crate::plots::boxplot::render,
        Violin => crate::plots::violin::render,
        Raincloud => crate::plots::violin::render_raincloud,
    },
    SpecializedLegacy {
        Heatmap => crate::plots::heatmap::render,
        Marginal => crate::plots::marginal::render,
        Matrix => crate::plots::matrix::render,
        Beeswarm => crate::plots::beeswarm::render,
        Waterfall => crate::plots::waterfall::render,
    },
}

impl InteriorRoute {
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

/// Font-independent scientific axis inputs retained by T5a5. Family statistics
/// and marks are still combined; these inputs must not be called fully prepared
/// family payloads. Categories (including categorical Trajectory) live in axes,
/// not in an inferred numeric-only classification.
pub(crate) struct PreparedInteriorInputs {
    pub(crate) route: InteriorRoute,
    pub(crate) axes: std::sync::Arc<Vec<(Option<String>, crate::plots::shared_x::PreparedAxes)>>,
    pub(crate) matrix: Option<std::sync::Arc<crate::plots::matrix::PreparedMatrixAxes>>,
}

impl PreparedInteriorInputs {
    pub(crate) fn attach(&self, mut plot: Plot) -> Result<Plot, Error> {
        self.route.validate(&plot)?;
        plot.prepared_axes = self.axes.clone();
        plot.prepared_matrix_axes.clone_from(&self.matrix);
        Ok(plot)
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

/// Closed retained-result contract consumed by capture. This variant retains
/// T5a5 axes and T5a3 text, NOT scientific mark geometry. Extraction must replace
/// it with selected DeferredMarkPlan/DeferredAnnotationPlan payloads whose
/// intrinsics, domains, reserves and obstacle/outward ink come from those same
/// payloads. They may not measure via a backend or reconstruct during capture.
#[derive(Clone, Copy)]
pub(crate) enum RetainedInteriorPayload<'a> {
    LegacyCombined(&'a PlotRenderResult),
}

#[cfg(test)]
mod tests;
