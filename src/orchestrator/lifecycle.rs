//! One-canvas chart lifecycle and immutable standalone scene planning.

use crate::Error;

use super::backend::{capture_text_replay_plan, validate_text_replay_plan, TextReplayPlan};
use crate::builder::Plot;
use crate::decorations::plan::PanelPlan;
use crate::layout::PanelBox;
use crate::render::backend::{Backend, SurfaceRole};
use crate::spec::{DecorationMode, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;

pub(super) struct PlotPlan {
    root: crate::layout::FrozenRoot<TextReplayPlan, PreparedPlotInputs>,
}

struct PreparedPlotInputs {
    axes: crate::layout::interior::PreparedInteriorInputs,
    theme: Box<ResolvedTheme>,
    text_census: crate::layout::PreparedTextCensus,
}

enum PlotLayoutPlan {
    Unfaceted(Box<UnfacetedPlotPlan>),
    Faceted(Box<super::facets::FacetPlan>),
}

#[cfg(all(test, unix, not(any(target_os = "macos", target_os = "android"))))]
mod soft_hyphen_tests {
    use super::*;

    #[test]
    fn production_title_census_prepares_soft_hyphen_for_unwrapped_and_wrapped_svg() {
        const SOURCE: &str = "A\u{00AD}AA";
        for (width, wrapped) in [(800, false), (450, true)] {
            let plot = Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0]),
            )
            .title(SOURCE)
            .x_label("")
            .y_label("")
            .x_ticks(vec![crate::layout::AxisTick::new(0.0, "A")])
            .y_ticks(vec![crate::layout::AxisTick::new(0.0, "A")])
            .size(width, 1000)
            .erase();
            let mut theme = crate::Theme::Publication.resolve();
            theme.typography.title.font_size = 240;
            theme.typography.title.weight = crate::TextWeight::Bold;
            assert_ne!(
                theme.typography.title.weight,
                theme.typography.tick_label.weight
            );
            let plan = PlotPlan::new_with_resolved_theme(&plot, PlotKind::Scatter, theme)
                .unwrap_or_else(|error| panic!("width {width}: {error}"));
            let prepared = plan.root.prepared();
            let captured = plan.root.plan().retained_text();
            let title_plan = captured
                .iter()
                .find(|(_, text)| text.source == SOURCE)
                .unwrap()
                .1;
            assert_eq!(title_plan.lines[0].selected_soft_hyphen.is_some(), wrapped);
            let census = &prepared.text_census;
            assert!(census
                .requirements
                .iter()
                .all(|requirement| requirement.generated_alphabet.is_empty()));
            let title = census
                .requirements
                .iter()
                .find(|requirement| requirement.text_role == crate::layout::PreparedTextRole::Title)
                .unwrap();
            assert_eq!(title.source, SOURCE);
            let pair = &census.pairs[title.pair_ordinal as usize];
            assert!(pair.required_scalars.contains(&'-'));
            assert!(census
                .pairs
                .iter()
                .filter(|pair| pair.pair_ordinal != title.pair_ordinal)
                .all(|pair| !pair.required_scalars.contains(&'-')));
            let environment = prepared.theme.prepared_font_environment().unwrap();
            assert!(environment.prepared_pairs_are_sealed().unwrap());
            let resolved = environment
                .prepared_pair(&pair.stack, crate::font::canonical::RequestedWeight::Bold)
                .unwrap()
                .unwrap();
            assert!(resolved.selected_scalars.iter().any(|selected| {
                selected.scalar == '-'
                    && matches!(
                        selected.selection,
                        crate::font::canonical::ScalarFaceSelection::Face(_)
                    )
            }));
            let mut backend = plan.take_svg_backend().unwrap();
            plan.render(&mut backend).unwrap();
            let svg = backend.into_bytes().unwrap();
            let xml = std::str::from_utf8(&svg).unwrap();
            let document = roxmltree::Document::parse(xml).unwrap();
            let titles = document
                .descendants()
                .filter(|node| {
                    node.has_tag_name("text") && node.attribute("font-size") == Some("240")
                })
                .collect::<Vec<_>>();
            let visible = titles
                .iter()
                .map(|title| {
                    title
                        .descendants()
                        .filter(roxmltree::Node::is_text)
                        .filter_map(|node| node.text())
                        .collect::<String>()
                })
                .collect::<Vec<_>>();
            if wrapped {
                assert_eq!(visible, ["A-", "AA"]);
                assert_eq!(titles[0].attribute("aria-label"), Some("A\u{00AD}"));
            } else {
                assert_eq!(visible, [SOURCE]);
            }
            let retained = titles
                .iter()
                .zip(&visible)
                .map(|(title, text)| title.attribute("aria-label").unwrap_or(text))
                .collect::<String>();
            assert_eq!(retained, SOURCE);
        }
    }
}

