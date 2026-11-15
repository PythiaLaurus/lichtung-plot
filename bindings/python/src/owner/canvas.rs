//! Canvas captures native configurations, never external Python Plot objects.

use std::path::PathBuf;
use std::sync::Arc;

use lichtung_plot::{Canvas, Text};
use pyo3::exceptions::PyMemoryError;
use pyo3::prelude::*;
use pyo3::{PyTraverseError, PyVisit};

use super::{cleared_owner, native_snapshot, policy, NativePlot, PyPlotOwner};
use crate::callbacks::{self, clone_roots, CallbackRoots};
use crate::output::{PyOutput, Terminal};
use crate::values::{self, PyLabels, PyPngOptions, PyTextLayout, PyTheme, TextIntent};

struct PanelRoots {
    anchor: (usize, usize),
    roots: CallbackRoots,
}

#[pyclass(name = "_CanvasOwner", module = "lichtung_plot._native")]
pub(crate) struct PyCanvasOwner {
    native: Option<Arc<Canvas>>,
    // Only actual Python references live here; core placements own all topology.
    roots: Vec<PanelRoots>,
}

#[derive(Clone, Copy)]
enum TextSlot {
    Title,
    Subtitle,
    Caption,
}

fn fresh_text(intent: TextIntent) -> Text {
    let mut text = intent.native;
    if intent.clear_style {
        text = text.clear_style();
    }
    if intent.clear_layout {
        text = text.clear_layout();
    }
    text
}

impl PyCanvasOwner {
    fn update(
        slf: &Bound<'_, Self>,
        edit: impl FnOnce(Canvas) -> lichtung_plot::Result<Canvas>,
    ) -> PyResult<()> {
        let retired = {
            let mut owner = slf.try_borrow_mut()?;
            let current = native_snapshot(owner.native.as_ref())?;
            let candidate = edit(current.as_ref().clone()).map_err(crate::native_error)?;
            owner.native.replace(Arc::new(candidate))
        };
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
        Self::update(slf, |canvas| {
            Ok(match slot {
                TextSlot::Title => {
                    let mut canvas = canvas.title(intent.native);
                    if intent.clear_style {
                        canvas = canvas.clear_title_style();
                    }
                    if intent.clear_layout {
                        canvas = canvas.clear_title_layout();
                    }
                    canvas
                }
                TextSlot::Subtitle => {
                    let mut canvas = canvas.subtitle(intent.native);
                    if intent.clear_style {
                        canvas = canvas.clear_subtitle_style();
                    }
                    if intent.clear_layout {
                        canvas = canvas.clear_subtitle_layout();
                    }
                    canvas
                }
                TextSlot::Caption => {
                    let mut canvas = canvas.caption(intent.native);
                    if intent.clear_style {
                        canvas = canvas.clear_caption_style();
                    }
                    if intent.clear_layout {
                        canvas = canvas.clear_caption_layout();
                    }
                    canvas
                }
            })
        })
    }

    fn publish(
        slf: &Bound<'_, Self>,
        path: &Bound<'_, PyAny>,
        terminal: Terminal,
    ) -> PyResult<PyOutput> {
        let path = path.extract::<PathBuf>()?;
        let (native, pins) = {
            let owner = slf.try_borrow()?;
            let native = native_snapshot(owner.native.as_ref())?;
            let count = owner.roots.iter().try_fold(0_usize, |count, panel| {
                count
                    .checked_add(panel.roots.len())
                    .ok_or_else(|| PyMemoryError::new_err("callback root count overflows storage"))
            })?;
            let mut pins = Vec::new();
            pins.try_reserve(count)
                .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
            pins.extend(
                owner
                    .roots
                    .iter()
                    .flat_map(|panel| panel.roots.iter())
                    .map(|root| root.clone_ref(slf.py())),
            );
            (native, pins)
        };
        callbacks::with_operation(slf.py(), pins, move || terminal.canvas(&native, &path))
            .map(PyOutput::new)
    }
}

#[pymethods]
impl PyCanvasOwner {
    #[staticmethod]
    fn grid(rows: usize, columns: usize) -> PyResult<Self> {
        Ok(Self {
            native: Some(Arc::new(
                Canvas::grid(rows, columns).map_err(crate::native_error)?,
            )),
            roots: Vec::new(),
        })
    }

