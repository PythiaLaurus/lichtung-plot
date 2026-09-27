use super::{ComposedInputs, Plot};
use crate::data::{FACET_COLUMN, FACET_ROW, FACET_WRAP};
use crate::spec::{AxisScale, FacetScales, FacetSpec, PlotKind};

#[cfg(test)]
mod interval_tests;

impl<K: crate::kind::Marker> Plot<K> {
    /// Apply one complete generic-facet layout.
    #[must_use]
    pub fn facet(mut self, facet: FacetSpec) -> Self
    where
        K: crate::kind::Facet,
    {
        self.facet = Some(facet);
        self
    }

    pub(crate) const fn facet_scales_spec(&self) -> FacetScales {
        match &self.facet {
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
            self.facet
                .as_ref()
                .and_then(FacetSpec::row_values)
                .map(|_| FACET_ROW),
            self.facet
                .as_ref()
                .and_then(FacetSpec::column_values)
                .map(|_| FACET_COLUMN),
            self.facet
                .as_ref()
                .and_then(FacetSpec::wrap_values)
                .map(|_| FACET_WRAP),
            self.facet.as_ref().and_then(FacetSpec::wrap_columns),
        )
    }

    pub(crate) fn without_facets(mut self) -> Self {
        self.facet = None;
        self
    }

    pub(crate) fn with_significance_subset<'a>(
        mut self,
        selectors: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Self {
        let selectors = selectors.into_iter().collect::<Vec<_>>();
        self.significance.retain(|annotation| {
            annotation
                .facet_cell()
                .is_some_and(|cell| cell.matches(selectors.iter().map(|(_, level)| *level)))
        });
        self
    }

    pub(crate) const fn has_facets(&self) -> bool {
        self.facet.is_some()
    }

