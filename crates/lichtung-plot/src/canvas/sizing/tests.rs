use super::*;
use crate::decorations::legend::discrete::scoped::{DependencyUse, GuideAllocation};
use crate::numeric::{decimal::NegativeZero, format::NumericFormat};
use crate::Plot;
use std::sync::Arc;

fn prepare_height_inputs<'src>(
    canvas: &'src Canvas,
    stage: &mut crate::layout::kernel::PreparedStage<'_>,
) -> Result<Vec<PlotRegistration<'src>>, Error> {
    let (_, appearances) = super::super::resolve_appearances(
        &canvas.theme,
        &canvas.placements,
        &canvas.guide_text_layouts,
    )?;
    let (mut inputs, _) = canvas.prepare_interiors(stage, appearances)?;
    let source = canvas.placements.plot(0).expect("configured Canvas plot");
    crate::layout::interior::guide::prepare_discrete_guides(
        source,
        &mut inputs.panels,
        &stage.operation(),
        false,
    )?;
    inputs.finish()
}

fn axis_operands(
    registered: &PlotRegistration<'_>,
) -> ([(f64, f64); 2], [crate::layout::RangeProvenance; 2]) {
    let axes = [
        registered.source.x_axis_spec(),
        registered.source.y_axis_spec(),
    ];
    let ranges = std::array::from_fn(|side| {
        axes[side].window.map_or(
            registered.axes.ranges()[side],
            crate::layout::RawWindow::pair,
        )
    });
    let provenance = axes.map(|axis| {
        if axis.window.is_some() {
            crate::layout::RangeProvenance::Configured
        } else {
            crate::layout::RangeProvenance::Automatic
        }
    });
    (ranges, provenance)
}

#[expect(
    clippy::too_many_arguments,
    reason = "adapt the original fixture to the real scalar record operands"
)]
fn height_inputs(
    source: &Plot,
    registered: Option<&PlotRegistration<'_>>,
    canvas: &Canvas,
    width: f64,
    insets: [i64; 4],
    round: &FeedbackRound,
    owner: &ComponentAddress,
    per_plot: bool,
    dependencies: &[ConsumedGuideDependency],
) -> Result<HeightInputs, Error> {
    let registered = registered.unwrap();
    assert_eq!(source.kind(), registered.kind());
    let (ranges, provenance) = axis_operands(registered);
    HeightInputs::new(
        registered,
        Some(canvas),
        None,
        ranges,
        provenance,
        None,
        width,
        insets,
        round,
        owner,
        per_plot,
        dependencies,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the fixture calls the real record, including its own key construction"
)]
fn record_height(
    measurements: &mut HeightMeasurements,
    source: &Plot,
    registered: Option<&PlotRegistration<'_>>,
    canvas: &Canvas,
    width: f64,
    height: f64,
    insets: [i64; 4],
    round: &FeedbackRound,
    owner: &ComponentAddress,
    dependencies: &[ConsumedGuideDependency],
) -> Result<(), Error> {
    let registered = registered.unwrap();
    assert_eq!(source.kind(), registered.kind());
    let (ranges, provenance) = axis_operands(registered);
    measurements.record(
        registered,
        Some(canvas),
        None,
        ranges,
        provenance,
        None,
        width,
        height,
        insets,
        round,
        owner,
        dependencies,
    )
}

