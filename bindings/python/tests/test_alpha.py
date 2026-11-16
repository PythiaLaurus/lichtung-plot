"""Alpha declarations, original scopes and borrow-free binding-time mappers."""

import xml.etree.ElementTree as ET

import pytest

import lichtung_plot as lp

SVG = "{http://www.w3.org/2000/svg}"


def texts(path):
    return [
        "".join(node.itertext())
        for node in ET.parse(path.with_suffix(".svg")).getroot().iter(f"{SVG}text")
    ]


def alphas(path, paint, tag=None):
    root = ET.parse(path.with_suffix(".svg")).getroot()
    nodes = root.iter(f"{SVG}{tag}") if tag else root.iter()
    return [
        round(float(node.get(f"{paint}-opacity", "1")) * 255)
        for node in nodes
        if node.get(paint) not in (None, "none")
    ]


def test_mappers_see_owned_inputs_and_original_errors_precede_native_and_stale_errors(tmp_path):
    np = pytest.importorskip("numpy")
    pd = pytest.importorskip("pandas")
    plot = lp.line(
        x=[0.0, 1.0], y=[1.0, 2.0], hue=["h", "h"], group=["g", "g"],
    ).legend("hidden")
    selector = pd.DataFrame({"hk": ["h"], "gk": ["g"]})
    q = np.array([0.5])
    def mutate_sources(value):
        q[:] = 0.9
        selector["hk"] = ["changed"]
        selector["gk"] = ["changed"]
        return 83 if value <= 0.5 else 171

    plot.line_alpha_keyed_map_from_data(
        selector, input=q, keys=[("hue", "hk"), ("group", "gk")],
        continuous=((0.0, 1.0), mutate_sources),
    )
    plot.svg_preserve_text(tmp_path / "owned-before-mapper")
    assert 83 in alphas(tmp_path / "owned-before-mapper", "stroke")

    owner = lp.scatter(x=[0.0, 1.0], y=[1.0, 2.0]).legend("hidden")
    owner.point_alpha_single(value=71)
    original = LookupError("original mapper exception")
    failed_calls = []

    def fail(value):
        failed_calls.append(value)
        owner.title("Inner edit survives failure")
        raise original

    with pytest.raises(LookupError) as raised:
        # Missing expansion is a later producer error; it must not mask the PyErr.
        owner.point_alpha_columns_map(
            [0.5], column_key=[0], continuous=((0.0, 1.0), fail),
        )
    assert raised.value is original and len(failed_calls) == 1

    def reenter(value):
        owner.title("Inner edit survives stale candidate")
        return 200

    with pytest.raises(RuntimeError, match="stale"):
        owner.point_alpha_map([0.5, 0.5], continuous=((0.0, 1.0), reenter))
    owner.svg_preserve_text(tmp_path / "stale-safe-alpha")
    assert "Inner edit survives stale candidate" in texts(tmp_path / "stale-safe-alpha")
    assert alphas(tmp_path / "stale-safe-alpha", "fill", "circle") == [71, 71]
