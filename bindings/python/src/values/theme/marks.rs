//! Complete mark and annotation role graphs, with native sibling validation.

use lichtung_plot as core;
use pyo3::prelude::*;

use super::paint::{
    PyDataFillTheme, PyDataLineTheme, PyFixedFillTheme, PyOutlineTheme, PyPointTheme,
    PySolidStrokeTheme, PyStrokeTheme,
};
use super::scales::PyAccentPalette;
use super::{is_omitted, omitted, PyColor, PyOpaqueColor, PyRelativeSpacing};
use crate::native_error;

theme_record!(PySizedPointTheme, "SizedPointTheme", SizedPointTheme, {
    appearance: PyRef<'_, PyPointTheme> => get_appearance (PyPointTheme) [borrow PyPointTheme],
    radius: u32 => get_radius (u32) [scalar],
});

theme_record!(PySupplementalPointTheme, "SupplementalPointTheme", SupplementalPointTheme, {
    radius: u32 => get_radius (u32) [scalar],
    fill_alpha: u8 => get_fill_alpha (u8) [scalar],
});

theme_record!(PyPointRadii, "PointRadii", PointRadii, {
    scatter: u32 => get_scatter (u32) [scalar],
    path: u32 => get_path (u32) [scalar],
    trajectory: u32 => get_trajectory (u32) [scalar],
    regression: u32 => get_regression (u32) [scalar],
    qq: u32 => get_qq (u32) [scalar],
    marginal: u32 => get_marginal (u32) [scalar],
    matrix: u32 => get_matrix (u32) [scalar],
    interval_estimate: u32 => get_interval_estimate (u32) [scalar],
    dumbbell: u32 => get_dumbbell (u32) [scalar],
    interaction: u32 => get_interaction (u32) [scalar],
    beeswarm: u32 => get_beeswarm (u32) [scalar],
    box_flier: u32 => get_box_flier (u32) [scalar],
});

theme_record!(PyPointRoles, "PointRoles", PointRoles, {
    primary: PyRef<'_, PyPointTheme> => get_primary (PyPointTheme) [borrow PyPointTheme],
    raw_overlay: PyRef<'_, PySizedPointTheme> => get_raw_overlay (PySizedPointTheme) [borrow PySizedPointTheme],
    mandatory_strip: PyRef<'_, PySizedPointTheme> => get_mandatory_strip (PySizedPointTheme) [borrow PySizedPointTheme],
    radii: PyRef<'_, PyPointRadii> => get_radii (PyPointRadii) [borrow PyPointRadii],
    supplemental: PyRef<'_, PySupplementalPointTheme> => get_supplemental (PySupplementalPointTheme) [borrow PySupplementalPointTheme],
});

theme_record!(PyBodyFillRoles, "BodyFillRoles", BodyFillRoles, {
    bar_body: PyRef<'_, PyDataFillTheme> => get_bar_body (PyDataFillTheme) [borrow PyDataFillTheme],
    box_body: PyRef<'_, PyDataFillTheme> => get_box_body (PyDataFillTheme) [borrow PyDataFillTheme],
    violin_body: PyRef<'_, PyDataFillTheme> => get_violin_body (PyDataFillTheme) [borrow PyDataFillTheme],
    inner_summary: PyRef<'_, PyFixedFillTheme> => get_inner_summary (PyFixedFillTheme) [borrow PyFixedFillTheme],
    histogram_bin: PyRef<'_, PyDataFillTheme> => get_histogram_bin (PyDataFillTheme) [borrow PyDataFillTheme],
    waterfall_bar: PyRef<'_, PyDataFillTheme> => get_waterfall_bar (PyDataFillTheme) [borrow PyDataFillTheme],
});

theme_record!(PyBandFillRoles, "BandFillRoles", BandFillRoles, {
    line_band: PyRef<'_, PyDataFillTheme> => get_line_band (PyDataFillTheme) [borrow PyDataFillTheme],
    regression_band: PyRef<'_, PyDataFillTheme> => get_regression_band (PyDataFillTheme) [borrow PyDataFillTheme],
});

theme_record!(PyDensityFillRoles, "DensityFillRoles", DensityFillRoles, {
    density_area: PyRef<'_, PyDataFillTheme> => get_density_area (PyDataFillTheme) [borrow PyDataFillTheme],
    marginal_density: PyRef<'_, PyDataFillTheme> => get_marginal_density (PyDataFillTheme) [borrow PyDataFillTheme],
    matrix_density: PyRef<'_, PyDataFillTheme> => get_matrix_density (PyDataFillTheme) [borrow PyDataFillTheme],
});

