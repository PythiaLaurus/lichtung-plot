//! Required operation-local materialization, independent of optional layout caches.

use std::sync::Arc;

use super::context::{ContextInfo, NumericPurpose, NumericSetContext, NumericValue};
use super::format::{NumericFormat, NumericPrecision};
use super::set::FormattedSet;
use crate::{Error, Result};

#[derive(Debug)]
struct Entry {
    policy: NumericFormat,
    info: Arc<ContextInfo>,
    result: FormattedSet,
}

#[derive(Debug, Default)]
pub(crate) struct PresentationTable {
    entries: Vec<Entry>,
    pub(crate) sealed: bool,
}

impl PresentationTable {
    pub(crate) fn seal(&mut self) {
        self.sealed = true;
        self.entries.clear();
    }
}

impl NumericFormat {
    pub(crate) fn materialize(
        &self,
        operation: &crate::font::canonical::OperationContext,
        info: ContextInfo,
        require_distinct: bool,
    ) -> Result<Option<FormattedSet>> {
        let mut table = operation.numeric_presentations.lock().map_err(|_| {
            Error::Render("numeric presentation table synchronization failed".into())
        })?;
        if table.sealed {
            return Err(Error::Render(
                "numeric presentation requested after final root sealing".into(),
            ));
        }
        let probe = info.purpose == NumericPurpose::IntrinsicProbe;
        if let Some(entry) = table
            .entries
            .iter()
            .find(|entry| entry.policy == *self && entry.info.as_ref() == &info)
        {
            let result = entry.result.try_copy()?;
            return accept(
                result,
                require_distinct && !probe,
                self.emitter.is_builtin() && !probe,
                self.get_precision(),
            );
        }
        let values = crate::data::source::collect_source(
            info.values
                .iter()
                .map(|value| Ok(f64::from_bits(value.bits))),
        )?;
        let power = info.values.first().is_some_and(|value| value.power);
        if info.values.iter().any(|value| value.power != power) {
            return Err(Error::Render(
                "numeric owner mixed incompatible operand representations".into(),
            ));
        }
        let mut result = if self.emitter.is_builtin() {
            self.function_builtin(&values, power)?
        } else {
            self.function_metadata(&values, power)?
        };
        if !values.is_empty() && !self.emitter.is_builtin() {
            let (labels, explanation, precision) =
                self.call_function(&info, &values, power, &result, require_distinct)?;
            if labels.len() != values.len() {
                return Err(Error::Data(format!(
                    "{:?} numeric function returned {} labels for {} occurrences",
                    info.role,
                    labels.len(),
                    values.len()
                )));
            }
            for (index, label) in labels.iter().enumerate() {
                crate::text::validate_for(
                    label,
                    &format!("{:?} numeric function occurrence {index}", info.role),
                )?;
            }
            result.labels = labels;
            result.precision = precision;
            if let Some(explanation) = explanation {
                crate::text::validate_for(explanation.as_str(), "numeric function explanation")?;
                result.explanation = Some(
                    self.units
                        .explanation
                        .replace_callback(result.explanation.take(), explanation)?,
                );
            }
        }
        let info = crate::data::source::source_arc(info)?;
        result.context = Some(info.clone());
        crate::data::source::reserve_source(&mut table.entries, 1)?;
        table.entries.push(Entry {
            policy: self.try_copy()?,
            info,
            result: result.try_copy()?,
        });
        accept(
            result,
            require_distinct && !probe,
            self.emitter.is_builtin() && !probe,
            self.get_precision(),
        )
    }

    fn call_function(
        &self,
        info: &ContextInfo,
        operands: &[f64],
        power: bool,
        metadata: &FormattedSet,
        required: bool,
    ) -> Result<(Vec<String>, Option<crate::Text>, u8)> {
        let values = crate::data::source::collect_source(
            info.values.iter().map(|value| Ok(NumericValue(value))),
        )?;
        let helper = std::cell::OnceCell::new();
        let context = NumericSetContext {
            info,
            values: &values,
            format: self,
            metadata,
            operands,
            power,
            required,
            helper: &helper,
        };
        let (labels, explanation) = self
            .emitter
            .call(&context)
            .map_err(|error| contextual(error, info))?
            .into_parts();
        Ok((
            labels,
            explanation,
            helper.get().map_or(metadata.precision, |set| set.precision),
        ))
    }
}

fn adaptive(precision: NumericPrecision) -> bool {
    matches!(
        precision,
        NumericPrecision::AdaptiveFractional { .. } | NumericPrecision::AdaptiveSignificant { .. }
    )
}

fn accept(
    result: FormattedSet,
    required: bool,
    builtin: bool,
    precision: NumericPrecision,
) -> Result<Option<FormattedSet>> {
    if (required || (builtin && adaptive(precision))) && !distinct(&result)? {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

fn distinct(set: &FormattedSet) -> Result<bool> {
    let mut order = crate::data::source::collect_source((0..set.members.len()).map(Ok))?;
    crate::data::source::sort_source_by(&mut order, |a, b| set.labels[*a].cmp(&set.labels[*b]));
    Ok(order.windows(2).all(|pair| {
        let (a, b) = (pair[0], pair[1]);
        set.labels[a] != set.labels[b]
            || set.members[a] == set.members[b]
            || (f64::from_bits(set.members[a]) == 0.0 && f64::from_bits(set.members[b]) == 0.0)
    }))
}

fn contextual(error: Error, info: &ContextInfo) -> Error {
    let context = format!(
        "{:?} numeric function for {:?} ({:?})",
        info.role, info.owner, info.purpose
    );
    match error {
        Error::Data(message) => Error::Data(format!("{context}: {message}")),
        Error::Render(message) => Error::Render(format!("{context}: {message}")),
        Error::Io(error) => Error::Io(std::io::Error::new(
            error.kind(),
            format!("{context}: {error}"),
        )),
    }
}
