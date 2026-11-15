//! Shared Text preparation and read-only native layout values.

use lichtung_plot as core;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyString;

use super::theme::PyTextTheme;
use super::{is_omitted, omitted};
use crate::native_error;

/// Transient owner-slot commands, not state carried by an immutable Text.
pub(crate) struct TextIntent {
    pub(crate) native: core::Text,
    pub(crate) clear_style: bool,
    pub(crate) clear_layout: bool,
}

fn prepare_instructions(
    mut native: core::Text,
    explicit_style: Option<&Bound<'_, PyAny>>,
    explicit_layout: Option<&Bound<'_, PyAny>>,
) -> PyResult<TextIntent> {
    let mut clear_style = false;
    let mut clear_layout = false;
    if let Some(style) = explicit_style {
        if style.is_none() {
            native = native.clear_style();
            clear_style = true;
        } else {
            native = native.style(style.extract::<PyRef<'_, PyTextTheme>>()?.native.clone());
        }
    }
    if let Some(layout) = explicit_layout {
        if layout.is_none() {
            native = native.clear_layout();
            clear_layout = true;
        } else {
            native = native.layout(layout.extract::<PyRef<'_, PyTextLayout>>()?.native.clone());
        }
    }
    Ok(TextIntent {
        native,
        clear_style,
        clear_layout,
    })
}

/// Prepare one str/Text entry before the owner commits any edit.
///
/// The supplied private sentinel means omission by identity; None means clear.
/// The owner must apply returned clear commands through its native slot methods.
pub(crate) fn text_intent(
    value: &Bound<'_, PyAny>,
    style: &Bound<'_, PyAny>,
    layout: &Bound<'_, PyAny>,
    unset: &Bound<'_, PyAny>,
) -> PyResult<TextIntent> {
    let native = if value.is_instance_of::<PyString>() {
        core::Text::new(value.extract::<String>()?)
    } else if let Ok(value) = value.extract::<PyRef<'_, PyText>>() {
        value.native.clone()
    } else {
        return Err(PyTypeError::new_err("text must be str or Text"));
    };
    prepare_instructions(
        native,
        (!style.is(unset)).then_some(style),
        (!layout.is(unset)).then_some(layout),
    )
}

#[pyclass(
    name = "Text",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyText {
    pub(crate) native: core::Text,
}

#[pymethods]
impl PyText {
    #[new]
    #[pyo3(signature = (source, *, style = None, layout = None))]
    fn new(
        source: String,
        style: Option<PyRef<'_, PyTextTheme>>,
        layout: Option<PyRef<'_, PyTextLayout>>,
    ) -> Self {
        let mut native = core::Text::new(source);
        if let Some(style) = style {
            native = native.style(style.native.clone());
        }
        if let Some(layout) = layout {
            native = native.layout(layout.native.clone());
        }
        Self { native }
    }

    #[getter]
    fn source(&self) -> &str {
        self.native.as_str()
    }

    #[getter]
    fn style(&self) -> Option<PyTextTheme> {
        self.native
            .get_style()
            .cloned()
            .map(|native| PyTextTheme { native })
    }

    #[getter]
    fn layout(&self) -> Option<PyTextLayout> {
        self.native
            .get_layout()
            .cloned()
            .map(|native| PyTextLayout { native })
    }

    #[pyo3(signature = (*, source = omitted(), style = omitted(), layout = omitted()),
        text_signature = "($self, *, source=..., style=..., layout=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        source: Py<PyAny>,
        style: Py<PyAny>,
        layout: Py<PyAny>,
    ) -> PyResult<Self> {
        let source = &source.into_bound(py);
        let style = &style.into_bound(py);
        let layout = &layout.into_bound(py);
        let native = if is_omitted(source) {
            self.native.clone()
        } else {
            let mut native = core::Text::new(source.extract::<String>()?);
            if let Some(style) = self.native.get_style() {
                native = native.style(style.clone());
            }
            if let Some(layout) = self.native.get_layout() {
                native = native.layout(layout.clone());
            }
            native
        };
        let intent = prepare_instructions(
            native,
            (!is_omitted(style)).then_some(style),
            (!is_omitted(layout)).then_some(layout),
        )?;
        Ok(Self {
            native: intent.native,
        })
    }
}

macro_rules! string_choice {
    ($parse:ident, $inspect:ident, $native:ty, $field:literal,
        {$($name:literal => $variant:path),+ $(,)?}) => {
        fn $parse(value: &str) -> PyResult<$native> {
            match value {
                $($name => Ok($variant),)+
                _ => Err(PyValueError::new_err(concat!(
                    $field, " must be one of: ", $($name, " ",)+
                ))),
            }
        }

        fn $inspect(value: $native) -> &'static str {
            match value {
                $($variant => $name,)+
            }
        }
    };
}

