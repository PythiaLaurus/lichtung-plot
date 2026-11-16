"""Public constructor consumers, not a family-by-container transport matrix."""

import xml.etree.ElementTree as ET

import pytest

import lichtung_plot as lp

SVG = "{http://www.w3.org/2000/svg}"


def texts(path):
    return [
        "".join(node.itertext())
        for node in ET.parse(path.with_suffix(".svg")).getroot().iter(f"{SVG}text")
    ]


def test_trajectory_axis_is_dtype_driven_and_ambiguity_is_not_fallback(tmp_path):
    np = pytest.importorskip("numpy")
    pd = pytest.importorskip("pandas")
    pl = pytest.importorskip("polars")
    frame = pd.DataFrame({1: pd.Series([1.0, 2.0, 3.0], dtype="Float64")})
    numeric = lp.trajectory(frame, x=lp.col[1], y=[2, 1, 3], group=["g"] * 3).x_limits(0, 4)
    categorical = lp.trajectory(
        x=np.array(["30", "10", "20"]), y=[2, 1, 3], group=["g"] * 3,
    )
    canvas = lp.Canvas.grid(1, 2).size(2400, 1000).panel(
        0, 0, numeric.title("Typed numeric"),
    ).panel(0, 1, categorical.title("Numeric-looking categories"))
    canvas.svg_preserve_text(tmp_path / "typed-axes")
    assert [text for text in texts(tmp_path / "typed-axes") if text in {"30", "10", "20"}] == [
        "30", "10", "20",
    ]
    for x in ([], [None, None], [True, False], np.array([True, False])):
        with pytest.raises(TypeError):
            lp.trajectory(x=x, y=[1, 2], group=["g", "g"])
    with pytest.raises(TypeError):
        lp.trajectory(x=pl.Series("untyped", [None, None]), y=[1, 2], group=["g", "g"])
    with pytest.raises(TypeError):
        lp.trajectory(x=pd.Series([None, None], dtype=object), y=[1, 2], group=["g", "g"])
    # Type establishes the axis even when the native useful-data check will fail.
    lp.trajectory(x=np.array([], dtype=float), y=[], group=[])
    lp.trajectory(x=pd.Series([None, None], dtype="string"), y=[1, 2], group=["g", "g"])
