//! Canonical payload/footprint owners, with a derived slot-to-anchor lookup.

use crate::{Error, Plot, TextPanel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Footprint {
    pub row: usize,
    pub column: usize,
    pub row_span: usize,
    pub column_span: usize,
}
impl Footprint {
    pub fn cell(row: usize, column: usize) -> Self {
        Self {
            row,
            column,
            row_span: 1,
            column_span: 1,
        }
    }
}

#[derive(Clone)]
pub(super) enum Payload {
    Plot(Box<Plot>),
    Text(TextPanel),
}

#[derive(Clone)]
struct Owner {
    footprint: Footprint,
    payload: Payload,
}

#[derive(Clone)]
pub(super) struct Placements {
    rows: usize,
    columns: usize,
    owners: Vec<Option<Owner>>,
    occupancy: Vec<Option<usize>>,
}
impl Placements {
    pub fn new(rows: usize, columns: usize) -> Result<Self, Error> {
        let count = rows
            .checked_mul(columns)
            .ok_or_else(|| Error::Data("Canvas topology overflows storage".into()))?;
        Ok(Self {
            rows,
            columns,
            owners: crate::data::source::collect_source((0..count).map(|_| Ok(None)))?,
            occupancy: crate::data::source::collect_source((0..count).map(|_| Ok(None)))?,
        })
    }
    pub fn len(&self) -> usize {
        self.owners.len()
    }
    pub fn shape(&self) -> (usize, usize) {
        (self.rows, self.columns)
    }
    pub fn plot(&self, index: usize) -> Option<&Plot> {
        match &self.owners.get(index)?.as_ref()?.payload {
            Payload::Plot(plot) => Some(plot),
            Payload::Text(_) => None,
        }
    }
    pub fn text(&self, index: usize) -> Option<&TextPanel> {
        match &self.owners.get(index)?.as_ref()?.payload {
            Payload::Text(text) => Some(text),
            Payload::Plot(_) => None,
        }
    }
    pub fn plots(&self) -> impl ExactSizeIterator<Item = Option<&Plot>> {
        (0..self.len()).map(|index| self.plot(index))
    }
    pub fn texts(&self) -> impl ExactSizeIterator<Item = Option<&TextPanel>> {
        (0..self.len()).map(|index| self.text(index))
    }
    pub fn footprint(&self, index: usize) -> Option<Footprint> {
        self.owners
            .get(index)?
            .as_ref()
            .map(|owner| owner.footprint)
    }

    pub fn insert(&mut self, footprint: Footprint, mut payload: Payload) -> Result<(), Error> {
        let row_end = footprint
            .row
            .checked_add(footprint.row_span)
            .filter(|&end| footprint.row_span > 0 && end <= self.rows);
        let column_end = footprint
            .column
            .checked_add(footprint.column_span)
            .filter(|&end| footprint.column_span > 0 && end <= self.columns);
        let (Some(row_end), Some(column_end)) = (row_end, column_end) else {
            return Err(Error::Data(
                "Canvas footprint is empty, overflowing or outside its declared grid".into(),
            ));
        };
        for row in footprint.row..row_end {
            for column in footprint.column..column_end {
                if self.occupancy[row * self.columns + column].is_some() {
                    return Err(Error::Data(format!(
                        "Canvas footprint overlaps occupied cell ({row}, {column})"
                    )));
                }
            }
        }
        let anchor = footprint.row * self.columns + footprint.column;
        if let Payload::Plot(plot) = &mut payload {
            plot.numeric_cell = Some(crate::data::source::source_arc((
                footprint.row,
                footprint.column,
            ))?);
        }
        self.owners[anchor] = Some(Owner { footprint, payload });
        for row in footprint.row..row_end {
            for column in footprint.column..column_end {
                self.occupancy[row * self.columns + column] = Some(anchor);
            }
        }
        Ok(())
    }
}