theme_record!(PyFillRoles, "FillRoles", FillRoles, {
    bodies: PyRef<'_, PyBodyFillRoles> => get_bodies (PyBodyFillRoles) [borrow PyBodyFillRoles],
    bands: PyRef<'_, PyBandFillRoles> => get_bands (PyBandFillRoles) [borrow PyBandFillRoles],
    densities: PyRef<'_, PyDensityFillRoles> => get_densities (PyDensityFillRoles) [borrow PyDensityFillRoles],
});

theme_record!(PyDataLineRoles, "DataLineRoles", DataLineRoles, {
    series_line: PyRef<'_, PyDataLineTheme> => get_series_line (PyDataLineTheme) [borrow PyDataLineTheme],
    path_line: PyRef<'_, PyDataLineTheme> => get_path_line (PyDataLineTheme) [borrow PyDataLineTheme],
    density_line: PyRef<'_, PyDataLineTheme> => get_density_line (PyDataLineTheme) [borrow PyDataLineTheme],
    interaction_interval: PyRef<'_, PyDataLineTheme> => get_interaction_interval (PyDataLineTheme) [borrow PyDataLineTheme],
    interaction_connector: PyRef<'_, PyDataLineTheme> => get_interaction_connector (PyDataLineTheme) [borrow PyDataLineTheme],
});

theme_record!(PyNeutralPaintRoles, "NeutralPaintRoles", NeutralPaintRoles, {
    path: PyRef<'_, PyOpaqueColor> => get_path (PyOpaqueColor) [value PyOpaqueColor],
    box_flier: PyRef<'_, PyOpaqueColor> => get_box_flier (PyOpaqueColor) [value PyOpaqueColor],
});

theme_record!(PyReferenceStrokeRoles, "ReferenceStrokeRoles", ReferenceStrokeRoles, {
    bar_zero: PyRef<'_, PyStrokeTheme> => get_bar_zero (PyStrokeTheme) [borrow PyStrokeTheme],
    waterfall_reference: PyRef<'_, PyStrokeTheme> => get_waterfall_reference (PyStrokeTheme) [borrow PyStrokeTheme],
    beeswarm_reference: PyRef<'_, PyStrokeTheme> => get_beeswarm_reference (PyStrokeTheme) [borrow PyStrokeTheme],
    point_interval_reference: PyRef<'_, PyStrokeTheme> => get_point_interval_reference (PyStrokeTheme) [borrow PyStrokeTheme],
    estimate_reference: PyRef<'_, PyStrokeTheme> => get_estimate_reference (PyStrokeTheme) [borrow PyStrokeTheme],
    qq_reference: PyRef<'_, PyStrokeTheme> => get_qq_reference (PyStrokeTheme) [borrow PyStrokeTheme],
});

theme_record!(PyConnectorStrokeRoles, "ConnectorStrokeRoles", ConnectorStrokeRoles, {
    waterfall_connector: PyRef<'_, PyStrokeTheme> => get_waterfall_connector (PyStrokeTheme) [borrow PyStrokeTheme],
    dumbbell_connector: PyRef<'_, PyStrokeTheme> => get_dumbbell_connector (PyStrokeTheme) [borrow PyStrokeTheme],
});

theme_record!(PySummaryStrokeRoles, "SummaryStrokeRoles", SummaryStrokeRoles, {
    uncertainty: PyRef<'_, PyStrokeTheme> => get_uncertainty (PyStrokeTheme) [borrow PyStrokeTheme],
    summary: PyRef<'_, PyStrokeTheme> => get_summary (PyStrokeTheme) [borrow PyStrokeTheme],
    estimate_interval: PyRef<'_, PyStrokeTheme> => get_estimate_interval (PyStrokeTheme) [borrow PyStrokeTheme],
    median: PyRef<'_, PyStrokeTheme> => get_median (PyStrokeTheme) [borrow PyStrokeTheme],
});

theme_record!(PyCurveStrokeRoles, "CurveStrokeRoles", CurveStrokeRoles, {
    histogram_curve: PyRef<'_, PyStrokeTheme> => get_histogram_curve (PyStrokeTheme) [borrow PyStrokeTheme],
    histogram_rug: PyRef<'_, PyStrokeTheme> => get_histogram_rug (PyStrokeTheme) [borrow PyStrokeTheme],
    regression_fit: PyRef<'_, PyStrokeTheme> => get_regression_fit (PyStrokeTheme) [borrow PyStrokeTheme],
});

theme_record!(PyStrokeRoles, "StrokeRoles", StrokeRoles, {
    references: PyRef<'_, PyReferenceStrokeRoles> => get_references (PyReferenceStrokeRoles) [borrow PyReferenceStrokeRoles],
    connectors: PyRef<'_, PyConnectorStrokeRoles> => get_connectors (PyConnectorStrokeRoles) [borrow PyConnectorStrokeRoles],
    summaries: PyRef<'_, PySummaryStrokeRoles> => get_summaries (PySummaryStrokeRoles) [borrow PySummaryStrokeRoles],
    curves: PyRef<'_, PyCurveStrokeRoles> => get_curves (PyCurveStrokeRoles) [borrow PyCurveStrokeRoles],
});

