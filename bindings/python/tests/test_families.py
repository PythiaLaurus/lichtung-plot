"""Public constructor consumers, not a family-by-container transport matrix."""

import gc
import weakref
import xml.etree.ElementTree as ET

import pytest

import lichtung_plot as lp


SVG = "{http://www.w3.org/2000/svg}"


def texts(path):
    return [
        "".join(node.itertext())
        for node in ET.parse(path.with_suffix(".svg")).getroot().iter(f"{SVG}text")
    ]


def test_family_constructors_return_their_exact_owners():
    # Native family publication is covered by the scientific gallery; this
    # boundary checks each Python constructor's exact returned owner.
    samples = [1.0, 1.5, 2.5, 3.0, 2.0, 2.5, 3.5, 4.0]
    categories = ["a"] * 4 + ["b"] * 4
    plots = [
        ("Histogram", lp.histogram(samples=samples)),
        ("Density", lp.density(value=samples)),
        ("Qq", lp.qq(value=samples)),
        ("PointInterval", lp.point_interval(
            category=["a", "b"], estimate=[1, 2], lower=[0.5, 1.5], upper=[1.5, 2.5],
        )),
        ("Line", lp.line(x=[0, 2, 7], y=[1, 3, 2])),
        ("Trajectory", lp.trajectory(
            x=["early", "late", "early", "late"], y=[1, 2, 2, 3],
            group=["first", "first", "second", "second"],
        )),
        ("LinePath", lp.line_path(
            x=[1, 0, 1], y=[1, 2, 3], group=["path"] * 3,
        )),
        ("Interaction", lp.interaction(
            condition=["before", "after"], estimate=[1, 2], group=["g", "g"],
            lower=[0.5, 1.5], upper=[1.5, 2.5],
        )),
        ("Coefficient", lp.coefficient(term=["a", "b"], estimate=[1, 2])),
        ("Forest", lp.forest(
            term=["a", "b"], estimate=[1, 2], lower=[0.5, 1.5],
            upper=[1.5, 2.5], reference=1,
        )),
        ("Dumbbell", lp.dumbbell(
            category=["a", "b"], left=[1, 2], right=[2, 3],
            left_label="Before", left_color=lp.Color(25, 74, 117, alpha=180),
            right_label="After", right_color=lp.Color(186, 91, 35, alpha=220),
        )),
        ("Heatmap", lp.heatmap(grid=[[1, 2], [3, 4]])),
        ("Bar", lp.bar(category=categories, samples=samples)),
        ("HorizontalBar", lp.horizontal_bar(category=["a", "b"], estimate=[1, 2])),
        ("StackedBar", lp.stacked_bar(
            category=["a", "a", "b", "b"], value=[1, 2, 2, 1],
            series=["first", "second"] * 2,
        )),
        ("StackedHorizontalBar", lp.stacked_horizontal_bar(
            category=["a", "a", "b", "b"], value=[1, 2, 2, 1],
            series=["first", "second"] * 2,
        )),
        ("Boxplot", lp.boxplot(category=categories, value=samples)),
        ("Violin", lp.violin(category=categories, value=samples)),
        ("Raincloud", lp.raincloud(category=categories, value=samples)),
        ("Regression", lp.regression(x=[0, 1, 2, 3, 4], y=[1, 2.2, 2.8, 4.1, 5])),
        ("Marginal", lp.marginal(x=[0, 1, 2, 3, 4], y=[1, 2, 1.5, 3, 4])),
        ("Matrix", lp.matrix(variables=[[0, 1], [1, 3], [2, 2], [3, 4]])),
        ("Beeswarm", lp.beeswarm(
            category=categories, value=samples, color=[0, 1, 2, 3, 0, 1, 2, 3],
        )),
        ("Waterfall", lp.waterfall(category=["first", "second"], delta=[2, -1], base=10)),
    ]
    assert len(plots) == 24
    for name, plot in plots:
        assert type(plot).__name__ == name


