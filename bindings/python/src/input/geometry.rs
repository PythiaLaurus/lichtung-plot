//! Flat complete geometry: three independent alignment domains, one retained source.

use lichtung_plot::{self as core, kind, Plot};
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pl;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use super::array::Role;
use super::binding;
use super::columns::{self, Scalar};
use super::families::{self, HistogramArgs, RaincloudArgs};
use super::{Config, InputRoots, Origin, Prepared};

fn complete(values: &[(&str, &Bound<'_, PyAny>)], unset: &Bound<'_, PyAny>) -> PyResult<()> {
    for (name, value) in values {
        if value.is(unset) {
            return Err(PyTypeError::new_err(format!(
                "complete geometry requires {name}"
            )));
        }
    }
    families::required(values)
}

#[expect(
    clippy::too_many_lines,
    reason = "complete sample/three-domain forms share one presence decision"
)]
pub(crate) fn histogram(
    data: Option<&Bound<'_, PyAny>>,
    args: HistogramArgs<'_, '_>,
) -> PyResult<Prepared<kind::Histogram>> {
    let fields = [
        ("bin_lower", args.bin_lower),
        ("bin_upper", args.bin_upper),
        ("bin_density", args.bin_density),
        ("kde_x", args.kde_x),
        ("kde_y", args.kde_y),
        ("rug_x", args.rug_x),
        ("rug_baseline", args.rug_baseline),
        ("rug_top", args.rug_top),
    ];
    let geometry = fields
        .iter()
        .any(|(_, value)| families::present(value, args.unset));
    let samples = families::present(args.samples, args.unset);
    if samples == geometry {
        return Err(PyTypeError::new_err(
            "supply samples or complete histogram geometry, exclusively",
        ));
    }
    if !geometry {
        if args.kde_data.is_some_and(|data| !data.is_none())
            || args.rug_data.is_some_and(|data| !data.is_none())
        {
            return Err(PyTypeError::new_err(
                "auxiliary sources belong to complete geometry",
            ));
        }
        families::required(&[("samples", args.samples)])?;
        let (source, mut roots) = families::context(data)?;
        let polars = columns::polars_route(source.as_ref(), &[args.samples])?;
        let mut aliases = Vec::new();
        let samples = Scalar::prepare(
            source.as_ref(),
            args.samples,
            Role::Numeric,
            "samples",
            &mut roots,
            &mut aliases,
        )?
        .numeric("samples")?;
        let config = if data.is_none_or(Bound::is_none) && binding::flat_native(args.samples)? {
            Config {
                native: Plot::histogram(core::HistogramInput::samples(samples)),
                origin: Origin::Core,
            }
        } else if polars {
            families::from_pl(pl::Histogram::new(samples).map_err(super::pl_error)?)
        } else {
            families::from_nd(nd::Histogram::new(samples).map_err(super::nd_error)?)
        };
        if let Some(source) = &source {
            source.verify()?;
        }
        return Ok(families::finish(config, roots, aliases));
    }
    complete(&fields, args.unset)?;
    let (source, mut roots) = families::context(data)?;
    let (kde_source, mut kde_roots) = families::context(args.kde_data)?;
    let (rug_source, mut rug_roots) = families::context(args.rug_data)?;
    let polars = columns::polars_route(
        source.as_ref(),
        &[args.bin_lower, args.bin_upper, args.bin_density],
    )?;
    let mut aliases = Vec::new();
    macro_rules! numeric {
        ($source:expr, $roots:expr, $field:ident) => {
            Scalar::prepare(
                $source,
                args.$field,
                Role::Numeric,
                stringify!($field),
                $roots,
                &mut aliases,
            )?
        };
    }
    let bin_lower = numeric!(source.as_ref(), &mut roots, bin_lower);
    let bin_upper = numeric!(source.as_ref(), &mut roots, bin_upper);
    let bin_density = numeric!(source.as_ref(), &mut roots, bin_density);
    let kde_x = numeric!(kde_source.as_ref(), &mut kde_roots, kde_x);
    let kde_y = numeric!(kde_source.as_ref(), &mut kde_roots, kde_y);
    let rug_x = numeric!(rug_source.as_ref(), &mut rug_roots, rug_x);
    let rug_baseline = numeric!(rug_source.as_ref(), &mut rug_roots, rug_baseline);
    let rug_top = numeric!(rug_source.as_ref(), &mut rug_roots, rug_top);
    // Deliberate internal metadata mapping; display labels never enter it.
    let value_dimension = rug_x.dimension("value");
    let density_dimension = bin_density.dimension("density");
    let bin_lower = bin_lower.numeric("bin_lower")?;
    let bin_upper = bin_upper.numeric("bin_upper")?;
    let bin_density = bin_density.numeric("bin_density")?;
    let kde_x = kde_x.numeric("kde_x")?;
    let kde_y = kde_y.numeric("kde_y")?;
    let rug_x = rug_x.numeric("rug_x")?;
    let rug_baseline = rug_baseline.numeric("rug_baseline")?;
    let rug_top = rug_top.numeric("rug_top")?;
    let config = if polars {
        families::from_pl(
            pl::Histogram::geometry(
                bin_lower,
                bin_upper,
                bin_density,
                kde_x,
                kde_y,
                rug_x,
                rug_baseline,
                rug_top,
                value_dimension,
                density_dimension,
            )
            .map_err(super::pl_error)?,
        )
    } else {
        families::from_nd(
            nd::Histogram::geometry(
                bin_lower,
                bin_upper,
                bin_density,
                kde_x,
                kde_y,
                rug_x,
                rug_baseline,
                rug_top,
                value_dimension,
                density_dimension,
            )
            .map_err(super::nd_error)?,
        )
    };
    for source in [&source, &kde_source, &rug_source].into_iter().flatten() {
        source.verify()?;
    }
    // Auxiliary roots, including their original Python keys, end here.
    Ok(families::finish(config, roots, aliases))
}