theme_record!(PyCellMarkRoles, "CellMarkRoles", CellMarkRoles, {
    fill_alpha: u8 => get_fill_alpha (u8) [scalar],
    cell_outline: Option<PyRef<'_, PyOutlineTheme>> = None => get_cell_outline (Option<PyOutlineTheme>) [optional PyOutlineTheme],
    missing_fill: PyRef<'_, PyColor> => get_missing_fill (PyColor) [value PyColor],
    missing_row_outline: Option<PyRef<'_, PyOutlineTheme>> = None => get_missing_row_outline (Option<PyOutlineTheme>) [optional PyOutlineTheme],
    missing_key_outline: Option<PyRef<'_, PyOutlineTheme>> = None => get_missing_key_outline (Option<PyOutlineTheme>) [optional PyOutlineTheme],
    missing_cross: PyRef<'_, PyStrokeTheme> => get_missing_cross (PyStrokeTheme) [borrow PyStrokeTheme],
    missing_cross_inset: PyRef<'_, PyRelativeSpacing> => get_missing_cross_inset (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyCategoryMarkGeometry, "CategoryMarkGeometry", CategoryMarkGeometry, {
    vertical_body_breadth: PyRef<'_, PyRelativeSpacing> => get_vertical_body_breadth (PyRelativeSpacing) [borrow PyRelativeSpacing],
    horizontal_body_breadth: PyRef<'_, PyRelativeSpacing> => get_horizontal_body_breadth (PyRelativeSpacing) [borrow PyRelativeSpacing],
    vertical_group_span: PyRef<'_, PyRelativeSpacing> => get_vertical_group_span (PyRelativeSpacing) [borrow PyRelativeSpacing],
    horizontal_group_span: PyRef<'_, PyRelativeSpacing> => get_horizontal_group_span (PyRelativeSpacing) [borrow PyRelativeSpacing],
    uncertainty_cap_ratio: PyRef<'_, PyRelativeSpacing> => get_uncertainty_cap_ratio (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyMarkExtentTheme, "MarkExtentTheme", MarkExtentTheme, {
    categories: PyRef<'_, PyCategoryMarkGeometry> => get_categories (PyCategoryMarkGeometry) [borrow PyCategoryMarkGeometry],
    waterfall_body_category_breadth: PyRef<'_, PyRelativeSpacing> => get_waterfall_body_category_breadth (PyRelativeSpacing) [borrow PyRelativeSpacing],
    point_interval_cap_half_height: u32 => get_point_interval_cap_half_height (u32) [scalar],
    sampled_rug_height: PyRef<'_, PyRelativeSpacing> => get_sampled_rug_height (PyRelativeSpacing) [borrow PyRelativeSpacing],
    marginal_density_fraction: PyRef<'_, PyRelativeSpacing> => get_marginal_density_fraction (PyRelativeSpacing) [borrow PyRelativeSpacing],
    matrix_density_fraction: PyRef<'_, PyRelativeSpacing> => get_matrix_density_fraction (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyMarkTheme, "MarkTheme", MarkTheme, {
    accents: PyRef<'_, PyAccentPalette> => get_accents (PyAccentPalette) [borrow PyAccentPalette],
    points: PyRef<'_, PyPointRoles> => get_points (PyPointRoles) [borrow PyPointRoles],
    fills: PyRef<'_, PyFillRoles> => get_fills (PyFillRoles) [borrow PyFillRoles],
    data_lines: PyRef<'_, PyDataLineRoles> => get_data_lines (PyDataLineRoles) [borrow PyDataLineRoles],
    neutral_paints: PyRef<'_, PyNeutralPaintRoles> => get_neutral_paints (PyNeutralPaintRoles) [borrow PyNeutralPaintRoles],
    strokes: PyRef<'_, PyStrokeRoles> => get_strokes (PyStrokeRoles) [borrow PyStrokeRoles],
    cells: PyRef<'_, PyCellMarkRoles> => get_cells (PyCellMarkRoles) [borrow PyCellMarkRoles],
    extents: PyRef<'_, PyMarkExtentTheme> => get_extents (PyMarkExtentTheme) [borrow PyMarkExtentTheme],
});