string_choice!(alignment, alignment_name, core::TextAlignment, "alignment", {
    "start" => core::TextAlignment::Start,
    "center" => core::TextAlignment::Center,
    "end" => core::TextAlignment::End,
});
string_choice!(direction, direction_name, core::TextDirection, "direction", {
    "auto" => core::TextDirection::Auto,
    "left_to_right" => core::TextDirection::LeftToRight,
    "right_to_left" => core::TextDirection::RightToLeft,
});
string_choice!(wrapping, wrapping_name, core::TextWrapping, "wrapping", {
    "legal" => core::TextWrapping::Legal,
    "no_soft_wrap" => core::TextWrapping::NoSoftWrap,
});
string_choice!(overlap, overlap_name, core::TextOverlap, "overlap", {
    "require_non_overlap" => core::TextOverlap::RequireNonOverlap,
    "allow" => core::TextOverlap::Allow,
});
string_choice!(fit_priority, fit_priority_name, core::TextFitPriority, "fit_priority", {
    "content_first" => core::TextFitPriority::ContentFirst,
    "preferred_size_first" => core::TextFitPriority::PreferredSizeFirst,
});
string_choice!(rotation_mode, rotation_mode_name, core::TextRotationMode, "rotation_mode", {
    "rotate_about_anchor" => core::TextRotationMode::RotateAboutAnchor,
    "align_after_rotation" => core::TextRotationMode::AlignAfterRotation,
});
string_choice!(offset_frame, offset_frame_name, core::TextOffsetFrame, "frame", {
    "screen" => core::TextOffsetFrame::Screen,
    "text_local" => core::TextOffsetFrame::TextLocal,
});

mod fit;
mod layout;
mod orientation;
mod position;

use fit::PyTextFit;
use layout::{PyTextAnchor, PyTextEllipsis, PyTextOverflow, PyTextVisibility};
use orientation::{PyTextOrientation, PyTextRotation};
pub(super) use position::PyDataCoordinate;
use position::PyDataPosition;

fn optional_choice<T>(
    value: &Bound<'_, PyAny>,
    inherited: Option<T>,
    parse: fn(&str) -> PyResult<T>,
) -> PyResult<Option<T>> {
    if is_omitted(value) {
        Ok(inherited)
    } else if value.is_none() {
        Ok(None)
    } else {
        parse(value.extract::<&str>()?).map(Some)
    }
}

#[pyclass(
    name = "TextOffset",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyTextOffset {
    pub(crate) native: core::TextOffset,
}

#[pymethods]
impl PyTextOffset {
    #[new]
    #[pyo3(signature = (dx, dy, *, frame = "screen"))]
    fn new(dx: f64, dy: f64, frame: &str) -> PyResult<Self> {
        Ok(Self {
            native: core::TextOffset::new(dx, dy, offset_frame(frame)?).map_err(native_error)?,
        })
    }

    #[getter]
    fn dx(&self) -> f64 {
        self.native.get_dx()
    }

    #[getter]
    fn dy(&self) -> f64 {
        self.native.get_dy()
    }

    #[getter]
    fn frame(&self) -> &'static str {
        offset_frame_name(self.native.get_frame())
    }

    #[pyo3(signature = (*, dx = omitted(), dy = omitted(), frame = omitted()),
        text_signature = "($self, *, dx=..., dy=..., frame=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        dx: Py<PyAny>,
        dy: Py<PyAny>,
        frame: Py<PyAny>,
    ) -> PyResult<Self> {
        let dx = &dx.into_bound(py);
        let dy = &dy.into_bound(py);
        let frame = &frame.into_bound(py);
        let dx = if is_omitted(dx) {
            self.native.get_dx()
        } else {
            dx.extract()?
        };
        let dy = if is_omitted(dy) {
            self.native.get_dy()
        } else {
            dy.extract()?
        };
        let frame = if is_omitted(frame) {
            self.native.get_frame()
        } else {
            offset_frame(frame.extract::<&str>()?)?
        };
        Ok(Self {
            native: core::TextOffset::new(dx, dy, frame).map_err(native_error)?,
        })
    }
}

#[pyclass(
    name = "TextBounds",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyTextBounds {
    pub(crate) native: core::TextBounds,
}