    fn copy(slf: &Bound<'_, Self>) -> PyResult<Self> {
        let mut roots = Vec::new();
        let native = {
            let owner = slf.try_borrow()?;
            let native = native_snapshot(owner.native.as_ref())?;
            roots
                .try_reserve(owner.roots.len())
                .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
            for panel in &owner.roots {
                roots.push(PanelRoots {
                    anchor: panel.anchor,
                    roots: clone_roots(slf.py(), &panel.roots)?,
                });
            }
            native
        };
        Ok(Self {
            native: Some(native),
            roots,
        })
    }

    #[pyo3(signature = (row, column, plot, *, row_span=1, column_span=1, overwrite=false))]
    fn panel(
        slf: &Bound<'_, Self>,
        row: usize,
        column: usize,
        plot: &Bound<'_, PyPlotOwner>,
        row_span: usize,
        column_span: usize,
        overwrite: bool,
    ) -> PyResult<()> {
        let (plot, mut incoming_roots) = plot.try_borrow()?.snapshot(slf.py())?;
        let mut retired = Vec::new();
        let result = (|| {
            let mut owner = slf.try_borrow_mut()?;
            let Self { native, roots } = &mut *owner;
            let native = native.as_mut().ok_or_else(cleared_owner)?;
            // Prepare every binding-owned allocation before core commits placement.
            roots
                .try_reserve(1)
                .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
            retired
                .try_reserve(roots.len())
                .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
            let canvas = Arc::make_mut(native);
            all_families!(call_configs, plot.as_ref(), config => canvas.place_panel(
                row,
                column,
                row_span,
                column_span,
                config.native.clone(),
                overwrite,
            ))
            .map_err(crate::native_error)?;

            let mut index = 0;
            while index < roots.len() {
                let anchor = roots[index].anchor;
                if anchor == (row, column) || !canvas.contains_plot_anchor(anchor.0, anchor.1) {
                    retired.push(roots.swap_remove(index));
                } else {
                    index += 1;
                }
            }
            if !incoming_roots.is_empty() {
                roots.push(PanelRoots {
                    anchor: (row, column),
                    roots: std::mem::take(&mut incoming_roots),
                });
            }
            Ok(())
        })();
        // Do not let callable finalizers run under the mutable Canvas borrow.
        drop(retired);
        drop(incoming_roots);
        result
    }