struct UnfacetedPlotPlan {
    plot: Plot,
    panel_plan: PanelPlan,
    canvas_layout: crate::layout::CanvasLayout,
}

impl PlotPlan {
    pub(super) fn new(source: &Plot, kind: PlotKind) -> Result<Self, Error> {
        Self::new_with_theme_factory(source, kind, || {
            Box::new(
                source
                    .theme_spec()
                    .unwrap_or(crate::Theme::Publication)
                    .resolve(),
            )
        })
    }

    #[cfg(test)]
    pub(super) fn new_with_resolved_theme(
        source: &Plot,
        kind: PlotKind,
        theme: ResolvedTheme,
    ) -> Result<Self, Error> {
        Self::new_with_theme_factory(source, kind, || Box::new(theme))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "ordered axis preparation, font preparation, measurement, and capture lifecycle"
    )]
    fn new_with_theme_factory(
        source: &Plot,
        kind: PlotKind,
        resolve_theme: impl FnOnce() -> Box<ResolvedTheme>,
    ) -> Result<Self, Error> {
        #[cfg(test)]
        super::PLOT_PLAN_CALLS.with(|calls| calls.set(calls.get() + 1));
        let mut children = crate::decorations::plan::children::ChildPlans::facets(source);
        let root = crate::layout::plan_exact_root(
            source.width(),
            source.height(),
            |prepared_stage| {
                let theme = resolve_theme();
                source.validate_configuration(kind)?;
                let axes = crate::plots::shared_x::prepare_plot(
                    source,
                    &prepared_stage.generation_resources(),
                )?;
                let requirements = crate::layout::foundational_plot_requirements(
                    Some(&axes),
                    source,
                    kind,
                    theme.as_ref(),
                    &crate::layout::ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(prepared_stage),
                )?;
                let text_topology =
                    crate::layout::PreparedTextCensus::structural_address_count(&requirements)
                        .map_err(|error| Error::Render(format!("prepared text census: {error}")))?;
                let mut scalar_presence = prepared_stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(prepared_stage),
                    Some(&mut scalar_presence),
                )
                .map_err(|error| Error::Render(format!("prepared text census: {error}")))?;
                if census.counts.structural_addresses != text_topology {
                    return Err(Error::Render(format!(
                        "prepared text topology counted {text_topology} addresses but materialized {}",
                        census.counts.structural_addresses
                    )));
                }
                theme.prepare_font_environment_from_census(
                    &mut census,
                    scalar_presence,
                    prepared_stage,
                )?;
                Ok(PreparedPlotInputs {
                    axes,
                    theme,
                    text_census: census,
                })
            },
            |prepared, assigned, round| {
                let (width, height) = assigned.dimensions();
                let source = prepared
                    .axes
                    .attach(source.clone())?
                    .with_canvas_size(width, height);
                let plan = if source.has_facets() {
                    PlotLayoutPlan::Faceted(Box::new(super::facets::FacetPlan::new(
                        &source,
                        kind,
                        &prepared.theme,
                        round,
                        &mut children,
                    )?))
                } else {
                    PlotLayoutPlan::Unfaceted(Box::new(UnfacetedPlotPlan::new(
                        &source,
                        kind,
                        &prepared.theme,
                        round,
                    )?))
                };
                children.finish_round(round);
                Ok(crate::layout::MeasuredLayout::new(assigned, plan))
            },
            |prepared, assigned, layout, final_stage| {
                let (width, height) = assigned.dimensions();
                let replay = capture_text_replay_plan(
                    width,
                    height,
                    "prepared plot scene capture",
                    &prepared.theme,
                    final_stage,
                    |backend| {
                        render_plot_scene(
                            source,
                            kind,
                            width,
                            height,
                            &prepared.theme,
                            &layout,
                            backend,
                        )
                    },
                )?;
                Ok(replay)
            },
        )?;
        let (width, height) = root.dimensions();
        validate_text_replay_plan(width, height, root.plan(), "prepared plot Null replay")?;
        Ok(Self { root })
    }

    pub(super) fn render(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        super::FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(calls.get() + 1));
        let _ = &self.root.prepared().text_census;
        let (width, height) = self.dimensions();
        self.root
            .plan()
            .replay(width, height, backend, "prepared plot scene replay")
    }

    pub(super) fn dimensions(&self) -> (u32, u32) {
        self.root.dimensions()
    }

    pub(super) fn take_svg_backend(&self) -> Result<crate::render::svg_backend::SvgBackend, Error> {
        self.root.plan().take_svg_backend()
    }
}

