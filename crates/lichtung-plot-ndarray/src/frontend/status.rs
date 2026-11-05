use super::PlotAdapter;
use crate::error::{binding, BindingResult};
use crate::{kind, AdapterError, BooleanInput, CategoryInput, EstimateStatus};

impl<K: kind::EstimateStatus> PlotAdapter<'_, K> {
    /// Explicitly map Boolean true/false to Estimated/NotEstimable; null is an error.
    pub fn estimate_status_boolean(
        mut self,
        input: impl BooleanInput,
    ) -> std::result::Result<Self, AdapterError> {
        if input.cell_shape().is_some() || input.selected_columns().is_some() {
            return Err(binding(
                "estimate status",
                "expected one Boolean variable, not a cell grid",
            ));
        }
        let (name, values) = input.raw_boolean(self.source.as_deref(), "estimate status")?;
        self.aligned("estimate status", &name, values.len())?;
        let statuses = values
            .into_iter()
            .enumerate()
            .map(|(slot, value)| match value {
                Some(true) => Ok(EstimateStatus::Estimated),
                Some(false) => Ok(EstimateStatus::NotEstimable),
                None => Err(binding(
                    "estimate status",
                    format!("missing Boolean at slot {slot}"),
                )),
            })
            .collect::<BindingResult<Vec<_>>>()?;
        self.plot = self.plot.estimate_status(statuses);
        Ok(self)
    }
    /// Map exact raw categorical statuses, rejecting missing/unmapped observations.
    pub fn estimate_status_categorical<I, S>(
        mut self,
        input: impl CategoryInput,
        mapping: I,
    ) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = (S, EstimateStatus)>,
        S: AsRef<str>,
    {
        let (name, values) = input.raw_category(self.source.as_deref(), "estimate status")?;
        self.aligned("estimate status", &name, values.len())?;
        let mapping = mapping
            .into_iter()
            .map(|(key, status)| (key.as_ref().to_owned(), status))
            .collect::<Vec<_>>();
        let statuses = values
            .into_iter()
            .enumerate()
            .map(|(slot, value)| {
                let value = value.ok_or_else(|| {
                    binding(
                        "estimate status",
                        format!("missing category at slot {slot}"),
                    )
                })?;
                let mut matches = mapping.iter().filter(|(key, _)| key == &value);
                let status = matches
                    .next()
                    .ok_or_else(|| {
                        binding(
                            "estimate status",
                            format!("unmapped category {value:?} at slot {slot}"),
                        )
                    })?
                    .1;
                if matches.next().is_some() {
                    return Err(binding(
                        "estimate status",
                        format!("duplicate mapping for {value:?}"),
                    ));
                }
                Ok(status)
            })
            .collect::<BindingResult<Vec<_>>>()?;
        self.plot = self.plot.estimate_status(statuses);
        Ok(self)
    }
}
