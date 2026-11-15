//! Mutable Python Plot handles over one exact-family native configuration.

use std::path::PathBuf;
use std::sync::Arc;

use lichtung_plot::{kind, Plot};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use pyo3::{PyTraverseError, PyVisit};

use super::{native_snapshot, CallbackSlot, NativePlot, PlotRoots};
use crate::callbacks::{self, CallbackRoots};
use crate::input;
use crate::output::{PyOutput, Terminal};
use crate::values::numeric::{PyNumericFormat, PyTickGeneration};
use crate::values::{self, PyLabels, PyPngOptions, PyTheme, TextIntent};

#[pyclass(name = "_PlotOwner", module = "lichtung_plot._native")]
pub(crate) struct PyPlotOwner {
    // None has exactly one meaning: GC has cleared this owner.
    native: Option<Arc<NativePlot>>,
    roots: PlotRoots,
    inputs: input::InputRoots,
}

#[derive(Clone, Copy)]
enum TextSlot {
    Title,
    Subtitle,
    Caption,
    XLabel,
    YLabel,
}

impl NativePlot {
    fn map_native(self, edit: impl FnOnce(Plot<kind::Scatter>) -> Plot<kind::Scatter>) -> Self {
        match self {
            Self::Scatter(config) => Self::Scatter(config.map_native(edit)),
        }
    }

    fn text(self, slot: TextSlot, intent: TextIntent) -> Self {
        self.map_native(|plot| match slot {
            TextSlot::Title => {
                let mut plot = plot.title(intent.native);
                if intent.clear_style {
                    plot = plot.clear_title_style();
                }
                if intent.clear_layout {
                    plot = plot.clear_title_layout();
                }
                plot
            }
            TextSlot::Subtitle => {
                let mut plot = plot.subtitle(intent.native);
                if intent.clear_style {
                    plot = plot.clear_subtitle_style();
                }
                if intent.clear_layout {
                    plot = plot.clear_subtitle_layout();
                }
                plot
            }
            TextSlot::Caption => {
                let mut plot = plot.caption(intent.native);
                if intent.clear_style {
                    plot = plot.clear_caption_style();
                }
                if intent.clear_layout {
                    plot = plot.clear_caption_layout();
                }
                plot
            }
            TextSlot::XLabel => {
                let mut plot = plot.x_label(intent.native);
                if intent.clear_style {
                    plot = plot.clear_x_label_style();
                }
                if intent.clear_layout {
                    plot = plot.clear_x_label_layout();
                }
                plot
            }
            TextSlot::YLabel => {
                let mut plot = plot.y_label(intent.native);
                if intent.clear_style {
                    plot = plot.clear_y_label_style();
                }
                if intent.clear_layout {
                    plot = plot.clear_y_label_layout();
                }
                plot
            }
        })
    }
}

impl PyPlotOwner {
    pub(super) fn snapshot(&self, py: Python<'_>) -> PyResult<(Arc<NativePlot>, CallbackRoots)> {
        Ok((native_snapshot(self.native.as_ref())?, self.roots.pins(py)?))
    }

    fn input_snapshot(slf: &Bound<'_, Self>) -> PyResult<(Arc<NativePlot>, input::InputRoots)> {
        let owner = slf.try_borrow()?;
        let native = native_snapshot(owner.native.as_ref())?;
        let roots = owner.inputs.clone_ref(slf.py());
        Ok((native, roots))
    }

    fn commit_input(
        slf: &Bound<'_, Self>,
        expected: &Arc<NativePlot>,
        prepared: input::Prepared<kind::Scatter>,
    ) -> PyResult<()> {
        let incoming = Arc::new(NativePlot::Scatter(prepared.config));
        let mut inputs = Some(prepared.roots);
        let mut retired = None;
        let result = (|| {
            let mut owner = slf.try_borrow_mut()?;
            let current = native_snapshot(owner.native.as_ref())?;
            if !Arc::ptr_eq(&current, expected) {
                return Err(PyRuntimeError::new_err(
                    "plot changed during input preparation; the stale binding was not committed",
                ));
            }
            retired = Some((
                owner.native.replace(incoming),
                std::mem::replace(
                    &mut owner.inputs,
                    inputs.take().expect("prepared inputs are committed once"),
                ),
            ));
            Ok(())
        })();
        // Both rejected and retired Python roots can run reentrant finalizers.
        drop(retired);
        drop(inputs);
        result
    }

    fn update(slf: &Bound<'_, Self>, edit: impl FnOnce(NativePlot) -> NativePlot) -> PyResult<()> {
        let retired = {
            let mut owner = slf.try_borrow_mut()?;
            let current = native_snapshot(owner.native.as_ref())?;
            let candidate = edit(current.as_ref().clone());
            owner.native.replace(Arc::new(candidate))
        };
        drop(retired);
        Ok(())
    }