fn render_plot_scene(
    source: &Plot,
    kind: PlotKind,
    width: u32,
    height: u32,
    theme: &ResolvedTheme,
    layout: &PlotLayoutPlan,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    backend.draw_surface(
        PanelBox::new(0.0, f64::from(width), 0.0, f64::from(height)),
        theme.surfaces.canvas_root,
        SurfaceRole::CanvasRoot,
    )?;
    match layout {
        PlotLayoutPlan::Unfaceted(plan) => plan.render(source, kind, theme, backend),
        PlotLayoutPlan::Faceted(plan) => plan.render(source, kind, theme, backend),
    }
}

impl UnfacetedPlotPlan {
    fn new(
        source: &Plot,
        kind: PlotKind,
        theme: &ResolvedTheme,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<Self, Error> {
        let canvas_layout = source.measured_canvas_layout(theme)?;
        round.observe_canvas(&canvas_layout)?;
        let cell = canvas_layout.panel_box;
        let panel_plan = crate::layout::fit::measured(crate::decorations::plan::plan_panel(
            source,
            kind,
            cell,
            crate::decorations::plan::LegendRole::Panel,
            theme,
            round,
            &crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::PlotBody, 0)?,
        ))
        .map_err(crate::layout::fit::PlanError::into_error)?;
        let plot = panel_plan.plot_with_planned_limits(source);
        Ok(Self {
            plot,
            panel_plan,
            canvas_layout,
        })
    }

    fn render(
        &self,
        source: &Plot,
        kind: PlotKind,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        backend.draw_surface(
            self.panel_plan.panel_box,
            theme.surfaces.plot_panel,
            SurfaceRole::PlotPanel,
        )?;
        if self.panel_plan.result.decoration_mode == DecorationMode::Standard {
            crate::decorations::apply::apply_grid(source, &self.panel_plan, backend)?;
        }
        let source_rows = self.plot.configured_source_rows()?;
        let context = source_rows
            .map_or_else(
                || PlotRenderContext::new(self.plot.data(), self.panel_plan.panel_box, theme),
                |rows| {
                    PlotRenderContext::with_source_rows(
                        self.plot.data(),
                        rows,
                        self.panel_plan.panel_box,
                        theme,
                    )
                },
            )
            .with_retained_body(self.panel_plan.result.retained_payload());
        let rendered = self.plot.render_panel(kind, context, backend)?;

        crate::significance::render_selected(source, &self.panel_plan.result, theme, backend)?;

        match self.panel_plan.result.decoration_mode {
            DecorationMode::Standard => {
                crate::decorations::apply::apply_axes(source, &self.panel_plan, backend)?;
                crate::decorations::apply::apply_planned_legend(
                    source,
                    self.panel_plan.legend.as_ref(),
                    backend,
                )?;
            }
            DecorationMode::SelfManaged => {}
        }
        crate::decorations::apply::apply_canvas_text(source, &self.canvas_layout, backend)?;
        debug_assert_eq!(rendered, self.panel_plan.result.decoration_mode);
        Ok(())
    }
}

