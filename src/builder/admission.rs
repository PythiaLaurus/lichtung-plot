//! Admission of the heap-owning configuration copied by composition adapters.
//!
//! Scientific inputs, values, labels and prepared payloads are immutable Arc
//! carriers. Only configuration storage is copied, before any mutation begins.

use super::Plot;
use crate::data::PlotData;
use crate::Error;

impl Plot {
    #[expect(
        clippy::too_many_lines,
        reason = "one boundary inventories every heap-owning Plot configuration field"
    )]
    pub(crate) fn admitted_clone(&self, data: PlotData<'_>) -> Result<Self, Error> {
        // Exhaustive inventory: adding a Plot field requires classifying its
        // ownership here rather than silently extending the derived Clone.
        let Self {
            input: _,
            source_rows: _,
            series: _,
            hue: _,
            group: _,
            title: _,
            subtitle: _,
            title_alignment: _,
            subtitle_alignment: _,
            caption: _,
            x_label: _,
            y_label: _,
            labels: _,
            x_axis_spec: _,
            x_limits_user_explicit: _,
            y_axis_spec: _,
            y_limits_user_explicit: _,
            theme: _,
            significance: _,
            width: _,
            height: _,
            matrix_variables: _,
            bar_spacing: _,
            histogram_bins: _,
            histogram_stat: _,
            histogram_composite: _,
            qq_line: _,
            point_outline_visible: _,
            legend_policy: _,
            suppress_continuous_guide: _,
            color_label: _,
            heatmap_scale: _,
            heatmap_layout: _,
            heatmap_label_fit: _,
            heatmap_color_limits: _,
            beeswarm_color_limits: _,
            heatmap_annotations: _,
            heatmap_null_row_annotations: _,
            heatmap_highlight: _,
            estimate_labels: _,
            significance_mark_style: _,
            category_orders: _,
            color_by_sign: _,
            zero_line: _,
            line_band: _,
            estimate_status: _,
            markers_visible: _,
            regression_band: _,
            line_path_style: _,
            raw_overlay: _,
            coefficient_reference_line: _,
            forest_reference: _,
            axis_scale: _,
            interaction_connectors: _,
            facet: _,
            resolved_facet_x_range: _,
            global_hue_levels: _,
            global_group_levels: _,
            global_collective_pairs: _,
            global_line_condition_levels: _,
            interaction_styles: _,
            categorical_colors: _,
            render_scope: _,
            prepared_axes: _,
            prepared_matrix_axes: _,
            prepared_scatter: _,
            prepared_line: _,
            prepared_qq: _,
            prepared_estimate: _,
            prepared_bars: _,
            prepared_marginal: _,
            prepared_waterfall: _,
            prepared_beeswarm: _,
            prepared_heatmap: _,
            prepared_interaction: _,
            prepared_distributions: _,
            prepared_dumbbell: _,
            prepared_regression: _,
            prepared_histogram: _,
            prepared_density: _,
            axis_gutter_owner: _,
            y_tick_labels_visible: _,
            waterfall_labels: _,
            guides: _,
            point_layers: _,
            bar_point_layers: _,
            panel_text_annotations: _,
            kind: _,
            marker: _,
        } = self;
        data.claim_source_records(1)?;
        for value in [
            &self.title,
            &self.subtitle,
            &self.caption,
            &self.x_label,
            &self.y_label,
            &self.color_label,
            &self.render_scope,
        ]
        .into_iter()
        .flatten()
        {
            data.admit_clone_str(value)?;
        }
        for axis in [&self.x_axis_spec, &self.y_axis_spec] {
            if let Some(ticks) = &axis.exact_ticks {
                data.admit_clone_slice(ticks)?;
                for tick in ticks {
                    data.admit_clone_str(&tick.label)?;
                }
            }
        }
        for values in [
            Some(&self.matrix_variables),
            self.global_hue_levels.as_ref(),
            self.global_group_levels.as_ref(),
            self.global_line_condition_levels.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            data.admit_clone_slice(values)?;
            for value in values {
                data.admit_clone_str(value)?;
            }
        }
        for order in [
            &self.category_orders.primary,
            &self.category_orders.x,
            &self.category_orders.y,
        ] {
            if let crate::CategoryOrder::Explicit(values) = order {
                data.admit_clone_slice(values)?;
                for value in values {
                    data.admit_clone_str(value)?;
                }
            }
        }
        if let Some(pairs) = &self.global_collective_pairs {
            data.admit_clone_slice(pairs)?;
            for pair in pairs {
                for text in [&pair.group, &pair.hue].into_iter().flatten() {
                    data.admit_clone_str(text)?;
                }
            }
        }
        data.admit_clone_slice(&self.significance)?;
        for annotation in &self.significance {
            let (a, b, facet) = match annotation {
                crate::significance::SignificanceAnnotation::Attached { target, facet, .. } => {
                    (target, None, facet)
                }
                crate::significance::SignificanceAnnotation::Pairwise {
                    endpoint_a,
                    endpoint_b,
                    facet,
                    ..
                } => (endpoint_a, Some(endpoint_b), facet),
            };
            for target in [Some(a), b].into_iter().flatten() {
                data.admit_clone_str(target.category_name())?;
                if let Some(series) = target.series_name() {
                    data.admit_clone_str(series)?;
                }
            }
            if let Some(facet) = facet {
                facet.admit_clone(data)?;
            }
        }
        if let Some(format) = &self.estimate_labels {
            format.admit_clone(data)?;
        }
        if let crate::HeatmapAnnotations::Values(format) = &self.heatmap_annotations {
            format.admit_clone(data)?;
        }
        data.admit_clone_slice(&self.heatmap_null_row_annotations)?;
        for annotation in &self.heatmap_null_row_annotations {
            data.admit_clone_str(annotation.row_label())?;
            data.admit_clone_str(annotation.text())?;
        }
        if let Some(styles) = &self.interaction_styles {
            data.admit_clone_slice(styles)?;
            for style in styles {
                data.admit_clone_str(style.group())?;
            }
        }
        if let Some(colors) = &self.categorical_colors {
            colors.admit_clone(data)?;
        }
        if let Some(labels) = &self.waterfall_labels {
            data.admit_clone_str(labels.base())?;
            data.admit_clone_str(labels.terminal())?;
        }
        data.admit_clone_slice(&self.guides)?;
        data.admit_clone_slice(&self.bar_point_layers)?;
        data.admit_clone_slice(&self.point_layers)?;
        for layer in &self.point_layers {
            if let Some(label) = layer.legend_label_text() {
                data.admit_clone_str(label)?;
            }
        }
        data.admit_clone_slice(&self.panel_text_annotations)?;
        for annotation in &self.panel_text_annotations {
            data.admit_clone_str(annotation.text())?;
        }
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::canonical::OperationGenerationResources;

    #[test]
    fn configuration_clone_has_exact_admission_and_shares_scientific_sources() {
        let source = |count| {
            Plot::scatter(
                crate::NumericValues::new("source x", (0..count).map(f64::from)),
                crate::NumericValues::new("source y", (0..count).map(f64::from)),
            )
            .title("Long root title".repeat(80))
            .subtitle("Subtitle")
            .caption("caption")
            .x_label("Explicit x")
            .y_label("Explicit y")
            .labels(crate::Labels::raw().alias("source x", "Display sample"))
            .x_ticks(vec![crate::AxisTick::new(1.0, "one".repeat(120))])
            .erase()
        };
        let run = |plot: &Plot, limits: Option<(u128, u64)>| {
            let resources = OperationGenerationResources::new();
            resources.set_test_limits(0, limits.map_or(u64::MAX, |(_, bytes)| bytes));
            if let Some((records, _)) = limits {
                resources.claim_slots(1_000_000 - records).unwrap();
            }
            let retained = plot.admitted_clone(plot.data().with_axis_resources(&resources))?;
            assert!(std::sync::Arc::ptr_eq(&plot.input, &retained.input));
            assert!(std::sync::Arc::ptr_eq(
                &plot.histogram_composite,
                &retained.histogram_composite
            ));
            assert_eq!(plot.labels, retained.labels);
            assert_eq!(plot.x_axis_spec, retained.x_axis_spec);
            assert_eq!(plot.title, retained.title);
            assert_eq!(resources.usage().unwrap().1, 0);
            Ok::<_, Error>((
                resources.usage().unwrap().0 - limits.map_or(0, |(records, _)| 1_000_000 - records),
                resources.byte_usage().unwrap(),
            ))
        };
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let small = source(2);
        let usage = run(&small, None).unwrap();
        assert_eq!(run(&source(10_000), None).unwrap(), usage);
        assert_eq!(run(&small, Some(usage)).unwrap(), usage);
        assert!(run(&small, Some((usage.0 - 1, usage.1))).is_err());
        assert!(run(&small, Some((usage.0, usage.1 - 1))).is_err());
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}
