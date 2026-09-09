//! Checked fixed-precision geometry shared by validating and drawing backends.

use std::collections::HashMap;
use std::fmt::{self, Write as _};

use crate::layout::PanelBox;
use crate::render::backend::{BracketOrientation, GradientStop, Point2D, SignificanceBracket};
use crate::Error;

const MAX_EXACT_UNITS: i64 = (1_i64 << 53) - 1;

struct CoordinateSpelling {
    bytes: [u8; 328],
    len: usize,
}

impl fmt::Write for CoordinateSpelling {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len.checked_add(text.len()).ok_or(fmt::Error)?;
        self.bytes
            .get_mut(self.len..end)
            .ok_or(fmt::Error)?
            .copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoordinatePrecision {
    Integer,
    Tenths,
    Hundredths,
    Thousandths,
    Millionths,
}

impl CoordinatePrecision {
    const fn decimals(self) -> usize {
        match self {
            Self::Integer => 0,
            Self::Tenths => 1,
            Self::Hundredths => 2,
            Self::Thousandths => 3,
            Self::Millionths => 6,
        }
    }

    const fn scale(self) -> i64 {
        match self {
            Self::Integer => 1,
            Self::Tenths => 10,
            Self::Hundredths => 100,
            Self::Thousandths => 1_000,
            Self::Millionths => 1_000_000,
        }
    }

