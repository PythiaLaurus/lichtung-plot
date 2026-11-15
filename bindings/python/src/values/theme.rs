//! Complete native appearance values and immutable typography derivation.

use lichtung_plot as core;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::{is_omitted, omitted};
use crate::native_error;

#[pyclass(
    name = "Color",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyColor {
    pub(crate) native: core::Color,
}

#[pymethods]
impl PyColor {
    #[new]
    #[pyo3(signature = (red, green, blue, *, alpha = 255))]
    fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            native: core::Color::rgba(red, green, blue, alpha),
        }
    }

    #[staticmethod]
    fn hex(value: &str) -> PyResult<Self> {
        Ok(Self {
            native: core::Color::hex(value).map_err(native_error)?,
        })
    }

    #[getter]
    fn red(&self) -> u8 {
        self.native.red()
    }

    #[getter]
    fn green(&self) -> u8 {
        self.native.green()
    }

    #[getter]
    fn blue(&self) -> u8 {
        self.native.blue()
    }

    #[getter]
    fn alpha(&self) -> u8 {
        self.native.get_alpha()
    }

    #[pyo3(signature = (*, red = omitted(), green = omitted(), blue = omitted(), alpha = omitted()),
        text_signature = "($self, *, red=..., green=..., blue=..., alpha=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        red: Py<PyAny>,
        green: Py<PyAny>,
        blue: Py<PyAny>,
        alpha: Py<PyAny>,
    ) -> PyResult<Self> {
        let red = &red.into_bound(py);
        let green = &green.into_bound(py);
        let blue = &blue.into_bound(py);
        let alpha = &alpha.into_bound(py);
        Ok(Self::new(
            if is_omitted(red) {
                self.native.red()
            } else {
                red.extract()?
            },
            if is_omitted(green) {
                self.native.green()
            } else {
                green.extract()?
            },
            if is_omitted(blue) {
                self.native.blue()
            } else {
                blue.extract()?
            },
            if is_omitted(alpha) {
                self.native.get_alpha()
            } else {
                alpha.extract()?
            },
        ))
    }
}

#[pyclass(
    name = "OpaqueColor",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyOpaqueColor {
    pub(crate) native: core::OpaqueColor,
}

#[pymethods]
impl PyOpaqueColor {
    #[new]
    fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            native: core::OpaqueColor::rgb(red, green, blue),
        }
    }

    #[staticmethod]
    fn hex(value: &str) -> PyResult<Self> {
        Ok(Self {
            native: core::OpaqueColor::hex(value).map_err(native_error)?,
        })
    }

    #[getter]
    fn red(&self) -> u8 {
        self.native.red()
    }

    #[getter]
    fn green(&self) -> u8 {
        self.native.green()
    }

    #[getter]
    fn blue(&self) -> u8 {
        self.native.blue()
    }

    #[pyo3(signature = (*, red = omitted(), green = omitted(), blue = omitted()),
        text_signature = "($self, *, red=..., green=..., blue=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        red: Py<PyAny>,
        green: Py<PyAny>,
        blue: Py<PyAny>,
    ) -> PyResult<Self> {
        let red = &red.into_bound(py);
        let green = &green.into_bound(py);
        let blue = &blue.into_bound(py);
        Ok(Self::new(
            if is_omitted(red) {
                self.native.red()
            } else {
                red.extract()?
            },
            if is_omitted(green) {
                self.native.green()
            } else {
                green.extract()?
            },
            if is_omitted(blue) {
                self.native.blue()
            } else {
                blue.extract()?
            },
        ))
    }
}

fn generic_family(value: &str) -> PyResult<core::GenericFontFamily> {
    match value {
        "serif" => Ok(core::GenericFontFamily::Serif),
        "sans_serif" => Ok(core::GenericFontFamily::SansSerif),
        "monospace" => Ok(core::GenericFontFamily::Monospace),
        _ => Err(PyValueError::new_err(
            "generic_family must be 'serif', 'sans_serif', or 'monospace'",
        )),
    }
}

fn generic_family_name(value: core::GenericFontFamily) -> &'static str {
    match value {
        core::GenericFontFamily::Serif => "serif",
        core::GenericFontFamily::SansSerif => "sans_serif",
        core::GenericFontFamily::Monospace => "monospace",
    }
}

