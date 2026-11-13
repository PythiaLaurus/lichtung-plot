//! Operation-local measurement state, borrowed for one measurement round.

use crate::decorations::plan::children::ChildPlans;

#[derive(Clone, Copy)]
pub(crate) struct GridSlot(pub(crate) usize);

/// One occurrence's post-font analysis. Native readers retain their original
/// lazy initialization; scientific axes own no measurement state.
#[derive(Debug)]
pub(crate) enum PlotIntrinsics {
    None,
    Coordinate {
        value: std::sync::OnceLock<crate::layout::interior::coordinate::Intrinsics>,
    },
    Heatmap {
        value: std::sync::OnceLock<crate::plots::heatmap::HeatmapIntrinsicCache>,
    },
}

impl PlotIntrinsics {
    pub(crate) const fn for_kind(kind: crate::PlotKind) -> Self {
        match kind {
            crate::PlotKind::Matrix => Self::None,
            crate::PlotKind::Heatmap => Self::Heatmap {
                value: std::sync::OnceLock::new(),
            },
            _ => Self::Coordinate {
                value: std::sync::OnceLock::new(),
            },
        }
    }
}

pub(crate) struct PlotIntrinsicInputs<'v, 'src> {
    pub(crate) registered: &'v crate::orchestrator::registration::PlotRegistration<'src>,
    pub(crate) request: crate::text::TextRequestContext<'v, 'src>,
    pub(crate) operation: &'v crate::font::canonical::OperationContext,
    pub(crate) y_visible: bool,
    pub(crate) diagnostic_scope: Option<&'v str>,
}

pub(crate) fn coordinate_intrinsic<'a>(
    slot: &'a PlotIntrinsics,
    input: &PlotIntrinsicInputs<'_, '_>,
) -> Result<&'a crate::layout::interior::coordinate::Intrinsics, crate::Error> {
    crate::layout::interior::coordinate::Intrinsics::prepare(slot, input)
}

pub(crate) fn prepare_plot_intrinsics(
    slot: &PlotIntrinsics,
    input: &PlotIntrinsicInputs<'_, '_>,
) -> Result<(), crate::Error> {
    let kind = input.registered.source.kind();
    if crate::layout::interior::InteriorRoute::classify(kind).is_coordinate() {
        coordinate_intrinsic(slot, input)?;
    } else if kind == crate::PlotKind::Heatmap {
        if !matches!(
            input.registered.family,
            crate::orchestrator::registration::PreparedFamily::Heatmap(_)
        ) {
            return Err(crate::Error::Render(
                "prepared heatmap intrinsic scope missing".into(),
            ));
        }
        crate::plots::heatmap::heatmap_minima(slot, input)?;
    }
    Ok(())
}

impl crate::Plot {
    pub(crate) fn intrinsic_inputs<'v, 'src>(
        &'v self,
        registered: Option<&'v crate::orchestrator::registration::PlotRegistration<'src>>,
        request: &crate::text::TextRequestContext<'v, 'src>,
        operation: &'v crate::font::canonical::OperationContext,
    ) -> Result<PlotIntrinsicInputs<'v, 'src>, crate::Error> {
        Ok(PlotIntrinsicInputs {
            registered: registered.ok_or_else(|| {
                crate::Error::Render("native coordinate panel requires root-prepared axes".into())
            })?,
            request: *request,
            operation,
            y_visible: self.y_tick_labels_visible(),
            diagnostic_scope: self.axis_scope(),
        })
    }
}

pub(crate) struct RootLayoutCaches {
    pub(crate) children: ChildPlans,
    pub(crate) heights: crate::canvas::HeightMeasurements,
    pub(crate) frames: Option<CachedRootFrame>,
}

pub(crate) struct CachedRootFrame {
    pub(crate) prose: [crate::Text; 3],
    pub(crate) dimensions: (u32, u32),
    pub(crate) minimum: [u64; 2],
    pub(crate) frame: std::sync::Arc<crate::layout::RootFramePlan>,
}

