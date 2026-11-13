use super::*;
use crate::orchestrator::registration::{PlotRegistryBuilder, PreparedFamily, RowSelection, Scope};

#[test]
fn shared_initial_axis_seed_drives_the_original_curve_finish_before_fonts() {
    let source = Plot::marginal(
        crate::NumericValues::new("x", [1.0, 2.0, 3.0]),
        crate::NumericValues::new("y", [2.0, 4.0, 3.0]),
    )
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 1,
            height: 1,
        },
        |stage| {
            let operation = stage.operation();
            let scope = Scope::new(&source, &operation, 0, RowSelection::Full, None, None)?;
            let mut registry = PlotRegistryBuilder::new(operation.root_identity());
            let id = registry.reserve(source.description(), scope, &operation)?;
            let panel = &mut registry.panels[id.slot];
            crate::plots::shared_x::prepare_plot_registration(panel, &source, None, &operation)?;
            let axes = panel.axes.as_ref().unwrap();
            let initial_y = axes.ranges()[1];
            assert!(axes
                .sources()
                .iter()
                .all(|source| source.label.get().is_none()));
            let Some(PreparedFamily::Marginal(marks)) = &panel.family else {
                panic!("prepared Marginal")
            };
            assert!(marks.curves.x_grid.is_empty() && marks.curves.y_grid.is_empty());

            let shared = (-10.0, 10.0);
            crate::plots::shared_x::prepare_reference_plot(
                &source,
                panel,
                Some(shared),
                None,
                &operation,
            )?;
            let axes = panel.axes.as_ref().unwrap();
            assert_eq!(axes.ranges(), [shared, initial_y]);
            assert_eq!(
                axes.sources()[0].base_provenance,
                layout::RangeProvenance::Shared
            );
            assert!(axes
                .sources()
                .iter()
                .all(|source| source.label.get().is_some()));
            let Some(PreparedFamily::Marginal(marks)) = &panel.family else {
                panic!("finished Marginal")
            };
            assert_eq!(marks.curves.x_grid.len(), 80);
            assert_eq!(marks.curves.y_grid.len(), 80);
            assert_eq!(marks.curves.x_grid.first(), Some(&shared.0));
            assert_eq!(marks.curves.x_grid.last(), Some(&shared.1));
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
            Ok(())
        },
        |(), assigned, _| {
            Ok(crate::layout::MeasuredLayout::new(
                assigned.assigned_size()?,
                (),
            ))
        },
        |(), _, (), _| Ok(()),
    )
    .unwrap();
}