#[pyclass(
    name = "FontFamily",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyFontFamily {
    pub(crate) native: core::FontFamily,
}

impl PyFontFamily {
    fn from_parts(name: Option<String>, generic: Option<String>) -> PyResult<Self> {
        let native = match (name, generic) {
            (Some(name), None) => core::FontFamily::named(name).map_err(native_error)?,
            (None, Some(generic)) => core::FontFamily::generic(generic_family(&generic)?),
            _ => {
                return Err(PyTypeError::new_err(
                    "FontFamily requires exactly one of name and generic_family",
                ))
            }
        };
        Ok(Self { native })
    }
}

#[pymethods]
impl PyFontFamily {
    #[new]
    #[pyo3(signature = (name = None, *, generic_family = None))]
    fn new(name: Option<String>, generic_family: Option<String>) -> PyResult<Self> {
        Self::from_parts(name, generic_family)
    }

    #[getter]
    fn name(&self) -> Option<&str> {
        self.native.name()
    }

    #[getter]
    fn generic_family(&self) -> Option<&'static str> {
        self.native.generic_family().map(generic_family_name)
    }

    #[pyo3(signature = (*, name = omitted(), generic_family = omitted()),
        text_signature = "($self, *, name=..., generic_family=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        name: Py<PyAny>,
        generic_family: Py<PyAny>,
    ) -> PyResult<Self> {
        let name = &name.into_bound(py);
        let generic_family = &generic_family.into_bound(py);
        Self::from_parts(
            if is_omitted(name) {
                self.native.name().map(str::to_owned)
            } else {
                name.extract()?
            },
            if is_omitted(generic_family) {
                self.generic_family().map(str::to_owned)
            } else {
                generic_family.extract()?
            },
        )
    }
}

#[pyclass(
    name = "FontStack",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyFontStack {
    pub(crate) native: core::FontStack,
}

impl PyFontStack {
    fn from_bound(families: &Bound<'_, PyAny>) -> PyResult<Self> {
        let families: Vec<PyRef<'_, PyFontFamily>> = families.extract()?;
        Ok(Self {
            native: core::FontStack::new(families.iter().map(|family| family.native.clone()))
                .map_err(native_error)?,
        })
    }
}

#[pymethods]
impl PyFontStack {
    #[new]
    fn new(families: &Bound<'_, PyAny>) -> PyResult<Self> {
        Self::from_bound(families)
    }

    #[getter]
    fn families<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let families = self
            .native
            .families()
            .iter()
            .map(|native| {
                Py::new(
                    py,
                    PyFontFamily {
                        native: native.clone(),
                    },
                )
            })
            .collect::<PyResult<Vec<_>>>()?;
        PyTuple::new(py, families)
    }

    #[pyo3(signature = (*, families = omitted()),
        text_signature = "($self, *, families=...)")]
    fn copy_with(&self, py: Python<'_>, families: Py<PyAny>) -> PyResult<Self> {
        let families = &families.into_bound(py);
        if is_omitted(families) {
            Ok(self.clone())
        } else {
            Self::from_bound(families)
        }
    }
}

#[pyclass(
    name = "RelativeSpacing",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyRelativeSpacing {
    pub(crate) native: core::RelativeSpacing,
}

#[pymethods]
impl PyRelativeSpacing {
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        Ok(Self {
            native: core::RelativeSpacing::new(value).map_err(native_error)?,
        })
    }

    #[staticmethod]
    fn ratio(numerator: u32, denominator: u32) -> PyResult<Self> {
        Ok(Self {
            native: core::RelativeSpacing::ratio(numerator, denominator).map_err(native_error)?,
        })
    }

    /// Nearest binary64 inspection, not the exact rational identity.
    #[getter]
    fn value(&self) -> f64 {
        self.native.as_f64()
    }

    #[pyo3(signature = (*, value = omitted()),
        text_signature = "($self, *, value=...)")]
    fn copy_with(&self, py: Python<'_>, value: Py<PyAny>) -> PyResult<Self> {
        let value = &value.into_bound(py);
        if is_omitted(value) {
            Ok(self.clone())
        } else {
            Self::new(value.extract()?)
        }
    }
}