#[test]
fn natural_columns_do_not_require_a_successful_uncapped_text_height_layout() {
    let source = Canvas::grid(1, 1)
        .unwrap()
        .text_panel(
            0,
            0,
            crate::TextPanel::new(
                "Width owner",
                crate::Text::new("Complete source with no permitted height").layout(
                    crate::TextLayout::default()
                        .bounds(crate::TextBounds::new(None, Some(0.0), None).unwrap()),
                ),
            )
            .unwrap(),
        )
        .unwrap();
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 800,
            height: 600,
        },
        |stage| {
            Root::prepare(
                crate::orchestrator::root::RootSource::Canvas(&source),
                stage,
            )
        },
        |prepared, assigned, round| {
            let request = prepared.request();
            let content = super::super::tracks::ContentSizing {
                source: &source,
                root: prepared,
                output_dimensions: None,
                initial_sizing: true,
                round,
                measurements: None,
                shared_allocation: None,
                natural_width: true,
                natural_height: true,
            };
            let width = fit::measured(super::super::tracks::natural_width(
                &source.placements,
                &[],
                &[],
                &[],
                &request,
                &FitScope::root(),
                &content,
            ))
            .map_err(fit::PlanError::into_error)?;
            assert!(width > 0.0);
            let full = super::super::tracks::allocate(
                PixelRect::new(0.0, 0.0, 0.0, 0.0),
                &source.placements,
                prepared,
                Some(round),
                &[],
                &[],
                &[],
                &[],
                &request,
                &FitScope::root(),
                Some(&content),
            );
            assert!(full.is_err() || matches!(full, Ok(fit::FitOutcome::Miss(_))));
            Ok(crate::layout::MeasuredLayout::new(
                assigned.assigned_size()?,
                (),
            ))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
}

