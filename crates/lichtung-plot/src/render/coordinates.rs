//! Checked fixed-precision geometry shared by validating and drawing backends.

use std::fmt::{self, Write as _};

use crate::layout::PanelBox;
use crate::render::backend::{BracketOrientation, GradientStop, Point2D, SignificanceBracket};
use crate::Error;

const MAX_EXACT_UNITS: i64 = (1_i64 << 53) - 1;

#[cfg(test)]
thread_local! {
    pub(crate) static SPATIAL_CONVERSIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static COORDINATE_CONVERSIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static Y_CONVERSIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

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
    CssQuantum,
    Integer,
    Millionths,
}

impl CoordinatePrecision {
    const fn decimals(self) -> usize {
        match self {
            Self::CssQuantum => 10,
            Self::Integer => 0,
            Self::Millionths => 6,
        }
    }

    const fn scale(self) -> i64 {
        match self {
            Self::CssQuantum => 1024,
            Self::Integer => 1,
            Self::Millionths => 1_000_000,
        }
    }

    const fn unsigned_scale(self) -> u64 {
        match self {
            Self::CssQuantum => 1024,
            Self::Integer => 1,
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
    pub(crate) const fn from_quanta(units: i64) -> Self {
        Self {
            units,
            precision: CoordinatePrecision::CssQuantum,
            negative_zero: false,
        }
    }
    pub(crate) fn nonnegative_length(value: f64, context: &str) -> Result<Self, Error> {
        let length = Self::from_f64(value, CoordinatePrecision::CssQuantum, context)?;
        if value < 0.0 || (value > 0.0 && length.units() == 0) {
            return Err(Error::Render(format!("{context} must be nonnegative and its positive extent must not collapse at 1/1024 CSS-pixel coordinate precision")));
        }
        Ok(length)
    }

    pub(crate) fn from_f64(
        value: f64,
        precision: CoordinatePrecision,
        context: impl fmt::Display,
    ) -> Result<Self, Error> {
        #[cfg(test)]
        COORDINATE_CONVERSIONS.with(|count| count.set(count.get() + 1));
        if precision == CoordinatePrecision::CssQuantum {
            #[cfg(test)]
            SPATIAL_CONVERSIONS.with(|count| count.set(count.get() + 1));
            return Ok(Self {
                units: crate::layout::kernel::nearest_spatial_quanta(value)
                    .map_err(|error| Error::Render(format!("{context}: {error}")))?,
                precision,
                negative_zero: false,
            });
        }
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
        if precision == CoordinatePrecision::CssQuantum {
            return Ok(Self {
                units,
                precision,
                negative_zero: false,
            });
        }
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
        reason = "geometry measurement may approximate signed quanta; final serialization uses integer units"
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
        #[cfg(test)]
        Y_CONVERSIONS.with(|count| count.set(count.get() + 1));
        let height_units = i128::from(canvas_height) * i128::from(self.precision.scale());
        let units = i64::try_from(height_units - i128::from(self.units))
            .map_err(|_| Error::Render("inverted coordinate exceeds signed range".into()))?;
        Self::from_derived_units(units, self.precision, "inverted coordinate")
    }

    fn checked_add(self, other: Self, context: impl fmt::Display) -> Result<Self, Error> {
        self.checked_binary(other, context, i128::checked_add)
    }

    fn checked_sub(self, other: Self, context: impl fmt::Display) -> Result<Self, Error> {
        self.checked_binary(other, context, i128::checked_sub)
    }

    fn checked_binary(
        self,
        other: Self,
        context: impl fmt::Display,
        operation: fn(i128, i128) -> Option<i128>,
    ) -> Result<Self, Error> {
        if self.precision != other.precision {
            return Err(Error::Render(format!(
                "{context} mixes incompatible coordinate precision"
            )));
        }
        let units = operation(i128::from(self.units), i128::from(other.units))
            .and_then(|units| i64::try_from(units).ok())
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
        if self.precision == CoordinatePrecision::CssQuantum {
            if fraction == 0 {
                return write!(formatter, "{whole}");
            }
            let mut decimal = fraction * 9_765_625;
            let mut width = 10;
            while decimal.is_multiple_of(10) {
                decimal /= 10;
                width -= 1;
            }
            return write!(formatter, "{whole}.{decimal:0width$}");
        }
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
    pub(crate) fn inverted(self, height: u32) -> Result<Self, Error> {
        Ok(Self {
            x: self.x,
            y: self.y.inverted(height)?,
        })
    }
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
                CoordinatePrecision::CssQuantum,
                format_args!("{context} point {index}"),
            )
        })
}

