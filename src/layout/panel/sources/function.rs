//! Actual-view position materialization, ordinary filtering and complete-set text.

use super::{AxisSide, NumericTickPlan, Projection, SourceKind, TickSource};
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::numeric::{context::DomainInfo, operand::Operand, source::AxisRequest};
use crate::spec::ticks::function::View;
use crate::theme::ResolvedTheme;
use crate::{AxisScale, CoordinateAxis, Error, TickPositions};

#[cfg(test)]
mod tests;

fn materialize(
    range: (f64, f64),
    log: bool,
    projection: &Projection,
    side: AxisSide,
    request: &AxisRequest<'_>,
) -> Result<std::sync::Arc<TickPositions>, Error> {
    let panel = projection.panel_box();
    let span = match side {
        AxisSide::Bottom => panel.width(),
        AxisSide::Left => panel.height(),
    };
    crate::layout::kernel::required_length(span)?;
    let domain = DomainInfo::axis(
        range,
        if log {
            AxisScale::Log10
        } else {
            AxisScale::Linear
        },
        request.provenance,
        if log {
            request.source.raw_limits(side)
        } else {
            Some(range)
        },
    );
    let view = View {
        owner: request.source.owner.try_copy()?,
        domain,
        axis: match side {
            AxisSide::Bottom => CoordinateAxis::X,
            AxisSide::Left => CoordinateAxis::Y,
        },
        span: span.to_bits(),
    };
    request
        .source
        .operation
        .tick_positions
        .lock()
        .map_err(|_| Error::Render("position table synchronization failed".into()))?
        .materialize(&request.generation, view)
}

pub(super) fn plan(
    range: (f64, f64),
    log: bool,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    request: Option<&AxisRequest<'_>>,
    scope: &FitScope,
) -> Result<NumericTickPlan, PlanError> {
    let request = request
        .ok_or_else(|| Error::Render("position function lacks its operation-bound owner".into()))?;
    let returned = materialize(range, log, projection, side, request)?;
    let (values, power) = match returned.as_ref() {
        TickPositions::Data(values) => (values, false),
        TickPositions::Log10Exponents(values) if log => (values, true),
        TickPositions::Log10Exponents(_) => {
            return Err(Error::Data(format!(
                "position exponents require a Log10 axis for {:?}",
                request.source.owner
            ))
            .into())
        }
    };
    let coordinate = |value| coordinate(value, range, log, power, &request.source.owner);
    let mut count = 0_usize;
    for &value in values {
        if coordinate(value)?.is_some() {
            count = count
                .checked_add(1)
                .ok_or_else(|| Error::Render("position occurrence count overflow".into()))?;
        }
    }
    if count > request.generation.get_maximum_count().unwrap_or(usize::MAX) {
        return Err(Error::Render(format!(
            "position count exceeds configured maximum for {:?}",
            request.source.owner
        ))
        .into());
    }
    let source = TickSource {
        kind: SourceKind::Function,
        ordinal: 0,
        cap: 1,
        precision: None,
    };
    if count == 0 {
        return Ok(NumericTickPlan {
            ticks: Vec::new(),
            source,
            numeric: None,
        });
    }
    let mut positions = Vec::new();
    let mut operands = Vec::new();
    let mut indices = Vec::new();
    super::reserve(&mut positions, count)?;
    super::reserve(&mut operands, count)?;
    super::reserve(&mut indices, count)?;
    for (index, &value) in values.iter().enumerate() {
        if let Some(position) = coordinate(value)? {
            positions.push(position);
            operands.push(Operand::number(value, power));
            indices.push(index);
        }
    }
    let request = AxisRequest {
        source_indices: Some(&indices),
        ..request.clone()
    };
    let selected = if request.format.is_some() {
        NumericTickPlan::formatted(
            &positions, &operands, log, &request, theme, source, projection, side,
        )?
    } else {
        NumericTickPlan::omitted(&positions, &operands, false, source)?
    };
    if let Some(mut selected) = selected {
        if request.format.is_none() {
            let mut set = selected
                .numeric
                .as_ref()
                .ok_or_else(|| Error::Render("omitted function text lost its members".into()))?
                .try_copy()?;
            set.context = Some(crate::data::source::source_arc(request.context(
                &operands,
                log,
                side,
                range,
                crate::NumericPurpose::ActualSet,
            )?)?);
            selected.numeric = Some(crate::data::source::source_arc(set)?);
        }
        if super::labels_fit(&selected.ticks, projection, side, theme)? {
            return Ok(selected);
        }
    }
    Err(scope
        .failure(
            FitKind::Tick,
            "complete function positions cannot fit their selected text",
        )
        .into())
}

fn coordinate(
    value: f64,
    range: (f64, f64),
    log: bool,
    power: bool,
    owner: &crate::numeric::context::OwnerInfo,
) -> Result<Option<f64>, Error> {
    if !value.is_finite() {
        return Err(Error::Render(format!(
            "unprojectable position for {owner:?}"
        )));
    }
    if log && !power && value <= 0.0 {
        return Ok(None);
    }
    let position = if log && !power { value.log10() } else { value };
    if !position.is_finite() {
        return Err(Error::Render(format!(
            "unprojectable position for {owner:?}"
        )));
    }
    Ok((position >= range.0 && position <= range.1).then_some(position))
}