#[expect(
    clippy::too_many_lines,
    reason = "complete sample/hue geometry forms keep all domains explicit"
)]
pub(crate) fn raincloud(
    data: Option<&Bound<'_, PyAny>>,
    args: RaincloudArgs<'_, '_>,
) -> PyResult<Prepared<kind::Raincloud>> {
    let fields = [
        ("q1", args.q1),
        ("median", args.median),
        ("q3", args.q3),
        ("density_category", args.density_category),
        ("density_y", args.density_y),
        ("normalized_width", args.normalized_width),
        ("point_category", args.point_category),
        ("point_y", args.point_y),
        ("normalized_strip", args.normalized_strip),
    ];
    let geometry = fields
        .iter()
        .any(|(_, value)| families::present(value, args.unset));
    let hues = [args.hue, args.density_hue, args.point_hue]
        .map(|value| value.filter(|value| !value.is_none()));
    if !geometry {
        if hues.iter().any(Option::is_some)
            || args.density_data.is_some_and(|data| !data.is_none())
            || args.point_data.is_some_and(|data| !data.is_none())
        {
            return Err(PyTypeError::new_err(
                "geometry hue and auxiliary sources exclude sample forms",
            ));
        }
        return super::wide::raincloud_samples(data, &args);
    }
    if families::present(args.value, args.unset)
        || args.by_column_name
        || args.numeric_only
        || args.series.is_some_and(|value| !value.is_none())
    {
        return Err(PyTypeError::new_err(
            "complete raincloud geometry excludes sample inputs and modes",
        ));
    }
    complete(&fields, args.unset)?;
    complete(&[("category", args.category)], args.unset)?;
    let hue_count = hues.iter().filter(|value| value.is_some()).count();
    if hue_count != 0 && hue_count != 3 {
        return Err(PyTypeError::new_err(
            "geometry hue requires hue, density_hue and point_hue together",
        ));
    }
    let (source, mut roots) = families::context(data)?;
    let (density_source, mut density_roots) = families::context(args.density_data)?;
    let (point_source, mut point_roots) = families::context(args.point_data)?;
    let polars = columns::polars_route(
        source.as_ref(),
        &[args.category, args.q1, args.median, args.q3],
    )?;
    let mut aliases = Vec::new();
    macro_rules! column {
        ($source:expr, $roots:expr, $field:ident, $role:ident) => {
            Scalar::prepare(
                $source,
                args.$field,
                Role::$role,
                stringify!($field),
                $roots,
                &mut aliases,
            )?
        };
    }
    let category = column!(source.as_ref(), &mut roots, category, Category);
    let hue = hues[0]
        .map(|value| {
            Scalar::prepare(
                source.as_ref(),
                value,
                Role::Category,
                "hue",
                &mut roots,
                &mut aliases,
            )
        })
        .transpose()?;
    let q1 = column!(source.as_ref(), &mut roots, q1, Numeric).numeric("q1")?;
    let median = column!(source.as_ref(), &mut roots, median, Numeric).numeric("median")?;
    let q3 = column!(source.as_ref(), &mut roots, q3, Numeric).numeric("q3")?;
    let density_category = column!(
        density_source.as_ref(),
        &mut density_roots,
        density_category,
        Category
    )
    .category("density_category")?;
    let density_hue = hues[1]
        .map(|value| {
            Scalar::prepare(
                density_source.as_ref(),
                value,
                Role::Category,
                "density_hue",
                &mut density_roots,
                &mut aliases,
            )?
            .category("density_hue")
        })
        .transpose()?;
    let density_y = column!(
        density_source.as_ref(),
        &mut density_roots,
        density_y,
        Numeric
    )
    .numeric("density_y")?;
    let normalized_width = column!(
        density_source.as_ref(),
        &mut density_roots,
        normalized_width,
        Numeric
    )
    .numeric("normalized_width")?;
    let point_category = column!(
        point_source.as_ref(),
        &mut point_roots,
        point_category,
        Category
    )
    .category("point_category")?;
    let point_hue = hues[2]
        .map(|value| {
            Scalar::prepare(
                point_source.as_ref(),
                value,
                Role::Category,
                "point_hue",
                &mut point_roots,
                &mut aliases,
            )?
            .category("point_hue")
        })
        .transpose()?;
    let point_y = column!(point_source.as_ref(), &mut point_roots, point_y, Numeric);
    let normalized_strip = column!(
        point_source.as_ref(),
        &mut point_roots,
        normalized_strip,
        Numeric
    )
    .numeric("normalized_strip")?;
    let category_dimension = category.dimension("category");
    let value_dimension = point_y.dimension("value");
    let hue_dimension = hue.as_ref().map(|hue| hue.dimension("hue"));
    let category = category.category("category")?;
    let hue = hue.map(|hue| hue.category("hue")).transpose()?;
    let point_y = point_y.numeric("point_y")?;
    let config = match (hue, density_hue, point_hue, hue_dimension) {
        (Some(hue), Some(density_hue), Some(point_hue), Some(hue_dimension)) => {
            if polars {
                families::from_pl(
                    pl::Raincloud::geometry_with_hue(
                        category,
                        hue,
                        q1,
                        median,
                        q3,
                        density_category,
                        density_hue,
                        density_y,
                        normalized_width,
                        point_category,
                        point_hue,
                        point_y,
                        normalized_strip,
                        category_dimension,
                        value_dimension,
                        hue_dimension,
                    )
                    .map_err(super::pl_error)?,
                )
            } else {
                families::from_nd(
                    nd::Raincloud::geometry_with_hue(
                        category,
                        hue,
                        q1,
                        median,
                        q3,
                        density_category,
                        density_hue,
                        density_y,
                        normalized_width,
                        point_category,
                        point_hue,
                        point_y,
                        normalized_strip,
                        category_dimension,
                        value_dimension,
                        hue_dimension,
                    )
                    .map_err(super::nd_error)?,
                )
            }
        }
        (None, None, None, None) => {
            if polars {
                families::from_pl(
                    pl::Raincloud::geometry(
                        category,
                        q1,
                        median,
                        q3,
                        density_category,
                        density_y,
                        normalized_width,
                        point_category,
                        point_y,
                        normalized_strip,
                        category_dimension,
                        value_dimension,
                    )
                    .map_err(super::pl_error)?,
                )
            } else {
                families::from_nd(
                    nd::Raincloud::geometry(
                        category,
                        q1,
                        median,
                        q3,
                        density_category,
                        density_y,
                        normalized_width,
                        point_category,
                        point_y,
                        normalized_strip,
                        category_dimension,
                        value_dimension,
                    )
                    .map_err(super::nd_error)?,
                )
            }
        }
        _ => unreachable!("complete hue combination was checked before lookup"),
    };
    for source in [&source, &density_source, &point_source]
        .into_iter()
        .flatten()
    {
        source.verify()?;
    }
    Ok(families::finish(config, roots, aliases))
}

