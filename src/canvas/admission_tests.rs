use super::*;

pub(super) fn capture_counters() -> [usize; 6] {
    [
        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
        crate::layout::Projection::test_projection_calls(),
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
        crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get),
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
        crate::orchestrator::SELECTED_INTERIOR_MEASUREMENTS.with(std::cell::Cell::get),
    ]
}

/// The exhaustive 25-family lifecycle fixture invokes this with its actual
/// source Plots. Matrix remains a root, never an expanded Canvas child.
pub(crate) fn mixed_family_parent_audit(plots: &[(PlotKind, Plot)]) {
    assert_eq!(plots.len(), 25);
    let mut canvas = Canvas::grid(4, 6)
        .unwrap()
        .size(2401, 2001)
        .title("All supported Canvas families")
        .subtitle("Retained child scopes")
        .caption("Independent root caption")
        .labels(Labels::raw().alias("x", "Shared sample"));
    let mut index = 0;
    for (kind, plot) in plots {
        if *kind == PlotKind::Matrix {
            let mut rejected = Canvas::grid(1, 1).unwrap();
            rejected.panels[0] = Some(plot.clone());
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            assert!(rejected.preflight_plan().is_err());
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
            continue;
        }
        canvas.panels[index] = Some(plot.clone().title(format!("{kind:?}")));
        index += 1;
    }
    assert_eq!(index, 24);
    // Mixed four-child roots stay within the existing operation budget; do not
    // relax the cap merely to fit all 24 supported families in one operation.
    let panels = std::mem::take(&mut canvas.panels);
    canvas.rows = 2;
    canvas.columns = 2;
    canvas.width = 1601;
    canvas.height = 1201;
    canvas.text_panels = vec![None; 4];
    for group in panels.chunks(4) {
        canvas.panels = group.to_vec();
        let cached = canvas.render_scene().unwrap();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|disabled| disabled.set(true));
        let uncached = canvas.render_scene();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE
            .with(|disabled| disabled.set(false));
        assert_eq!(cached.svg_bytes(), uncached.unwrap().svg_bytes());
    }
}

#[test]
fn parent_metadata_exact_refusal_precedes_fonts_and_backends() {
    let canvas = Canvas::grid(3, 5)
        .unwrap()
        .column_header(0, 5, "column span")
        .unwrap()
        .row_header(0, 3, "row span")
        .unwrap()
        .row_title(2, "row title")
        .unwrap();
    let run = |limits: Option<(u128, u64)>| {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        if let Some((visits, bytes)) = limits {
            resources.set_test_limits(visits, bytes);
        }
        canvas.admit_parent_metadata(&resources)?;
        Ok::<_, Error>((
            resources.usage().unwrap().1,
            resources.byte_usage().unwrap(),
        ))
    };
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
    let (visits, bytes) = run(None).unwrap();
    assert_eq!(run(Some((visits, bytes))).unwrap(), (visits, bytes));
    assert!(run(Some((visits - 1, bytes)))
        .unwrap_err()
        .to_string()
        .contains("admission"));
    assert!(run(Some((visits, bytes - 1)))
        .unwrap_err()
        .to_string()
        .contains("admission"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    assert_eq!(
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn complete_parent_admission_has_exact_work_and_storage_boundaries() {
    let canvas = Canvas::grid(2, 2)
        .unwrap()
        .panel(
            0,
            0,
            Plot::scatter(
                crate::NumericValues::new("raw_x", [1.0, 2.0, 3.0, 4.0]),
                crate::NumericValues::new("raw_y", [1.0, 3.0, 2.0, 4.0]),
            )
            .hue(crate::CategoryValues::new("hue", ["a", "a", "b", "b"]).unwrap())
            .x_ticks(vec![crate::AxisTick::new(2.0, "two")])
            .title("Panel title")
            .subtitle("Panel subtitle"),
        )
        .unwrap()
        .text_panel(
            1,
            1,
            TextPanel::new("Prose title", "First sentence and a second sentence.").unwrap(),
        )
        .unwrap()
        .row_header(0, 2, "Rows")
        .unwrap()
        .column_header(0, 2, "Columns")
        .unwrap()
        .row_title(1, "Row title")
        .unwrap()
        .title("Root title")
        .subtitle("Root subtitle")
        .caption("Root caption")
        .labels(Labels::raw().alias("raw_x", "Sample"))
        .legend_policy(CanvasLegendPolicy::Shared)
        .size(1201, 901);
    let run = |limits: Option<(u128, u64)>| {
        let themes = Arc::new(ResolvedCanvasThemes::new(canvas.theme, &canvas.panels));
        let mut children = crate::decorations::plan::children::ChildPlans::canvas(&canvas);
        crate::layout::plan_exact_root(
            canvas.width,
            canvas.height,
            |stage| {
                let resources = stage.generation_resources();
                if let Some((visits, bytes)) = limits {
                    resources.set_test_limits(visits, bytes);
                }
                canvas.admit_parent_metadata(&resources)?;
                let (interiors, shared) = canvas.prepare_interiors(stage)?;
                let (mut census, presence) =
                    canvas.foundational_text_census(&themes, &interiors, stage)?;
                themes.prepare_font_environment(&mut census, presence, stage)?;
                Ok((interiors, shared, resources))
            },
            |(interiors, shared, _), assigned, round| {
                let plan = canvas.build_canvas_plan(
                    Arc::clone(&themes),
                    assigned,
                    round,
                    &mut children,
                    shared,
                    interiors,
                )?;
                children.finish_round(round);
                Ok(crate::layout::MeasuredLayout::new(assigned, plan))
            },
            |(_, _, resources), _, _, _| {
                let usage = (
                    resources.usage().unwrap().1,
                    resources.byte_usage().unwrap(),
                );
                resources.set_test_limits(u128::MAX, u64::MAX);
                Ok(usage)
            },
        )
        .map(|root| *root.plan())
    };
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
    let usage = run(None).unwrap();
    assert_eq!(run(Some(usage)).unwrap(), usage);
    for limits in [(usage.0 - 1, usage.1), (usage.0, usage.1 - 1)] {
        assert!(run(Some(limits)).is_err());
    }
    assert_eq!(
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
        0
    );
}