    fn update_callback(
        slf: &Bound<'_, Self>,
        slot: CallbackSlot,
        roots: CallbackRoots,
        edit: impl FnOnce(NativePlot) -> NativePlot,
    ) -> PyResult<()> {
        let retired = {
            let mut owner = slf.try_borrow_mut()?;
            let current = native_snapshot(owner.native.as_ref())?;
            let candidate = edit(current.as_ref().clone());
            let native = owner.native.replace(Arc::new(candidate));
            let roots = owner.roots.replace(slot, roots);
            (native, roots)
        };
        // A released callable may run a finalizer which reenters this owner.
        drop(retired);
        Ok(())
    }

    fn edit_text(
        slf: &Bound<'_, Self>,
        slot: TextSlot,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let intent = values::text_intent(text, style, layout, unset)?;
        Self::update(slf, |native| native.text(slot, intent))
    }

    fn publish(
        slf: &Bound<'_, Self>,
        path: &Bound<'_, PyAny>,
        terminal: Terminal,
    ) -> PyResult<PyOutput> {
        let path = path.extract::<PathBuf>()?;
        let (native, pins) = {
            let owner = slf.try_borrow()?;
            owner.snapshot(slf.py())?
        };
        callbacks::with_operation(slf.py(), pins, move || match native.as_ref() {
            NativePlot::Scatter(config) => terminal.plot(&config.native, &path),
        })
        .map(PyOutput::new)
    }
}

#[pymethods]
impl PyPlotOwner {
    #[staticmethod]
    #[pyo3(signature = (data=None, *, x, y, hue=None, by_row_position=false, numeric_only=false, unset))]
    fn scatter(
        data: Option<&Bound<'_, PyAny>>,
        x: &Bound<'_, PyAny>,
        y: &Bound<'_, PyAny>,
        hue: Option<&Bound<'_, PyAny>>,
        by_row_position: bool,
        numeric_only: bool,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let prepared = input::scatter(data, x, y, hue, by_row_position, numeric_only, unset)?;
        Ok(Self {
            native: Some(Arc::new(NativePlot::Scatter(prepared.config))),
            roots: PlotRoots::default(),
            inputs: prepared.roots,
        })
    }

    fn copy(slf: &Bound<'_, Self>) -> PyResult<Self> {
        let mut roots = PlotRoots::default();
        let (native, inputs) = {
            let owner = slf.try_borrow()?;
            let native = native_snapshot(owner.native.as_ref())?;
            owner.roots.clone_into(slf.py(), &mut roots)?;
            (native, owner.inputs.clone_ref(slf.py()))
        };
        Ok(Self {
            native: Some(native),
            roots,
            inputs,
        })
    }

    fn title(
        slf: &Bound<'_, Self>,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        Self::edit_text(slf, TextSlot::Title, text, style, layout, unset)
    }

    fn subtitle(
        slf: &Bound<'_, Self>,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        Self::edit_text(slf, TextSlot::Subtitle, text, style, layout, unset)
    }

    fn caption(
        slf: &Bound<'_, Self>,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        Self::edit_text(slf, TextSlot::Caption, text, style, layout, unset)
    }

    fn x_label(
        slf: &Bound<'_, Self>,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        Self::edit_text(slf, TextSlot::XLabel, text, style, layout, unset)
    }

    fn y_label(
        slf: &Bound<'_, Self>,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        Self::edit_text(slf, TextSlot::YLabel, text, style, layout, unset)
    }

    fn theme(slf: &Bound<'_, Self>, theme: PyRef<'_, PyTheme>) -> PyResult<()> {
        let theme_native = theme.native.clone();
        drop(theme);
        Self::update(slf, |native| {
            native.map_native(|plot| plot.theme(theme_native))
        })
    }

    fn labels(slf: &Bound<'_, Self>, labels: PyRef<'_, PyLabels>) -> PyResult<()> {
        let labels_native = labels.native.clone();
        drop(labels);
        Self::update(slf, |native| match native {
            NativePlot::Scatter(config) => NativePlot::Scatter(config.labels(labels_native)),
        })
    }

    fn width(slf: &Bound<'_, Self>, width: u32) -> PyResult<()> {
        Self::update(slf, |native| native.map_native(|plot| plot.width(width)))
    }

    fn max_width(slf: &Bound<'_, Self>, maximum: u32) -> PyResult<()> {
        Self::update(slf, |native| {
            native.map_native(|plot| plot.max_width(maximum))
        })
    }

    fn size(slf: &Bound<'_, Self>, width: u32, height: u32) -> PyResult<()> {
        Self::update(slf, |native| {
            native.map_native(|plot| plot.size(width, height))
        })
    }