impl RootLayoutCaches {
    pub(crate) fn new(children: ChildPlans) -> Self {
        Self {
            children,
            heights: crate::canvas::HeightMeasurements::default(),
            frames: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CategoryValues, NumericValues, Plot, PlotKind};

    #[test]
    fn native_intrinsic_slots_wait_for_the_original_hard_minimum_reader() {
        let categories = CategoryValues::new("group", ["a", "b", "c"]).unwrap();
        let x = NumericValues::new("x", [1.0, 2.0, 3.0]);
        let y = NumericValues::new("y", [2.0, 4.0, 3.0]);
        let sources = [
            Plot::beeswarm(categories.clone(), x.clone(), y.clone()).erase(),
            Plot::waterfall(categories, x.clone(), 0.0).erase(),
            Plot::marginal(x, y).erase(),
        ];
        let appearance = crate::Theme::publication().resolve().unwrap();
        let text = crate::text::TextService::default();
        let request = crate::text::TextRequestContext::new(&appearance, &text);
        for source in sources {
            crate::orchestrator::with_prepared_plot_for_test(&source, &request, |plot, inputs, request, operation| {
                let registered = &inputs.plots[0];
                let intrinsic = inputs.intrinsic(registered);
                let PlotIntrinsics::Coordinate { value } = intrinsic else {
                    panic!("native coordinate slot")
                };
                assert!(value.get().is_none(), "post-font preparation must remain lazy");
                let plot = plot.clone().with_y_tick_labels_visible(plot.kind() != PlotKind::Beeswarm);
                plot.intrinsic_body_minimum(Some(registered), intrinsic, operation, request).unwrap();
                let initialized = value.get().unwrap();
                let same = coordinate_intrinsic(intrinsic, &plot.intrinsic_inputs(Some(registered), request, operation).unwrap()).unwrap();
                assert!(std::ptr::eq(initialized, same));
                if plot.kind() == PlotKind::Beeswarm {
                    let visible = plot.with_y_tick_labels_visible(true);
                    let error = coordinate_intrinsic(intrinsic, &visible.intrinsic_inputs(Some(registered), request, operation).unwrap()).unwrap_err();
                    assert_eq!(error.to_string(), "render error: previously suppressed axis text cannot become active in prepared intrinsics");
                }
            }).unwrap();
        }
    }

    #[test]
    fn coordinate_intrinsic_mismatch_remains_fatal_without_reanalysis() {
        let source = Plot::scatter(
            NumericValues::new("x", [0.0, 1.0]),
            NumericValues::new("y", [1.0, 2.0]),
        )
        .erase();
        let appearance = crate::Theme::publication().resolve().unwrap();
        let text = crate::text::TextService::default();
        let request = crate::text::TextRequestContext::new(&appearance, &text);
        crate::orchestrator::with_prepared_plot_for_test(
            &source,
            &request,
            |plot, inputs, request, operation| {
                let intrinsic = inputs.intrinsic(&inputs.plots[0]);
                let PlotIntrinsics::Coordinate { value } = intrinsic else {
                    panic!("coordinate slot")
                };
                assert!(value.get().is_some());
                let mut changed = request.theme.clone();
                changed.typography.axes.tick_label.font_size += 1.0;
                let changed = crate::text::TextRequestContext::new(&changed, request.text);
                crate::orchestrator::measurements::start();
                let error = coordinate_intrinsic(
                    intrinsic,
                    &plot
                        .intrinsic_inputs(inputs.plots.first(), &changed, operation)
                        .unwrap(),
                )
                .unwrap_err();
                let measured = crate::orchestrator::measurements::finish();
                assert_eq!(
                    error.to_string(),
                    "render error: prepared panel intrinsic typography changed"
                );
                assert_eq!(
                    measured.counters
                        [crate::orchestrator::measurements::Counter::StandardPanelIntrinsic
                            as usize],
                    0
                );
            },
        )
        .unwrap();
    }
}
