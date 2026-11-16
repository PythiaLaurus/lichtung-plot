use super::*;
use lichtung_plot::{FontFamily, FontStack};

#[test]
fn active_custom_font_roles_are_required_while_unused_preferences_are_not() {
    let base = Theme::publication();
    let title = base
        .typography()
        .get_root()
        .get_title()
        .clone()
        .stack(
            FontStack::new([FontFamily::named("Unavailable public heading role").unwrap()])
                .unwrap(),
        )
        .unwrap();
    let root = base.typography().get_root().clone().title(title).unwrap();
    let typography = base.typography().clone().root(root).unwrap();
    let theme = base.customize().typography(typography).build().unwrap();
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0]),
        NumericValues::new("y", [2.0, 1.0]),
    )
    .theme(theme);
    plot.preflight().unwrap();
    let error = plot
        .title("This role is now consumed")
        .preflight()
        .unwrap_err();
    assert!(matches!(error, lichtung_plot::Error::Render(_)), "{error}");
    assert!(
        error.to_string().contains("font candidate list is empty"),
        "{error}"
    );
}
