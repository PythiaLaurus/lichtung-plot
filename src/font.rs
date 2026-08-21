//! System-font defaults used by SVG and PNG rendering.

/// Preferred system font family.
pub const DEFAULT_FONT_FAMILY: &str = "Noto Sans";

/// Ordered CSS system-font fallback stack used by built-in themes.
pub const DEFAULT_FONT_FAMILY_STACK: &str =
    "Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif";

/// Ordered concrete families considered when configuring PNG text rendering.
pub const SYSTEM_FONT_FAMILIES: &[&str] = &[
    "Noto Sans",
    "Noto Sans CJK SC",
    "DejaVu Sans",
    "Liberation Sans",
    "Arial",
];

#[cfg(feature = "png")]
pub(crate) fn configure_system_fonts(
    database: &mut resvg::usvg::fontdb::Database,
) -> crate::Result<()> {
    use resvg::usvg::fontdb::{Family, Query};

    database.load_system_fonts();
    if database.is_empty() {
        return Err(crate::Error::Render(
            "no system fonts were found for PNG text rendering".into(),
        ));
    }

    let selected = SYSTEM_FONT_FAMILIES
        .iter()
        .find(|family| {
            database
                .query(&Query {
                    families: &[Family::Name(family)],
                    ..Query::default()
                })
                .is_some()
        })
        .map(|family| (*family).to_owned())
        .or_else(|| {
            database
                .faces()
                .find_map(|face| face.families.first().map(|family| family.0.clone()))
        })
        .ok_or_else(|| crate::Error::Render("system font database contains no families".into()))?;

    database.set_sans_serif_family(selected);
    Ok(())
}