fn text_weight(value: &str) -> PyResult<core::TextWeight> {
    match value {
        "normal" => Ok(core::TextWeight::Normal),
        "semibold" => Ok(core::TextWeight::Semibold),
        "bold" => Ok(core::TextWeight::Bold),
        _ => Err(PyValueError::new_err(
            "weight must be 'normal', 'semibold', or 'bold'",
        )),
    }
}

fn text_weight_name(value: core::TextWeight) -> &'static str {
    match value {
        core::TextWeight::Normal => "normal",
        core::TextWeight::Semibold => "semibold",
        core::TextWeight::Bold => "bold",
    }
}

#[pyclass(
    name = "TextTheme",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyTextTheme {
    pub(crate) native: core::TextTheme,
}

#[pymethods]
impl PyTextTheme {
    #[new]
    #[pyo3(signature = (*, stack, color, font_size, weight, line_height))]
    fn new(
        stack: PyRef<'_, PyFontStack>,
        color: PyRef<'_, PyColor>,
        font_size: f64,
        weight: &str,
        line_height: f64,
    ) -> PyResult<Self> {
        let native_stack = stack.native.clone();
        let native_color = color.native;
        drop(stack);
        drop(color);
        Ok(Self {
            native: core::TextTheme::new(
                native_stack,
                native_color,
                font_size,
                text_weight(weight)?,
                line_height,
            )
            .map_err(native_error)?,
        })
    }

    #[getter]
    fn stack(&self) -> PyFontStack {
        PyFontStack {
            native: self.native.get_stack().clone(),
        }
    }

    #[getter]
    fn color(&self) -> PyColor {
        PyColor {
            native: self.native.get_color(),
        }
    }

    #[getter]
    fn font_size(&self) -> f64 {
        self.native.get_font_size()
    }

    #[getter]
    fn weight(&self) -> &'static str {
        text_weight_name(self.native.get_weight())
    }

    #[getter]
    fn line_height(&self) -> f64 {
        self.native.get_line_height()
    }

    #[pyo3(signature = (*, stack = omitted(), color = omitted(), font_size = omitted(), weight = omitted(), line_height = omitted()),
        text_signature = "($self, *, stack=..., color=..., font_size=..., weight=..., line_height=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        stack: Py<PyAny>,
        color: Py<PyAny>,
        font_size: Py<PyAny>,
        weight: Py<PyAny>,
        line_height: Py<PyAny>,
    ) -> PyResult<Self> {
        let stack = &stack.into_bound(py);
        let color = &color.into_bound(py);
        let font_size = &font_size.into_bound(py);
        let weight = &weight.into_bound(py);
        let line_height = &line_height.into_bound(py);
        let stack = if is_omitted(stack) {
            self.native.get_stack().clone()
        } else {
            stack.extract::<PyRef<'_, PyFontStack>>()?.native.clone()
        };
        let color = if is_omitted(color) {
            self.native.get_color()
        } else {
            color.extract::<PyRef<'_, PyColor>>()?.native
        };
        let font_size = if is_omitted(font_size) {
            self.native.get_font_size()
        } else {
            font_size.extract()?
        };
        let weight = if is_omitted(weight) {
            self.native.get_weight()
        } else {
            text_weight(weight.extract::<&str>()?)?
        };
        let line_height = if is_omitted(line_height) {
            self.native.get_line_height()
        } else {
            line_height.extract()?
        };
        Ok(Self {
            native: core::TextTheme::new(stack, color, font_size, weight, line_height)
                .map_err(native_error)?,
        })
    }
}