    pub(crate) fn with_resolved_facet_ranges(
        mut self,
        kind: PlotKind,
        x_range: Option<(f64, f64)>,
        y_range: Option<(f64, f64)>,
    ) -> Self {
        if let Some(range) = x_range {
            if self.x_axis_spec.provenance != crate::layout::RangeProvenance::Configured {
                self.x_axis_spec.provenance = crate::layout::RangeProvenance::Shared;
            }
            if kind == PlotKind::Forest && self.axis_scale == AxisScale::Log10 {
                self.resolved_facet_x_range = Some(range);
            } else {
                self.x_axis_spec.limits = Some(range);
            }
        }
        if let Some(range) = y_range {
            if self.y_axis_spec.provenance != crate::layout::RangeProvenance::Configured {
                self.y_axis_spec.provenance = crate::layout::RangeProvenance::Shared;
            }
            self.y_axis_spec.limits = Some(range);
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
        data: crate::data::PlotData<'_>,
    ) -> Result<bool, crate::Error> {
        fn same_values<T: PartialEq>(
            a: Option<&[T]>,
            b: Option<&[T]>,
            data: crate::data::PlotData<'_>,
        ) -> Result<bool, crate::Error> {
            match (a, b) {
                (None, None) => Ok(true),
                (Some(a), Some(b)) if a.len() == b.len() => {
                    for (a, b) in a.iter().zip(b) {
                        data.claim_source_visits(1)?;
                        if a != b {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
        for (axis, other) in [
            (&self.x_axis_spec, &other.x_axis_spec),
            (&self.y_axis_spec, &other.y_axis_spec),
        ] {
            if axis.limits != other.limits
                || axis.provenance != other.provenance
                || axis.interval != other.interval
                || !same_values(
                    axis.exact_ticks.as_deref(),
                    other.exact_ticks.as_deref(),
                    data,
                )?
            {
                return Ok(false);
            }
        }
        let range = |value: Option<(f64, f64)>| value.map(|(a, b)| (a.to_bits(), b.to_bits()));
        Ok(
            range(self.x_axis_spec.limits) == range(other.x_axis_spec.limits)
                && range(self.y_axis_spec.limits) == range(other.y_axis_spec.limits)
                && range(self.resolved_facet_x_range) == range(other.resolved_facet_x_range)
                && same_values(
                    self.global_hue_levels.as_deref(),
                    other.global_hue_levels.as_deref(),
                    data,
                )?
                && same_values(
                    self.global_group_levels.as_deref(),
                    other.global_group_levels.as_deref(),
                    data,
                )?
                && same_values(
                    self.global_collective_pairs.as_deref(),
                    other.global_collective_pairs.as_deref(),
                    data,
                )?
                && same_values(
                    self.global_line_condition_levels.as_deref(),
                    other.global_line_condition_levels.as_deref(),
                    data,
                )?
                && self.axis_gutter_owner == other.axis_gutter_owner
                && self.y_tick_labels_visible == other.y_tick_labels_visible
                && self.render_scope == other.render_scope
                && self.legend_policy == other.legend_policy
                && self.suppress_continuous_guide == other.suppress_continuous_guide
                && (self.width, self.height) == (other.width, other.height),
        )
    }

    pub(crate) fn retain_composed_inputs(
        &self,
        environment: &crate::font::canonical::PreparedFontEnvironment,
    ) -> Result<ComposedInputs, crate::Error> {
        let copy = |source: &str| -> Result<String, crate::Error> {
            environment
                .claim_unicode_bytes(
                    u64::try_from(source.len())
                        .map_err(|_| crate::Error::Render("child input bytes overflow".into()))?,
                )
                .map_err(|error| crate::Error::Render(error.to_string()))?;
            let mut value = String::new();
            value
                .try_reserve_exact(source.len())
                .map_err(|error| crate::Error::Render(error.to_string()))?;
            value.push_str(source);
            Ok(value)
        };
        let mut axes = [
            crate::layout::AxisSpec::default(),
            crate::layout::AxisSpec::default(),
        ];
        for (target, source) in axes.iter_mut().zip([&self.x_axis_spec, &self.y_axis_spec]) {
            target.limits = source.limits;
            target.provenance = source.provenance;
            target.interval = source.interval;
            if let Some(ticks) = &source.exact_ticks {
                let mut retained = Vec::new();
                crate::render::backend::reserve_text_records(
                    &mut retained,
                    ticks.len(),
                    environment,
                )?;
                for tick in ticks {
                    retained.push(crate::AxisTick {
                        position: tick.position,
                        label: copy(&tick.label)?,
                    });
                }
                target.exact_ticks = Some(retained);
            }
        }
        let mut levels = [None, None, None];
        for (target, source) in levels.iter_mut().zip([
            &self.global_hue_levels,
            &self.global_group_levels,
            &self.global_line_condition_levels,
        ]) {
            if let Some(source) = source {
                let mut retained = Vec::new();
                crate::render::backend::reserve_text_records(
                    &mut retained,
                    source.len(),
                    environment,
                )?;
                for value in source {
                    retained.push(copy(value)?);
                }
                *target = Some(retained);
            }
        }
        let pairs = if let Some(source) = &self.global_collective_pairs {
            let mut retained = Vec::new();
            crate::render::backend::reserve_text_records(&mut retained, source.len(), environment)?;
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
            x_axis_spec,
            y_axis_spec,
            global_hue_levels,
            global_group_levels,
            global_line_condition_levels,
            global_collective_pairs: pairs,
            resolved_facet_x_range: self.resolved_facet_x_range,
            axis_gutter_owner: self.axis_gutter_owner,
            y_tick_labels_visible: self.y_tick_labels_visible,
            render_scope: self.render_scope.as_deref().map(copy).transpose()?,
            legend_policy: self.legend_policy,
            suppress_continuous_guide: self.suppress_continuous_guide,
            width: self.width,
            height: self.height,
        })
    }

    pub(crate) fn with_global_hue_levels(mut self, levels: Vec<String>) -> Self {
        self.global_hue_levels = Some(levels);
        self
    }

    pub(crate) fn global_hue_levels(&self) -> Option<&[String]> {
        self.global_hue_levels.as_deref()
    }

    pub(crate) fn with_global_group_levels(mut self, levels: Vec<String>) -> Self {
        self.global_group_levels = Some(levels);
        self
    }

    pub(crate) fn global_group_levels(&self) -> Option<&[String]> {
        self.global_group_levels.as_deref()
    }

    pub(crate) fn with_global_collective_pairs(
        mut self,
        identities: Vec<crate::data::CollectivePair>,
    ) -> Self {
        self.global_collective_pairs = Some(identities);
        self
    }

    pub(crate) fn global_collective_pairs(&self) -> Option<&[crate::data::CollectivePair]> {
        self.global_collective_pairs.as_deref()
    }

    pub(crate) fn with_global_line_condition_levels(mut self, levels: Vec<String>) -> Self {
        self.global_line_condition_levels = Some(levels);
        self
    }

    pub(crate) fn global_line_condition_levels(&self) -> Option<&[String]> {
        self.global_line_condition_levels.as_deref()
    }

    pub(crate) fn hue_color_index(&self, label: &str, default: usize) -> usize {
        self.global_hue_levels
            .as_ref()
            .and_then(|levels| levels.iter().position(|level| level == label))
            .unwrap_or(default)
    }

    pub(crate) fn with_render_scope(mut self, scope: impl Into<String>) -> Self {
        self.render_scope = Some(scope.into());
        self
    }

    pub(crate) fn with_parent_axis_gutters(mut self) -> Self {
        self.axis_gutter_owner = super::AxisGutterOwner::Composition;
        self
    }

    pub(crate) fn layer_name(&self, name: &str) -> String {
        self.render_scope
            .as_ref()
            .map_or_else(|| name.to_owned(), |scope| format!("{scope}-{name}"))
    }

    pub(crate) fn with_canvas_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}