#[pymethods]
impl PyTextBounds {
    #[new]
    #[pyo3(signature = (*, max_width = None, max_height = None, max_lines = None))]
    fn new(
        max_width: Option<f64>,
        max_height: Option<f64>,
        max_lines: Option<usize>,
    ) -> PyResult<Self> {
        Ok(Self {
            native: core::TextBounds::new(max_width, max_height, max_lines)
                .map_err(native_error)?,
        })
    }

    #[getter]
    fn max_width(&self) -> Option<f64> {
        self.native.get_max_width()
    }

    #[getter]
    fn max_height(&self) -> Option<f64> {
        self.native.get_max_height()
    }

    #[getter]
    fn max_lines(&self) -> Option<usize> {
        self.native.get_max_lines()
    }

    #[pyo3(signature = (*, max_width = omitted(), max_height = omitted(), max_lines = omitted()),
        text_signature = "($self, *, max_width=..., max_height=..., max_lines=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        max_width: Py<PyAny>,
        max_height: Py<PyAny>,
        max_lines: Py<PyAny>,
    ) -> PyResult<Self> {
        let max_width = &max_width.into_bound(py);
        let max_height = &max_height.into_bound(py);
        let max_lines = &max_lines.into_bound(py);
        Self::new(
            if is_omitted(max_width) {
                self.native.get_max_width()
            } else {
                max_width.extract()?
            },
            if is_omitted(max_height) {
                self.native.get_max_height()
            } else {
                max_height.extract()?
            },
            if is_omitted(max_lines) {
                self.native.get_max_lines()
            } else {
                max_lines.extract()?
            },
        )
    }
}

/// Complete native request, with thirteen independently optional instructions.
#[pyclass(
    name = "TextLayout",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyTextLayout {
    pub(crate) native: core::TextLayout,
}

