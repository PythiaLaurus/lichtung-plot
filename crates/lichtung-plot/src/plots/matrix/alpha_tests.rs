use super::*;
use crate::alpha::{AlphaEncoding, AlphaValues};

#[test]
fn every_off_diagonal_replays_original_observation_alpha_without_changing_diagonal_sources() {
    let source = Plot::matrix(vec![
        crate::NumericValues::new("a", [1.0, 2.0, 3.0]),
        crate::NumericValues::new("b", [2.0, 5.0, 7.0]),
    ])
    .hue(crate::CategoryValues::new("cohort", ["seen"; 3]).unwrap())
    .source_rows(crate::SourceRows::new([100, 200, 300]).unwrap())
    .with_alpha(
        AlphaTarget::Point,
        AlphaEncoding::observations(AlphaValues::new("confidence", [0, 128, 0]).unwrap()),
    )
    .with_alpha(
        AlphaTarget::Fill,
        AlphaEncoding::keyed(
            [crate::CategoryValues::new("cohort", ["\u{10ffff}", "seen"]).unwrap()],
            AlphaValues::new("density confidence", [0, 64]).unwrap(),
        )
        .unwrap(),
    )
    .hide_point_outline()
    .erase();
    let operation = crate::font::canonical::OperationContext::new();
    let prepare = |source: &Plot, rows| {
        use crate::orchestrator::registration::{PlotRegistryBuilder, Scope};
        let scope = Scope::new(source, &operation, 0, rows, None, None)?;
        let mut registry = PlotRegistryBuilder::new(operation.root_identity());
        let id = registry.reserve(source.description(), scope, &operation)?;
        prepare_axes(source, registry.data(&id, &operation)?, &id)
    };
    let (prepared, _) = prepare(
        &source,
        crate::orchestrator::registration::RowSelection::Selected {
            input_slots: vec![2, 0, 1],
            diagnostic_rows: vec![300, 100, 200],
        },
    )
    .unwrap();
    assert_eq!(
        prepared.groups[0]
            .alphas
            .iter()
            .map(|alpha| (alpha.slot, alpha.source_row, alpha.value.unwrap().alpha))
            .collect::<Vec<_>>(),
        [(2, 300, 0), (0, 100, 0), (1, 200, 128)]
    );
    let (before, before_axes) = prepare(
        &source,
        crate::orchestrator::registration::RowSelection::Full,
    )
    .unwrap();
    let control = source
        .clone()
        .with_alpha(
            AlphaTarget::Point,
            AlphaEncoding::single(AlphaValues::new("confidence", [0]).unwrap()).unwrap(),
        )
        .with_alpha(
            AlphaTarget::Fill,
            AlphaEncoding::single(AlphaValues::new("density confidence", [0]).unwrap()).unwrap(),
        );
    let (after, after_axes) = prepare(
        &control,
        crate::orchestrator::registration::RowSelection::Full,
    )
    .unwrap();
    let (before_variables, before_density) = before_axes.matrix().unwrap();
    let (after_variables, after_density) = after_axes.matrix().unwrap();
    assert_eq!(
        before_variables
            .iter()
            .map(|source| source.base_seed.unwrap().pair())
            .collect::<Vec<_>>(),
        after_variables
            .iter()
            .map(|source| source.base_seed.unwrap().pair())
            .collect::<Vec<_>>()
    );
    for (before, after) in before.diagonals.iter().zip(&after.diagonals) {
        assert_eq!(before.positions, after.positions);
        assert_eq!(before.densities, after.densities);
    }
    assert_eq!(
        before_density
            .iter()
            .map(|source| source.base_seed.unwrap().upper.to_bits())
            .collect::<Vec<_>>(),
        after_density
            .iter()
            .map(|source| source.base_seed.unwrap().upper.to_bits())
            .collect::<Vec<_>>()
    );
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let primary = theme
        .marks
        .get_points()
        .get_primary()
        .clone()
        .fill_alpha(0)
        .unwrap();
    let points = theme.marks.get_points().clone().primary(primary).unwrap();
    theme.marks = theme.marks.clone().points(points).unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    crate::orchestrator::with_prepared_plot_for_test(&source, &theme, |chart, inputs, theme, _operation| {
    let chart = chart.clone();
    let operation = theme.text.prepared_font_environment().unwrap().operation();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            inputs.plots[0].data(&operation).unwrap(),
            PixelRect::new(20.0, 800.0, 20.0, 620.0),
            theme,
        ).with_registration(&inputs.plots[0], inputs.intrinsic(&inputs.plots[0])))
        .unwrap();
    let layout::interior::MarkPayload::Matrix(payload) = &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Matrix payload")
    };
    assert_eq!(payload.cells.len(), 4);
    for index in [1, 2] {
        let points = payload.cells[index]
            .marks
            .iter()
            .map(|mark| match mark {
                MatrixPrimitive::Scatter(points, members, style) => {
                    assert_eq!(points.len(), members.len());
                    assert!(style.outline.is_none());
                    (
                        members[0].slot,
                        members[0].source_row,
                        style.color.get_alpha(),
                    )
                }
                MatrixPrimitive::Polygon(..) => panic!("off-diagonal polygon"),
            })
            .collect::<Vec<_>>();
        assert_eq!(points, [(0, 100, 0), (1, 200, 128), (2, 300, 0)]);
    }
    for index in [0, 3] {
        assert!(
            matches!(&payload.cells[index].marks[0], MatrixPrimitive::Polygon(_, style) if style.color.get_alpha() == 64 && style.stroke.get_alpha() == 255)
        );
    }
    }).unwrap();
}