    const fn unsigned_scale(self) -> u64 {
        match self {
            Self::Integer => 1,
            Self::Tenths => 10,
            Self::Hundredths => 100,
            Self::Thousandths => 1_000,
            Self::Millionths => 1_000_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalCoordinate {
    units: i64,
    precision: CoordinatePrecision,
    negative_zero: bool,
}

impl CanonicalCoordinate {
    pub(crate) fn from_f64(
        value: f64,
        precision: CoordinatePrecision,
        context: impl fmt::Display,
    ) -> Result<Self, Error> {
        if !value.is_finite() {
            return Err(Error::Render(format!(
                "{context} must be finite; got {value}"
            )));
        }

        let decimals = precision.decimals();
        // Even f64::MAX in fixed notation fits this bounded scalar scratch.
        let mut scratch = CoordinateSpelling {
            bytes: [0; 328],
            len: 0,
        };
        write!(scratch, "{value:.decimals$}")
            .map_err(|_| Error::Render(format!("{context} coordinate spelling overflow")))?;
        let spelling = std::str::from_utf8(&scratch.bytes[..scratch.len])
            .map_err(|_| Error::Render(format!("{context} coordinate spelling is not UTF-8")))?;
        let negative = spelling.starts_with('-');
        let unsigned = spelling.strip_prefix('-').unwrap_or(spelling);
        let (whole, fraction) = if decimals == 0 {
            (unsigned, "")
        } else {
            unsigned.split_once('.').ok_or_else(|| {
                Error::Render(format!(
                    "{context} is not representable at {decimals}-decimal precision; got {value}"
                ))
            })?
        };
        if fraction.len() != decimals {
            return Err(Error::Render(format!(
                "{context} is not representable at {decimals}-decimal precision; got {value}"
            )));
        }

        let whole = whole.parse::<i128>().map_err(|_| {
            Error::Render(format!(
                "{context} exceeds the signed coordinate range; got {value}"
            ))
        })?;
        let fraction = if fraction.is_empty() {
            0
        } else {
            fraction.parse::<i128>().map_err(|_| {
                Error::Render(format!(
                    "{context} is not representable at {decimals}-decimal precision; got {value}"
                ))
            })?
        };
        let magnitude = whole
            .checked_mul(i128::from(precision.scale()))
            .and_then(|units| units.checked_add(fraction))
            .ok_or_else(|| {
                Error::Render(format!(
                    "{context} exceeds the signed coordinate range; got {value}"
                ))
            })?;
        let signed = if negative { -magnitude } else { magnitude };
        let units = i64::try_from(signed).map_err(|_| {
            Error::Render(format!(
                "{context} exceeds the signed coordinate range; got {value}"
            ))
        })?;

        Self::from_units(units, precision, units == 0 && negative, context)
    }

    fn from_units(
        units: i64,
        precision: CoordinatePrecision,
        negative_zero: bool,
        context: impl fmt::Display,
    ) -> Result<Self, Error> {
        let canvas_reserve = i64::from(u32::MAX)
            .checked_mul(precision.scale())
            .ok_or_else(|| {
                Error::Render(format!("{context} exceeds the signed coordinate range"))
            })?;
        let limit = MAX_EXACT_UNITS - canvas_reserve;
        Self::from_units_with_limit(units, precision, negative_zero, context, limit)
    }

    fn from_derived_units(
        units: i64,
        precision: CoordinatePrecision,
        context: impl fmt::Display,
    ) -> Result<Self, Error> {
        Self::from_units_with_limit(units, precision, false, context, MAX_EXACT_UNITS)
    }

    fn from_units_with_limit(
        units: i64,
        precision: CoordinatePrecision,
        negative_zero: bool,
        context: impl fmt::Display,
        limit: i64,
    ) -> Result<Self, Error> {
        if units.unsigned_abs() > limit.unsigned_abs() {
            return Err(Error::Render(format!(
                "{context} exceeds the exactly representable signed coordinate range"
            )));
        }
        Ok(Self {
            units,
            precision,
            negative_zero,
        })
    }

    pub(crate) fn units(self) -> i64 {
        self.units
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "coordinate units are constrained to f64's exact integer range"
    )]
    pub(crate) fn value(self) -> f64 {
        let value = self.units as f64 / self.precision.scale() as f64;
        if self.negative_zero {
            -0.0
        } else {
            value
        }
    }

    pub(crate) fn inverted(self, canvas_height: u32) -> Result<Self, Error> {
        let height_units = i64::from(canvas_height)
            .checked_mul(self.precision.scale())
            .ok_or_else(|| Error::Render("canvas height exceeds coordinate range".into()))?;
        let units = height_units
            .checked_sub(self.units)
            .ok_or_else(|| Error::Render("inverted coordinate exceeds signed range".into()))?;
        Self::from_derived_units(units, self.precision, "inverted coordinate")
    }

    fn checked_add(self, other: Self, context: impl fmt::Display) -> Result<Self, Error> {
        self.checked_binary(other, context, i64::checked_add)
    }

    fn checked_sub(self, other: Self, context: impl fmt::Display) -> Result<Self, Error> {
        self.checked_binary(other, context, i64::checked_sub)
    }

    fn checked_binary(
        self,
        other: Self,
        context: impl fmt::Display,
        operation: fn(i64, i64) -> Option<i64>,
    ) -> Result<Self, Error> {
        if self.precision != other.precision {
            return Err(Error::Render(format!(
                "{context} mixes incompatible coordinate precision"
            )));
        }
        let units = operation(self.units, other.units)
            .ok_or_else(|| Error::Render(format!("{context} exceeds signed coordinate range")))?;
        Self::from_derived_units(units, self.precision, context)
    }
}

impl fmt::Display for CanonicalCoordinate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let negative = self.units < 0 || self.negative_zero;
        let magnitude = self.units.unsigned_abs();
        if negative {
            formatter.write_str("-")?;
        }
        if self.precision == CoordinatePrecision::Integer {
            return write!(formatter, "{magnitude}");
        }
        let scale = self.precision.unsigned_scale();
        let whole = magnitude / scale;
        let fraction = magnitude % scale;
        write!(
            formatter,
            "{whole}.{fraction:0width$}",
            width = self.precision.decimals()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalPoint {
    pub(crate) x: CanonicalCoordinate,
    pub(crate) y: CanonicalCoordinate,
}

impl CanonicalPoint {
    fn from_raw(
        point: Point2D,
        precision: CoordinatePrecision,
        context: impl fmt::Display,
    ) -> Result<Self, Error> {
        Ok(Self {
            x: CanonicalCoordinate::from_f64(point.x, precision, format_args!("{context} x"))?,
            y: CanonicalCoordinate::from_f64(point.y, precision, format_args!("{context} y"))?,
        })
    }

    pub(crate) fn value(self) -> Point2D {
        Point2D::new(self.x.value(), self.y.value())
    }

    pub(crate) fn same_position(self, other: Self) -> bool {
        self.x.units() == other.x.units() && self.y.units() == other.y.units()
    }
}

pub(crate) fn canonical_points(
    points: &[Point2D],
    context: &str,
) -> Result<Vec<CanonicalPoint>, Error> {
    let mut converted = Vec::new();
    converted
        .try_reserve_exact(points.len())
        .map_err(|error| Error::Render(format!("{context} point reservation: {error}")))?;
    for point in canonical_points_iter(points, context) {
        converted.push(point?);
    }
    Ok(converted)
}

pub(crate) fn canonical_points_iter<'a>(
    points: &'a [Point2D],
    context: &'a str,
) -> impl Iterator<Item = Result<CanonicalPoint, Error>> + 'a {
    points
        .iter()
        .copied()
        .enumerate()
        .map(move |(index, point)| {
            if !point.x.is_finite() || !point.y.is_finite() {
                return Err(Error::Render(format!(
                    "{context} point {index} must contain finite coordinates; got ({}, {})",
                    point.x, point.y
                )));
            }
            CanonicalPoint::from_raw(
                point,
                CoordinatePrecision::Thousandths,
                format_args!("{context} point {index}"),
            )
        })
}

#[derive(Default)]
pub(crate) struct ValidationScratch {
    points: Vec<(CanonicalPoint, Point2D, usize)>,
    fixed: bool,
}

impl ValidationScratch {
    pub(crate) fn prepare(
        &mut self,
        count: usize,
        stage: &mut crate::layout::kernel::FinalStage<'_>,
        payload: &str,
    ) -> Result<(), Error> {
        super::svg_backend::reserve_storage(stage, &mut self.points, count, payload)?;
        self.fixed = true;
        Ok(())
    }