def test_complete_combinations_and_required_none_precede_lookup():
    pd = pytest.importorskip("pandas")
    guarded = False

    class MetadataLookup(RuntimeError):
        pass

    class GuardedFrame(pd.DataFrame):
        @property
        def shape(self):
            if guarded:
                raise MetadataLookup("argument combination must precede source metadata")
            return super().shape

    table = GuardedFrame({"present": [1.0]})
    guarded = True
    with pytest.raises(TypeError):
        lp.bar(table, category=lp.col["c"], samples=lp.col["v"], estimate=lp.col["e"])
    with pytest.raises(TypeError):
        lp.bar(table, category=lp.col["c"], estimate=lp.col["e"], lower=lp.col["lo"])
    with pytest.raises(TypeError):
        lp.bar(table, category=lp.col["c"], estimate=lp.col["e"], error_bar="hidden")
    with pytest.raises(TypeError):
        lp.bar(table, category=lp.col["c"], samples=lp.col["v"], error_bar=None)
    with pytest.raises(TypeError):
        lp.heatmap(table, grid=lp.cols["a"], x=lp.col["x"])
    with pytest.raises(TypeError):
        lp.histogram(table, samples=lp.col["v"], bin_lower=lp.col["lo"])
    with pytest.raises(TypeError):
        lp.histogram(table, bin_lower=lp.col["lo"])
    with pytest.raises(TypeError):
        lp.raincloud(table, category=lp.col["c"], value=lp.col["v"], q1=lp.col["q"])
    with pytest.raises(TypeError):
        lp.line(table, x=lp.col["x"], y=lp.cols["y"], by_row_position=True)
    with pytest.raises(TypeError):
        lp.matrix(table, variables=lp.cols["v"], numeric_only=True)
    with pytest.raises(TypeError):
        lp.forest(table, term=lp.col["t"], estimate=lp.col["e"],
                  lower=lp.col["lo"], upper=lp.col["hi"], reference=None)
    with pytest.raises(TypeError):
        lp.waterfall(table, category=lp.col["c"], delta=lp.col["d"], base=True)
    with pytest.raises(TypeError):
        lp.density(table, value=None)