#[pymethods]
impl PyTextLayout {
    #[new]
    #[pyo3(signature = (*, alignment = None, fit_priority = None, direction = None,
        rotation_mode = None, offset = None, bounds = None, wrapping = None, overlap = None,
        fit = None, orientation = None, anchor = None, overflow = None, visibility = None))]
    #[expect(
        clippy::too_many_arguments,
        reason = "each supported native layout instruction is an independent keyword"
    )]
    fn new(
        alignment: Option<&str>,
        fit_priority: Option<&str>,
        direction: Option<&str>,
        rotation_mode: Option<&str>,
        offset: Option<PyRef<'_, PyTextOffset>>,
        bounds: Option<PyRef<'_, PyTextBounds>>,
        wrapping: Option<&str>,
        overlap: Option<&str>,
        fit: Option<PyRef<'_, PyTextFit>>,
        orientation: Option<PyRef<'_, PyTextOrientation>>,
        anchor: Option<PyRef<'_, PyTextAnchor>>,
        overflow: Option<PyRef<'_, PyTextOverflow>>,
        visibility: Option<PyRef<'_, PyTextVisibility>>,
    ) -> PyResult<Self> {
        Ok(Self {
            native: build_layout(
                alignment.map(self::alignment).transpose()?,
                fit.map(|value| value.native),
                fit_priority.map(self::fit_priority).transpose()?,
                direction.map(self::direction).transpose()?,
                orientation.map(|value| value.native.clone()),
                rotation_mode.map(self::rotation_mode).transpose()?,
                anchor.map(|value| value.native),
                offset.map(|value| value.native),
                bounds.map(|value| value.native),
                wrapping.map(self::wrapping).transpose()?,
                overflow.map(|value| value.native.clone()),
                overlap.map(self::overlap).transpose()?,
                visibility.map(|value| value.native.clone()),
            ),
        })
    }

    #[getter]
    fn alignment(&self) -> Option<&'static str> {
        self.native.get_alignment().map(alignment_name)
    }

    #[getter]
    fn fit_priority(&self) -> Option<&'static str> {
        self.native.get_fit_priority().map(fit_priority_name)
    }

    #[getter]
    fn direction(&self) -> Option<&'static str> {
        self.native.get_direction().map(direction_name)
    }

    #[getter]
    fn rotation_mode(&self) -> Option<&'static str> {
        self.native.get_rotation_mode().map(rotation_mode_name)
    }

    #[getter]
    fn offset(&self) -> Option<PyTextOffset> {
        self.native
            .get_offset()
            .map(|native| PyTextOffset { native })
    }

    #[getter]
    fn bounds(&self) -> Option<PyTextBounds> {
        self.native
            .get_bounds()
            .map(|native| PyTextBounds { native })
    }

    #[getter]
    fn wrapping(&self) -> Option<&'static str> {
        self.native.get_wrapping().map(wrapping_name)
    }

    #[getter]
    fn overlap(&self) -> Option<&'static str> {
        self.native.get_overlap().map(overlap_name)
    }

    #[getter]
    fn fit(&self) -> Option<PyTextFit> {
        self.native.get_fit().map(|native| PyTextFit { native })
    }

    #[getter]
    fn orientation(&self) -> Option<PyTextOrientation> {
        self.native
            .get_orientation()
            .cloned()
            .map(|native| PyTextOrientation { native })
    }

    #[getter]
    fn anchor(&self) -> Option<PyTextAnchor> {
        self.native
            .get_anchor()
            .map(|native| PyTextAnchor { native })
    }

    #[getter]
    fn overflow(&self) -> Option<PyTextOverflow> {
        self.native
            .get_overflow()
            .cloned()
            .map(|native| PyTextOverflow { native })
    }

    #[getter]
    fn visibility(&self) -> Option<PyTextVisibility> {
        self.native
            .get_visibility()
            .cloned()
            .map(|native| PyTextVisibility { native })
    }

    #[pyo3(signature = (*, alignment = omitted(), fit_priority = omitted(), direction = omitted(),
        rotation_mode = omitted(), offset = omitted(), bounds = omitted(),
        wrapping = omitted(), overlap = omitted(), fit = omitted(), orientation = omitted(),
        anchor = omitted(), overflow = omitted(), visibility = omitted()),
        text_signature = "($self, *, alignment=..., fit_priority=..., direction=..., rotation_mode=..., offset=..., bounds=..., wrapping=..., overlap=..., fit=..., orientation=..., anchor=..., overflow=..., visibility=...)")]
    #[expect(
        clippy::too_many_arguments,
        reason = "each supported native layout instruction is independently omissible"
    )]
    fn copy_with(
        &self,
        py: Python<'_>,
        alignment: Py<PyAny>,
        fit_priority: Py<PyAny>,
        direction: Py<PyAny>,
        rotation_mode: Py<PyAny>,
        offset: Py<PyAny>,
        bounds: Py<PyAny>,
        wrapping: Py<PyAny>,
        overlap: Py<PyAny>,
        fit: Py<PyAny>,
        orientation: Py<PyAny>,
        anchor: Py<PyAny>,
        overflow: Py<PyAny>,
        visibility: Py<PyAny>,
    ) -> PyResult<Self> {
        let alignment = &alignment.into_bound(py);
        let fit_priority = &fit_priority.into_bound(py);
        let direction = &direction.into_bound(py);
        let rotation_mode = &rotation_mode.into_bound(py);
        let offset = &offset.into_bound(py);
        let bounds = &bounds.into_bound(py);
        let wrapping = &wrapping.into_bound(py);
        let overlap = &overlap.into_bound(py);
        let fit = &fit.into_bound(py);
        let orientation = &orientation.into_bound(py);
        let anchor = &anchor.into_bound(py);
        let overflow = &overflow.into_bound(py);
        let visibility = &visibility.into_bound(py);
        let alignment = optional_choice(alignment, self.native.get_alignment(), self::alignment)?;
        let fit_priority = optional_choice(
            fit_priority,
            self.native.get_fit_priority(),
            self::fit_priority,
        )?;
        let direction = optional_choice(direction, self.native.get_direction(), self::direction)?;
        let rotation_mode = optional_choice(
            rotation_mode,
            self.native.get_rotation_mode(),
            self::rotation_mode,
        )?;
        let offset = if is_omitted(offset) {
            self.native.get_offset()
        } else if offset.is_none() {
            None
        } else {
            Some(offset.extract::<PyRef<'_, PyTextOffset>>()?.native)
        };
        let bounds = if is_omitted(bounds) {
            self.native.get_bounds()
        } else if bounds.is_none() {
            None
        } else {
            Some(bounds.extract::<PyRef<'_, PyTextBounds>>()?.native)
        };
        let wrapping = optional_choice(wrapping, self.native.get_wrapping(), self::wrapping)?;
        let overlap = optional_choice(overlap, self.native.get_overlap(), self::overlap)?;
        let fit = if is_omitted(fit) {
            self.native.get_fit()
        } else {
            fit.extract::<Option<PyRef<'_, PyTextFit>>>()?
                .map(|value| value.native)
        };
        let orientation = if is_omitted(orientation) {
            self.native.get_orientation().cloned()
        } else {
            orientation
                .extract::<Option<PyRef<'_, PyTextOrientation>>>()?
                .map(|value| value.native.clone())
        };
        let anchor = if is_omitted(anchor) {
            self.native.get_anchor()
        } else {
            anchor
                .extract::<Option<PyRef<'_, PyTextAnchor>>>()?
                .map(|value| value.native)
        };
        let overflow = if is_omitted(overflow) {
            self.native.get_overflow().cloned()
        } else {
            overflow
                .extract::<Option<PyRef<'_, PyTextOverflow>>>()?
                .map(|value| value.native.clone())
        };
        let visibility = if is_omitted(visibility) {
            self.native.get_visibility().cloned()
        } else {
            visibility
                .extract::<Option<PyRef<'_, PyTextVisibility>>>()?
                .map(|value| value.native.clone())
        };
        Ok(Self {
            native: build_layout(
                alignment,
                fit,
                fit_priority,
                direction,
                orientation,
                rotation_mode,
                anchor,
                offset,
                bounds,
                wrapping,
                overflow,
                overlap,
                visibility,
            ),
        })
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one call-local reconstruction retains all thirteen native instructions"
)]
fn build_layout(
    alignment: Option<core::TextAlignment>,
    fit: Option<core::TextFit>,
    fit_priority: Option<core::TextFitPriority>,
    direction: Option<core::TextDirection>,
    orientation: Option<core::TextOrientation>,
    rotation_mode: Option<core::TextRotationMode>,
    anchor: Option<core::TextAnchor>,
    offset: Option<core::TextOffset>,
    bounds: Option<core::TextBounds>,
    wrapping: Option<core::TextWrapping>,
    overflow: Option<core::TextOverflow>,
    overlap: Option<core::TextOverlap>,
    visibility: Option<core::TextVisibility>,
) -> core::TextLayout {
    let mut native = core::TextLayout::default();
    if let Some(value) = alignment {
        native = native.alignment(value);
    }
    if let Some(value) = fit {
        native = native.fit(value);
    }
    if let Some(value) = fit_priority {
        native = native.fit_priority(value);
    }
    if let Some(value) = direction {
        native = native.direction(value);
    }
    if let Some(value) = orientation {
        native = native.orientation(value);
    }
    if let Some(value) = rotation_mode {
        native = native.rotation_mode(value);
    }
    if let Some(value) = anchor {
        native = native.anchor(value);
    }
    if let Some(value) = offset {
        native = native.offset(value);
    }
    if let Some(value) = bounds {
        native = native.bounds(value);
    }
    if let Some(value) = wrapping {
        native = native.wrapping(value);
    }
    if let Some(value) = overflow {
        native = native.overflow(value);
    }
    if let Some(value) = overlap {
        native = native.overlap(value);
    }
    if let Some(value) = visibility {
        native = native.visibility(value);
    }
    native
}

pub(super) fn register_components(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTextOffset>()?;
    m.add_class::<PyTextBounds>()?;
    m.add_class::<PyTextFit>()?;
    m.add_class::<PyTextRotation>()?;
    m.add_class::<PyDataCoordinate>()?;
    m.add_class::<PyDataPosition>()?;
    m.add_class::<PyTextOrientation>()?;
    m.add_class::<PyTextAnchor>()?;
    m.add_class::<PyTextEllipsis>()?;
    m.add_class::<PyTextOverflow>()?;
    m.add_class::<PyTextVisibility>()
}

#[cfg(test)]
mod tests {
    use pyo3::types::PyDict;

    use super::*;

    #[test]
    fn layout_copy_clears_one_field_without_losing_opaque_native_requests() {
        Python::initialize();
        Python::attach(|py| -> PyResult<()> {
            let module = PyModule::new(py, "value_contract")?;
            crate::values::register(&module)?;
            let opaque = core::TextLayout::default()
                .fit(core::TextFit::range(8.0, 13.0).map_err(native_error)?)
                .orientation(core::TextOrientation::fixed(core::TextRotation::tilt_up()))
                .anchor(core::TextAnchor::bottom_right())
                .overflow(core::TextOverflow::ellipsis(
                    core::TextEllipsis::new(core::TextElision::Middle).marker(".."),
                    core::TextFailure::Clip,
                ))
                .visibility(
                    core::TextVisibility::all()
                        .boundary_tolerance(Some(2.0))
                        .map_err(native_error)?,
                )
                .direction(core::TextDirection::RightToLeft);
            let original = opaque.clone().alignment(core::TextAlignment::End);
            let value = Py::new(
                py,
                PyTextLayout {
                    native: original.clone(),
                },
            )?;
            let keywords = PyDict::new(py);
            keywords.set_item("alignment", py.None())?;
            let changed = value
                .bind(py)
                .call_method("copy_with", (), Some(&keywords))?;
            assert_eq!(changed.extract::<PyRef<'_, PyTextLayout>>()?.native, opaque);
            assert_eq!(value.bind(py).get().native, original);
            Ok(())
        })
        .unwrap();
    }
}