impl Plot {
    #[cfg(test)]
    pub(super) fn render_kind_erased(
        &self,
        kind: PlotKind,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        PlotPlan::new(self, kind)?.render(backend)
    }

    #[cfg(test)]
    pub(crate) fn preflight_panel(
        &self,
        kind: PlotKind,
        panel: crate::layout::PanelBox,
        theme: &ResolvedTheme,
    ) -> Result<PlotRenderResult, Error> {
        crate::layout::fit::measured(self.preflight_measurement(
            kind,
            panel,
            theme,
            &crate::layout::fit::ROOT_SCOPE,
        ))
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    pub(crate) fn preflight_measurement(
        &self,
        kind: PlotKind,
        panel: crate::layout::PanelBox,
        theme: &ResolvedTheme,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<crate::layout::fit::FitOutcome<PlotRenderResult>, Error> {
        self.validate_category_order_configuration(kind)?;
        if !self.significance_annotations().is_empty() && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}; use a supported categorical family or remove the annotations"
            )));
        }
        let source_rows = self.configured_source_rows()?;
        let context = source_rows.map_or_else(
            || PlotRenderContext::new(self.data(), panel, theme),
            |rows| PlotRenderContext::with_source_rows(self.data(), rows, panel, theme),
        );
        crate::layout::fit::outcome(self.measure_panel(kind, context.with_fit_scope(scope)))
    }

    pub(crate) fn configured_source_rows(&self) -> Result<Option<&[usize]>, Error> {
        if let Some(rows) = self.source_row_map() {
            if rows.len() != self.data().height() {
                return Err(Error::Data(format!(
                    "chart source-row map has length {}, but chart data has {} rows",
                    rows.len(),
                    self.data().height()
                )));
            }
        }
        Ok(self.source_row_map())
    }
}

#[cfg(test)]
mod root_title_tests {
    use super::*;
    use crate::theme::TitleAlignment::{Center, End, Start};