    fn fill(&mut self, points: &[Point2D], context: &str) -> Result<(), Error> {
        self.points.clear();
        if points.len() > self.points.capacity() {
            if self.fixed {
                return Err(Error::Render(
                    "canonical validation exceeded reserved scratch".into(),
                ));
            }
            self.points
                .try_reserve_exact(points.len())
                .map_err(|error| {
                    Error::Render(format!("canonical validation reservation: {error}"))
                })?;
        }
        for (index, canonical) in canonical_points_iter(points, context).enumerate() {
            self.points.push((canonical?, points[index], index));
        }
        Ok(())
    }

    pub(crate) fn line(&mut self, points: &[Point2D]) -> Result<(), Error> {
        self.fill(points, "line")?;
        self.points
            .sort_unstable_by_key(|(point, _, index)| (point.x.units(), point.y.units(), *index));
        let conflict = self
            .points
            .windows(2)
            .filter(|pair| {
                pair[0].0.same_position(pair[1].0) && !same_finite_point(pair[0].1, pair[1].1)
            })
            .min_by_key(|pair| pair[1].2);
        if let Some(pair) = conflict {
            let (_, first, first_index) = pair[0];
            let (_, second, second_index) = pair[1];
            return Err(Error::Render(format!("line distinct points {first_index} ({}, {}) and {second_index} ({}, {}) collapse at 3-decimal coordinate precision", first.x, first.y, second.x, second.y)));
        }
        Ok(())
    }