#[expect(
    clippy::too_many_lines,
    reason = "one cache contract compares complete scalar assignment inputs and declared guide dependencies"
)]
fn assert_changed_height_inputs(
    source: &Plot,
    variants: impl IntoIterator<Item = Plot>,
    include_spanning: bool,
) {
    let mut canvas = Canvas::grid(2, 2).unwrap();
    Arc::make_mut(&mut canvas.placements)
        .insert(
            super::super::placement::Footprint::cell(0, 0),
            super::super::placement::Payload::Plot(source.clone().into_description()),
        )
        .unwrap();
    let spanning = include_spanning.then(|| {
        let mut spanning = canvas.clone();
        Arc::make_mut(&mut spanning.placements)
            .replace(
                super::super::placement::Footprint {
                    row: 0,
                    column: 0,
                    row_span: 2,
                    column_span: 1,
                },
                super::super::placement::Payload::Plot(source.clone().into_description()),
            )
            .unwrap();
        spanning
    });
    let variants = variants
        .into_iter()
        .map(|plot| {
            let mut changed = canvas.clone();
            Arc::make_mut(&mut changed.placements)
                .replace(
                    super::super::placement::Footprint::cell(0, 0),
                    super::super::placement::Payload::Plot(plot.into_description()),
                )
                .unwrap();
            changed
        })
        .collect::<Vec<_>>();
    let mut measurements = HeightMeasurements::default();
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 800,
            height: 600,
        },
        |stage| {
            let inputs = prepare_height_inputs(&canvas, stage)?;
            let variants = crate::data::source::collect_source(
                variants
                    .iter()
                    .map(|canvas| prepare_height_inputs(canvas, stage)),
            )?;
            Ok((inputs, variants))
        },
        |(inputs, variant_inputs), assigned, round| {
            inputs[0]
                .scope
                .render_name
                .as_deref()
                .expect("Canvas scope");
            let source = source.clone();
            let owner = ComponentAddress::root().child(AddressRole::CanvasCell, 0)?;
            let ancestor = ComponentAddress::root().child(AddressRole::CanvasSharedLegend, 0)?;
            let dependencies = [ConsumedGuideDependency::new(
                ancestor.clone(),
                DependencyUse::AllocationOnly,
                GuideAllocation::width(400.0)?,
                round,
            )?];
            record_height(
                &mut measurements,
                &source,
                inputs.first(),
                &canvas,
                400.0,
                120.0,
                [0; 4],
                round,
                &owner,
                &dependencies,
            )?;
            let same = height_inputs(
                &source,
                inputs.first(),
                &canvas,
                400.0,
                [0; 4],
                round,
                &owner,
                true,
                &dependencies,
            )?;
            assert_eq!(measurements.measured(&same, &owner, round)?, Some(120.0));
            let undeclared = height_inputs(
                &source,
                inputs.first(),
                &canvas,
                400.0,
                [0; 4],
                round,
                &owner,
                true,
                &[],
            )?;
            assert_eq!(measurements.measured(&undeclared, &owner, round)?, None);
            let narrower = [ConsumedGuideDependency::new(
                ancestor,
                DependencyUse::AllocationOnly,
                GuideAllocation::width(400.0 - 1.0 / 1024.0)?,
                round,
            )?];
            let different = height_inputs(
                &source,
                inputs.first(),
                &canvas,
                400.0,
                [0; 4],
                round,
                &owner,
                true,
                &narrower,
            )?;
            assert_eq!(same.width, different.width);
            assert_eq!(measurements.measured(&different, &owner, round)?, None);
            for (changed, changed_inputs) in variants.iter().zip(variant_inputs) {
                let changed_source = changed.placements.plot(0).expect("changed Canvas plot");
                let (ranges, provenance) = axis_operands(&changed_inputs[0]);
                assert_eq!(changed_source.kind(), changed_inputs[0].kind());
                let different = HeightInputs::new(
                    &changed_inputs[0],
                    Some(&canvas),
                    None,
                    ranges,
                    provenance,
                    None,
                    400.0,
                    [0; 4],
                    round,
                    &owner,
                    true,
                    &dependencies,
                )?;
                assert_eq!(same.width, different.width);
                assert_eq!(same.ranges, different.ranges);
                assert_eq!(measurements.measured(&different, &owner, round)?, None);
                measurements.record(
                    &changed_inputs[0],
                    Some(&canvas),
                    None,
                    ranges,
                    provenance,
                    None,
                    400.0,
                    120.0,
                    [0; 4],
                    round,
                    &owner,
                    &dependencies,
                )?;
                assert_eq!(
                    measurements.measured(&different, &owner, round)?,
                    Some(120.0)
                );
                assert_eq!(measurements.measured(&same, &owner, round)?, None);
            }
            if let Some(spanning) = &spanning {
                record_height(
                    &mut measurements,
                    &source,
                    inputs.first(),
                    &canvas,
                    400.0,
                    120.0,
                    [0; 4],
                    round,
                    &owner,
                    &dependencies,
                )?;
                let different = height_inputs(
                    &source,
                    inputs.first(),
                    spanning,
                    400.0,
                    [0; 4],
                    round,
                    &owner,
                    true,
                    &dependencies,
                )?;
                assert_eq!(same.width, different.width);
                assert_eq!(same.ranges, different.ranges);
                assert_eq!(measurements.measured(&different, &owner, round)?, None);
                record_height(
                    &mut measurements,
                    &source,
                    inputs.first(),
                    spanning,
                    400.0,
                    120.0,
                    [0; 4],
                    round,
                    &owner,
                    &dependencies,
                )?;
                assert_eq!(
                    measurements.measured(&different, &owner, round)?,
                    Some(120.0)
                );
                assert_eq!(measurements.measured(&same, &owner, round)?, None);
            }
            Ok(crate::layout::MeasuredLayout::new(
                assigned.assigned_size()?,
                (),
            ))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
}

#[test]
fn recorded_heights_require_complete_text_requests_even_when_geometry_can_match() {
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 0.0]),
    )
    .title("Heading")
    .x_label("Horizontal")
    .erase();
    let exact = crate::Theme::publication()
        .typography()
        .get_root()
        .get_title()
        .clone();
    let variants = [
        source
            .clone()
            .title(crate::Text::new("Heading").style(exact)),
        source
            .clone()
            .title(crate::Text::new("Heading").layout(crate::TextLayout::default())),
        source
            .clone()
            .x_label(crate::Text::new("Horizontal").layout(crate::TextLayout::default())),
    ];
    assert_changed_height_inputs(&source, variants, false);
}