pub(crate) fn canonical_line_points(points: &[Point2D]) -> Result<Vec<CanonicalPoint>, Error> {
    let converted = canonical_points(points, "line")?;
    let mut indices = Vec::new();
    indices
        .try_reserve_exact(points.len())
        .map_err(|error| Error::Render(format!("line validation reservation: {error}")))?;
    indices.extend(0..points.len());
    validate_line_geometry(points, &converted, &mut indices)?;
    Ok(converted)
}

pub(crate) fn validate_line_geometry(
    points: &[Point2D],
    converted: &[CanonicalPoint],
    indices: &mut [usize],
) -> Result<(), Error> {
    indices.sort_unstable_by_key(|&index| {
        (
            converted[index].x.units(),
            converted[index].y.units(),
            index,
        )
    });
    let conflict = indices
        .windows(2)
        .filter(|pair| {
            converted[pair[0]].same_position(converted[pair[1]])
                && !same_finite_point(points[pair[0]], points[pair[1]])
        })
        .min_by_key(|pair| pair[1]);
    if let Some(pair) = conflict {
        let (first_index, second_index) = (pair[0], pair[1]);
        let (first, second) = (points[first_index], points[second_index]);
        return Err(Error::Render(format!("line distinct points {first_index} ({}, {}) and {second_index} ({}, {}) collapse at 1/1024 CSS-pixel coordinate precision", first.x, first.y, second.x, second.y)));
    }
    Ok(())
}

pub(crate) fn canonical_polygon_points(points: &[Point2D]) -> Result<Vec<CanonicalPoint>, Error> {
    let converted = canonical_points(points, "polygon")?;
    validate_polygon_geometry(points, &converted)?;
    Ok(converted)
}