    pub(crate) fn polygon(&mut self, points: &[Point2D]) -> Result<(), Error> {
        self.fill(points, "polygon")?;
        for index in 0..points.len() {
            let next = (index + 1) % points.len();
            if !same_finite_point(points[index], points[next])
                && self.points[index].0.same_position(self.points[next].0)
            {
                return Err(Error::Render(format!("polygon distinct adjacent points {index} ({}, {}) and {next} ({}, {}) collapse at 3-decimal coordinate precision", points[index].x, points[index].y, points[next].x, points[next].y)));
            }
        }
        if distinct_up_to_three(
            points
                .iter()
                .map(|point| (normalized_bits(point.x), normalized_bits(point.y))),
        ) >= 3
            && distinct_up_to_three(
                self.points
                    .iter()
                    .map(|(point, _, _)| (point.x.units(), point.y.units())),
            ) < 3
        {
            return Err(Error::Render(
                "polygon required distinct points collapse at 3-decimal coordinate precision"
                    .into(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn canonical_line_points(points: &[Point2D]) -> Result<Vec<CanonicalPoint>, Error> {
    canonical_distinct_points(points, "line")
}

pub(crate) fn canonical_polygon_points(points: &[Point2D]) -> Result<Vec<CanonicalPoint>, Error> {
    let converted = canonical_points(points, "polygon")?;
    for index in 0..points.len() {
        let next = (index + 1) % points.len();
        let raw = (
            normalized_bits(points[index].x),
            normalized_bits(points[index].y),
        );
        let next_raw = (
            normalized_bits(points[next].x),
            normalized_bits(points[next].y),
        );
        if raw != next_raw && converted[index].same_position(converted[next]) {
            return Err(Error::Render(format!(
                "polygon distinct adjacent points {index} ({}, {}) and {next} ({}, {}) collapse at 3-decimal coordinate precision",
                points[index].x, points[index].y, points[next].x, points[next].y
            )));
        }
    }
    let raw_distinct = distinct_up_to_three(
        points
            .iter()
            .map(|point| (normalized_bits(point.x), normalized_bits(point.y))),
    );
    let converted_distinct = distinct_up_to_three(
        converted
            .iter()
            .map(|point| (point.x.units(), point.y.units())),
    );
    if raw_distinct >= 3 && converted_distinct < 3 {
        return Err(Error::Render(
            "polygon required distinct points collapse at 3-decimal coordinate precision".into(),
        ));
    }
    Ok(converted)
}

fn distinct_up_to_three<T: Copy + PartialEq>(values: impl Iterator<Item = T>) -> usize {
    let mut distinct = [None; 3];
    let mut count = 0;
    for value in values {
        if !distinct[..count].contains(&Some(value)) {
            distinct[count] = Some(value);
            count += 1;
            if count == 3 {
                break;
            }
        }
    }
    count
}

fn canonical_distinct_points(
    points: &[Point2D],
    context: &str,
) -> Result<Vec<CanonicalPoint>, Error> {
    let converted = canonical_points(points, context)?;
    let mut seen = HashMap::new();
    seen.try_reserve(points.len())
        .map_err(|error| Error::Render(format!("{context} distinct-point reservation: {error}")))?;
    for (index, (point, canonical)) in points.iter().zip(&converted).enumerate() {
        let raw = (normalized_bits(point.x), normalized_bits(point.y));
        let key = (canonical.x.units(), canonical.y.units());
        if let Some((previous_index, previous_point, previous_raw)) =
            seen.insert(key, (index, *point, raw))
        {
            if previous_raw != raw {
                return Err(Error::Render(format!(
                    "{context} distinct points {previous_index} ({}, {}) and {index} ({}, {}) collapse at 3-decimal coordinate precision",
                    previous_point.x, previous_point.y, point.x, point.y
                )));
            }
        }
    }
    Ok(converted)
}

pub(crate) fn canonical_triangles(
    points: &[Point2D],
    radius: u32,
) -> Result<Vec<[CanonicalPoint; 3]>, Error> {
    let mut converted = Vec::new();
    converted
        .try_reserve_exact(points.len())
        .map_err(|error| Error::Render(format!("triangle reservation: {error}")))?;
    for (index, point) in points.iter().copied().enumerate() {
        converted.push(canonical_triangle(point, radius, index)?);
    }
    Ok(converted)
}

pub(crate) fn canonical_triangle(
    point: Point2D,
    radius: u32,
    index: usize,
) -> Result<[CanonicalPoint; 3], Error> {
    let radius = f64::from(radius);
    let half_width = radius * 0.866_025_403_784_438_6;
    let raw = [
        Point2D::new(point.x, point.y + radius),
        Point2D::new(point.x - half_width, point.y - radius * 0.5),
        Point2D::new(point.x + half_width, point.y - radius * 0.5),
    ];
    let mut result = [CanonicalPoint::from_raw(
        raw[0],
        CoordinatePrecision::Thousandths,
        format_args!("triangle {index} point 0"),
    )?; 3];
    for i in 1..3 {
        result[i] = CanonicalPoint::from_raw(
            raw[i],
            CoordinatePrecision::Thousandths,
            format_args!("triangle {index} point {i}"),
        )?;
        for j in 0..i {
            if !same_finite_point(raw[i], raw[j]) && result[i].same_position(result[j]) {
                return Err(Error::Render(format!(
                    "triangle {index} distinct points collapse at 3-decimal coordinate precision"
                )));
            }
        }
    }
    Ok(result)
}

fn normalized_bits(value: f64) -> u64 {
    if value == 0.0 {
        0
    } else {
        value.to_bits()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalRect {
    pub(crate) left: CanonicalCoordinate,
    pub(crate) right: CanonicalCoordinate,
    pub(crate) bottom: CanonicalCoordinate,
    pub(crate) top: CanonicalCoordinate,
    pub(crate) width: CanonicalCoordinate,
    pub(crate) height: CanonicalCoordinate,
}

impl CanonicalRect {
    pub(crate) fn panel(self) -> PanelBox {
        PanelBox::new(
            self.left.value(),
            self.right.value(),
            self.bottom.value(),
            self.top.value(),
        )
    }
}

pub(crate) fn canonical_clip(region: PanelBox) -> Result<CanonicalRect, Error> {
    canonical_panel_rect(region, CoordinatePrecision::Thousandths, "clip region")
}

fn canonical_panel_rect(
    region: PanelBox,
    precision: CoordinatePrecision,
    context: &str,
) -> Result<CanonicalRect, Error> {
    let values = [region.x0, region.x1, region.y0, region.y1];
    let width = region.width();
    let height = region.height();
    if values.iter().any(|value| !value.is_finite())
        || !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
    {
        return Err(Error::Render(format!(
            "{context} must have finite coordinates and finite positive width and height; got {region:?}"
        )));
    }
    canonical_rect(
        region.x0, region.y1, width, height, precision, context, true,
    )
}

fn canonical_rect(
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    precision: CoordinatePrecision,
    context: &str,
    require_positive: bool,
) -> Result<CanonicalRect, Error> {
    let positive_width = width > 0.0;
    let positive_height = height > 0.0;
    let left = CanonicalCoordinate::from_f64(left, precision, format_args!("{context} left"))?;
    let top = CanonicalCoordinate::from_f64(top, precision, format_args!("{context} top"))?;
    let width = CanonicalCoordinate::from_f64(width, precision, format_args!("{context} width"))?;
    let height =
        CanonicalCoordinate::from_f64(height, precision, format_args!("{context} height"))?;
    if (positive_width && width.units() <= 0)
        || (positive_height && height.units() <= 0)
        || (require_positive && (width.units() <= 0 || height.units() <= 0))
        || (!require_positive && (width.units() < 0 || height.units() < 0))
    {
        return Err(Error::Render(format!(
            "{context} positive extent collapses at {}-decimal coordinate precision",
            precision.decimals()
        )));
    }
    let right = left.checked_add(width, format_args!("{context} right edge"))?;
    let bottom = top.checked_sub(height, format_args!("{context} bottom edge"))?;
    Ok(CanonicalRect {
        left,
        right,
        bottom,
        top,
        width,
        height,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalGradientStop {
    pub(crate) offset: CanonicalCoordinate,
    pub(crate) color: (u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalGradient {
    pub(crate) bounds: CanonicalRect,
    pub(crate) start: CanonicalPoint,
    pub(crate) end: CanonicalPoint,
    pub(crate) stops: Vec<CanonicalGradientStop>,
}

pub(crate) fn canonical_gradient(
    bounds: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: &[GradientStop],
) -> Result<CanonicalGradient, Error> {
    let mut converted_stops = Vec::new();
    converted_stops
        .try_reserve_exact(stops.len())
        .map_err(|error| Error::Render(format!("gradient stop reservation: {error}")))?;
    let (bounds, start, end) = visit_canonical_gradient(bounds, start, end, stops, |stop| {
        converted_stops.push(stop);
    })?;
    Ok(CanonicalGradient {
        bounds,
        start,
        end,
        stops: converted_stops,
    })
}

pub(crate) fn validate_gradient_geometry(
    bounds: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: &[GradientStop],
) -> Result<(), Error> {
    visit_canonical_gradient(bounds, start, end, stops, |_| {}).map(|_| ())
}

pub(crate) fn visit_canonical_gradient(
    bounds: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: &[GradientStop],
    mut emit: impl FnMut(CanonicalGradientStop),
) -> Result<(CanonicalRect, CanonicalPoint, CanonicalPoint), Error> {
    let bounds = canonical_panel_rect(
        bounds,
        CoordinatePrecision::Thousandths,
        "linear gradient bounds",
    )?;
    let mut endpoints = [CanonicalPoint::from_raw(
        start,
        CoordinatePrecision::Thousandths,
        "linear gradient vector point 0",
    )?; 2];
    endpoints[1] = CanonicalPoint::from_raw(
        end,
        CoordinatePrecision::Thousandths,
        "linear gradient vector point 1",
    )?;
    if same_finite_point(start, end) {
        return Err(Error::Render(
            "linear gradient vector endpoints must be distinct".into(),
        ));
    }
    if endpoints[0] == endpoints[1] {
        return Err(Error::Render(
            "linear gradient vector endpoints collapse at 3-decimal coordinate precision".into(),
        ));
    }
    if stops.len() < 2 {
        return Err(Error::Render(format!(
            "linear gradient requires at least two stops; got {}",
            stops.len()
        )));
    }
    for (index, stop) in stops.iter().enumerate() {
        if !stop.offset.is_finite() || !(0.0..=1.0).contains(&stop.offset) {
            return Err(Error::Render(format!(
                "linear gradient stop {index} offset must be finite and in [0, 1]; got {}",
                stop.offset
            )));
        }
    }
    if stops[0].offset.to_bits() != 0.0_f64.to_bits()
        || stops[stops.len() - 1].offset.to_bits() != 1.0_f64.to_bits()
    {
        return Err(Error::Render(format!(
            "linear gradient stops must begin at 0 and end at 1; got {}..{}",
            stops[0].offset,
            stops[stops.len() - 1].offset
        )));
    }
    for (index, pair) in stops.windows(2).enumerate() {
        if pair[0].offset >= pair[1].offset {
            return Err(Error::Render(format!(
                "linear gradient stop offsets must be strictly increasing; stops {index} and {} are {} and {}",
                index + 1,
                pair[0].offset,
                pair[1].offset
            )));
        }
    }

    let mut previous = None;
    for (index, stop) in stops.iter().enumerate() {
        let converted = CanonicalGradientStop {
            offset: CanonicalCoordinate::from_f64(
                stop.offset,
                CoordinatePrecision::Millionths,
                format_args!("linear gradient stop {index} offset"),
            )?,
            color: stop.color,
        };
        if previous.is_some_and(|offset| offset >= converted.offset.units()) {
            return Err(Error::Render(
                "linear gradient stop offsets collapse at 6-decimal coordinate precision".into(),
            ));
        }
        previous = Some(converted.offset.units());
        emit(converted);
    }
    Ok((bounds, endpoints[0], endpoints[1]))
}

fn same_finite_point(left: Point2D, right: Point2D) -> bool {
    same_finite_coordinate(left.x, right.x) && same_finite_coordinate(left.y, right.y)
}

fn same_finite_coordinate(left: f64, right: f64) -> bool {
    normalized_bits(left) == normalized_bits(right)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalText {
    pub(crate) position: CanonicalPoint,
    pub(crate) rotation: CanonicalCoordinate,
}

pub(crate) fn canonical_text(position: Point2D, rotation: f64) -> Result<CanonicalText, Error> {
    Ok(CanonicalText {
        position: CanonicalPoint::from_raw(
            position,
            CoordinatePrecision::Integer,
            "text position",
        )?,
        rotation: CanonicalCoordinate::from_f64(
            rotation,
            CoordinatePrecision::Tenths,
            "text rotation",
        )?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalRoundedRect {
    pub(crate) rect: CanonicalRect,
    pub(crate) radius: CanonicalCoordinate,
}

pub(crate) fn canonical_rounded_rect(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) -> Result<CanonicalRoundedRect, Error> {
    if [x, y, width, height, radius]
        .iter()
        .any(|value| !value.is_finite())
    {
        return Err(Error::Render(format!(
            "rounded rectangle must contain finite geometry; got ({x}, {y}, {width}, {height}, {radius})"
        )));
    }
    if width < 0.0 || height < 0.0 || radius < 0.0 {
        return Err(Error::Render(format!(
            "rounded rectangle dimensions and radius must be non-negative; got ({width}, {height}, {radius})"
        )));
    }
    let top = y + height;
    if !(x + width).is_finite() || !top.is_finite() {
        return Err(Error::Render(format!(
            "rounded rectangle far edges must be finite; got ({x}, {y}, {width}, {height})"
        )));
    }
    if radius > width / 2.0 || radius > height / 2.0 {
        return Err(Error::Render(format!(
            "rounded rectangle radius must not exceed half either extent; got radius {radius} for {width}x{height}"
        )));
    }

    let rect = canonical_rect(
        x,
        top,
        width,
        height,
        CoordinatePrecision::Hundredths,
        "rounded rectangle",
        false,
    )?;
    let radius = CanonicalCoordinate::from_f64(
        radius,
        CoordinatePrecision::Tenths,
        "rounded rectangle radius",
    )?;
    if radius.value() > rect.width.value() / 2.0 || radius.value() > rect.height.value() / 2.0 {
        return Err(Error::Render(
            "rounded rectangle radius exceeds a post-conversion half extent".into(),
        ));
    }
    Ok(CanonicalRoundedRect { rect, radius })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalBracket {
    pub(crate) orientation: BracketOrientation,
    pub(crate) line_start: CanonicalPoint,
    pub(crate) line_end: CanonicalPoint,
    pub(crate) first_tick_end: CanonicalPoint,
    pub(crate) second_tick_end: CanonicalPoint,
    pub(crate) label_position: CanonicalPoint,
}

pub(crate) fn canonical_bracket(
    bracket: &SignificanceBracket,
    tick_height: f64,
    label_gap: f64,
) -> Result<CanonicalBracket, Error> {
    if [
        bracket.x_left,
        bracket.x_right,
        bracket.y,
        tick_height,
        label_gap,
    ]
    .iter()
    .any(|value| !value.is_finite())
    {
        return Err(Error::Render(format!(
            "significance bracket must contain finite coordinates; got ({}, {}, {})",
            bracket.x_left, bracket.x_right, bracket.y
        )));
    }
    if same_finite_coordinate(bracket.x_left, bracket.x_right) {
        return Err(Error::Render(
            "significance bracket must span two distinct rendered anchors".into(),
        ));
    }

    let point = |x, y, context| {
        CanonicalPoint::from_raw(
            Point2D::new(x, y),
            CoordinatePrecision::Thousandths,
            context,
        )
    };
    let (line_start, line_end, first_tick_end, second_tick_end, label_position) =
        match bracket.orientation {
            BracketOrientation::Vertical => (
                point(bracket.x_left, bracket.y, "vertical bracket first anchor")?,
                point(bracket.x_right, bracket.y, "vertical bracket second anchor")?,
                point(
                    bracket.x_left,
                    bracket.y - tick_height,
                    "vertical bracket first tick",
                )?,
                point(
                    bracket.x_right,
                    bracket.y - tick_height,
                    "vertical bracket second tick",
                )?,
                point(
                    f64::midpoint(bracket.x_left, bracket.x_right),
                    bracket.y + label_gap,
                    "vertical bracket label",
                )?,
            ),
            BracketOrientation::Horizontal => (
                point(bracket.y, bracket.x_left, "horizontal bracket first anchor")?,
                point(
                    bracket.y,
                    bracket.x_right,
                    "horizontal bracket second anchor",
                )?,
                point(
                    bracket.y - tick_height,
                    bracket.x_left,
                    "horizontal bracket first tick",
                )?,
                point(
                    bracket.y - tick_height,
                    bracket.x_right,
                    "horizontal bracket second tick",
                )?,
                point(
                    bracket.y + label_gap,
                    f64::midpoint(bracket.x_left, bracket.x_right),
                    "horizontal bracket label",
                )?,
            ),
        };
    if line_start == line_end {
        return Err(Error::Render(
            "significance bracket anchors collapse at 3-decimal coordinate precision".into(),
        ));
    }
    Ok(CanonicalBracket {
        orientation: bracket.orientation,
        line_start,
        line_end,
        first_tick_end,
        second_tick_end,
        label_position,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_signed_coordinate_range_is_closed_under_canvas_inversion() {
        let reserve = i64::from(u32::MAX);
        let input_limit = MAX_EXACT_UNITS - reserve;
        let coordinate = CanonicalCoordinate::from_units(
            -input_limit,
            CoordinatePrecision::Integer,
            false,
            "test coordinate",
        )
        .unwrap();

        let inverted = coordinate.inverted(u32::MAX).unwrap();

        assert_eq!(inverted.units(), MAX_EXACT_UNITS);
    }
}
