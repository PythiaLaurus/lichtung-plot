use super::{ComposedInputs, Plot, PlotDescription};
use crate::data::category::CategoryOrderRef;
use crate::data::{FACET_COLUMN, FACET_ROW, FACET_WRAP};
use crate::orchestrator::registration::{legend_policy, render_name};
use crate::spec::{AxisScale, FacetScales, FacetSpec, PlotKind};

#[cfg(test)]
mod generation_tests;

impl<K: crate::kind::Marker> Plot<K> {
    /// Apply one complete generic-facet layout.
    #[must_use]
    pub fn facet(mut self, facet: FacetSpec) -> Self
    where
        K: crate::kind::Facet,
    {
        self.description.payload_mut().facet = Some(facet);
        self
    }
}

impl PlotDescription {
    pub(crate) fn facet_spec(&self) -> Option<&FacetSpec> {
        self.payload.facet.as_ref()
    }

    pub(crate) fn facet_scales_spec(&self) -> FacetScales {
        match &self.payload.facet {
            Some(facet) => facet.scale_policy(),
            None => FacetScales::Fixed,
        }
    }

    pub(crate) fn facet_columns(
        &self,
    ) -> (
        Option<&'static str>,
        Option<&'static str>,
        Option<&'static str>,
        Option<usize>,
    ) {
        (
            self.payload
                .facet
                .as_ref()
                .and_then(FacetSpec::row_values)
                .map(|_| FACET_ROW),
            self.payload
                .facet
                .as_ref()
                .and_then(FacetSpec::column_values)
                .map(|_| FACET_COLUMN),
            self.payload
                .facet
                .as_ref()
                .and_then(FacetSpec::wrap_values)
                .map(|_| FACET_WRAP),
            self.payload
                .facet
                .as_ref()
                .and_then(FacetSpec::wrap_columns),
        )
    }

    pub(crate) fn has_facets(&self) -> bool {
        self.payload.facet.is_some()
    }
}

impl<K: crate::kind::Marker> Plot<K> {
    pub(crate) fn without_facets(mut self) -> Self {
        self.description.payload_mut().facet = None;
        self
    }

    pub(crate) fn with_resolved_facet_ranges(
        mut self,
        kind: PlotKind,
        x_range: Option<(f64, f64)>,
        y_range: Option<(f64, f64)>,
    ) -> Self {
        if let Some(range) = x_range {
            if self.description.payload.axes[0].provenance
                != crate::layout::RangeProvenance::Configured
            {
                self.description.payload_mut().axes[0].provenance =
                    crate::layout::RangeProvenance::Shared;
            }
            if kind == PlotKind::Forest && self.description.payload.axis_scale == AxisScale::Log10 {
                self.resolved_facet_x_range = Some(range);
            } else {
                self.description.payload_mut().axes[0].window =
                    Some(crate::layout::RawWindow::from_pair(range));
            }
        }
        if let Some(range) = y_range {
            if self.description.payload.axes[1].provenance
                != crate::layout::RangeProvenance::Configured
            {
                self.description.payload_mut().axes[1].provenance =
                    crate::layout::RangeProvenance::Shared;
            }
            self.description.payload_mut().axes[1].window =
                Some(crate::layout::RawWindow::from_pair(range));
        }
        self
    }

    pub(crate) const fn resolved_facet_x_range(&self) -> Option<(f64, f64)> {
        self.resolved_facet_x_range
    }

