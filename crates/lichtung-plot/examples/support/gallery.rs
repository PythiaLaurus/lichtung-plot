//! Shared public-API appearance for gallery examples.

use lichtung_plot::{
    AccentPalette, Color, FrameSides, FrameTheme, OpaqueColor, QualitativePalette, StrokePattern,
    StrokeTheme, Theme,
};

// ANCHOR: gallery-theme
pub fn theme() -> lichtung_plot::Result<Theme> {
    let base = Theme::publication();
    let colors = ["#207F83", "#D47758", "#7772A5", "#B39545"]
        .map(OpaqueColor::hex)
        .into_iter()
        .collect::<lichtung_plot::Result<Vec<_>>>()?;
    let axes = base.axes().clone().grid(None)?.frame(Some(FrameTheme::new(
        FrameSides::new(false, false, true, true)?,
        StrokeTheme::new(Color::hex("#9CA8AE")?, 0.8, StrokePattern::Solid)?,
    )?))?;
    let root = base
        .typography()
        .get_root()
        .clone()
        .title(
            base.typography()
                .get_root()
                .get_title()
                .clone()
                .font_size(22.0)?
                .color(Color::hex("#263C46")?)?,
        )?
        .subtitle(
            base.typography()
                .get_root()
                .get_subtitle()
                .clone()
                .font_size(14.0)?
                .color(Color::hex("#63737A")?)?,
        )?
        .caption(
            base.typography()
                .get_root()
                .get_caption()
                .clone()
                .font_size(11.0)?,
        )?;
    let radii = base
        .marks()
        .get_points()
        .get_radii()
        .clone()
        .scatter(4)?
        .regression(3)?
        .interval_estimate(5)?;
    let points = base.marks().get_points().clone().radii(radii)?;
    base.customize()
        .axes(axes)
        .typography(base.typography().clone().root(root)?)
        .marks(
            base.marks()
                .clone()
                .points(points)?
                .accents(AccentPalette::new(colors.clone())?)?,
        )
        .color_scales(
            base.color_scales()
                .clone()
                .qualitative(QualitativePalette::fixed(colors)?)?,
        )
        .build()
}
// ANCHOR_END: gallery-theme
