use lichtung_plot::SourceRows;
use ndarray::ArrayView1;

use crate::{error, snapshot, Error, Result, SourceRowElement};

pub(crate) fn ids<T: SourceRowElement>(view: ArrayView1<'_, T>) -> Result<Vec<usize>> {
    let mut ids = snapshot::buffer(view.len(), "source rows")?;
    for (slot, value) in view.iter().enumerate() {
        ids.push(value.to_usize().ok_or_else(|| Error::SourceRow {
            slot,
            value: format!("{value:?}"),
        })?);
    }
    Ok(ids)
}

/// Snapshot exact unique integer IDs; null, negative and out-of-range IDs fail.
///
/// Slots are logical input positions, not guessed original application row IDs.
/// Attach the resulting map once with the native `Plot::source_rows` setter.
pub fn source_rows<T: SourceRowElement>(view: ArrayView1<'_, T>) -> Result<SourceRows> {
    SourceRows::new(ids(view)?).map_err(|source| error::native("source rows", source))
}