def test_error_policies_and_supplied_endpoints_are_distinct_consumers(tmp_path):
    pd = pytest.importorskip("pandas")
    units = pd.DataFrame({"unit": ["one", "one", "two", "two", "three", "three"]})
    units_ref = weakref.ref(units)
    bootstrap = lp.Bootstrap(resamples=31, seed=7, units=lp.col["unit"], data=units)
    samples = [1, 2, 2, 3, 3, 4]
    category = ["a"] * 6
    policies = [
        ("Student", lp.ConfidenceInterval(level=0.9, bootstrap=None)),
        ("Units", lp.ConfidenceInterval(bootstrap=bootstrap)),
    ]
    canvas = lp.Canvas.grid(2, 4).size(4000, 1800)
    for index, (title, policy) in enumerate(policies):
        plot = lp.bar(category=category, samples=samples, error_bar=policy).title(title)
        canvas.panel(index // 4, index % 4, plot)
    canvas.panel(1, 2, lp.horizontal_bar(
        category=["a", "b"], estimate=[1, 2], lower=[0.5, 1.5], upper=[1.5, 2.5],
    ).title("Supplied bars"))
    canvas.panel(1, 3, lp.coefficient(
        term=["a", "b"], estimate=[1, 2], lower=[0.5, 1.5], upper=[1.5, 2.5],
    ).title("Supplied coefficients"))
    del units, bootstrap
    gc.collect()
    assert units_ref() is None
    canvas.svg_preserve_text(tmp_path / "error-policy-consumers")
    assert {"Units", "Student", "Supplied bars", "Supplied coefficients"} <= set(
        texts(tmp_path / "error-policy-consumers")
    )
    # Opaque units are checked by the real raw-sample consumer, not its value class.
    bad = lp.bar(category=category, samples=samples, error_bar=lp.ConfidenceInterval(
        bootstrap=lp.Bootstrap(resamples=5, units=["one"]),
    ))
    with pytest.raises(lp.DataError):
        bad.svg_preserve_text(tmp_path / "misaligned-units")


def test_explicit_discovery_and_generated_positions_use_real_wide_producers(tmp_path):
    pd = pytest.importorskip("pandas")
    table = pd.DataFrame({
        "measurement": [1.0, 2.0, 1.5, 3.0],
        "other": [2.0, 3.0, 2.5, 4.0],
        "numeric_id": [10, 11, 12, 13],
        "metadata": ["not a measurement"] * 4,
        "flag": [True, False, True, False],
    }, index=[100, 200, 500, 900])
    plots = [
        lp.line(table, by_row_position=True, numeric_only=True),
        lp.boxplot(table, by_column_name=True, numeric_only=True),
        lp.violin(table, by_column_name=True, numeric_only=True),
        lp.raincloud(table, by_column_name=True, numeric_only=True),
        lp.heatmap(x=["left", "right"], y=["row", "row"], fill=[1, 2]).title("Record heatmap"),
    ]
    matrix = lp.matrix(table, numeric_only=True)
    canvas = lp.Canvas.grid(2, 3).size(3600, 2400)
    for index, plot in enumerate(plots):
        canvas.panel(index // 3, index % 3, plot)
    canvas.svg_preserve_text(tmp_path / "discovery")
    matrix.size(1800, 1800).svg_preserve_text(tmp_path / "discovery-matrix")
    rendered = set(texts(tmp_path / "discovery") + texts(tmp_path / "discovery-matrix"))
    assert {"measurement", "other", "numeric_id"} <= rendered
    assert "metadata" not in rendered and "flag" not in rendered
    with pytest.raises(TypeError):
        lp.matrix(table)
    with pytest.raises(TypeError):
        lp.boxplot(table, value=lp.cols["measurement", "metadata"], by_column_name=True)
    # Direct rectangles are explicit collections, even when they have one column.
    one = lp.line(y=[[1], [2], [3]], by_row_position=True)
    with pytest.raises(lp.DataError):
        one.band([0, 1, 2], [2, 3, 4])
    one.band([[0], [1], [2]], [[2], [3], [4]])


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


def test_flat_geometry_has_independent_domains_and_releases_auxiliary_sources(tmp_path):
    np = pytest.importorskip("numpy")
    pd = pytest.importorskip("pandas")
    bins = pd.DataFrame({"lower": [0.0, 1.0], "upper": [1.0, 2.0], "density": [0.5, 0.5]})
    kde = np.array([[0.0, 0.0], [1.0, 0.5], [2.0, 0.0]])
    rugs = pd.DataFrame({"measurement": [0.25, 1.75], "base": [0.0, 0.0], "top": [0.1, 0.1]})
    main_ref, kde_ref, rugs_ref = weakref.ref(bins), weakref.ref(kde), weakref.ref(rugs)
    histogram = lp.histogram(
        bins, bin_lower="lower", bin_upper="upper", bin_density="density",
        kde_data=kde, kde_x=lp.col[0], kde_y=lp.col[1],
        rug_data=rugs, rug_x="measurement", rug_baseline="base", rug_top="top",
        value_label=lp.Text("Supplied measurement"), density_label="Supplied density",
    )
    raw = ["A  B", "A B"]
    summary = pd.DataFrame({"condition": raw, "treatment": ["x", "y"],
                            "q1": [1.0, 1.0], "median": [2.0, 2.0], "q3": [3.0, 3.0]})
    density = pd.DataFrame({
        "condition": raw * 3, "treatment": ["x", "y"] * 3,
        "support": [0.0, 0.0, 2.0, 2.0, 4.0, 4.0],
        "width": [0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
    })
    points = pd.DataFrame({"condition": raw[::-1], "treatment": ["y", "x"],
                           "measurement": [1.0, 3.0], "strip": [0.25, 0.75]})
    density_ref, points_ref = weakref.ref(density), weakref.ref(points)
    raincloud = lp.raincloud(
        summary, category="condition", hue="treatment", q1="q1", median="median", q3="q3",
        density_data=density, density_category="condition", density_hue="treatment",
        density_y="support", normalized_width="width",
        point_data=points, point_category="condition", point_hue="treatment",
        point_y="measurement", normalized_strip="strip",
        category_label="Raw conditions", value_label="Original observations", hue_label="Treatment",
    ).labels(lp.Labels(mode="raw"))
    plain = lp.raincloud(
        category=["a"], q1=[1], median=[2], q3=[3],
        density_category=["a"] * 3, density_y=[0, 2, 4], normalized_width=[0, 1, 0],
        point_category=["a", "a"], point_y=[1, 3], normalized_strip=[0.25, 0.75],
    ).title("No hue geometry")
    del bins, kde, rugs, density, points
    gc.collect()
    assert main_ref() is not None
    assert kde_ref() is None and rugs_ref() is None
    assert density_ref() is None and points_ref() is None
    canvas = lp.Canvas.grid(1, 3).size(3600, 1200).panel(0, 0, histogram).panel(
        0, 1, raincloud,
    ).panel(0, 2, plain)
    canvas.svg_preserve_text(tmp_path / "complete-geometry")
    assert {"Supplied measurement", "Supplied density", "Raw conditions",
            "Original observations", "Treatment", *raw} <= set(texts(tmp_path / "complete-geometry"))


def test_auxiliary_none_never_falls_back_and_geometry_keeps_native_validation(tmp_path):
    pd = pytest.importorskip("pandas")
    main = pd.DataFrame({"lo": [0.0], "hi": [2.0], "density": [0.5],
                         "x": [1.0], "y": [0.5]})
    with pytest.raises(TypeError):
        lp.histogram(
            main, bin_lower="lo", bin_upper="hi", bin_density="density",
            kde_x=lp.col["x"], kde_y=lp.col["y"],
            rug_x=[1], rug_baseline=[0], rug_top=[0.1],
        )
    samples = lp.histogram(main, samples="x")
    with pytest.raises(TypeError):
        samples.composite_precomputed_kde(x=lp.col["x"], y=lp.col["y"])
    kde = pd.DataFrame({"x": [0.25, 1.0, 1.75], "y": [0.0, 0.5, 0.0]})
    sample_owned = lp.histogram(samples=[0.25, 0.75, 1.25, 1.75]).stat("density")
    sample_owned.composite_precomputed_kde_from_data(kde, x="x", y="y")
    sample_owned.svg_preserve_text(tmp_path / "independent-kde")
    unchanged_stat = lp.histogram(samples=[0.25, 0.75, 1.25, 1.75])
    unchanged_stat.composite_precomputed_kde(x=[0.25, 1.0, 1.75], y=[0.0, 0.5, 0.0])
    with pytest.raises(lp.DataError, match="Density"):
        unchanged_stat.svg_preserve_text(tmp_path / "kde-does-not-change-stat")
    invalid = lp.raincloud(
        category=["a"], q1=[1], median=[2], q3=[3],
        density_category=["a"] * 3, density_y=[0, 2, 4], normalized_width=[0, 1, 0],
        point_category=["unknown"], point_y=[1], normalized_strip=[0.5],
    )
    with pytest.raises(lp.DataError):
        invalid.svg_preserve_text(tmp_path / "unknown-geometry-group")
    with pytest.raises(TypeError):
        lp.raincloud(
            category=["a"], q1=[1], median=[2], q3=[3], hue=["h"],
            density_category=["a"] * 3, density_y=[0, 2, 4], normalized_width=[0, 1, 0],
            point_category=["a"], point_y=[1], normalized_strip=[0.5],
        )
