//! Shared locator identity and required operation-local actual-view results.

use std::sync::Arc;

use crate::numeric::context::{DomainInfo, OwnerInfo};
use crate::{CoordinateAxis, NumericDomain, NumericOwner, Result};

/// One immutable, actually assigned numeric-axis view.
#[derive(Debug, Clone)]
pub struct TickContext<'a> {
    pub(crate) axis: CoordinateAxis,
    pub(crate) domain: &'a DomainInfo,
    pub(crate) owner: &'a OwnerInfo,
    pub(crate) span: f64,
}
impl<'a> TickContext<'a> {
    /// Horizontal or vertical scientific coordinate.
    pub const fn axis(&self) -> CoordinateAxis {
        self.axis
    }
    /// Final resolved scientific domain and provenance.
    pub fn domain(&self) -> NumericDomain<'a> {
        NumericDomain(self.domain)
    }
    /// Semantic producer, including raw facet and Canvas anchor identity.
    pub fn owner(&self) -> NumericOwner<'a> {
        NumericOwner(self.owner)
    }
    /// Actual assigned axis-parallel span in CSS pixels.
    pub const fn span(&self) -> f64 {
        self.span
    }
}

/// Complete source-order positions, without labels or drawing instructions.
#[derive(Debug, Clone)]
pub enum TickPositions {
    /// Raw data values, transformed by the actual scientific scale.
    Data(Vec<f64>),
    /// Mathematical powers of ten on an actual Log10 axis.
    Log10Exponents(Vec<f64>),
}
impl From<Vec<f64>> for TickPositions {
    fn from(values: Vec<f64>) -> Self {
        Self::Data(values)
    }
}

type Callback = dyn for<'a> Fn(&TickContext<'a>) -> Result<TickPositions> + Send + Sync;

#[derive(Clone)]
pub(crate) struct Function(Arc<Callback>);
impl Function {
    pub(super) fn new<F>(function: F) -> Self
    where
        F: for<'a> Fn(&TickContext<'a>) -> Result<TickPositions> + Send + Sync + 'static,
    {
        Self(Arc::new(function))
    }
    fn call(&self, context: &TickContext<'_>) -> Result<TickPositions> {
        self.0(context)
    }
}
impl std::fmt::Debug for Function {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PositionFunction")
    }
}
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Function {}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct View {
    pub(crate) owner: OwnerInfo,
    pub(crate) domain: DomainInfo,
    pub(crate) axis: CoordinateAxis,
    pub(crate) span: u64,
}

#[derive(Debug)]
struct Entry {
    policy: crate::TickGeneration,
    view: View,
    result: Arc<TickPositions>,
}

#[derive(Debug, Default)]
pub(crate) struct PositionTable {
    entries: Vec<Entry>,
    sealed: bool,
}
impl PositionTable {
    pub(crate) fn seal(&mut self) {
        self.sealed = true;
        self.entries.clear();
    }

    pub(crate) fn materialize(
        &mut self,
        policy: &crate::TickGeneration,
        view: View,
    ) -> Result<Arc<TickPositions>> {
        if self.sealed {
            return Err(crate::Error::Render(
                "tick positions requested after final root sealing".into(),
            ));
        }
        if let Some(entry) = self
            .entries
            .iter()
            .find(|entry| entry.policy == *policy && entry.view == view)
        {
            return Ok(entry.result.clone());
        }
        let function = policy
            .position_function()
            .ok_or_else(|| crate::Error::Render("position source lacks its function".into()))?;
        let result = function
            .call(&TickContext {
                axis: view.axis,
                domain: &view.domain,
                owner: &view.owner,
                span: f64::from_bits(view.span),
            })
            .map_err(|error| contextual(error, &view))?;
        let result = crate::data::source::source_arc(result)?;
        crate::data::source::reserve_source(&mut self.entries, 1)?;
        self.entries.push(Entry {
            policy: policy.clone(),
            view,
            result: result.clone(),
        });
        Ok(result)
    }
}

fn contextual(error: crate::Error, view: &View) -> crate::Error {
    let context = format!("{:?} position function for {:?}", view.axis, view.owner);
    match error {
        crate::Error::Data(message) => crate::Error::Data(format!("{context}: {message}")),
        crate::Error::Render(message) => crate::Error::Render(format!("{context}: {message}")),
        crate::Error::Io(error) => crate::Error::Io(std::io::Error::new(
            error.kind(),
            format!("{context}: {error}"),
        )),
    }
}
