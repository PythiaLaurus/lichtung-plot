use super::PlotAdapter;
use crate::error::{binding, BindingResult};
use crate::{kind, AdapterError, ColumnInput, EstimateStatus};

impl<K: kind::EstimateStatus> PlotAdapter<'_, K> {
    /// Explicitly map Boolean true/false to Estimated/NotEstimable; null never infers availability.
    pub fn estimate_status_boolean(
        mut self,
        input: impl ColumnInput,
    ) -> std::result::Result<Self, AdapterError> {
        let column = input.bind_column(self.source, "estimate status")?;
        let values = crate::boolean::values(&column)?;
        self.aligned("estimate status", column.name().as_str(), values.len())?;
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
    /// Map categorical statuses by exact raw keys, rejecting missing or unmapped observations.
    pub fn estimate_status_categorical<I, S>(
        mut self,
        input: impl ColumnInput,
        mapping: I,
    ) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = (S, EstimateStatus)>,
        S: AsRef<str>,
    {
        let column = input.bind_column(self.source, "estimate status")?;
        let values = crate::category::values(&column)?;
        self.aligned("estimate status", column.name().as_str(), values.len())?;
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