#[test]
fn recorded_heights_require_complete_numeric_and_guide_inputs_at_equal_width_and_domain() {
    let format = NumericFormat::default();
    let make = |guide, values| {
        Plot::heatmap(
            crate::CategoryValues::new("x", ["a"]).unwrap(),
            crate::CategoryValues::new("y", ["one"]).unwrap(),
            crate::NumericValues::new("value", [1.0]),
        )
        .color_label_format(guide)
        .color_label("Value")
        .annotations(crate::HeatmapAnnotations::Values(values))
    };
    let normalized = format
        .clone()
        .negative_zero(NegativeZero::Normalize)
        .unwrap();
    assert_changed_height_inputs(
        &make(format.clone(), format.clone()).erase(),
        vec![
            make(format.clone(), format.clone())
                .x_category_labels(crate::CategoryLabelSelection::every(2).unwrap())
                .erase(),
            make(format.clone(), format.clone())
                .y_category_labels(crate::CategoryLabelSelection::every(2).unwrap())
                .erase(),
            make(normalized.clone(), format.clone()).erase(),
            make(format.clone(), normalized).erase(),
            make(format.clone(), format)
                .color_label(crate::Text::new("Value").layout(crate::TextLayout::default()))
                .erase(),
        ],
        true,
    );

    let source = crate::Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .x_limits(0.0, 1.0)
    .y_limits(0.0, 1.0)
    .x_tick_generation(crate::TickGeneration::target(5).unwrap());
    let variants = [
        source
            .clone()
            .x_tick_generation(crate::TickGeneration::maximum(5).unwrap())
            .erase(),
        source
            .clone()
            .x_tick_generation(crate::TickGeneration::target(4).unwrap())
            .erase(),
        source
            .clone()
            .x_tick_generation(crate::TickGeneration::automatic())
            .erase(),
        source
            .clone()
            .y_tick_generation(crate::TickGeneration::maximum(5).unwrap())
            .erase(),
    ];
    assert_changed_height_inputs(&source.erase(), variants, false);

    let source = crate::Plot::line(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .with_alpha(
        crate::alpha::AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("confidence", [0, 127]).unwrap(),
        ),
    );
    let variants = [
        source
            .clone()
            .with_alpha(
                crate::alpha::AlphaTarget::Point,
                crate::alpha::AlphaEncoding::observations(
                    crate::alpha::AlphaValues::new("confidence", [0, 128]).unwrap(),
                ),
            )
            .erase(),
        source
            .clone()
            .with_alpha(
                crate::alpha::AlphaTarget::Line,
                crate::alpha::AlphaEncoding::single(
                    crate::alpha::AlphaValues::new("confidence", [127]).unwrap(),
                )
                .unwrap(),
            )
            .erase(),
        source
            .clone()
            .with_alpha(
                crate::alpha::AlphaTarget::Fill,
                crate::alpha::AlphaEncoding::single(
                    crate::alpha::AlphaValues::new("confidence", [127]).unwrap(),
                )
                .unwrap(),
            )
            .erase(),
    ];
    assert_changed_height_inputs(&source.erase(), variants, false);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one real cell-height call witnesses lookup, invalidation and all three preceding fallible stages"
)]
fn cell_height_hits_still_run_hard_preference_and_prose_before_key_lookup() {
    let _trace = HeightTraceGuard::start();
    let scatter = || {
        Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 0.0]),
        )
    };
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, scatter().title("Local heading"))
        .unwrap();
    let changed_source = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, scatter().title("Different complete prose"))
        .unwrap();
    let shared = canvas.clone().x_range_group([(0, 0)]).unwrap();
    let mut measured = HeightMeasurements::default();
    let mut rounds = 0;
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 800,
            height: 600,
        },
        |stage| {
            Ok((
                Root::prepare(
                    crate::orchestrator::root::RootSource::Canvas(&canvas),
                    stage,
                )?,
                Root::prepare(
                    crate::orchestrator::root::RootSource::Canvas(&changed_source),
                    stage,
                )?,
                Root::prepare(
                    crate::orchestrator::root::RootSource::Canvas(&shared),
                    stage,
                )?,
            ))
        },
        |(root, changed_root, shared_root), assigned, round| {
            rounds += 1;
            let registered = root.canvas_plot(0).unwrap();
            let owner = ComponentAddress::root()
                .child(AddressRole::CanvasGrid, 0)?
                .child(AddressRole::CanvasCell, 0)?;
            let axes = axis_inputs(root, registered, round, true);
            let request = crate::text::TextRequestContext::new(&registered.appearance, &root.text);
            let call = |input: &Root<'_>,
                        canvas: &Canvas,
                        request: &crate::text::TextRequestContext<'_, '_>,
                        measured: &HeightMeasurements,
                        insets,
                        dependencies: &[ConsumedGuideDependency]| {
                CELL_HEIGHT_STEPS.with(|steps| steps.borrow_mut().clear());
                let registered = input.canvas_plot(0).unwrap();
                cell_height(
                    registered,
                    &input.operation,
                    canvas,
                    request,
                    400.0,
                    input.plot_intrinsic(&registered.scope.id)?,
                    None,
                    axis_inputs(input, registered, round, true),
                    true,
                    round,
                    &owner,
                    insets,
                    Some(measured),
                    dependencies,
                )
            };
            let steps = || CELL_HEIGHT_STEPS.with(|steps| steps.borrow().clone());
            let hit = ["hard", "preference", "prose", "key", "lookup", "hit"];
            let miss = ["hard", "preference", "prose", "key", "lookup", "computed"];
            if rounds == 1 {
                measured.record(
                    registered,
                    Some(&canvas),
                    None,
                    axes.0,
                    axes.1,
                    axes.2,
                    400.0,
                    123.25,
                    [0; 4],
                    round,
                    &owner,
                    &[],
                )?;
                assert_eq!(
                    call(root, &canvas, &request, &measured, [0; 4], &[])?.to_bits(),
                    123.25_f64.to_bits()
                );
                assert_eq!(steps(), hit);
                call(root, &canvas, &request, &measured, [1, 0, 0, 0], &[])?;
                assert_eq!(steps(), miss);

                // Real preparations share this operation/occurrence identity:
                // neither rejection can hide behind a different root token.
                let changed = changed_root.canvas_plot(0).unwrap();
                assert_eq!(registered.scope.id, changed.scope.id);
                call(
                    changed_root,
                    &changed_source,
                    &crate::text::TextRequestContext::new(&changed.appearance, &changed_root.text),
                    &measured,
                    [0; 4],
                    &[],
                )?;
                assert_eq!(steps(), miss);

                let shared_plot = shared_root.canvas_plot(0).unwrap();
                assert_eq!(registered.scope.id, shared_plot.scope.id);
                let shared_axes = axis_inputs(shared_root, shared_plot, round, true);
                assert_eq!(axes.0, shared_axes.0);
                assert_eq!(axes.1[0], crate::layout::RangeProvenance::Automatic);
                assert_eq!(shared_axes.1[0], crate::layout::RangeProvenance::Shared);
                let shared_request = crate::text::TextRequestContext::new(
                    &shared_plot.appearance,
                    &shared_root.text,
                );
                call(
                    shared_root,
                    &shared,
                    &shared_request,
                    &measured,
                    [0; 4],
                    &[],
                )?;
                assert_eq!(steps(), miss);

                let dependencies = shared.consumed_guide_dependencies(0, 400.0, None, round)?;
                assert_eq!(dependencies.len(), 1);
                let mut shared_heights = HeightMeasurements::default();
                shared_heights.record(
                    shared_plot,
                    Some(&shared),
                    None,
                    shared_axes.0,
                    shared_axes.1,
                    shared_axes.2,
                    400.0,
                    345.5,
                    [0; 4],
                    round,
                    &owner,
                    &dependencies,
                )?;
                assert_eq!(
                    call(
                        shared_root,
                        &shared,
                        &shared_request,
                        &shared_heights,
                        [0; 4],
                        &dependencies
                    )?
                    .to_bits(),
                    345.5_f64.to_bits()
                );
                assert_eq!(steps(), hit);
                call(
                    shared_root,
                    &shared,
                    &shared_request,
                    &shared_heights,
                    [0; 4],
                    &[],
                )?;
                assert_eq!(steps(), miss);

                let mut hard = registered.appearance.clone();
                hard.typography.axes.tick_label.font_size += 1.0;
                assert!(call(
                    root,
                    &canvas,
                    &crate::text::TextRequestContext::new(&hard, &root.text),
                    &measured,
                    [0; 4],
                    &[],
                )
                .is_err());
                assert_eq!(steps(), ["hard"]);

                let unavailable = crate::text::TextService::default();
                assert!(unavailable
                    .fonts
                    .set(Err("height preference witness".into()))
                    .is_ok());
                assert!(call(
                    root,
                    &canvas,
                    &crate::text::TextRequestContext::new(&registered.appearance, &unavailable),
                    &measured,
                    [0; 4],
                    &[],
                )
                .is_err());
                assert_eq!(steps(), ["hard", "preference"]);

                let mut prose = registered.appearance.clone();
                prose.typography.root.title.stack =
                    crate::FontStack::new([crate::FontFamily::named(
                        "Absent height-cache prose witness",
                    )?])?;
                assert!(call(
                    root,
                    &canvas,
                    &crate::text::TextRequestContext::new(&prose, &root.text),
                    &measured,
                    [0; 4],
                    &[],
                )
                .is_err());
                assert_eq!(steps(), ["hard", "preference", "prose"]);
                assert_eq!(
                    call(root, &canvas, &request, &measured, [0; 4], &[])?.to_bits(),
                    123.25_f64.to_bits()
                );
                assert_eq!(steps(), hit);
            } else {
                assert_eq!(axes.0[0], (-2.0, 3.0));
                call(root, &canvas, &request, &measured, [0; 4], &[])?;
                assert_eq!(
                    steps(),
                    miss,
                    "real consumed domain invalidates the prior height"
                );
            }
            round.request_domain(
                &registered.scope.address,
                &registered.scope.address,
                FeedbackKind::AutomaticDomainX,
                (-2.0, 3.0),
            )?;
            Ok(crate::layout::MeasuredLayout::new(
                assigned.assigned_size()?,
                (),
            ))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
    assert_eq!(rounds, 2);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the real parent path records only successful child preferences at the actual cell width"
)]
fn successful_child_height_records_cell_width_then_publishes_content_height_and_misses_do_not_record(
) {
    let _trace = HeightTraceGuard::start();
    let plot = || {
        Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 0.0]),
        )
        .title("Local heading")
        .hue(crate::CategoryValues::new("cohort", ["A", "B"]).unwrap())
        .legend(crate::LegendPolicy::Visible(
            crate::LegendPosition::OutsideRight,
        ))
    };
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .width(800)
        .panel(0, 0, plot())
        .unwrap();
    crate::layout::plan_root(
        canvas.sizing_request(),
        |stage| {
            Root::prepare(
                crate::orchestrator::root::RootSource::Canvas(&canvas),
                stage,
            )
        },
        Root::measure,
        |root, _, plan, _| {
            let crate::orchestrator::plan::MeasuredRootContent::Canvas(content) = &plan.content
            else {
                panic!("Canvas plan")
            };
            let cell = content.grid.owners[0].unwrap();
            let selected = &content.plots[0].plot;
            let operands = [
                cell.width(),
                selected.panel_box.width(),
                cell.height() - selected.selected.cell.height(),
            ];
            assert!(operands[0] > operands[1]);
            assert!(operands[2] > 0.0);
            let caches = root.caches.borrow();
            let record = caches.heights.records[0]
                .as_ref()
                .expect("successful child record");
            assert_eq!(
                record.inputs.width,
                crate::layout::kernel::nearest_spatial_quanta(cell.width())?
            );
            assert_ne!(
                record.inputs.width,
                crate::layout::kernel::nearest_spatial_quanta(selected.panel_box.width())?
            );
            assert_eq!(
                record.inputs.output_dimensions,
                Some(plan.measurement.measurement_size)
            );
            CHILD_HEIGHT_STEPS.with(|steps| {
                let steps = steps.borrow();
                let final_steps = &steps[steps.len() - 4..];
                assert_eq!(
                    final_steps.iter().map(|step| step.0).collect::<Vec<_>>(),
                    [
                        "child-measured",
                        "measured-preference",
                        "record",
                        "content-height"
                    ]
                );
                for (_, index, _) in final_steps {
                    assert_eq!(*index, 0);
                }
                assert_eq!(
                    final_steps[0].2.map(f64::to_bits),
                    operands.map(f64::to_bits)
                );
                assert_eq!(
                    final_steps[1].2.map(f64::to_bits),
                    [operands[1], operands[2], 0.0].map(f64::to_bits)
                );
                assert_eq!(final_steps[2].2[0].to_bits(), operands[0].to_bits());
                assert_eq!(
                    final_steps[2].2.map(f64::to_bits),
                    final_steps[3].2.map(f64::to_bits)
                );
                assert_eq!(
                    crate::layout::kernel::required_length(final_steps[2].2[1])
                        .unwrap()
                        .to_bits(),
                    record.height.to_bits()
                );
            });
            Ok(())
        },
    )
    .unwrap();

    let miss = Canvas::grid(1, 1)
        .unwrap()
        .width(800)
        .panel(
            0,
            0,
            plot().text_annotation(
                crate::PlotTextAnnotation::new(
                    crate::DataAreaCorner::TopLeft,
                    crate::Text::new("Cannot fit a positive line").layout(
                        crate::TextLayout::default()
                            .bounds(crate::TextBounds::new(None, Some(0.0), None).unwrap()),
                    ),
                )
                .unwrap(),
            ),
        )
        .unwrap();
    CHILD_HEIGHT_STEPS.with(|steps| steps.borrow_mut().clear());
    let failed = crate::layout::plan_root(
        miss.sizing_request(),
        |stage| Root::prepare(crate::orchestrator::root::RootSource::Canvas(&miss), stage),
        |root, assigned, round| {
            let error = root
                .measure(assigned, round)
                .err()
                .expect("child must miss");
            assert!(root
                .caches
                .borrow()
                .heights
                .records
                .iter()
                .all(Option::is_none));
            Err::<crate::layout::MeasuredLayout<()>, _>(error)
        },
        |_, _, (), _| Ok(()),
    );
    assert!(failed.is_err());
    CHILD_HEIGHT_STEPS.with(|steps| {
        let steps = steps.borrow();
        assert!(steps.iter().any(|step| step.0 == "child-miss"));
        assert!(!steps
            .iter()
            .any(|step| matches!(step.0, "record" | "content-height")));
    });
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one real Forest occurrence carries initial working I and distinct consumed shared G through both actual key entry points"
)]
fn forest_cell_height_and_child_keys_keep_initial_working_range_separate_from_resolved_x() {
    let forest = Plot::forest(
        crate::CategoryValues::new("term", ["A", "B"]).unwrap(),
        crate::NumericValues::new("ratio", [100.0, 10_000.0]),
        crate::Bounds::new(
            crate::NumericValues::new("lower", [80.0, 8_000.0]),
            crate::NumericValues::new("upper", [120.0, 12_000.0]),
        ),
        1_000.0,
    )
    .scale(crate::AxisScale::Log10);
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .width(800)
        .x_range_group([(0, 0)])
        .unwrap()
        .panel(0, 0, forest)
        .unwrap();
    let mut rounds = 0;
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 800,
            height: 600,
        },
        |stage| {
            Root::prepare(
                crate::orchestrator::root::RootSource::Canvas(&canvas),
                stage,
            )
        },
        |root, assigned, round| {
            rounds += 1;
            let registered = root.canvas_plot(0).unwrap();
            let initial = registered.axes.ranges()[0];
            assert!(initial.0 > 0.0);
            let feedback = (initial.0 / 2.0, initial.1 + 1.0);
            let owner = &registered.scope.address;
            if rounds > 1 {
                let _trace = HeightTraceGuard::start();
                let output = assigned.assigned_size()?;
                let request =
                    crate::text::TextRequestContext::new(&registered.appearance, &root.text);
                let grid_owner = ComponentAddress::root()
                    .child(AddressRole::CanvasGrid, 0)?
                    .child(AddressRole::CanvasCell, 0)?;
                let intrinsic = root.plot_intrinsic(&registered.scope.id)?;
                for (initial_sizing, output) in [(true, None), (false, Some(output))] {
                    cell_height(
                        registered,
                        &root.operation,
                        &canvas,
                        &request,
                        600.0,
                        intrinsic,
                        output,
                        axis_inputs(root, registered, round, initial_sizing),
                        true,
                        round,
                        &grid_owner,
                        [0; 4],
                        None,
                        &[],
                    )?;
                }
                HEIGHT_AXIS_INPUTS.with(|snapshots| {
                    let snapshots = snapshots.borrow();
                    assert_eq!(snapshots.len(), 2);
                    assert_eq!(
                        snapshots[0].1[0],
                        [initial.0.to_bits(), initial.1.to_bits()]
                    );
                    assert_eq!(
                        snapshots[1].1[0],
                        [initial.0.to_bits(), initial.1.to_bits()]
                    );
                    for (_, _, resolved_x) in snapshots.iter() {
                        assert_eq!(
                            *resolved_x,
                            Some([feedback.0.to_bits(), feedback.1.to_bits()])
                        );
                    }
                    assert!(snapshots[0].0.is_none());
                    assert_eq!(snapshots[1].0, Some(output));
                });
                let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
                let dimensions = super::super::PlotMeasureDimensions {
                    canvas: measurement.measurement_size,
                    output_override: Some(measurement.measurement_size),
                };
                let mut requirements = [(
                    0,
                    0,
                    EdgeSpaceRequirements {
                        left: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        top: 0.0,
                    },
                )];
                let cells = [Some(PixelRect::new(20.0, 620.0, 20.0, 500.0))];
                let _axis_trace =
                    crate::decorations::plan::children::ChildAxisInputsTraceGuard::start();
                let mut children = crate::decorations::plan::children::ChildPlans::canvas();
                for _ in 0..2 {
                    assert!(matches!(
                        canvas.complete_cell_gutters(
                            root,
                            &measurement,
                            dimensions,
                            &cells,
                            &root.request(),
                            &mut children,
                            round,
                            &mut requirements,
                            None,
                        )?,
                        fit::FitOutcome::Measured(())
                    ));
                }
                crate::decorations::plan::children::CHILD_AXIS_INPUTS.with(|snapshots| {
                    let snapshots = snapshots.borrow();
                    assert_eq!(
                        snapshots.len(),
                        1,
                        "second real probe must hit its complete key"
                    );
                    assert_eq!(snapshots[0].0, *owner);
                    assert_eq!(
                        snapshots[0].1[0], None,
                        "round key retains the absent raw window"
                    );
                    assert_eq!(snapshots[0].2, Some(feedback));
                });
            }
            // Seed through the real feedback publisher, never by editing a key,
            // source, context or assignment; the next call consumes it normally.
            round.request_domain(owner, owner, FeedbackKind::AutomaticDomainX, feedback)?;
            Ok(crate::layout::MeasuredLayout::new(
                assigned.assigned_size()?,
                (),
            ))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
    assert_eq!(rounds, 2);
}
