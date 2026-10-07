use super::*;

fn fractional_prose() -> (ResolvedTheme, i64, i64) {
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    for role in [
        &mut theme.typography.root.title,
        &mut theme.typography.root.subtitle,
        &mut theme.typography.root.caption,
    ] {
        *role = role.clone().with_font_size(1).unwrap();
    }
    let zero = crate::theme::RelativeSpacing::ratio(0, 1).unwrap();
    let third = crate::theme::RelativeSpacing::ratio(1, 3072).unwrap();
    let spacing = theme
        .composition
        .root_title_block()
        .clone()
        .with_title_block_to_content_clearance(third.clone())
        .unwrap()
        .with_caption_to_content_clearance(third)
        .unwrap();
    let padding =
        crate::theme::composition::RootPadding::new(zero.clone(), zero.clone(), zero.clone(), zero)
            .unwrap();
    theme.composition = theme
        .composition
        .clone()
        .with_root_title_block(spacing)
        .unwrap()
        .with_root_padding(padding)
        .unwrap();
    let height = |role| {
        theme
            .measure_text(
                "0",
                role,
                crate::layout::PhysicalTextAlignment::Start,
                "fractional root prose fixture",
                false,
            )
            .unwrap()
            .height_quanta
    };
    let top = height(&theme.typography.root.title);
    let bottom = height(&theme.typography.root.caption);
    (theme, top, bottom)
}

#[test]
fn root_frame_combines_two_third_quantum_clearances_at_its_real_height_boundary() {
    let (theme, top, bottom) = fractional_prose();
    let body_quanta = 10 * 1024 - top - bottom - 1;
    let body = f64::from(u32::try_from(body_quanta).unwrap()) / 1024.0;
    let frame = crate::layout::RootFramePlan::measure(
        (32, 10),
        &theme,
        [Some("0"), None, Some("0")],
        RootAlignments::resolve(&theme, [None; 2], [None; 2]),
        (1.0 / 1024.0, body),
    )
    .expect("the complete root requirement is N-1/3 quantum and must fit at N");
    assert!(frame.panel_box.height() >= body);
    assert!(matches!(
        crate::layout::RootFramePlan::measure(
            (32, 10),
            &theme,
            [Some("0"), None, Some("0")],
            RootAlignments::resolve(&theme, [None; 2], [None; 2]),
            (1.0 / 1024.0, body + 1.0 / 1024.0)
        ),
        Err(crate::layout::fit::PlanError::Fit(_))
    ));
    let mut ink = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
    frame.root_text.geometry(&mut ink).unwrap();
    assert!(!ink.bounds().is_empty());
    assert!(ink.bounds().iter().all(|bounds| bounds.left >= 0.0
        && bounds.right <= 32.0
        && bounds.bottom >= 0.0
        && bounds.top <= 10.0));
}

#[test]
fn embedded_frame_combines_two_third_quantum_clearances_without_losing_body_capacity() {
    let (theme, top, bottom) = fractional_prose();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 0.0]),
    )
    .title("0")
    .caption("0")
    .erase();
    let plot = crate::plots::shared_x::prepare_plot(&plot, &resources, None)
        .unwrap()
        .attach(plot)
        .unwrap();
    let minimum = plot.intrinsic_body_minimum(&theme).unwrap();
    let height = f64::from(
        u32::try_from(
            top + bottom + crate::layout::kernel::ceil_spatial_quanta(minimum.1).unwrap() + 1,
        )
        .unwrap(),
    ) / 1024.0;
    let intrinsics =
        RootTitleBlockPlan::intrinsic_widths(&theme, [Some("0"), None, Some("0")], None).unwrap();
    assert!(matches!(
        PlotLocalTextPlan::measure(
            &plot,
            PanelBox::new(0.0, 32.0, 0.0, height - 1.0 / 1024.0),
            &theme,
            [None; 2],
            intrinsics,
            &FitScope::root()
        )
        .unwrap(),
        FitOutcome::Miss(_)
    ));
    let FitOutcome::Measured(frame) = PlotLocalTextPlan::measure(
        &plot,
        PanelBox::new(0.0, 32.0, 0.0, height),
        &theme,
        [None; 2],
        intrinsics,
        &FitScope::root(),
    )
    .unwrap() else {
        panic!("exact embedded clearances must leave the complete scientific body minimum");
    };
    assert!(frame.body.height() >= minimum.1);
    let mut ink = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
    frame.text.geometry(&mut ink).unwrap();
    assert!(ink
        .bounds()
        .iter()
        .all(|bounds| bounds.bottom >= 0.0 && bounds.top <= height));
    assert_eq!(
        frame
            .text
            .placed_text()
            .map(|(_, text)| text.source.as_str())
            .collect::<Vec<_>>(),
        ["0", "0"]
    );
}

