use lichtung_plot::render::backend::{Point2D, PointStyle};
use lichtung_plot::{BarAggregation, Chart, ComparisonAnnotation, ValueSort};
use polars::prelude::*;

#[path = "../support/visual.rs"]
mod support;

use support::*;

mod contract;
mod labels;
mod public_contracts;
