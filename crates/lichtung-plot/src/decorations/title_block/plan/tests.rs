use super::*;

#[test]
fn fitted_prose_uses_permitted_minima_and_selected_gaps_without_changing_root_padding() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let request = crate::TextLayout::default().fit(crate::TextFit::range(14.0, 14.0).unwrap());
    let original = crate::Text::new("Complete fitted heading")
        .style(
            theme
                .typography
                .root
                .title
                .clone()
                .font_size(1_000_000_000.0)
                .unwrap(),
        )
        .layout(request.clone());
    let nominal = crate::Text::new("Complete fitted heading")
        .style(theme.typography.root.title.clone().font_size(14.0).unwrap())
        .layout(request);
    let subtitle = crate::Text::new("Independent subtitle");
    let caption = crate::Text::new("Independent caption");
    let alignments = RootAlignments::resolve(&theme, [None; 3], [None; 2]);
    let padding = theme.root_padding_lengths().unwrap();
    let measure = |text: &crate::Text| {
        let sources = [Some(text), Some(&subtitle), Some(&caption)];
        let minimum = RootTitleBlockPlan::minimum_height(&theme, sources, 300.0).unwrap();
        let plan = RootTitleBlockPlan::measure(
            &theme,
            sources,
            PanelBox::new(0.0, 400.0, 0.0, 600.0),
            &padding,
            400.0,
            alignments,
            300.0,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .unwrap();
        (minimum, plan)
    };
    let (fitted_minimum, fitted) = measure(&original);
    let (nominal_minimum, nominal) = measure(&nominal);
    assert_eq!(fitted_minimum.to_bits(), nominal_minimum.to_bits());
    assert_eq!(fitted.reserves, nominal.reserves);
    for ((fitted_placement, fitted_text), (nominal_placement, nominal_text)) in
        fitted.placed_text().zip(nominal.placed_text())
    {
        assert_eq!(fitted_placement, nominal_placement);
        assert_eq!(fitted_text.lines, nominal_text.lines);
    }
    assert_eq!(
        fitted
            .reserves
            .body(PanelBox::new(0.0, 400.0, 0.0, 600.0), &padding)
            .unwrap(),
        nominal
            .reserves
            .body(PanelBox::new(0.0, 400.0, 0.0, 600.0), &padding)
            .unwrap(),
    );
}
