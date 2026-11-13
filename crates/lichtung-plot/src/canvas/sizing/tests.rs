use super::*;
use crate::decorations::legend::discrete::scoped::{DependencyUse, GuideAllocation};
use crate::numeric::{decimal::NegativeZero, format::NumericFormat};
use std::sync::Arc;

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
    let themes = super::super::ResolvedCanvasThemes::new(
        &source.theme,
        &source.placements,
        &source.guide_text_layouts,
    )
    .unwrap();
    let interiors = vec![None];
    let plots = vec![None];
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 800,
            height: 600,
        },
        |stage| {
            let operation = stage.operation();
            let (_, text, prose) = source.prepare_intrinsics(&themes, &interiors, &operation)?;
            Ok((text, prose, operation))
        },
        |(text, prose, operation), assigned, round| {
            let content = super::super::tracks::ContentSizing {
                operation,
                source: &source,
                interiors: &interiors,
                round,
                measurements: None,
                shared_allocation: None,
                natural_width: true,
                natural_height: true,
            };
            let width = fit::measured(super::super::tracks::natural_width(
                &source.placements,
                &plots,
                text,
                prose,
                &[],
                &[],
                &[],
                &themes,
                &FitScope::root(),
                &content,
            ))
            .map_err(fit::PlanError::into_error)?;
            assert!(width > 0.0);
            let full = super::super::tracks::allocate(
                PixelRect::new(0.0, 0.0, 0.0, 0.0),
                &source.placements,
                &plots,
                &interiors,
                operation,
                text,
                prose,
                &[],
                &[],
                &[],
                &[],
                &themes,
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
            super::super::placement::Payload::Plot(Arc::new(source.clone())),
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
                super::super::placement::Payload::Plot(Arc::new(source.clone())),
            )
            .unwrap();
        spanning
    });
    let appearance = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let request = crate::text::TextRequestContext::new(&appearance, &text);
    let themes = super::super::ResolvedCanvasThemes::new(
        &canvas.theme,
        &canvas.placements,
        &canvas.guide_text_layouts,
    )
    .unwrap();
    let variants = variants.into_iter().collect::<Vec<_>>();
    let mut measurements = HeightMeasurements::default();
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 800,
            height: 600,
        },
        |stage| {
            let (mut interiors, _) = canvas.prepare_interiors(stage, Some(&themes))?;
            let inputs = interiors[0].take().expect("prepared Canvas occurrence");
            inputs.prepare_discrete_guides(
                source,
                &request,
                &stage.operation(),
                &ComponentAddress::root(),
                false,
            )?;
            Ok(inputs)
        },
        |inputs, assigned, round| {
            let scope = inputs.plots[0]
                .scope
                .render_name
                .as_deref()
                .expect("Canvas scope");
            let source = inputs.attach(source.clone().with_render_scope(scope.to_owned()))?;
            let owner = ComponentAddress::root().child(AddressRole::CanvasCell, 0)?;
            let ancestor = ComponentAddress::root().child(AddressRole::CanvasSharedLegend, 0)?;
            let dependencies = [ConsumedGuideDependency::new(
                ancestor.clone(),
                DependencyUse::AllocationOnly,
                GuideAllocation::width(400.0)?,
                round,
            )?];
            measurements.record(
                &source,
                inputs.plots.first(),
                &canvas,
                400.0,
                120.0,
                [0; 4],
                round,
                &owner,
                &dependencies,
            )?;
            let same = HeightInputs::new(
                &source,
                inputs.plots.first(),
                &canvas,
                400.0,
                [0; 4],
                round,
                &owner,
                true,
                &dependencies,
            )?;
            assert_eq!(measurements.measured(&same, &owner, round)?, Some(120.0));
            let undeclared = HeightInputs::new(
                &source,
                inputs.plots.first(),
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
            let different = HeightInputs::new(
                &source,
                inputs.plots.first(),
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
            for changed in &variants {
                let changed = inputs.attach(changed.clone().with_render_scope(scope.to_owned()))?;
                let different = HeightInputs::new(
                    &changed,
                    inputs.plots.first(),
                    &canvas,
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
                    &changed,
                    inputs.plots.first(),
                    &canvas,
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
                measurements.record(
                    &source,
                    inputs.plots.first(),
                    &canvas,
                    400.0,
                    120.0,
                    [0; 4],
                    round,
                    &owner,
                    &dependencies,
                )?;
                let different = HeightInputs::new(
                    &source,
                    inputs.plots.first(),
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
                measurements.record(
                    &source,
                    inputs.plots.first(),
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