    #[test]
    fn matrix_root_uses_one_title_producer_with_independent_odd_size_alignments() {
        for preset in [crate::Theme::Publication, crate::Theme::Science] {
            for explicit in [false, true] {
                let mut plot = Plot::matrix(vec![
                    crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
                    crate::NumericValues::new("y", [2.0, 1.0, 3.0, 4.0]),
                ])
                .title("Title\nx")
                .subtitle("Subtitle\ny")
                .caption("Caption\nz")
                .size(533, 681)
                .erase();
                if explicit {
                    plot.title_alignment = Some(End);
                    plot.subtitle_alignment = Some(Start);
                }
                let theme = preset.resolve();
                assert_eq!(theme.composition.root_title_alignment, Center);
                assert_eq!(theme.composition.root_subtitle_alignment, None);
                assert_eq!(theme.composition.root_caption_alignment, Start);
                let canvas = plot.measured_canvas_layout(&theme).unwrap();
                assert!(matches!(
                    canvas.root_text,
                    crate::layout::canvas::CanvasRootText::Planned(_)
                ));
                assert!(
                    canvas.title_y.is_none()
                        && canvas.subtitle_y.is_none()
                        && canvas.caption_y.is_none()
                );
                let plan =
                    PlotPlan::new_with_resolved_theme(&plot, PlotKind::Matrix, theme).unwrap();
                let retained = plan.root.plan().retained_text();
                for (source, alignment) in [
                    (
                        "Title\nx",
                        if explicit {
                            crate::layout::PhysicalTextAlignment::End
                        } else {
                            crate::layout::PhysicalTextAlignment::Center
                        },
                    ),
                    (
                        "Subtitle\ny",
                        if explicit {
                            crate::layout::PhysicalTextAlignment::Start
                        } else {
                            crate::layout::PhysicalTextAlignment::Center
                        },
                    ),
                    ("Caption\nz", crate::layout::PhysicalTextAlignment::Start),
                ] {
                    let matching = retained
                        .iter()
                        .filter(|(_, text)| text.source == source)
                        .collect::<Vec<_>>();
                    assert_eq!(matching.len(), 1);
                    assert_eq!(matching[0].1.style.alignment, alignment);
                    assert_eq!(matching[0].1.lines.len(), 2);
                }
                let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
                let mut backend = plan.take_svg_backend().unwrap();
                plan.render(&mut backend).unwrap();
                let svg = backend.into_bytes().unwrap();
                let document =
                    roxmltree::Document::parse(std::str::from_utf8(&svg).unwrap()).unwrap();
                assert_eq!(
                    document.root_element().attribute("viewBox"),
                    Some("0 0 533 681")
                );
                assert_eq!(
                    document
                        .descendants()
                        .filter(|node| node.has_tag_name("clipPath"))
                        .count(),
                    4
                );
                assert_eq!(
                    shapes,
                    crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get)
                );
            }
        }
    }

    #[test]
    fn standalone_root_freezes_independent_alignment_before_null_and_svg() {
        for preset in [
            crate::Theme::Publication,
            crate::Theme::Nature,
            crate::Theme::Science,
            crate::Theme::CHI,
        ] {
            let mut plot = Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0]),
            )
            .title("Title\nx")
            .subtitle("Subtitle\ny")
            .caption("Caption\nz")
            .size(333, 481)
            .theme(preset)
            .erase();
            plot.title_alignment = Some(End);
            plot.subtitle_alignment = Some(Start);
            let mut theme = preset.resolve();
            theme.composition.root_caption_alignment = Center;
            let plan = PlotPlan::new_with_resolved_theme(&plot, PlotKind::Scatter, theme).unwrap();
            let retained = plan.root.plan().retained_text();
            for (source, alignment) in [
                ("Title\nx", crate::layout::PhysicalTextAlignment::End),
                ("Subtitle\ny", crate::layout::PhysicalTextAlignment::Start),
                ("Caption\nz", crate::layout::PhysicalTextAlignment::Center),
            ] {
                let roles = retained
                    .iter()
                    .filter(|(_, text)| text.source == source)
                    .collect::<Vec<_>>();
                assert_eq!(roles.len(), 1);
                let (placement, text) = roles[0];
                assert_eq!(text.style.alignment, alignment);
                assert_eq!(text.lines.len(), 2);
                let canonical =
                    crate::render::backend::validate_planned_text(*placement, text).unwrap();
                assert!((canonical.position.x.value() - placement.position.x).abs() < 0.000_001);
                assert!((canonical.position.y.value() - placement.position.y).abs() < 0.000_001);
            }
            let mut backend = plan.take_svg_backend().unwrap();
            plan.render(&mut backend).unwrap();
            let svg = backend.into_bytes().unwrap();
            let xml = std::str::from_utf8(&svg).unwrap();
            let document = roxmltree::Document::parse(xml).unwrap();
            for source in ["Title", "x", "Subtitle", "y", "Caption", "z"] {
                assert!(document
                    .descendants()
                    .any(|node| node.has_tag_name("text") && node.text() == Some(source)));
            }
        }
    }
}