    fn hue(slf: &Bound<'_, Self>, values: &Bound<'_, PyAny>) -> PyResult<()> {
        let legacy_native =
            if values.is_instance_of::<PyList>() || values.is_instance_of::<PyTuple>() {
                match slf.try_borrow() {
                    Ok(owner) => match owner.native.as_deref() {
                        Some(NativePlot::Scatter(config)) => config.is_core(),
                        None => true,
                    },
                    // Keep routing diagnostics behind the existing native
                    // conversion; update reports the owner failure afterwards.
                    Err(_) => true,
                }
            } else {
                false
            };
        if legacy_native {
            let values = input::category(values, "hue")?;
            return Self::update(slf, |native| native.map_native(|plot| plot.hue(values)));
        }
        let (expected, roots) = Self::input_snapshot(slf)?;
        let prepared = match expected.as_ref() {
            NativePlot::Scatter(config) => input::hue(config, &roots, values)?,
        };
        Self::commit_input(slf, &expected, prepared)
    }

    fn source_rows(slf: &Bound<'_, Self>, values: &Bound<'_, PyAny>) -> PyResult<()> {
        let (expected, roots) = Self::input_snapshot(slf)?;
        let prepared = match expected.as_ref() {
            NativePlot::Scatter(config) => input::source_rows(config, &roots, values)?,
        };
        Self::commit_input(slf, &expected, prepared)
    }

    fn x_limits(slf: &Bound<'_, Self>, minimum: f64, maximum: f64) -> PyResult<()> {
        Self::update(slf, |native| {
            native.map_native(|plot| plot.x_limits(minimum, maximum))
        })
    }

    fn y_limits(slf: &Bound<'_, Self>, minimum: f64, maximum: f64) -> PyResult<()> {
        Self::update(slf, |native| {
            native.map_native(|plot| plot.y_limits(minimum, maximum))
        })
    }

    fn x_ticks(slf: &Bound<'_, Self>, ticks: &Bound<'_, PyAny>) -> PyResult<()> {
        let ticks = input::ticks(ticks)?;
        Self::update(slf, |native| native.map_native(|plot| plot.x_ticks(ticks)))
    }

    fn y_ticks(slf: &Bound<'_, Self>, ticks: &Bound<'_, PyAny>) -> PyResult<()> {
        let ticks = input::ticks(ticks)?;
        Self::update(slf, |native| native.map_native(|plot| plot.y_ticks(ticks)))
    }

    fn x_tick_format(slf: &Bound<'_, Self>, format: PyRef<'_, PyNumericFormat>) -> PyResult<()> {
        let (native, roots) = format.snapshot(slf.py())?;
        drop(format);
        Self::update_callback(slf, CallbackSlot::XFormat, roots, |plot| {
            plot.map_native(|plot| plot.x_tick_format(native))
        })
    }

    fn y_tick_format(slf: &Bound<'_, Self>, format: PyRef<'_, PyNumericFormat>) -> PyResult<()> {
        let (native, roots) = format.snapshot(slf.py())?;
        drop(format);
        Self::update_callback(slf, CallbackSlot::YFormat, roots, |plot| {
            plot.map_native(|plot| plot.y_tick_format(native))
        })
    }

    fn x_tick_generation(
        slf: &Bound<'_, Self>,
        generation: PyRef<'_, PyTickGeneration>,
    ) -> PyResult<()> {
        let (native, roots) = generation.snapshot(slf.py())?;
        drop(generation);
        Self::update_callback(slf, CallbackSlot::XTick, roots, |plot| {
            plot.map_native(|plot| plot.x_tick_generation(native))
        })
    }

    fn y_tick_generation(
        slf: &Bound<'_, Self>,
        generation: PyRef<'_, PyTickGeneration>,
    ) -> PyResult<()> {
        let (native, roots) = generation.snapshot(slf.py())?;
        drop(generation);
        Self::update_callback(slf, CallbackSlot::YTick, roots, |plot| {
            plot.map_native(|plot| plot.y_tick_generation(native))
        })
    }

    fn svg(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<PyOutput> {
        Self::publish(slf, path, Terminal::Svg)
    }

    fn svg_preserve_text(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<PyOutput> {
        Self::publish(slf, path, Terminal::SvgPreserveText)
    }

    fn png(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<PyOutput> {
        Self::publish(slf, path, Terminal::Png)
    }

    fn png_preview(slf: &Bound<'_, Self>, path: &Bound<'_, PyAny>) -> PyResult<PyOutput> {
        Self::publish(slf, path, Terminal::PngPreview)
    }

    fn png_with_options(
        slf: &Bound<'_, Self>,
        path: &Bound<'_, PyAny>,
        options: PyRef<'_, PyPngOptions>,
    ) -> PyResult<PyOutput> {
        let native = options.native;
        drop(options);
        Self::publish(slf, path, Terminal::PngWithOptions(native))
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "PyO3 GC traversal requires an owned PyVisit argument"
    )]
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        self.roots.traverse(&visit)?;
        self.inputs.traverse(&visit)
    }

    fn __clear__(slf: &Bound<'_, Self>) -> PyResult<()> {
        let retired = {
            let mut owner = slf.try_borrow_mut()?;
            (
                owner.native.take(),
                std::mem::take(&mut owner.roots),
                std::mem::take(&mut owner.inputs),
            )
        };
        drop(retired);
        Ok(())
    }
}