pub(crate) fn composite_precomputed_kde(
    config: &Config<kind::Histogram>,
    roots: &InputRoots,
    data: Option<&Bound<'_, PyAny>>,
    x: &Bound<'_, PyAny>,
    y: &Bound<'_, PyAny>,
) -> PyResult<Prepared<kind::Histogram>> {
    families::required(&[("KDE x", x), ("KDE y", y)])?;
    let roots = roots.stage(x.py())?;
    let (source, mut auxiliary_roots) = families::context(data)?;
    let mut aliases = Vec::new();
    let x = Scalar::prepare(
        source.as_ref(),
        x,
        Role::Numeric,
        "KDE x",
        &mut auxiliary_roots,
        &mut aliases,
    )?
    .numeric("KDE x")?;
    let y = Scalar::prepare(
        source.as_ref(),
        y,
        Role::Numeric,
        "KDE y",
        &mut auxiliary_roots,
        &mut aliases,
    )?
    .numeric("KDE y")?;
    let config = match &config.origin {
        Origin::Core => config.clone().map_native(|native| {
            native.composite(core::HistogramComposite::KdeAndRug(
                core::HistogramKde::Precomputed { x, y },
            ))
        }),
        Origin::Nd(metadata) => families::from_nd(
            nd::Histogram::from_parts(config.native.clone(), metadata.clone())
                .composite_precomputed_kde(x, y)
                .map_err(super::nd_error)?,
        ),
        Origin::Polars(metadata) => families::from_pl(
            pl::Histogram::from_parts(config.native.clone(), metadata.clone())
                .composite_precomputed_kde(x, y)
                .map_err(super::pl_error)?,
        ),
    };
    Ok(families::finish(config, roots, aliases))
}
