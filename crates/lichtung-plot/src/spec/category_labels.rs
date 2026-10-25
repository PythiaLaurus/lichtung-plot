use crate::{Error, Result};

/// Full scientific-domain endpoints forced into a categorical label selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LabelEndpoints {
    /// No forced endpoints.
    #[default]
    None,
    /// The first category.
    First,
    /// The last category.
    Last,
    /// Both endpoints, without duplicate occurrences.
    Both,
}

/// Label eligibility only: categories, ticks, grid lines and marks remain complete.
/// ```
/// use lichtung_plot::{CategoryLabelSelection, LabelEndpoints};
/// let months = CategoryLabelSelection::every(2)?;
/// let shifted = months.offset(1)?.endpoints(LabelEndpoints::Both);
/// # let _ = shifted;
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryLabelSelection {
    stride: usize,
    offset: usize,
    endpoints: LabelEndpoints,
}

#[cfg(test)]
mod tests;

impl Default for CategoryLabelSelection {
    fn default() -> Self {
        Self::all()
    }
}

impl CategoryLabelSelection {
    /// Make every category label eligible (the default).
    pub const fn all() -> Self {
        Self {
            stride: 1,
            offset: 0,
            endpoints: LabelEndpoints::None,
        }
    }

    /// Select every positive-stride label, initially at offset zero without endpoints.
    pub fn every(stride: usize) -> Result<Self> {
        if stride == 0 {
            return Err(Error::Data("category label stride must be positive".into()));
        }
        Ok(Self {
            stride,
            ..Self::all()
        })
    }

    /// Select the zero-based offset, strictly less than the configured stride.
    pub fn offset(mut self, offset: usize) -> Result<Self> {
        if offset >= self.stride {
            return Err(Error::Data(
                "category label offset must be smaller than its stride".into(),
            ));
        }
        self.offset = offset;
        Ok(self)
    }

    /// Union full-domain endpoints into the selected indices. All stays normalized.
    #[must_use]
    pub const fn endpoints(mut self, value: LabelEndpoints) -> Self {
        if self.stride != 1 {
            self.endpoints = value;
        }
        self
    }

    pub(crate) const fn is_all(&self) -> bool {
        self.stride == 1
    }

    pub(crate) fn includes(&self, index: usize, count: usize) -> bool {
        index < count
            && (index % self.stride == self.offset
                || (index == 0
                    && matches!(self.endpoints, LabelEndpoints::First | LabelEndpoints::Both))
                || (index == count - 1
                    && matches!(self.endpoints, LabelEndpoints::Last | LabelEndpoints::Both)))
    }

    pub(crate) fn indices(
        &self,
        count: usize,
    ) -> impl ExactSizeIterator<Item = usize> + Clone + '_ {
        let base = if self.offset < count {
            (count - 1 - self.offset) / self.stride + 1
        } else {
            0
        };
        let first = count > 0
            && self.offset != 0
            && matches!(self.endpoints, LabelEndpoints::First | LabelEndpoints::Both);
        let last = count > 0
            && (count - 1) % self.stride != self.offset
            && matches!(self.endpoints, LabelEndpoints::Last | LabelEndpoints::Both)
            && (count != 1 || !first);
        let leading = usize::from(first);
        (0..(base + leading + usize::from(last))).map(move |ordinal| {
            if first && ordinal == 0 {
                0
            } else if ordinal - leading < base {
                self.offset + (ordinal - leading) * self.stride
            } else {
                count - 1
            }
        })
    }

    pub(crate) fn observe(&self, round: &mut crate::layout::kernel::FeedbackRound) -> Result<()> {
        round.observe_count(self.stride)?;
        round.observe_count(self.offset)?;
        round.observe_count(self.endpoints as usize)
    }
}
