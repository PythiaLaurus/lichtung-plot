//! Exact native configurations with Python references owned at their real roots.

use std::sync::Arc;

use lichtung_plot::kind;
use pyo3::exceptions::{PyMemoryError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::PyTraverseError;
use pyo3::PyVisit;

use crate::callbacks::{clone_roots, CallbackRoots};

// The single exact-family inventory drives compile-time dispatch. Scientific
// owners never cross an erased Plot<Any> boundary.
macro_rules! all_families {
    ($consumer:ident, $($args:tt)*) => {
        $consumer! {
            [Scatter, Histogram, Density, Qq, PointInterval, Line, Trajectory,
             LinePath, Interaction, Coefficient, Forest, Dumbbell, Heatmap, Bar,
             HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin,
             Raincloud, Regression, Marginal, Matrix, Beeswarm, Waterfall]
            $($args)*
        }
    };
}

macro_rules! declare_families {
    ([$($kind:ident),+]) => {
        /// Family erasure is private; each variant preserves its exact native
        /// capability type and matching binding origin in the same Config.
        #[derive(Clone)]
        pub(crate) enum NativePlot {
            $($kind(crate::input::Config<kind::$kind>)),+
        }
    };
}

all_families!(declare_families,);

#[macro_use]
mod dispatch;
#[macro_use]
mod methods;
mod canvas;
mod plot;
mod policy;

pub(crate) use plot::PyPlotOwner;

#[derive(Clone, Copy)]
enum CallbackSlot {
    XFormat,
    YFormat,
    ColorFormat,
    HeatmapValues,
    EstimateLabels,
    TotalFormat,
    XTick,
    YTick,
}

impl CallbackSlot {
    const fn index(self) -> usize {
        match self {
            Self::XFormat => 0,
            Self::YFormat => 1,
            Self::ColorFormat => 2,
            Self::HeatmapValues => 3,
            Self::EstimateLabels => 4,
            Self::TotalFormat => 5,
            Self::XTick => 6,
            Self::YTick => 7,
        }
    }
}

#[derive(Default)]
struct PlotRoots {
    slots: [CallbackRoots; 8],
}

impl PlotRoots {
    /// Write into an outer temporary so a partial failed copy is dropped only
    /// after the caller has released its PyO3 borrow.
    fn clone_into(&self, py: Python<'_>, target: &mut Self) -> PyResult<()> {
        for (source, target) in self.slots.iter().zip(&mut target.slots) {
            *target = clone_roots(py, source)?;
        }
        Ok(())
    }

    fn pins(&self, py: Python<'_>) -> PyResult<CallbackRoots> {
        let count = self.slots.iter().try_fold(0_usize, |count, roots| {
            count
                .checked_add(roots.len())
                .ok_or_else(|| PyMemoryError::new_err("callback root count overflows storage"))
        })?;
        let mut pins = Vec::new();
        pins.try_reserve(count)
            .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
        pins.extend(self.slots.iter().flatten().map(|root| root.clone_ref(py)));
        Ok(pins)
    }

    fn replace(&mut self, slot: CallbackSlot, incoming: CallbackRoots) -> CallbackRoots {
        std::mem::replace(&mut self.slots[slot.index()], incoming)
    }

    fn traverse(&self, visit: &PyVisit<'_>) -> Result<(), PyTraverseError> {
        for root in self.slots.iter().flatten() {
            visit.call(root)?;
        }
        Ok(())
    }
}

fn cleared_owner() -> PyErr {
    PyRuntimeError::new_err("this configuration owner has been cleared by garbage collection")
}

fn native_snapshot<T>(native: Option<&Arc<T>>) -> PyResult<Arc<T>> {
    native.map(Arc::clone).ok_or_else(cleared_owner)
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<plot::PyPlotOwner>()?;
    module.add_class::<canvas::PyCanvasOwner>()
}
