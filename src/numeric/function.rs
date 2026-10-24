//! Opaque shared emission identity; clones never compare application output.

use std::sync::Arc;

use super::context::{NumericLabels, NumericSetContext, NumericValueContext};
use super::format::NumericFormat;
use crate::Result;

#[cfg(test)]
mod tests;

type ValueFunction = dyn for<'a> Fn(&NumericValueContext<'a>) -> Result<String> + Send + Sync;
type SetFunction = dyn for<'a> Fn(&NumericSetContext<'a>) -> Result<NumericLabels> + Send + Sync;

enum Function {
    Value(Box<ValueFunction>),
    Set(Box<SetFunction>),
}

#[derive(Clone, Default)]
pub(crate) struct Emitter(Option<Arc<Function>>);
impl std::fmt::Debug for Emitter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self.0.as_deref() {
            None => "Builtin",
            Some(Function::Value(_)) => "ValueFunction",
            Some(Function::Set(_)) => "SetFunction",
        })
    }
}
impl PartialEq for Emitter {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for Emitter {}

impl Emitter {
    pub(crate) const fn is_builtin(&self) -> bool {
        self.0.is_none()
    }
    pub(super) fn call(&self, context: &NumericSetContext<'_>) -> Result<NumericLabels> {
        match self.0.as_deref() {
            None => context.format_builtin(),
            Some(Function::Set(function)) => function(context),
            Some(Function::Value(function)) => {
                let labels = crate::data::source::collect_source((0..context.values().len()).map(
                    |index| {
                        function(&NumericValueContext {
                            set: context,
                            index,
                        })
                    },
                ))?;
                Ok(NumericLabels::new(labels))
            }
        }
    }
}

impl NumericFormat {
    /// Select final per-occurrence text, retaining built-in options for helpers.
    #[must_use]
    pub fn value_function<F>(mut self, function: F) -> Self
    where
        F: for<'a> Fn(&NumericValueContext<'a>) -> Result<String> + Send + Sync + 'static,
    {
        self.emitter = Emitter(Some(Arc::new(Function::Value(Box::new(function)))));
        self
    }
    /// Select final complete-set text and optional common explanatory Text.
    #[must_use]
    pub fn set_function<F>(mut self, function: F) -> Self
    where
        F: for<'a> Fn(&NumericSetContext<'a>) -> Result<NumericLabels> + Send + Sync + 'static,
    {
        self.emitter = Emitter(Some(Arc::new(Function::Set(Box::new(function)))));
        self
    }
    /// Restore built-in emission without resetting any normalized options.
    #[must_use]
    pub fn builtin(mut self) -> Self {
        self.emitter = Emitter::default();
        self
    }
}
