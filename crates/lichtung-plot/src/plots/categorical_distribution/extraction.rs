use crate::{CategoryValues, NumericValues, Plot, RaincloudInput};

pub(crate) fn supplied() -> Plot {
    Plot::raincloud(RaincloudInput::geometry(crate::RaincloudGeometry {
        category_dimension: "category".into(),
        value_dimension: "value".into(),
        hue_dimension: Some("color".into()),
        category: CategoryValues::new("category", ["supplied"]).unwrap(),
        hue: Some(CategoryValues::new("color", ["one"]).unwrap()),
        q1: NumericValues::new("q1", [1.0]),
        median: NumericValues::new("median", [3.0]),
        q3: NumericValues::new("q3", [5.0]),
        density_category: CategoryValues::new("category", ["supplied"; 3]).unwrap(),
        density_hue: Some(CategoryValues::new("color", ["one"; 3]).unwrap()),
        density_y: NumericValues::new("y", [0.0, 3.0, 6.0]),
        normalized_width: NumericValues::new("width", [0.0, 0.8, 0.0]),
        point_category: CategoryValues::new("category", ["supplied"; 2]).unwrap(),
        point_hue: Some(CategoryValues::new("color", ["one"; 2]).unwrap()),
        point_y: NumericValues::new("y", [2.0, 4.0]),
        normalized_strip: NumericValues::new("strip", [0.25, 0.75]),
    }))
    .erase()
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one real Root witnesses guide preparation, assigned coordinate marks and paint without rescanning either free-x child"
)]
fn free_x_sampled_boxplot_keeps_each_childs_first_palette_color_without_source_rescans() {
    use crate::decorations::legend::discrete::scoped::{GuideConsumption, GuideUse};
    use crate::layout::{fit::FitScope, PixelRect};
    use crate::orchestrator::assignment::{
        GuideTrialAssignment, ParentInsets, PlotAllocationInputs, PlotMeasureDimensions,
    };
    use crate::orchestrator::root::{Root, RootSource};
    use crate::plots::categorical_distribution::{Mark, FACET_CATEGORY_SCANS};
    use crate::render::backend::Backend;

    let source = Plot::boxplot(
        CategoryValues::new("category", ["A", "A", "A", "B", "B", "B"]).unwrap(),
        NumericValues::new("value", [1.0, 2.0, 3.0, 2.0, 3.0, 4.0]),
    )
    .facet(
        crate::FacetSpec::column(
            CategoryValues::new("facet", ["left", "left", "left", "right", "right", "right"])
                .unwrap(),
        )
        .scales(crate::FacetScales::FreeX),
    )
    .size(1200, 800);
    FACET_CATEGORY_SCANS.with(|scans| scans.set(0));
    crate::layout::plan_root(
        source.description().sizing_request(),
        |stage| {
            let root = Root::prepare(RootSource::Plot(source.description()), stage)?;
            // Includes the real guide preparation, even though no hue contributes
            // an entry. The canonical source still has its authored facets.
            FACET_CATEGORY_SCANS.with(|scans| assert_eq!(scans.get(), 0));
            Ok(root)
        },
        |root, assigned, round| {
            let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
            assert_eq!(root.plots.len(), 2);
            for (index, registered) in root.plots.iter().enumerate() {
                assert!(std::ptr::eq(registered.source, source.description()));
                assert!(registered.source.has_facets());
                assert_eq!(
                    registered.axes.categories(0).unwrap().raw,
                    [if index == 0 { "A" } else { "B" }],
                );
                let first = registered
                    .appearance
                    .color_scales
                    .get_qualitative()
                    .colors()[0]
                    .as_rgb();
                let second = registered
                    .appearance
                    .color_scales
                    .get_qualitative()
                    .colors()[1]
                    .as_rgb();
                assert_ne!(first, second);
                let factory = root.plot_assignment_factory(&measurement, &registered.scope.id)?;
                let panel = PixelRect::new(80.0, 560.0, 80.0, 680.0);
                let entry = factory.assign(
                    PlotAllocationInputs {
                        dimensions: PlotMeasureDimensions {
                            canvas: measurement.measurement_size,
                            output_override: None,
                        },
                        cell: panel,
                        frame: panel,
                        body: panel,
                        axis_label_width: panel.width(),
                        content_height: false,
                        fill_content_height: false,
                        content_preference: None,
                        insets: ParentInsets::consumed(round, &registered.scope.address),
                        guide: GuideTrialAssignment {
                            edge: crate::LegendPosition::Auto,
                            compact: false,
                            allocation: None,
                        },
                    },
                    round,
                    None,
                )?;
                let local = registered.local_guide_slot()?;
                let guides = GuideConsumption {
                    use_kind: GuideUse::Local,
                    local_guide: local.as_ref(),
                    dependencies: &[],
                };
                let scope = FitScope::new(&registered.scope.address, 0);
                let context = factory.context(&entry, guides, &scope)?;
                let request = context.request();
                let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                let result = crate::layout::interior::InteriorRoute::classify(source.kind())
                    .measure(registered.source, &context, &request)
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                let crate::layout::interior::MarkPayload::Distribution(payload) =
                    &result.marks.as_ref().unwrap().payload
                else {
                    panic!("Boxplot marks")
                };
                let fills = payload
                    .marks
                    .iter()
                    .filter_map(|mark| match mark {
                        Mark::Polygon(_, style) => {
                            Some((style.color.red(), style.color.green(), style.color.blue()))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let mut backend = crate::render::svg_backend::SvgBackend::in_memory(1200, 800);
                backend.begin(1200, 800)?;
                result.marks.as_ref().unwrap().draw(&mut backend)?;
                backend.end()?;
                assert_eq!(
                    fills,
                    [first],
                    "child {index} must use its local first color"
                );
                assert_eq!(
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    reads
                );
                FACET_CATEGORY_SCANS.with(|scans| assert_eq!(scans.get(), 0));
            }
            Ok(crate::layout::MeasuredLayout::new(
                measurement.measurement_size,
                (),
            ))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
}
