//! Exact-family constructor calls. Argument records exist only during preparation.

use lichtung_plot::{self as core, kind, Plot};
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pl;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use super::binding::{self, Source};
use super::columns::{self, Scalar};
use super::{Config, InputRoots, Origin, Prepared};

macro_rules! arguments {
    ($name:ident {
        $first:ident: &$a:lifetime Bound<$py:lifetime, PyAny>
        $(, $field:ident: $ty:ty)* $(,)?
    }) => {
        #[derive(Clone, Copy)]
        pub(crate) struct $name<$a, $py> {
            pub(crate) $first: &$a Bound<$py, PyAny>,
            $(pub(crate) $field: $ty,)*
        }
    };
}

arguments!(DensityArgs {
    value: &'a Bound<'py, PyAny>,
    hue: Option<&'a Bound<'py, PyAny>>,
    group: Option<&'a Bound<'py, PyAny>>,
});
arguments!(QqArgs { value: &'a Bound<'py, PyAny> });
arguments!(PointIntervalArgs {
    category: &'a Bound<'py, PyAny>, estimate: &'a Bound<'py, PyAny>,
    lower: &'a Bound<'py, PyAny>, upper: &'a Bound<'py, PyAny>,
});
arguments!(LineArgs {
    x: &'a Bound<'py, PyAny>, y: &'a Bound<'py, PyAny>,
    hue: Option<&'a Bound<'py, PyAny>>, group: Option<&'a Bound<'py, PyAny>>,
    by_row_position: bool, numeric_only: bool, unset: &'a Bound<'py, PyAny>,
});
arguments!(TrajectoryArgs {
    x: &'a Bound<'py, PyAny>, y: &'a Bound<'py, PyAny>, group: &'a Bound<'py, PyAny>,
    hue: Option<&'a Bound<'py, PyAny>>,
});
arguments!(LinePathArgs {
    x: &'a Bound<'py, PyAny>, y: &'a Bound<'py, PyAny>, group: &'a Bound<'py, PyAny>,
    hue: Option<&'a Bound<'py, PyAny>>,
});
arguments!(InteractionArgs {
    condition: &'a Bound<'py, PyAny>, estimate: &'a Bound<'py, PyAny>,
    group: &'a Bound<'py, PyAny>, lower: &'a Bound<'py, PyAny>,
    upper: &'a Bound<'py, PyAny>, hue: Option<&'a Bound<'py, PyAny>>,
});
arguments!(CoefficientArgs {
    term: &'a Bound<'py, PyAny>, estimate: &'a Bound<'py, PyAny>,
    lower: &'a Bound<'py, PyAny>, upper: &'a Bound<'py, PyAny>,
    unset: &'a Bound<'py, PyAny>,
});
arguments!(ForestArgs {
    term: &'a Bound<'py, PyAny>, estimate: &'a Bound<'py, PyAny>,
    lower: &'a Bound<'py, PyAny>, upper: &'a Bound<'py, PyAny>,
    reference: &'a Bound<'py, PyAny>,
});
pub(crate) struct DumbbellArgs<'a, 'py> {
    pub(crate) category: &'a Bound<'py, PyAny>,
    pub(crate) left: &'a Bound<'py, PyAny>,
    pub(crate) right: &'a Bound<'py, PyAny>,
    pub(crate) endpoints: core::DumbbellEndpoints,
    pub(crate) group: Option<&'a Bound<'py, PyAny>>,
}
arguments!(HeatmapArgs {
    grid: &'a Bound<'py, PyAny>, x: &'a Bound<'py, PyAny>,
    y: &'a Bound<'py, PyAny>, fill: &'a Bound<'py, PyAny>,
    unset: &'a Bound<'py, PyAny>,
});
arguments!(BarArgs {
    category: &'a Bound<'py, PyAny>, samples: &'a Bound<'py, PyAny>,
    estimate: &'a Bound<'py, PyAny>, lower: &'a Bound<'py, PyAny>,
    upper: &'a Bound<'py, PyAny>, error_bar: &'a Bound<'py, PyAny>,
    series: Option<&'a Bound<'py, PyAny>>, unset: &'a Bound<'py, PyAny>,
});
arguments!(HorizontalBarArgs {
    category: &'a Bound<'py, PyAny>, samples: &'a Bound<'py, PyAny>,
    estimate: &'a Bound<'py, PyAny>, lower: &'a Bound<'py, PyAny>,
    upper: &'a Bound<'py, PyAny>, error_bar: &'a Bound<'py, PyAny>,
    series: Option<&'a Bound<'py, PyAny>>, unset: &'a Bound<'py, PyAny>,
});
arguments!(StackedBarArgs {
    category: &'a Bound<'py, PyAny>, value: &'a Bound<'py, PyAny>,
    series: &'a Bound<'py, PyAny>,
});
arguments!(StackedHorizontalBarArgs {
    category: &'a Bound<'py, PyAny>, value: &'a Bound<'py, PyAny>,
    series: &'a Bound<'py, PyAny>,
});
arguments!(BoxplotArgs {
    category: &'a Bound<'py, PyAny>, value: &'a Bound<'py, PyAny>,
    series: Option<&'a Bound<'py, PyAny>>, by_column_name: bool,
    numeric_only: bool, unset: &'a Bound<'py, PyAny>,
});
arguments!(ViolinArgs {
    category: &'a Bound<'py, PyAny>, value: &'a Bound<'py, PyAny>,
    series: Option<&'a Bound<'py, PyAny>>, by_column_name: bool,
    numeric_only: bool, unset: &'a Bound<'py, PyAny>,
});
arguments!(RaincloudArgs {
    category: &'a Bound<'py, PyAny>, value: &'a Bound<'py, PyAny>,
    series: Option<&'a Bound<'py, PyAny>>, by_column_name: bool, numeric_only: bool,
    q1: &'a Bound<'py, PyAny>, median: &'a Bound<'py, PyAny>, q3: &'a Bound<'py, PyAny>,
    density_data: Option<&'a Bound<'py, PyAny>>,
    density_category: &'a Bound<'py, PyAny>, density_y: &'a Bound<'py, PyAny>,
    normalized_width: &'a Bound<'py, PyAny>,
    point_data: Option<&'a Bound<'py, PyAny>>,
    point_category: &'a Bound<'py, PyAny>, point_y: &'a Bound<'py, PyAny>,
    normalized_strip: &'a Bound<'py, PyAny>,
    hue: Option<&'a Bound<'py, PyAny>>, density_hue: Option<&'a Bound<'py, PyAny>>,
    point_hue: Option<&'a Bound<'py, PyAny>>, unset: &'a Bound<'py, PyAny>,
});
arguments!(HistogramArgs {
    samples: &'a Bound<'py, PyAny>,
    bin_lower: &'a Bound<'py, PyAny>, bin_upper: &'a Bound<'py, PyAny>,
    bin_density: &'a Bound<'py, PyAny>,
    kde_data: Option<&'a Bound<'py, PyAny>>,
    kde_x: &'a Bound<'py, PyAny>, kde_y: &'a Bound<'py, PyAny>,
    rug_data: Option<&'a Bound<'py, PyAny>>,
    rug_x: &'a Bound<'py, PyAny>, rug_baseline: &'a Bound<'py, PyAny>,
    rug_top: &'a Bound<'py, PyAny>, unset: &'a Bound<'py, PyAny>,
});
arguments!(RegressionArgs { x: &'a Bound<'py, PyAny>, y: &'a Bound<'py, PyAny> });
arguments!(MarginalArgs {
    x: &'a Bound<'py, PyAny>, y: &'a Bound<'py, PyAny>,
    hue: Option<&'a Bound<'py, PyAny>>, group: Option<&'a Bound<'py, PyAny>>,
});
arguments!(MatrixArgs {
    variables: &'a Bound<'py, PyAny>, numeric_only: bool,
    hue: Option<&'a Bound<'py, PyAny>>, group: Option<&'a Bound<'py, PyAny>>,
    unset: &'a Bound<'py, PyAny>,
});
arguments!(BeeswarmArgs {
    category: &'a Bound<'py, PyAny>, value: &'a Bound<'py, PyAny>,
    color: &'a Bound<'py, PyAny>,
});
arguments!(WaterfallArgs {
    category: &'a Bound<'py, PyAny>, delta: &'a Bound<'py, PyAny>,
    base: &'a Bound<'py, PyAny>,
});