#[test]
fn natural_root_height_combines_exact_prose_reserves_before_integral_root_selection() {
    let (theme, top, bottom) = fractional_prose();
    let body = f64::from(u32::try_from(10 * 1024 - top - bottom - 1).unwrap()) / 1024.0;
    let source = crate::Canvas::grid(1, 1).unwrap().title("0").caption("0");
    let frame = crate::canvas::sizing::content_frame(&source, &theme, 32, body).unwrap();
    assert_eq!(
        frame.height_for_body(body, &theme).unwrap().to_bits(),
        10.0_f64.to_bits(),
        "selected-body recomposition must also keep the exact joint reserve"
    );
    assert_eq!(
        (frame.panel_box.y1 + frame.insets.top).to_bits(),
        10.0_f64.to_bits(),
        "natural root height must select N, not N+1 from separate clearance ceilings"
    );
}

#[test]
fn embedded_prose_uses_plot_owned_padding_only_when_present_and_keeps_body_horizontal_span() {
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let mut theme = crate::Theme::Publication.resolve(&resources).unwrap();
    let padding = crate::theme::composition::RootPadding::new(
        crate::theme::RelativeSpacing::ratio(32, 17).unwrap(),
        crate::theme::RelativeSpacing::ratio(4, 17).unwrap(),
        crate::theme::RelativeSpacing::ratio(3, 17).unwrap(),
        crate::theme::RelativeSpacing::ratio(7, 17).unwrap(),
    )
    .unwrap();
    theme.composition = theme
        .composition
        .clone()
        .with_root_padding(padding)
        .unwrap();
    let cell = PanelBox::new(0.0, 300.0, 0.0, 200.0);
    for present in [false, true] {
        let mut plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 0.0]),
        )
        .erase();
        if present {
            plot = plot.title("Plot-owned title");
        }
        plot.title_alignment = Some(TitleAlignment::Start);
        let plot = crate::plots::shared_x::prepare_plot(&plot, &resources, None)
            .unwrap()
            .attach(plot)
            .unwrap();
        let intrinsics =
            RootTitleBlockPlan::intrinsic_widths(&theme, [plot.title_text(), None, None], None)
                .unwrap();
        let FitOutcome::Measured(plan) = PlotLocalTextPlan::measure(
            &plot,
            cell,
            &theme,
            [None; 2],
            intrinsics,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("complete embedded prose must fit");
        };
        assert_eq!(plan.body.x0.to_bits(), cell.x0.to_bits());
        assert_eq!(plan.body.x1.to_bits(), cell.x1.to_bits());
        if present {
            assert_eq!(plan.body.y0.to_bits(), 3.0_f64.to_bits());
            assert_eq!(
                plan.body.y1.to_bits(),
                (193.0
                    - crate::theme::composition::hard(
                        &plan.text.reserves.top,
                        &theme
                            .prepared_font_environment()
                            .unwrap()
                            .generation_resources()
                    )
                    .unwrap())
                .to_bits()
            );
            let (placement, text) = plan.text.placed_text().next().unwrap();
            assert_eq!(placement.position.x.to_bits(), 32.0_f64.to_bits());
            assert_eq!(text.source, "Plot-owned title");
        } else {
            assert_eq!(plan.body, cell);
            assert_eq!(plan.text.placed_text().count(), 0);
        }
    }
}
