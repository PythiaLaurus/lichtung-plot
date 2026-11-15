//! Independently bound append-only point layers, never reusable Python owners.

use lichtung_plot::{self as core, kind};
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pl;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyBool;

use super::array::Role;
use super::columns::{self, Scalar};
use super::families;
use super::{Config, InputRoots, Prepared};
use crate::values::{text_intent, PyColor, PyTextLayout, TextIntent};

type Input<'a, 'py> = &'a Bound<'py, PyAny>;

#[derive(Clone, Copy)]
pub(crate) struct LinePointLayerArgs<'a, 'py> {
    pub(crate) data: Option<Input<'a, 'py>>,
    pub(crate) x: Input<'a, 'py>,
    pub(crate) y: Input<'a, 'py>,
    pub(crate) labels: Input<'a, 'py>,
    pub(crate) color: Input<'a, 'py>,
    pub(crate) radius: Input<'a, 'py>,
    pub(crate) alpha: Input<'a, 'py>,
    pub(crate) legend_label: Input<'a, 'py>,
    pub(crate) legend_label_style: Input<'a, 'py>,
    pub(crate) legend_label_layout: Input<'a, 'py>,
    pub(crate) label_text_layout: Input<'a, 'py>,
    pub(crate) unset: Input<'a, 'py>,
}

#[derive(Clone, Copy)]
pub(crate) struct BarPointLayerArgs<'a, 'py> {
    pub(crate) data: Option<Input<'a, 'py>>,
    pub(crate) category: Input<'a, 'py>,
    pub(crate) value: Input<'a, 'py>,
    pub(crate) series: Input<'a, 'py>,
    pub(crate) color: Input<'a, 'py>,
    pub(crate) radius: Input<'a, 'py>,
}

pub(crate) fn line_point_layer<'py>(
    py: Python<'py>,
    config: &Config<kind::Line>,
    roots: &InputRoots,
    args: LinePointLayerArgs<'_, 'py>,
) -> PyResult<Prepared<kind::Line>> {
    append_line(py, config, roots, args)
}

pub(crate) fn bar_point_layer<'py>(
    py: Python<'py>,
    config: &Config<kind::Bar>,
    roots: &InputRoots,
    args: BarPointLayerArgs<'_, 'py>,
) -> PyResult<Prepared<kind::Bar>> {
    append_bar(
        py,
        config,
        roots,
        args,
        core::Plot::<kind::Bar>::point_layer,
    )
}

pub(crate) fn horizontal_bar_point_layer<'py>(
    py: Python<'py>,
    config: &Config<kind::HorizontalBar>,
    roots: &InputRoots,
    args: BarPointLayerArgs<'_, 'py>,
) -> PyResult<Prepared<kind::HorizontalBar>> {
    append_bar(
        py,
        config,
        roots,
        args,
        core::Plot::<kind::HorizontalBar>::point_layer,
    )
}

/// These are instructions for one new layer, not owner defaults or retained args.
struct LineOptions {
    color: Option<core::Color>,
    radius: Option<u32>,
    alpha: Option<u8>,
    legend: Option<TextIntent>,
    label_layout: Option<core::TextLayout>,
}

impl LineOptions {
    fn prepare(args: &LinePointLayerArgs<'_, '_>) -> PyResult<Self> {
        if args.legend_label.is_none()
            && (!args.legend_label_style.is(args.unset) || !args.legend_label_layout.is(args.unset))
        {
            return Err(PyTypeError::new_err(
                "legend_label_style and legend_label_layout require legend_label",
            ));
        }
        Ok(Self {
            color: (!args.color.is_none())
                .then(|| {
                    args.color
                        .extract::<PyRef<'_, PyColor>>()
                        .map(|color| color.native)
                })
                .transpose()?,
            radius: (!args.radius.is_none())
                .then(|| radius(args.radius))
                .transpose()?,
            alpha: (!args.alpha.is_none())
                .then(|| {
                    integer(args.alpha, "alpha")?;
                    args.alpha.extract::<u8>()
                })
                .transpose()?,
            legend: (!args.legend_label.is_none())
                .then(|| {
                    text_intent(
                        args.legend_label,
                        args.legend_label_style,
                        args.legend_label_layout,
                        args.unset,
                    )
                })
                .transpose()?,
            label_layout: (!args.label_text_layout.is_none())
                .then(|| {
                    args.label_text_layout
                        .extract::<PyRef<'_, PyTextLayout>>()
                        .map(|layout| layout.native.clone())
                })
                .transpose()?,
        })
    }

    fn apply(self, mut layer: core::PointLayer) -> PyResult<core::PointLayer> {
        if let Some(color) = self.color {
            layer = layer.color(color);
        }
        if let Some(radius) = self.radius {
            layer = layer.radius(radius).map_err(crate::native_error)?;
        }
        if let Some(alpha) = self.alpha {
            layer = layer.alpha(alpha);
        }
        if let Some(legend) = self.legend {
            layer = layer.legend_label(legend.native);
            if legend.clear_style {
                layer = layer.clear_legend_label_style();
            }
            if legend.clear_layout {
                layer = layer.clear_legend_label_layout();
            }
        }
        if let Some(layout) = self.label_layout {
            layer = layer.label_text_layout(layout);
        }
        Ok(layer)
    }
}

