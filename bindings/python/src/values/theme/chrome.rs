//! Complete axis, guide and composition records through public native readers.

use lichtung_plot as core;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::paint::{PyOutlineTheme, PyStrokeTheme};
use super::{is_omitted, omitted, PyColor, PyRelativeSpacing};
use crate::native_error;

theme_record!(PyFrameSides, "FrameSides", FrameSides, {
    top: bool => get_top (bool) [scalar],
    right: bool => get_right (bool) [scalar],
    bottom: bool => get_bottom (bool) [scalar],
    left: bool => get_left (bool) [scalar],
});

theme_record!(PyFrameTheme, "FrameTheme", FrameTheme, {
    sides: PyRef<'_, PyFrameSides> => get_sides (PyFrameSides) [borrow PyFrameSides],
    stroke: PyRef<'_, PyStrokeTheme> => get_stroke (PyStrokeTheme) [borrow PyStrokeTheme],
});

theme_record!(PyTickTheme, "TickTheme", TickTheme, {
    stroke: PyRef<'_, PyStrokeTheme> => get_stroke (PyStrokeTheme) [borrow PyStrokeTheme],
    length: u32 => get_length (u32) [scalar],
});

theme_record!(PyAxisSpacing, "AxisSpacing", AxisSpacing, {
    x_tick_label_gap: PyRef<'_, PyRelativeSpacing> => get_x_tick_label_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    y_tick_label_gap: PyRef<'_, PyRelativeSpacing> => get_y_tick_label_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    x_title_tick_label_gap: PyRef<'_, PyRelativeSpacing> => get_x_title_tick_label_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    x_title_without_tick_labels_gap: PyRef<'_, PyRelativeSpacing> => get_x_title_without_tick_labels_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    y_title_tick_label_gap: PyRef<'_, PyRelativeSpacing> => get_y_title_tick_label_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    y_title_edge_padding: PyRef<'_, PyRelativeSpacing> => get_y_title_edge_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyAxisTheme, "AxisTheme", AxisTheme, {
    frame: Option<PyRef<'_, PyFrameTheme>> = None => get_frame (Option<PyFrameTheme>) [optional PyFrameTheme],
    cell_frame: Option<PyRef<'_, PyFrameTheme>> = None => get_cell_frame (Option<PyFrameTheme>) [optional PyFrameTheme],
    ticks: Option<PyRef<'_, PyTickTheme>> = None => get_ticks (Option<PyTickTheme>) [optional PyTickTheme],
    grid: Option<PyRef<'_, PyStrokeTheme>> = None => get_grid (Option<PyStrokeTheme>) [optional PyStrokeTheme],
    spacing: PyRef<'_, PyAxisSpacing> => get_spacing (PyAxisSpacing) [borrow PyAxisSpacing],
});

theme_record!(PyLegendSpacing, "LegendSpacing", LegendSpacing, {
    padding: PyRef<'_, PyRelativeSpacing> => get_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
    key_width: PyRef<'_, PyRelativeSpacing> => get_key_width (PyRelativeSpacing) [borrow PyRelativeSpacing],
    key_label_gap: PyRef<'_, PyRelativeSpacing> => get_key_label_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    outside_gap: PyRef<'_, PyRelativeSpacing> => get_outside_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    shared_gap: PyRef<'_, PyRelativeSpacing> => get_shared_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    row_gap: PyRef<'_, PyRelativeSpacing> => get_row_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    column_gap: PyRef<'_, PyRelativeSpacing> => get_column_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    title_gap: PyRef<'_, PyRelativeSpacing> => get_title_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    glyph_edge_padding: PyRef<'_, PyRelativeSpacing> => get_glyph_edge_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyGradientLengths, "GradientLengths", GradientLengths, {
    minimum: PyRef<'_, PyRelativeSpacing> => get_minimum (PyRelativeSpacing) [borrow PyRelativeSpacing],
    preferred: PyRef<'_, PyRelativeSpacing> => get_preferred (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyLegendFrameTheme, "LegendFrameTheme", LegendFrameTheme, {
    fill: PyRef<'_, PyColor> => get_fill (PyColor) [value PyColor],
    outline: Option<PyRef<'_, PyOutlineTheme>> = None => get_outline (Option<PyOutlineTheme>) [optional PyOutlineTheme],
    radius: PyRef<'_, PyRelativeSpacing> => get_radius (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyLegendTheme, "LegendTheme", LegendTheme, {
    spacing: PyRef<'_, PyLegendSpacing> => get_spacing (PyLegendSpacing) [borrow PyLegendSpacing],
    gradient_lengths: PyRef<'_, PyGradientLengths> => get_gradient_lengths (PyGradientLengths) [borrow PyGradientLengths],
    frame: Option<PyRef<'_, PyLegendFrameTheme>> = None => get_frame (Option<PyLegendFrameTheme>) [optional PyLegendFrameTheme],
});

theme_record!(PyRootTitleSpacing, "RootTitleSpacing", RootTitleSpacing, {
    title_subtitle_gap: PyRef<'_, PyRelativeSpacing> => get_title_subtitle_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    title_block_to_content_clearance: PyRef<'_, PyRelativeSpacing> => get_title_block_to_content_clearance (PyRelativeSpacing) [borrow PyRelativeSpacing],
    caption_to_content_clearance: PyRef<'_, PyRelativeSpacing> => get_caption_to_content_clearance (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyRootPadding, "RootPadding", RootPadding, {
    left: PyRef<'_, PyRelativeSpacing> => get_left (PyRelativeSpacing) [borrow PyRelativeSpacing],
    right: PyRef<'_, PyRelativeSpacing> => get_right (PyRelativeSpacing) [borrow PyRelativeSpacing],
    bottom: PyRef<'_, PyRelativeSpacing> => get_bottom (PyRelativeSpacing) [borrow PyRelativeSpacing],
    top: PyRef<'_, PyRelativeSpacing> => get_top (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyTextPanelSpacing, "TextPanelSpacing", TextPanelSpacing, {
    inset: PyRef<'_, PyRelativeSpacing> => get_inset (PyRelativeSpacing) [borrow PyRelativeSpacing],
    gap: PyRef<'_, PyRelativeSpacing> => get_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyHeaderSpacing, "HeaderSpacing", HeaderSpacing, {
    column_header_padding: PyRef<'_, PyRelativeSpacing> => get_column_header_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
    row_header_padding: PyRef<'_, PyRelativeSpacing> => get_row_header_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
    row_title_horizontal_padding: PyRef<'_, PyRelativeSpacing> => get_row_title_horizontal_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
    row_title_vertical_padding: PyRef<'_, PyRelativeSpacing> => get_row_title_vertical_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyFacetStripSpacing, "FacetStripSpacing", FacetStripSpacing, {
    horizontal_inset: PyRef<'_, PyRelativeSpacing> => get_horizontal_inset (PyRelativeSpacing) [borrow PyRelativeSpacing],
    content_clearance: PyRef<'_, PyRelativeSpacing> => get_content_clearance (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

fn alignment(value: &str) -> PyResult<core::TextAlignment> {
    match value {
        "start" => Ok(core::TextAlignment::Start),
        "center" => Ok(core::TextAlignment::Center),
        "end" => Ok(core::TextAlignment::End),
        _ => Err(PyValueError::new_err(
            "alignment must be 'start', 'center', or 'end'",
        )),
    }
}

fn alignment_name(value: core::TextAlignment) -> &'static str {
    match value {
        core::TextAlignment::Start => "start",
        core::TextAlignment::Center => "center",
        core::TextAlignment::End => "end",
    }
}

theme_record!(PyCompositionTheme, "CompositionTheme", CompositionTheme, {
    root_title_alignment: &str => get_root_title_alignment (&'static str) [choice alignment alignment_name],
    root_subtitle_alignment: Option<&str> = None => get_root_subtitle_alignment (Option<&'static str>) [optional_choice alignment alignment_name],
    root_caption_alignment: &str => get_root_caption_alignment (&'static str) [choice alignment alignment_name],
    root_title_block: PyRef<'_, PyRootTitleSpacing> => get_root_title_block (PyRootTitleSpacing) [borrow PyRootTitleSpacing],
    root_padding: PyRef<'_, PyRootPadding> => get_root_padding (PyRootPadding) [borrow PyRootPadding],
    panel_edge_clearance: PyRef<'_, PyRelativeSpacing> => get_panel_edge_clearance (PyRelativeSpacing) [borrow PyRelativeSpacing],
    text_panel: PyRef<'_, PyTextPanelSpacing> => get_text_panel (PyTextPanelSpacing) [borrow PyTextPanelSpacing],
    facet_gap: PyRef<'_, PyRelativeSpacing> => get_facet_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    canvas_grid_gap: PyRef<'_, PyRelativeSpacing> => get_canvas_grid_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    headers: PyRef<'_, PyHeaderSpacing> => get_headers (PyHeaderSpacing) [borrow PyHeaderSpacing],
    facet_strip: PyRef<'_, PyFacetStripSpacing> => get_facet_strip (PyFacetStripSpacing) [borrow PyFacetStripSpacing],
});

pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFrameSides>()?;
    m.add_class::<PyFrameTheme>()?;
    m.add_class::<PyTickTheme>()?;
    m.add_class::<PyAxisSpacing>()?;
    m.add_class::<PyAxisTheme>()?;
    m.add_class::<PyLegendSpacing>()?;
    m.add_class::<PyGradientLengths>()?;
    m.add_class::<PyLegendFrameTheme>()?;
    m.add_class::<PyLegendTheme>()?;
    m.add_class::<PyRootTitleSpacing>()?;
    m.add_class::<PyRootPadding>()?;
    m.add_class::<PyTextPanelSpacing>()?;
    m.add_class::<PyHeaderSpacing>()?;
    m.add_class::<PyFacetStripSpacing>()?;
    m.add_class::<PyCompositionTheme>()
}