pub(crate) fn validate_polygon_geometry(
    points: &[Point2D],
    converted: &[CanonicalPoint],
) -> Result<(), Error> {
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
                "polygon distinct adjacent points {index} ({}, {}) and {next} ({}, {}) collapse at 1/1024 CSS-pixel coordinate precision",
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
            "polygon required distinct points collapse at 1/1024 CSS-pixel coordinate precision"
                .into(),
        ));
    }
    Ok(())
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
        CoordinatePrecision::CssQuantum,
        format_args!("triangle {index} point 0"),
    )?; 3];
    for i in 1..3 {
        result[i] = CanonicalPoint::from_raw(
            raw[i],
            CoordinatePrecision::CssQuantum,
            format_args!("triangle {index} point {i}"),
        )?;
        for j in 0..i {
            if !same_finite_point(raw[i], raw[j]) && result[i].same_position(result[j]) {
                return Err(Error::Render(format!(
                    "triangle {index} distinct points collapse at 1/1024 CSS-pixel coordinate precision"
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

/// Final top-down rectangle; canvas conversion precedes counting and replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalBox {
    pub(crate) x: CanonicalCoordinate,
    pub(crate) y: CanonicalCoordinate,
    pub(crate) width: CanonicalCoordinate,
    pub(crate) height: CanonicalCoordinate,
}

impl CanonicalBox {
    pub(crate) fn from_bottom_up(rect: CanonicalRect, height: u32) -> Result<Self, Error> {
        rect.bottom.inverted(height)?;
        Ok(Self {
            x: rect.left,
            y: rect.top.inverted(height)?,
            width: rect.width,
            height: rect.height,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CanonicalRoundedBox {
    pub(crate) bounds: CanonicalBox,
    pub(crate) radius: CanonicalCoordinate,
    pub(crate) stroke_width: CanonicalCoordinate,
    pub(crate) color: crate::theme::Color,
    pub(crate) stroke: crate::theme::Color,
}

impl CanonicalRoundedBox {
    pub(crate) fn from_bottom_up(
        rounded: CanonicalRoundedRect,
        style: &crate::render::backend::FillStyle,
        height: u32,
    ) -> Result<Self, Error> {
        Ok(Self {
            bounds: CanonicalBox::from_bottom_up(rounded.rect, height)?,
            radius: rounded.radius,
            stroke_width: CanonicalCoordinate::nonnegative_length(
                f64::from(style.stroke_width),
                "rectangle stroke width",
            )?,
            color: style.color,
            stroke: style.stroke,
        })
    }
}

pub(crate) fn canonical_clip(region: PanelBox) -> Result<CanonicalRect, Error> {
    canonical_panel_rect(region, CoordinatePrecision::CssQuantum, "clip region")
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
            "{context} positive extent collapses at {} coordinate precision",
            if precision == CoordinatePrecision::CssQuantum {
                "1/1024 CSS-pixel".into()
            } else {
                format!("{}-decimal", precision.decimals())
            }
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

pub(crate) fn visit_canonical_gradient(
    bounds: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: &[GradientStop],
    mut emit: impl FnMut(CanonicalGradientStop),
) -> Result<(CanonicalRect, CanonicalPoint, CanonicalPoint), Error> {
    let bounds = canonical_panel_rect(
        bounds,
        CoordinatePrecision::CssQuantum,
        "linear gradient bounds",
    )?;
    let mut endpoints = [CanonicalPoint::from_raw(
        start,
        CoordinatePrecision::CssQuantum,
        "linear gradient vector point 0",
    )?; 2];
    endpoints[1] = CanonicalPoint::from_raw(
        end,
        CoordinatePrecision::CssQuantum,
        "linear gradient vector point 1",
    )?;
    if same_finite_point(start, end) {
        return Err(Error::Render(
            "linear gradient vector endpoints must be distinct".into(),
        ));
    }
    if endpoints[0] == endpoints[1] {
        return Err(Error::Render(
            "linear gradient vector endpoints collapse at 1/1024 CSS-pixel coordinate precision"
                .into(),
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
pub(crate) struct CanonicalAngle(crate::TextRotation);

impl CanonicalAngle {
    pub(crate) const ZERO: Self = Self(crate::TextRotation::horizontal());
    pub(crate) fn is_rotated(self) -> bool {
        self.0 != crate::TextRotation::horizontal()
    }

    pub(crate) fn value(self) -> f64 {
        self.0 .0.get()
    }
}

impl fmt::Display for CanonicalAngle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value().fmt(formatter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalText {
    pub(crate) position: CanonicalPoint,
    pub(crate) rotation: CanonicalAngle,
}

pub(crate) fn canonical_text(
    position: Point2D,
    rotation: f64,
    precision: CoordinatePrecision,
) -> Result<CanonicalText, Error> {
    let angle = crate::TextRotation::degrees(rotation)?;
    Ok(CanonicalText {
        position: CanonicalPoint::from_raw(position, precision, "text position")?,
        rotation: CanonicalAngle(angle),
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
        CoordinatePrecision::CssQuantum,
        "rounded rectangle",
        false,
    )?;
    let radius = CanonicalCoordinate::nonnegative_length(radius, "rounded rectangle radius")?;
    if i128::from(radius.units()) * 2 > i128::from(rect.width.units())
        || i128::from(radius.units()) * 2 > i128::from(rect.height.units())
    {
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
        CanonicalPoint::from_raw(Point2D::new(x, y), CoordinatePrecision::CssQuantum, context)
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
    fn spatial_quantum_ties_zero_and_spelling_are_exact() {
        for (value, units, spelling) in [
            (-0.0, 0, "0"),
            (-0.5 / 1024.0, 0, "0"),
            (0.5 / 1024.0, 0, "0"),
            (1.5 / 1024.0, 2, "0.001953125"),
            (2.5 / 1024.0, 2, "0.001953125"),
            (-1.5 / 1024.0, -2, "-0.001953125"),
            (-1.0 / 1024.0, -1, "-0.0009765625"),
            (1.25, 1280, "1.25"),
        ] {
            let coordinate =
                CanonicalCoordinate::from_f64(value, CoordinatePrecision::CssQuantum, "test")
                    .unwrap();
            assert_eq!(coordinate.units(), units);
            assert_eq!(coordinate.to_string(), spelling);
        }
        for value in [f64::NAN, f64::INFINITY, f64::MAX] {
            assert!(
                CanonicalCoordinate::from_f64(value, CoordinatePrecision::CssQuantum, "test")
                    .is_err()
            );
        }
    }

    #[test]
    fn spatial_signed_range_and_bottom_up_overflow_are_checked() {
        let minimum = CanonicalCoordinate::from_f64(
            -9_007_199_254_740_992.0,
            CoordinatePrecision::CssQuantum,
            "test",
        )
        .unwrap();
        assert_eq!(minimum.units(), i64::MIN);
        assert!(minimum.inverted(1).is_err());
        assert!(CanonicalCoordinate::from_f64(
            9_007_199_254_740_992.0,
            CoordinatePrecision::CssQuantum,
            "test"
        )
        .is_err());
        let rect = canonical_clip(PanelBox::new(-0.25, 1.25, -2.5, 3.125)).unwrap();
        let native = CanonicalBox::from_bottom_up(rect, 100).unwrap();
        assert_eq!(native.x.to_string(), "-0.25");
        assert_eq!(native.y.to_string(), "96.875");
        assert_eq!(native.width.to_string(), "1.5");
        assert_eq!(native.height.to_string(), "5.625");
    }

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