pub(super) fn required(values: &[(&str, &Bound<'_, PyAny>)]) -> PyResult<()> {
    for (name, value) in values {
        if value.is_none() {
            return Err(PyTypeError::new_err(format!(
                "{name} is required and cannot be None"
            )));
        }
    }
    Ok(())
}

pub(super) fn present(value: &Bound<'_, PyAny>, unset: &Bound<'_, PyAny>) -> bool {
    !value.is(unset)
}

pub(super) fn pair(
    lower: &Bound<'_, PyAny>,
    upper: &Bound<'_, PyAny>,
    unset: &Bound<'_, PyAny>,
) -> PyResult<bool> {
    if present(lower, unset) != present(upper, unset) {
        return Err(PyTypeError::new_err(
            "lower and upper must be supplied together",
        ));
    }
    if present(lower, unset) {
        required(&[("lower", lower), ("upper", upper)])?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub(super) fn context<'py>(
    data: Option<&Bound<'py, PyAny>>,
) -> PyResult<(Option<Source<'py>>, InputRoots)> {
    let data = data.filter(|data| !data.is_none());
    let source = data.map(Source::new).transpose()?;
    Ok((
        source,
        InputRoots {
            source: data.map(|data| data.clone().unbind()),
            keys: None,
        },
    ))
}

pub(super) fn finish<K: kind::Marker>(
    config: Config<K>,
    roots: InputRoots,
    aliases: Vec<(String, String)>,
) -> Prepared<K> {
    Prepared {
        config: binding::defaults(config, aliases),
        roots,
    }
}

pub(super) fn from_nd<K: kind::Marker>(adapter: nd::PlotAdapter<'_, K>) -> Config<K> {
    let (native, metadata) = adapter.into_parts();
    Config {
        native,
        origin: Origin::Nd(metadata),
    }
}

pub(super) fn from_pl<K: kind::Marker>(adapter: pl::PlotAdapter<'_, K>) -> Config<K> {
    let (native, metadata) = adapter.into_parts();
    Config {
        native,
        origin: Origin::Polars(metadata),
    }
}

fn native_call(data: Option<&Bound<'_, PyAny>>, values: &[&Bound<'_, PyAny>]) -> PyResult<bool> {
    if data.is_some_and(|data| !data.is_none()) {
        return Ok(false);
    }
    for value in values {
        if !binding::flat_native(value)? {
            return Ok(false);
        }
    }
    Ok(true)
}

// Only scalar records use prebound native columns. Each role was admitted as a
// single variable; these constructors have no wide/grid repetition to bypass.
// Shape-owning constructors and row/shape-sensitive edits use ordinary inputs.
macro_rules! bind_role {
    ($source:expr, $value:expr, $roots:expr, $aliases:expr, $role:ident, numeric) => {
        Scalar::prepare(
            $source,
            $value,
            super::array::Role::Numeric,
            stringify!($role),
            $roots,
            $aliases,
        )?
        .numeric(stringify!($role))
    };
    ($source:expr, $value:expr, $roots:expr, $aliases:expr, $role:ident, category) => {
        Scalar::prepare(
            $source,
            $value,
            super::array::Role::Category,
            stringify!($role),
            $roots,
            $aliases,
        )?
        .category(stringify!($role))
    };
    ($source:expr, $value:expr, $roots:expr, $aliases:expr, $role:ident, axis) => {
        columns::axis($source, $value, $roots, $aliases)
    };
}

macro_rules! record {
    ($function:ident, $args:ident, $kind:ident, $method:ident,
        ($($field:ident: $role:ident),+), ($($policy:ident),*), $native:expr,
        ($($optional:ident),*)) => {
        pub(crate) fn $function(
            data: Option<&Bound<'_, PyAny>>, args: $args<'_, '_>,
        ) -> PyResult<Prepared<kind::$kind>> {
            required(&[$((stringify!($field), args.$field)),+])?;
            $(let $policy = columns::literal(args.$policy, stringify!($policy))?;)*
            let native = native_call(data, &[$(args.$field),+])?;
            let (source, mut roots) = context(data)?;
            let polars = columns::polars_route(source.as_ref(), &[$(args.$field),+])?;
            let mut aliases = Vec::new();
            $(let $field = bind_role!(source.as_ref(), args.$field, &mut roots,
                &mut aliases, $field, $role)?;)+
            let config = if native {
                Config { native: $native, origin: Origin::Core }
            } else if polars {
                from_pl(pl::$kind::$method($($field,)+ $($policy),*).map_err(super::pl_error)?)
            } else {
                from_nd(nd::$kind::$method($($field,)+ $($policy),*).map_err(super::nd_error)?)
            };
            if let Some(source) = &source { source.verify()?; }
            let prepared = finish(config, roots, aliases);
            $(let prepared = if let Some(value) = args.$optional.filter(|value| !value.is_none()) {
                super::$optional(&prepared.config, &prepared.roots, value)?
            } else { prepared };)*
            Ok(prepared)
        }
    };
}

record!(density, DensityArgs, Density, new, (value: numeric), (), Plot::density(value), (hue, group));
record!(qq, QqArgs, Qq, new, (value: numeric), (), Plot::qq(value), ());
record!(point_interval, PointIntervalArgs, PointInterval, new,
    (category: category, estimate: numeric, lower: numeric, upper: numeric), (),
    Plot::point_interval(category, estimate, core::Bounds::new(lower, upper)), ());
record!(trajectory, TrajectoryArgs, Trajectory, new,
    (x: axis, y: numeric, group: category), (), Plot::trajectory(x, y, group), (hue));
record!(line_path, LinePathArgs, LinePath, new,
    (x: numeric, y: numeric, group: category), (), Plot::line_path(x, y, group), (hue));
record!(interaction, InteractionArgs, Interaction, new,
    (condition: category, estimate: numeric, group: category, lower: numeric, upper: numeric), (),
    Plot::interaction(condition, estimate, group, core::Bounds::new(lower, upper)), (hue));
record!(forest, ForestArgs, Forest, new,
    (term: category, estimate: numeric, lower: numeric, upper: numeric), (reference),
    Plot::forest(term, estimate, core::Bounds::new(lower, upper), reference), ());
record!(stacked_bar, StackedBarArgs, StackedBar, new,
    (category: category, value: numeric, series: category), (),
    Plot::stacked_bar(category, value, series), ());
record!(stacked_horizontal_bar, StackedHorizontalBarArgs, StackedHorizontalBar, new,
    (category: category, value: numeric, series: category), (),
    Plot::stacked_horizontal_bar(category, value, series), ());
record!(regression, RegressionArgs, Regression, new,
    (x: numeric, y: numeric), (), Plot::regression(x, y), ());
record!(marginal, MarginalArgs, Marginal, new,
    (x: numeric, y: numeric), (), Plot::marginal(x, y), (hue, group));
record!(beeswarm, BeeswarmArgs, Beeswarm, new,
    (category: category, value: numeric, color: numeric), (), Plot::beeswarm(category, value, color), ());
record!(waterfall, WaterfallArgs, Waterfall, new,
    (category: category, delta: numeric), (base), Plot::waterfall(category, delta, base), ());

pub(crate) fn dumbbell(
    data: Option<&Bound<'_, PyAny>>,
    args: DumbbellArgs<'_, '_>,
) -> PyResult<Prepared<kind::Dumbbell>> {
    required(&[
        ("category", args.category),
        ("left", args.left),
        ("right", args.right),
    ])?;
    let native = native_call(data, &[args.category, args.left, args.right])?;
    let (source, mut roots) = context(data)?;
    let polars = columns::polars_route(source.as_ref(), &[args.category, args.left, args.right])?;
    let mut aliases = Vec::new();
    let category = bind_role!(
        source.as_ref(),
        args.category,
        &mut roots,
        &mut aliases,
        category,
        category
    )?;
    let left = bind_role!(
        source.as_ref(),
        args.left,
        &mut roots,
        &mut aliases,
        left,
        numeric
    )?;
    let right = bind_role!(
        source.as_ref(),
        args.right,
        &mut roots,
        &mut aliases,
        right,
        numeric
    )?;
    let config = if native {
        Config {
            native: Plot::dumbbell(category, left, right, args.endpoints),
            origin: Origin::Core,
        }
    } else if polars {
        from_pl(pl::Dumbbell::new(category, left, right, args.endpoints).map_err(super::pl_error)?)
    } else {
        from_nd(nd::Dumbbell::new(category, left, right, args.endpoints).map_err(super::nd_error)?)
    };
    if let Some(source) = &source {
        source.verify()?;
    }
    let mut prepared = finish(config, roots, aliases);
    if let Some(value) = args.group.filter(|value| !value.is_none()) {
        prepared = super::group(&prepared.config, &prepared.roots, value)?;
    }
    Ok(prepared)
}

pub(crate) fn coefficient(
    data: Option<&Bound<'_, PyAny>>,
    args: CoefficientArgs<'_, '_>,
) -> PyResult<Prepared<kind::Coefficient>> {
    required(&[("term", args.term), ("estimate", args.estimate)])?;
    let intervals = pair(args.lower, args.upper, args.unset)?;
    let mut values = vec![args.term, args.estimate];
    if intervals {
        values.extend([args.lower, args.upper]);
    }
    let native = native_call(data, &values)?;
    let (source, mut roots) = context(data)?;
    let polars = columns::polars_route(source.as_ref(), &values)?;
    let mut aliases = Vec::new();
    let term = bind_role!(
        source.as_ref(),
        args.term,
        &mut roots,
        &mut aliases,
        term,
        category
    )?;
    let estimate = bind_role!(
        source.as_ref(),
        args.estimate,
        &mut roots,
        &mut aliases,
        estimate,
        numeric
    )?;
    let config = if intervals {
        let lower = bind_role!(
            source.as_ref(),
            args.lower,
            &mut roots,
            &mut aliases,
            lower,
            numeric
        )?;
        let upper = bind_role!(
            source.as_ref(),
            args.upper,
            &mut roots,
            &mut aliases,
            upper,
            numeric
        )?;
        if native {
            Config {
                native: Plot::coefficient(
                    term,
                    core::CoefficientInput::estimates_with_intervals(
                        estimate,
                        core::Bounds::new(lower, upper),
                    ),
                ),
                origin: Origin::Core,
            }
        } else if polars {
            from_pl(
                pl::Coefficient::with_intervals(term, estimate, lower, upper)
                    .map_err(super::pl_error)?,
            )
        } else {
            from_nd(
                nd::Coefficient::with_intervals(term, estimate, lower, upper)
                    .map_err(super::nd_error)?,
            )
        }
    } else if native {
        Config {
            native: Plot::coefficient(term, core::CoefficientInput::estimates(estimate)),
            origin: Origin::Core,
        }
    } else if polars {
        from_pl(pl::Coefficient::new(term, estimate).map_err(super::pl_error)?)
    } else {
        from_nd(nd::Coefficient::new(term, estimate).map_err(super::nd_error)?)
    };
    if let Some(source) = &source {
        source.verify()?;
    }
    Ok(finish(config, roots, aliases))
}

macro_rules! bar {
    ($function:ident, $args:ident, $kind:ident, $native:ident) => {
        #[allow(
            clippy::too_many_lines,
            reason = "complete Bar alternatives share one presence decision"
        )]
        pub(crate) fn $function(
            data: Option<&Bound<'_, PyAny>>,
            args: $args<'_, '_>,
        ) -> PyResult<Prepared<kind::$kind>> {
            required(&[("category", args.category)])?;
            let samples = present(args.samples, args.unset);
            if samples == present(args.estimate, args.unset) {
                return Err(PyTypeError::new_err(
                    "supply exactly one of samples and estimate",
                ));
            }
            let intervals = pair(args.lower, args.upper, args.unset)?;
            let policy = present(args.error_bar, args.unset);
            if (samples && intervals) || (!samples && policy) {
                return Err(PyTypeError::new_err(
                    "error_bar is sample-only; endpoints are estimate-only",
                ));
            }
            let value = if samples { args.samples } else { args.estimate };
            required(&[("value", value)])?;
            let policy = if policy {
                Some(crate::values::error_bar(args.error_bar)?)
            } else {
                None
            };
            let mut values = vec![args.category, value];
            if intervals {
                values.extend([args.lower, args.upper]);
            }
            let native = native_call(data, &values)?;
            let (source, mut roots) = context(data)?;
            let polars = columns::polars_route(source.as_ref(), &values)?;
            let mut aliases = Vec::new();
            let category = bind_role!(
                source.as_ref(),
                args.category,
                &mut roots,
                &mut aliases,
                category,
                category
            )?;
            let role = if samples { "samples" } else { "estimate" };
            let value = Scalar::prepare(
                source.as_ref(),
                value,
                super::array::Role::Numeric,
                role,
                &mut roots,
                &mut aliases,
            )?
            .numeric(role)?;
            let config = if samples {
                if native {
                    let mut input = core::BarInput::samples(value);
                    if let Some(policy) = policy {
                        input = input.error_bar(policy);
                    }
                    Config {
                        native: Plot::$native(category, input),
                        origin: Origin::Core,
                    }
                } else if polars {
                    from_pl(
                        if let Some(policy) = policy {
                            pl::$kind::with_error_bar(category, value, policy)
                        } else {
                            pl::$kind::new(category, value)
                        }
                        .map_err(super::pl_error)?,
                    )
                } else {
                    from_nd(
                        if let Some(policy) = policy {
                            nd::$kind::with_error_bar(category, value, policy)
                        } else {
                            nd::$kind::new(category, value)
                        }
                        .map_err(super::nd_error)?,
                    )
                }
            } else if intervals {
                let lower = bind_role!(
                    source.as_ref(),
                    args.lower,
                    &mut roots,
                    &mut aliases,
                    lower,
                    numeric
                )?;
                let upper = bind_role!(
                    source.as_ref(),
                    args.upper,
                    &mut roots,
                    &mut aliases,
                    upper,
                    numeric
                )?;
                if native {
                    Config {
                        native: Plot::$native(
                            category,
                            core::BarInput::estimates_with_error_bars(
                                value,
                                core::Bounds::new(lower, upper),
                            ),
                        ),
                        origin: Origin::Core,
                    }
                } else if polars {
                    from_pl(
                        pl::$kind::estimates_with_error_bars(category, value, lower, upper)
                            .map_err(super::pl_error)?,
                    )
                } else {
                    from_nd(
                        nd::$kind::estimates_with_error_bars(category, value, lower, upper)
                            .map_err(super::nd_error)?,
                    )
                }
            } else if native {
                Config {
                    native: Plot::$native(category, core::BarInput::estimates(value)),
                    origin: Origin::Core,
                }
            } else if polars {
                from_pl(pl::$kind::estimates(category, value).map_err(super::pl_error)?)
            } else {
                from_nd(nd::$kind::estimates(category, value).map_err(super::nd_error)?)
            };
            if let Some(source) = &source {
                source.verify()?;
            }
            let mut prepared = finish(config, roots, aliases);
            if let Some(value) = args.series.filter(|value| !value.is_none()) {
                prepared = super::series(&prepared.config, &prepared.roots, value)?;
            }
            Ok(prepared)
        }
    };
}
bar!(bar, BarArgs, Bar, bar);
bar!(
    horizontal_bar,
    HorizontalBarArgs,
    HorizontalBar,
    horizontal_bar
);