theme_record!(PySignificanceGeometry, "SignificanceGeometry", SignificanceGeometry, {
    bracket_stroke: PyRef<'_, PySolidStrokeTheme> => get_bracket_stroke (PySolidStrokeTheme) [borrow PySolidStrokeTheme],
    label_gap: PyRef<'_, PyRelativeSpacing> => get_label_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    tick_height: PyRef<'_, PyRelativeSpacing> => get_tick_height (PyRelativeSpacing) [borrow PyRelativeSpacing],
    data_gap: PyRef<'_, PyRelativeSpacing> => get_data_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    level_step: PyRef<'_, PyRelativeSpacing> => get_level_step (PyRelativeSpacing) [borrow PyRelativeSpacing],
    panel_padding: PyRef<'_, PyRelativeSpacing> => get_panel_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
    attached_extra_clearance: PyRef<'_, PyRelativeSpacing> => get_attached_extra_clearance (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyCellAnnotationGeometry, "CellAnnotationGeometry", CellAnnotationGeometry, {
    highlight_outline: PyRef<'_, PyOutlineTheme> => get_highlight_outline (PyOutlineTheme) [borrow PyOutlineTheme],
    value_horizontal_padding: PyRef<'_, PyRelativeSpacing> => get_value_horizontal_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
    value_vertical_padding: PyRef<'_, PyRelativeSpacing> => get_value_vertical_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
    null_row_horizontal_padding: PyRef<'_, PyRelativeSpacing> => get_null_row_horizontal_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
    null_row_vertical_padding: PyRef<'_, PyRelativeSpacing> => get_null_row_vertical_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyEstimateAnnotationGeometry, "EstimateAnnotationGeometry", EstimateAnnotationGeometry, {
    panel_inset: PyRef<'_, PyRelativeSpacing> => get_panel_inset (PyRelativeSpacing) [borrow PyRelativeSpacing],
    guide_clearance: PyRef<'_, PyRelativeSpacing> => get_guide_clearance (PyRelativeSpacing) [borrow PyRelativeSpacing],
    numeric_label_gap: PyRef<'_, PyRelativeSpacing> => get_numeric_label_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyValueShelfSpacing, "ValueShelfSpacing", ValueShelfSpacing, {
    padding: PyRef<'_, PyRelativeSpacing> => get_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
    gap: PyRef<'_, PyRelativeSpacing> => get_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    row_gap: PyRef<'_, PyRelativeSpacing> => get_row_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
});

theme_record!(PyAnnotationTheme, "AnnotationTheme", AnnotationTheme, {
    significance: PyRef<'_, PySignificanceGeometry> => get_significance (PySignificanceGeometry) [borrow PySignificanceGeometry],
    cells: PyRef<'_, PyCellAnnotationGeometry> => get_cells (PyCellAnnotationGeometry) [borrow PyCellAnnotationGeometry],
    estimates: PyRef<'_, PyEstimateAnnotationGeometry> => get_estimates (PyEstimateAnnotationGeometry) [borrow PyEstimateAnnotationGeometry],
    point_label_gap: PyRef<'_, PyRelativeSpacing> => get_point_label_gap (PyRelativeSpacing) [borrow PyRelativeSpacing],
    panel_text_padding: PyRef<'_, PyRelativeSpacing> => get_panel_text_padding (PyRelativeSpacing) [borrow PyRelativeSpacing],
    waterfall_shelf: PyRef<'_, PyValueShelfSpacing> => get_waterfall_shelf (PyValueShelfSpacing) [borrow PyValueShelfSpacing],
    planning_backdrop: PyRef<'_, PyOpaqueColor> => get_planning_backdrop (PyOpaqueColor) [value PyOpaqueColor],
});

pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySizedPointTheme>()?;
    m.add_class::<PySupplementalPointTheme>()?;
    m.add_class::<PyPointRadii>()?;
    m.add_class::<PyPointRoles>()?;
    m.add_class::<PyBodyFillRoles>()?;
    m.add_class::<PyBandFillRoles>()?;
    m.add_class::<PyDensityFillRoles>()?;
    m.add_class::<PyFillRoles>()?;
    m.add_class::<PyDataLineRoles>()?;
    m.add_class::<PyNeutralPaintRoles>()?;
    m.add_class::<PyReferenceStrokeRoles>()?;
    m.add_class::<PyConnectorStrokeRoles>()?;
    m.add_class::<PySummaryStrokeRoles>()?;
    m.add_class::<PyCurveStrokeRoles>()?;
    m.add_class::<PyStrokeRoles>()?;
    m.add_class::<PyCellMarkRoles>()?;
    m.add_class::<PyCategoryMarkGeometry>()?;
    m.add_class::<PyMarkExtentTheme>()?;
    m.add_class::<PyMarkTheme>()?;
    m.add_class::<PySignificanceGeometry>()?;
    m.add_class::<PyCellAnnotationGeometry>()?;
    m.add_class::<PyEstimateAnnotationGeometry>()?;
    m.add_class::<PyValueShelfSpacing>()?;
    m.add_class::<PyAnnotationTheme>()
}
