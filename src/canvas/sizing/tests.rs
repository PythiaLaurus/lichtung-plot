use super::*;
use crate::decorations::legend::discrete::scoped::{DependencyUse, GuideAllocation};
use crate::numeric::{decimal::NegativeZero, format::NumericFormat};

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
    let (_, text, prose) = source.prepare_intrinsics(&themes, &interiors).unwrap();
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 800,
            height: 600,
        },
        |_| Ok(()),
        |(), assigned, round| {
            let content = super::super::tracks::ContentSizing {
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
                &text,
                &prose,
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
                PanelBox::new(0.0, 0.0, 0.0, 0.0),
                &source.placements,
                &plots,
                &text,
                &prose,
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
        |(), _, (), _| Ok(()),
    )
    .unwrap();
}

fn assert_changed_height_inputs(source: Plot, variants: impl IntoIterator<Item = Plot>) {
    let operation = crate::font::canonical::OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &operation,
        Some(&crate::Theme::publication()),
    )
    .unwrap();
    inputs
        .prepare_discrete_guides(
            &source,
            &crate::Theme::publication().resolve().unwrap(),
            &operation,
            &ComponentAddress::root(),
            false,
        )
        .unwrap();
    let variants = variants
        .into_iter()
        .map(|variant| inputs.attach(variant).unwrap())
        .collect::<Vec<_>>();
    let source = inputs.attach(source).unwrap();
    let measurements = HeightMeasurements::default();
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 800,
            height: 600,
        },
        |_| Ok(()),
        |(), assigned, round| {
            let owner = ComponentAddress::root().child(AddressRole::CanvasCell, 0)?;
            let ancestor = ComponentAddress::root().child(AddressRole::CanvasSharedLegend, 0)?;
            let dependencies = [ConsumedGuideDependency::new(
                ancestor.clone(),
                DependencyUse::AllocationOnly,
                GuideAllocation::width(400.0)?,
                round,
            )?];
            measurements.record(&source, 400.0, 120.0, [0; 4], round, &owner, &dependencies)?;
            let same =
                HeightInputs::new(&source, 400.0, [0; 4], round, &owner, true, &dependencies)?;
            assert_eq!(measurements.measured(&same, &owner, round)?, Some(120.0));
            let undeclared = HeightInputs::new(&source, 400.0, [0; 4], round, &owner, true, &[])?;
            assert_eq!(measurements.measured(&undeclared, &owner, round)?, None);
            let narrower = [ConsumedGuideDependency::new(
                ancestor,
                DependencyUse::AllocationOnly,
                GuideAllocation::width(400.0 - 1.0 / 1024.0)?,
                round,
            )?];
            let different =
                HeightInputs::new(&source, 400.0, [0; 4], round, &owner, true, &narrower)?;
            assert_eq!(same.width, different.width);
            assert_eq!(measurements.measured(&different, &owner, round)?, None);
            for changed in &variants {
                let different =
                    HeightInputs::new(changed, 400.0, [0; 4], round, &owner, true, &dependencies)?;
                assert_eq!(same.width, different.width);
                assert_eq!(same.ranges, different.ranges);
                assert_eq!(measurements.measured(&different, &owner, round)?, None);
                measurements.record(changed, 400.0, 120.0, [0; 4], round, &owner, &dependencies)?;
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
        |(), _, (), _| Ok(()),
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
    assert_changed_height_inputs(source, variants);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one identity witness contrasts complete numeric, guide and footprint inputs at equal width/domain"
)]
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
    let mut spanning = make(format.clone(), format.clone()).erase();
    spanning.canvas_owner = Some(crate::data::source::source_arc([2, 2, 0, 0, 2, 1]).unwrap());
    assert_changed_height_inputs(
        make(format.clone(), format.clone()).erase(),
        vec![
            spanning,
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
    assert_changed_height_inputs(source.erase(), variants);

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
    assert_changed_height_inputs(source.erase(), variants);
}