    /// Compare the computed inputs that Canvas/facet coordination can replace.
    /// All other source configuration is immutable for a root-local child owner.
    pub(crate) fn same_composed_inputs(
        &self,
        other: &ComposedInputs,
        guides: &crate::decorations::legend::discrete::scoped::ScopedGuideInputs,
        registration: &crate::orchestrator::registration::PlotRegistration<'_>,
        canvas: Option<&crate::Canvas>,
    ) -> bool {
        fn same_values<T: PartialEq>(a: Option<&[T]>, b: Option<&[T]>) -> bool {
            match (a, b) {
                (None, None) => true,
                (Some(a), Some(b)) if a.len() == b.len() => {
                    for (a, b) in a.iter().zip(b) {
                        if a != b {
                            return false;
                        }
                    }
                    true
                }
                _ => false,
            }
        }
        let registered = Some(registration);
        let same_order = |order: Option<CategoryOrderRef<'_>>, values: Option<&[String]>| match (
            order, values,
        ) {
            (None, None) => true,
            (Some(order), Some(values)) if order.len() == values.len() => {
                order.values().zip(values).all(|(a, b)| a == b)
            }
            _ => false,
        };
        for (axis, other) in [
            (&self.description.payload.axes[0], &other.x_axis_spec),
            (&self.description.payload.axes[1], &other.y_axis_spec),
        ] {
            if axis.window.map(crate::layout::RawWindow::pair) != other.0
                || axis.provenance != other.1
                || axis.generation != other.2
                || !same_values(axis.exact_ticks.as_deref(), other.5.as_deref())
            {
                return false;
            }
        }
        let range = |value: Option<(f64, f64)>| value.map(|(a, b)| (a.to_bits(), b.to_bits()));
        self.description().same_prose(&other.prose)
            && self.description().same_axis_titles(&other.axis_titles)
            && range(
                self.description.payload.axes[0]
                    .window
                    .map(crate::layout::RawWindow::pair),
            ) == range(other.x_axis_spec.0)
            && range(
                self.description.payload.axes[1]
                    .window
                    .map(crate::layout::RawWindow::pair),
            ) == range(other.y_axis_spec.0)
            && range(self.resolved_facet_x_range) == range(other.resolved_facet_x_range)
            && same_order(
                crate::orchestrator::registration::parent_hues(registered)
                    .map(CategoryOrderRef::Keys),
                other.global_hue_levels.as_deref(),
            )
            && same_order(
                crate::orchestrator::registration::composed_group_levels(registered),
                other.global_group_levels.as_deref(),
            )
            && same_values(
                crate::orchestrator::registration::collective_pairs(registered),
                other.global_collective_pairs.as_deref(),
            )
            && same_values(
                crate::orchestrator::registration::line_condition_levels(registered),
                other.global_line_condition_levels.as_deref(),
            )
            && registered.map_or(
                crate::orchestrator::registration::GutterOwner::Root,
                |registered| registered.policy.gutter_owner,
            ) == other.axis_gutter_owner
            && crate::orchestrator::registration::axis_tick_labels_visible(registered, 1)
                == other.y_tick_labels_visible
            && render_name(registered) == other.render_scope.as_deref()
            && legend_policy(self.description(), registered) == other.legend_policy
            && crate::orchestrator::registration::continuous_guide_suppressed(registered)
                == other.suppress_continuous_guide
            && self.description.root_size == other.root_size
            && other.numeric.matches(registration, canvas)
            && self.description.payload.alpha == other.alpha
            && self.description.payload.text_layouts == other.text_layouts
            && self.output_dimensions == other.output_dimensions
            && guides.matches_source(self, registered)
            && *guides == other.guides
    }

    #[expect(
        clippy::too_many_lines,
        reason = "retain only the original narrow comparison values at their original fallible copy boundary"
    )]
    pub(crate) fn retain_composed_inputs(
        &self,
        guides: crate::decorations::legend::discrete::scoped::ScopedGuideInputs,
        registration: &crate::orchestrator::registration::PlotRegistration<'_>,
        canvas: Option<&crate::Canvas>,
    ) -> Result<ComposedInputs, crate::Error> {
        let registered = Some(registration);
        if !guides.matches_source(self, registered) {
            return Err(crate::Error::Render(
                "retained guide inputs do not belong to their computed source".into(),
            ));
        }
        let copy = |source: &str| -> Result<String, crate::Error> {
            let mut value = String::new();
            value
                .try_reserve_exact(source.len())
                .map_err(|error| crate::Error::Render(error.to_string()))?;
            value.push_str(source);
            Ok(value)
        };
        let mut axes = std::array::from_fn::<_, 2, _>(|_| {
            (
                None,
                crate::layout::RangeProvenance::Automatic,
                crate::TickGeneration::automatic(),
                crate::CategoryLabelSelection::all(),
                None,
                None,
            )
        });
        for (target, source) in axes.iter_mut().zip(&self.description.payload.axes) {
            target.0 = source.window.map(crate::layout::RawWindow::pair);
            target.1 = source.provenance;
            target.2.clone_from(&source.generation);
            target.3.clone_from(&source.category_labels);
            target.4.clone_from(&source.format);
            if let Some(ticks) = &source.exact_ticks {
                let mut retained = Vec::new();
                crate::render::backend::reserve_text_records(&mut retained, ticks.len())?;
                for tick in ticks {
                    retained.push(crate::AxisTick {
                        position: tick.position,
                        label: copy(&tick.label)?,
                    });
                }
                target.5 = Some(retained);
            }
        }
        let mut levels = [None, None, None];
        for (target, source) in levels.iter_mut().zip([
            crate::orchestrator::registration::parent_hues(registered).map(CategoryOrderRef::Keys),
            crate::orchestrator::registration::composed_group_levels(registered),
            crate::orchestrator::registration::line_condition_levels(registered)
                .map(CategoryOrderRef::Raw),
        ]) {
            if let Some(source) = source {
                let mut retained = Vec::new();
                crate::render::backend::reserve_text_records(&mut retained, source.len())?;
                for value in source.values() {
                    retained.push(copy(value)?);
                }
                *target = Some(retained);
            }
        }
        let pairs =
            if let Some(source) = crate::orchestrator::registration::collective_pairs(registered) {
                let mut retained = Vec::new();
                crate::render::backend::reserve_text_records(&mut retained, source.len())?;
                for pair in source {
                    retained.push(crate::data::CollectivePair {
                        group: pair.group.as_deref().map(&copy).transpose()?,
                        hue: pair.hue.as_deref().map(&copy).transpose()?,
                    });
                }
                Some(retained)
            } else {
                None
            };
        let [x_axis_spec, y_axis_spec] = axes;
        let [global_hue_levels, global_group_levels, global_line_condition_levels] = levels;
        Ok(ComposedInputs {
            text_layouts: self.description.payload.text_layouts.clone(),
            output_dimensions: self.output_dimensions,
            prose: self.description().copy_prose()?,
            axis_titles: self.description().copy_axis_titles()?,
            x_axis_spec,
            y_axis_spec,
            global_hue_levels,
            global_group_levels,
            global_line_condition_levels,
            global_collective_pairs: pairs,
            resolved_facet_x_range: self.resolved_facet_x_range,
            axis_gutter_owner: registered.map_or(
                crate::orchestrator::registration::GutterOwner::Root,
                |registered| registered.policy.gutter_owner,
            ),
            y_tick_labels_visible: crate::orchestrator::registration::axis_tick_labels_visible(
                registered, 1,
            ),
            render_scope: render_name(registered).map(copy).transpose()?,
            legend_policy: legend_policy(self.description(), registered),
            suppress_continuous_guide:
                crate::orchestrator::registration::continuous_guide_suppressed(registered),
            root_size: self.description.root_size,
            numeric: crate::numeric::inputs::NumericInputs::new(registration, canvas)?,
            alpha: self.description.payload.alpha.clone(),
            guides,
        })
    }

    pub(crate) fn with_canvas_size(mut self, width: u32, height: u32) -> Self {
        self.description.root_size = crate::layout::RootRequest::ExactSize { width, height };
        self
    }

    pub(crate) fn with_output_dimensions(mut self, width: u32, height: u32) -> Self {
        self.output_dimensions = Some((width, height));
        self
    }
}
