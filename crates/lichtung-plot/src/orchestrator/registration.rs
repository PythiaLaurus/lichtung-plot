//! Operation-local occurrence identity and selected input rows.

use crate::{builder::PlotDescription, Error};

pub(crate) enum RowSelection {
    Selected {
        input_slots: Vec<usize>,
        diagnostic_rows: Vec<usize>,
    },
}

impl RowSelection {
    pub(crate) fn selected(&self) -> (&[usize], &[usize]) {
        match self {
            Self::Selected {
                input_slots,
                diagnostic_rows,
            } => (input_slots, diagnostic_rows),
        }
    }
}

pub(crate) fn select_rows(
    source: &PlotDescription,
    input_slots: Vec<usize>,
) -> Result<RowSelection, Error> {
    let rows = source.source_row_map();
    let diagnostic_rows = crate::data::source::collect_source(
        input_slots
            .iter()
            .map(|index| Ok(rows.map_or(*index, |rows| rows.as_slice()[*index]))),
    )?;
    Ok(RowSelection::Selected {
        input_slots,
        diagnostic_rows,
    })
}