fn integer(value: &Bound<'_, PyAny>, name: &str) -> PyResult<()> {
    if value.is_instance_of::<PyBool>() || super::instance(value, "numpy", "bool_")? {
        return Err(PyTypeError::new_err(format!(
            "{name} requires an integer, not bool"
        )));
    }
    Ok(())
}

fn radius(value: &Bound<'_, PyAny>) -> PyResult<u32> {
    integer(value, "radius")?;
    value.extract()
}

fn append_line<'py>(
    py: Python<'py>,
    config: &Config<kind::Line>,
    roots: &InputRoots,
    args: LinePointLayerArgs<'_, 'py>,
) -> PyResult<Prepared<kind::Line>> {
    families::required(&[("x", args.x), ("y", args.y)])?;
    // All incomplete forms and literal instructions precede source discovery.
    let options = LineOptions::prepare(&args)?;
    let (source, mut local_roots) = families::context(args.data)?;
    let polars = columns::polars_route(source.as_ref(), &[args.x, args.y, args.labels])?;
    let mut aliases = Vec::new();
    let x = Scalar::prepare(
        source.as_ref(),
        args.x,
        Role::Numeric,
        "point layer x",
        &mut local_roots,
        &mut aliases,
    )?;
    let y = Scalar::prepare(
        source.as_ref(),
        args.y,
        Role::Numeric,
        "point layer y",
        &mut local_roots,
        &mut aliases,
    )?;
    let layer: core::PointLayer = if polars {
        let x = x.column();
        let y = y.column();
        let mut layer = pl::PointLayerAdapter::new(&x, &y).map_err(super::pl_error)?;
        if !args.labels.is_none() {
            let labels = Scalar::prepare(
                source.as_ref(),
                args.labels,
                Role::Category,
                "point layer labels",
                &mut local_roots,
                &mut aliases,
            )?
            .column();
            layer = layer.labels(&labels).map_err(super::pl_error)?;
        }
        layer.into()
    } else {
        let x = x.numeric("point layer x")?;
        let y = y.numeric("point layer y")?;
        let mut layer = nd::PointLayerAdapter::new(x, y).map_err(super::nd_error)?;
        if !args.labels.is_none() {
            let labels = Scalar::prepare(
                source.as_ref(),
                args.labels,
                Role::Category,
                "point layer labels",
                &mut local_roots,
                &mut aliases,
            )?
            .category("point layer labels")?;
            layer = layer.labels(labels).map_err(super::nd_error)?;
        }
        layer.into()
    };
    let layer = options.apply(layer)?;
    if let Some(source) = &source {
        source.verify()?;
    }
    // Auxiliary field names need no future owner lookup or axis-label aliases.
    // Their source, key registry and all imported buffers end before commit.
    Ok(Prepared {
        config: config
            .clone()
            .map_native(|native| native.point_layer(layer)),
        roots: roots.clone_ref(py),
    })
}

fn append_bar<'py, K: kind::Marker>(
    py: Python<'py>,
    config: &Config<K>,
    roots: &InputRoots,
    args: BarPointLayerArgs<'_, 'py>,
    append: fn(core::Plot<K>, core::BarPointLayer) -> core::Plot<K>,
) -> PyResult<Prepared<K>> {
    families::required(&[
        ("category", args.category),
        ("value", args.value),
        ("color", args.color),
        ("radius", args.radius),
    ])?;
    let color = args.color.extract::<PyRef<'_, PyColor>>()?.native;
    let radius = radius(args.radius)?;
    let (source, mut local_roots) = families::context(args.data)?;
    let polars = columns::polars_route(source.as_ref(), &[args.category, args.value, args.series])?;
    let mut aliases = Vec::new();
    let category = Scalar::prepare(
        source.as_ref(),
        args.category,
        Role::Category,
        "bar point layer category",
        &mut local_roots,
        &mut aliases,
    )?;
    let value = Scalar::prepare(
        source.as_ref(),
        args.value,
        Role::Numeric,
        "bar point layer value",
        &mut local_roots,
        &mut aliases,
    )?;
    let layer: core::BarPointLayer = if polars {
        let category = category.column();
        let value = value.column();
        let mut layer = pl::BarPointLayerAdapter::new(&category, &value, color, radius)
            .map_err(super::pl_error)?;
        if !args.series.is_none() {
            let series = Scalar::prepare(
                source.as_ref(),
                args.series,
                Role::Category,
                "bar point layer series",
                &mut local_roots,
                &mut aliases,
            )?
            .column();
            layer = layer.series(&series).map_err(super::pl_error)?;
        }
        layer.into()
    } else {
        let category = category.category("bar point layer category")?;
        let value = value.numeric("bar point layer value")?;
        // The original constructor owns radius validation, before series binding.
        let mut layer = nd::BarPointLayerAdapter::new(category, value, color, radius)
            .map_err(super::nd_error)?;
        if !args.series.is_none() {
            let series = Scalar::prepare(
                source.as_ref(),
                args.series,
                Role::Category,
                "bar point layer series",
                &mut local_roots,
                &mut aliases,
            )?
            .category("bar point layer series")?;
            layer = layer.series(series).map_err(super::nd_error)?;
        }
        layer.into()
    };
    if let Some(source) = &source {
        source.verify()?;
    }
    Ok(Prepared {
        config: config.clone().map_native(|native| append(native, layer)),
        roots: roots.clone_ref(py),
    })
}