macro_rules! text_roles {
    ($wrapper:ident, $name:literal, $native:ident,
        {$($field:ident: $child:ident => $get:ident),+ $(,)?}) => {
        #[pyclass(name = $name, module = "lichtung_plot", frozen, eq, skip_from_py_object)]
        #[derive(Clone, PartialEq)]
        pub(crate) struct $wrapper {
            pub(crate) native: core::$native,
        }

        #[pymethods]
        impl $wrapper {
            #[new]
            #[pyo3(signature = (*, $($field),+))]
            fn new($($field: PyRef<'_, $child>),+) -> PyResult<Self> {
                Ok(Self {
                    native: core::$native::new($($field.native.clone()),+)
                        .map_err(native_error)?,
                })
            }

            $(
                #[getter]
                fn $field(&self) -> $child {
                    $child { native: self.native.$get().clone() }
                }
            )+

            #[pyo3(signature = (*, $($field = omitted()),+))]
            #[allow(
                clippy::too_many_arguments,
                reason = "complete native role groups expose independently omissible fields"
            )]
            fn copy_with(&self, py: Python<'_>, $($field: Py<PyAny>),+) -> PyResult<Self> {
                $(
                    let $field = &$field.into_bound(py);
                    let $field = if is_omitted($field) {
                        self.native.$get().clone()
                    } else {
                        $field.extract::<PyRef<'_, $child>>()?.native.clone()
                    };
                )+
                Ok(Self { native: core::$native::new($($field),+).map_err(native_error)? })
            }
        }
    };
}

text_roles!(PyRootTextRoles, "RootTextRoles", RootTextRoles, {
    title: PyTextTheme => get_title,
    subtitle: PyTextTheme => get_subtitle,
    caption: PyTextTheme => get_caption,
});
text_roles!(PyAxisTextRoles, "AxisTextRoles", AxisTextRoles, {
    axis_title: PyTextTheme => get_axis_title,
    tick_label: PyTextTheme => get_tick_label,
});
text_roles!(PyGuideTextRoles, "GuideTextRoles", GuideTextRoles, {
    legend_title: PyTextTheme => get_legend_title,
    legend_label: PyTextTheme => get_legend_label,
});
text_roles!(PyCompositionTextRoles, "CompositionTextRoles", CompositionTextRoles, {
    facet_header: PyTextTheme => get_facet_header,
    composition_header: PyTextTheme => get_composition_header,
    text_panel_title: PyTextTheme => get_text_panel_title,
    body: PyTextTheme => get_body,
});
text_roles!(PyContrastTextRoles, "ContrastTextRoles", ContrastTextRoles, {
    light: PyTextTheme => get_light,
    dark: PyTextTheme => get_dark,
});
text_roles!(PyAnnotationTextRoles, "AnnotationTextRoles", AnnotationTextRoles, {
    significance_label: PyTextTheme => get_significance_label,
    estimate_detail: PyTextTheme => get_estimate_detail,
    waterfall_total: PyTextTheme => get_waterfall_total,
    point_layer_label: PyTextTheme => get_point_layer_label,
    panel_text: PyTextTheme => get_panel_text,
    contrast: PyContrastTextRoles => get_contrast,
});
text_roles!(PyTypographyTheme, "TypographyTheme", TypographyTheme, {
    root: PyRootTextRoles => get_root,
    axes: PyAxisTextRoles => get_axes,
    guides: PyGuideTextRoles => get_guides,
    composition: PyCompositionTextRoles => get_composition,
    annotations: PyAnnotationTextRoles => get_annotations,
});

#[pyclass(
    name = "SurfaceTheme",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq)]
pub(crate) struct PySurfaceTheme {
    pub(crate) native: core::SurfaceTheme,
}

#[pymethods]
impl PySurfaceTheme {
    #[new]
    #[pyo3(signature = (*, canvas_root, plot_panel))]
    fn new(canvas_root: PyRef<'_, PyColor>, plot_panel: PyRef<'_, PyColor>) -> PyResult<Self> {
        let native_canvas_root = canvas_root.native;
        let native_plot_panel = plot_panel.native;
        drop(canvas_root);
        drop(plot_panel);
        Ok(Self {
            native: core::SurfaceTheme::new(native_canvas_root, native_plot_panel)
                .map_err(native_error)?,
        })
    }

    #[getter]
    fn canvas_root(&self) -> PyColor {
        PyColor {
            native: self.native.get_canvas_root(),
        }
    }

    #[getter]
    fn plot_panel(&self) -> PyColor {
        PyColor {
            native: self.native.get_plot_panel(),
        }
    }