    fn text_panel(
        slf: &Bound<'_, Self>,
        row: usize,
        column: usize,
        title: &Bound<'_, PyAny>,
        body: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let title = values::text_intent(title, unset, unset, unset)?.native;
        let body = values::text_intent(body, unset, unset, unset)?.native;
        let panel = lichtung_plot::TextPanel::new(title, body).map_err(crate::native_error)?;
        Self::update(slf, |mut canvas| {
            canvas.place_text_panel(row, column, panel)?;
            Ok(canvas)
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

    fn column_header(
        slf: &Bound<'_, Self>,
        start_column: usize,
        span: usize,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let text = fresh_text(values::text_intent(text, style, layout, unset)?);
        Self::update(slf, |canvas| canvas.column_header(start_column, span, text))
    }

    fn row_header(
        slf: &Bound<'_, Self>,
        start_row: usize,
        span: usize,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let text = fresh_text(values::text_intent(text, style, layout, unset)?);
        Self::update(slf, |canvas| canvas.row_header(start_row, span, text))
    }

    fn row_title(
        slf: &Bound<'_, Self>,
        row: usize,
        text: &Bound<'_, PyAny>,
        style: &Bound<'_, PyAny>,
        layout: &Bound<'_, PyAny>,
        unset: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let text = fresh_text(values::text_intent(text, style, layout, unset)?);
        Self::update(slf, |canvas| canvas.row_title(row, text))
    }

    fn theme(slf: &Bound<'_, Self>, theme: PyRef<'_, PyTheme>) -> PyResult<()> {
        let native = theme.native.clone();
        drop(theme);
        Self::update(slf, |canvas| Ok(canvas.theme(native)))
    }

    fn labels(slf: &Bound<'_, Self>, labels: PyRef<'_, PyLabels>) -> PyResult<()> {
        let native = labels.native.clone();
        drop(labels);
        Self::update(slf, |canvas| Ok(canvas.labels(native)))
    }

    fn width(slf: &Bound<'_, Self>, width: u32) -> PyResult<()> {
        Self::update(slf, |canvas| Ok(canvas.width(width)))
    }

    fn max_width(slf: &Bound<'_, Self>, maximum: u32) -> PyResult<()> {
        Self::update(slf, |canvas| Ok(canvas.max_width(maximum)))
    }

    fn size(slf: &Bound<'_, Self>, width: u32, height: u32) -> PyResult<()> {
        Self::update(slf, |canvas| Ok(canvas.size(width, height)))
    }

    fn legend_policy(slf: &Bound<'_, Self>, policy: &str) -> PyResult<()> {
        let policy = policy::canvas_legend(policy)?;
        Self::update(slf, |canvas| Ok(canvas.legend_policy(policy)))
    }

    fn legend_composition(slf: &Bound<'_, Self>, composition: &str) -> PyResult<()> {
        let composition = policy::composition(composition)?;
        Self::update(slf, |canvas| Ok(canvas.legend_composition(composition)))
    }

    fn guide_text_layout(
        slf: &Bound<'_, Self>,
        role: &str,
        layout: PyRef<'_, PyTextLayout>,
    ) -> PyResult<()> {
        let layout_native = layout.native.clone();
        drop(layout);
        let role = policy::guide_role(role)?;
        Self::update(slf, |canvas| {
            Ok(canvas.guide_text_layout(role, layout_native))
        })
    }

    fn panel_title_alignment(slf: &Bound<'_, Self>, alignment: &str) -> PyResult<()> {
        let alignment = policy::alignment(alignment)?;
        Self::update(slf, |canvas| Ok(canvas.panel_title_alignment(alignment)))
    }

    fn panel_subtitle_alignment(slf: &Bound<'_, Self>, alignment: &str) -> PyResult<()> {
        let alignment = policy::alignment(alignment)?;
        Self::update(slf, |canvas| Ok(canvas.panel_subtitle_alignment(alignment)))
    }

    fn category_labels(slf: &Bound<'_, Self>, policy: &str) -> PyResult<()> {
        let policy = policy::canvas_categories(policy)?;
        Self::update(slf, |canvas| Ok(canvas.category_labels(policy)))
    }

    fn x_range_group(slf: &Bound<'_, Self>, cells: Vec<(usize, usize)>) -> PyResult<()> {
        Self::update(slf, |canvas| canvas.x_range_group(cells))
    }

    fn clear_title_style(slf: &Bound<'_, Self>) -> PyResult<()> {
        Self::update(slf, |canvas| Ok(canvas.clear_title_style()))
    }

    fn clear_title_layout(slf: &Bound<'_, Self>) -> PyResult<()> {
        Self::update(slf, |canvas| Ok(canvas.clear_title_layout()))
    }

    fn clear_subtitle_style(slf: &Bound<'_, Self>) -> PyResult<()> {
        Self::update(slf, |canvas| Ok(canvas.clear_subtitle_style()))
    }

    fn clear_subtitle_layout(slf: &Bound<'_, Self>) -> PyResult<()> {
        Self::update(slf, |canvas| Ok(canvas.clear_subtitle_layout()))
    }

    fn clear_caption_style(slf: &Bound<'_, Self>) -> PyResult<()> {
        Self::update(slf, |canvas| Ok(canvas.clear_caption_style()))
    }

    fn clear_caption_layout(slf: &Bound<'_, Self>) -> PyResult<()> {
        Self::update(slf, |canvas| Ok(canvas.clear_caption_layout()))
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
        for root in self.roots.iter().flat_map(|panel| panel.roots.iter()) {
            visit.call(root)?;
        }
        Ok(())
    }

    fn __clear__(slf: &Bound<'_, Self>) -> PyResult<()> {
        let retired = {
            let mut owner = slf.try_borrow_mut()?;
            (owner.native.take(), std::mem::take(&mut owner.roots))
        };
        drop(retired);
        Ok(())
    }
}