    #[pyo3(signature = (*, canvas_root = omitted(), plot_panel = omitted()),
        text_signature = "($self, *, canvas_root=..., plot_panel=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        canvas_root: Py<PyAny>,
        plot_panel: Py<PyAny>,
    ) -> PyResult<Self> {
        let canvas_root = &canvas_root.into_bound(py);
        let plot_panel = &plot_panel.into_bound(py);
        let canvas_root = if is_omitted(canvas_root) {
            self.native.get_canvas_root()
        } else {
            canvas_root.extract::<PyRef<'_, PyColor>>()?.native
        };
        let plot_panel = if is_omitted(plot_panel) {
            self.native.get_plot_panel()
        } else {
            plot_panel.extract::<PyRef<'_, PyColor>>()?.native
        };
        Ok(Self {
            native: core::SurfaceTheme::new(canvas_root, plot_panel).map_err(native_error)?,
        })
    }
}

/// Complete native Theme. Unbound components are retained, never mirrored.
#[pyclass(
    name = "Theme",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq)]
pub(crate) struct PyTheme {
    pub(crate) native: core::Theme,
}

impl PyTheme {
    fn derive(
        original: &core::Theme,
        typography: &Bound<'_, PyAny>,
        surfaces: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let mut builder = original.customize();
        if !is_omitted(typography) {
            builder = builder.typography(
                typography
                    .extract::<PyRef<'_, PyTypographyTheme>>()?
                    .native
                    .clone(),
            );
        }
        if !is_omitted(surfaces) {
            builder = builder.surfaces(
                surfaces
                    .extract::<PyRef<'_, PySurfaceTheme>>()?
                    .native
                    .clone(),
            );
        }
        Ok(Self {
            native: builder.build().map_err(native_error)?,
        })
    }
}

#[pymethods]
impl PyTheme {
    /// Start with publication appearance and replace supplied complete components.
    #[new]
    #[pyo3(signature = (*, typography = omitted(), surfaces = omitted()),
        text_signature = "(*, typography=..., surfaces=...)")]
    fn new(py: Python<'_>, typography: Py<PyAny>, surfaces: Py<PyAny>) -> PyResult<Self> {
        let typography = &typography.into_bound(py);
        let surfaces = &surfaces.into_bound(py);
        Self::derive(&core::Theme::publication(), typography, surfaces)
    }

    #[staticmethod]
    fn publication() -> Self {
        Self {
            native: core::Theme::publication(),
        }
    }

    #[staticmethod]
    fn nature() -> Self {
        Self {
            native: core::Theme::nature(),
        }
    }

    #[staticmethod]
    fn science() -> Self {
        Self {
            native: core::Theme::science(),
        }
    }

    #[staticmethod]
    fn chi() -> Self {
        Self {
            native: core::Theme::chi(),
        }
    }

    #[getter]
    fn typography(&self) -> PyTypographyTheme {
        PyTypographyTheme {
            native: self.native.typography().clone(),
        }
    }

    #[getter]
    fn surfaces(&self) -> PySurfaceTheme {
        PySurfaceTheme {
            native: self.native.surfaces().clone(),
        }
    }

    #[pyo3(signature = (*, typography = omitted(), surfaces = omitted()),
        text_signature = "($self, *, typography=..., surfaces=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        typography: Py<PyAny>,
        surfaces: Py<PyAny>,
    ) -> PyResult<Self> {
        let typography = &typography.into_bound(py);
        let surfaces = &surfaces.into_bound(py);
        Self::derive(&self.native, typography, surfaces)
    }
}

pub(super) fn register_components(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyColor>()?;
    m.add_class::<PyOpaqueColor>()?;
    m.add_class::<PyFontFamily>()?;
    m.add_class::<PyFontStack>()?;
    m.add_class::<PyRelativeSpacing>()?;
    m.add_class::<PyRootTextRoles>()?;
    m.add_class::<PyAxisTextRoles>()?;
    m.add_class::<PyGuideTextRoles>()?;
    m.add_class::<PyCompositionTextRoles>()?;
    m.add_class::<PyContrastTextRoles>()?;
    m.add_class::<PyAnnotationTextRoles>()?;
    m.add_class::<PyTypographyTheme>()?;
    m.add_class::<PySurfaceTheme>()
}
