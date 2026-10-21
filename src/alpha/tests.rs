use super::*;

#[test]
fn linear_alpha_quantizes_once_with_even_ties_and_preserves_extreme_endpoints() {
    let source = NumericValues::new("confidence", [-1.0, 0.0, 0.25, 0.5, 0.75, 1.0, 2.0]);
    let alpha = AlphaValues::linear(source.clone(), (0.0, 1.0), (0, 255)).unwrap();
    assert_eq!(alpha.data.values, [0, 0, 64, 128, 191, 255, 255]);
    assert_eq!(alpha.semantics(), AlphaSemantics::Continuous);
    assert_eq!(
        AlphaValues::linear(source.clone(), (0.0, 1.0), (255, 0))
            .unwrap()
            .data
            .values,
        [255, 255, 191, 128, 64, 0, 0],
    );
    assert_eq!(
        AlphaValues::linear(source, (0.0, 1.0), (17, 17))
            .unwrap()
            .data
            .values,
        [17; 7],
    );
    let middle = NumericValues::new("middle", [0.5; 2]);
    assert_eq!(
        AlphaValues::linear(middle.clone(), (0.0, 1.0), (0, 1))
            .unwrap()
            .data
            .values,
        [0, 0]
    );
    assert_eq!(
        AlphaValues::linear(middle, (0.0, 1.0), (1, 2))
            .unwrap()
            .data
            .values,
        [2, 2]
    );
    let extreme = AlphaValues::linear(
        NumericValues::new(
            "extreme",
            [-f64::MAX, -f64::MAX / 2.0, 0.0, f64::MAX / 2.0, f64::MAX],
        ),
        (-f64::MAX, f64::MAX),
        (0, 255),
    )
    .unwrap();
    assert_eq!(extreme.data.values, [0, 64, 128, 191, 255]);
    let tiny = f64::from_bits(1);
    let subnormal = AlphaValues::linear(
        NumericValues::new("tiny", [0.0, tiny, f64::from_bits(2)]),
        (0.0, f64::from_bits(2)),
        (0, 255),
    )
    .unwrap();
    assert_eq!(subnormal.data.values, [0, 128, 255]);
    let adjacent = AlphaValues::linear(
        NumericValues::new("adjacent", [1.0, 1.0_f64.next_up()]),
        (1.0, 1.0_f64.next_up()),
        (7, 213),
    )
    .unwrap();
    assert_eq!(adjacent.data.values, [7, 213]);
    let AlphaDeclaration::Continuous {
        examples, alphas, ..
    } = adjacent.declaration.as_ref()
    else {
        panic!("continuous declaration")
    };
    assert_eq!(examples.members.len(), 2);
    assert_ne!(examples.labels[0], examples.labels[1]);
    assert_eq!(alphas, &[7, 213]);
    for domain in [
        (1.0, 1.0),
        (2.0, 1.0),
        (f64::NAN, 1.0),
        (0.0, f64::INFINITY),
    ] {
        assert!(matches!(
            AlphaValues::linear(NumericValues::new("invalid", []), domain, (0, 255)),
            Err(Error::Data(_))
        ));
    }
    for bad in [
        None,
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
    ] {
        let error = AlphaValues::linear(
            NumericValues::nullable("invalid", [Some(0.5), bad]),
            (0.0, 1.0),
            (0, 255),
        )
        .unwrap_err();
        assert!(matches!(error, Error::Data(_)));
        assert!(error.to_string().contains("index 1"), "{error}");
    }
    let fraction = crate::numeric::mapping::normalized_fraction;
    assert_eq!(
        fraction((-0.0, 0.0), 10.0).unwrap().to_bits(),
        0.5_f64.to_bits()
    );
    assert!(fraction((0.0, 1.0), f64::NAN).is_err());
    assert!(fraction((1.0, 0.0), 0.5).is_err());
}

#[test]
fn log_alpha_maps_positive_coordinates_without_broadening_or_losing_clipped_members() {
    let alpha = AlphaValues::log10(
        NumericValues::new(
            "quality",
            [0.1, 1.0, 10.0, 100.0, 1000.0, 10000.0, 100_000.0],
        ),
        (1.0, 10000.0),
        (0, 255),
    )
    .unwrap();
    assert_eq!(alpha.data.values, [0, 0, 64, 128, 191, 255, 255]);
    assert_eq!(
        alpha.data.members,
        AlphaMembers::Numeric(
            [0.1_f64, 1.0, 10.0, 100.0, 1000.0, 10000.0, 100_000.0]
                .map(f64::to_bits)
                .to_vec()
        )
    );
    let AlphaDeclaration::Continuous {
        domain,
        rule,
        examples,
        ..
    } = alpha.declaration.as_ref()
    else {
        panic!("continuous declaration")
    };
    assert_eq!(*domain, [1.0_f64.to_bits(), 10000.0_f64.to_bits()]);
    assert!(
        matches!(rule, ContinuousRule::Log10 { range: (0, 255), coordinates } if *coordinates == [0, 4.0_f64.to_bits()])
    );
    // Explanation coordinates are original-domain midpoints, not exponent quarters.
    assert_eq!(
        examples.members,
        [1.0_f64, 2500.75, 5000.5, 7500.25, 10000.0].map(f64::to_bits)
    );
    let extreme = AlphaValues::log10(
        NumericValues::new("extreme", [f64::from_bits(1), f64::MAX]),
        (f64::from_bits(1), f64::MAX),
        (255, 0),
    )
    .unwrap();
    assert_eq!(extreme.data.values, [255, 0]);
    for domain in [(0.0, 1.0), (-1.0, 10.0), (f64::MAX.next_down(), f64::MAX)] {
        assert!(matches!(
            AlphaValues::log10(NumericValues::new("bad", []), domain, (0, 255)),
            Err(Error::Data(_))
        ));
    }
    for bad in [
        None,
        Some(-1.0),
        Some(0.0),
        Some(-0.0),
        Some(f64::NAN),
        Some(f64::INFINITY),
    ] {
        let error = AlphaValues::log10(
            NumericValues::nullable("bad", [Some(10.0), bad]),
            (1.0, 100.0),
            (0, 255),
        )
        .unwrap_err();
        assert!(matches!(error, Error::Data(_)));
        assert!(error.to_string().contains("index 1"), "{error}");
    }
}

#[test]
fn custom_continuous_materializes_each_sorted_clamped_coordinate_and_truthful_example_once() {
    let mut calls = Vec::new();
    let alpha = AlphaValues::continuous(
        NumericValues::new("reliability", [2.0, 0.125, -0.0, -1.0, 0.125, 0.5, 0.0]),
        (0.0, 1.0),
        |value| {
            calls.push(value.to_bits());
            if value < 0.5 {
                0
            } else {
                230
            }
        },
    )
    .unwrap();
    assert_eq!(
        calls,
        [0.0_f64, 0.125, 0.25, 0.5, 0.75, 1.0].map(f64::to_bits)
    );
    assert_eq!(alpha.data.values, [230, 0, 0, 0, 0, 230, 0]);
    assert_eq!(
        alpha.data.members,
        AlphaMembers::Numeric(
            [2.0_f64, 0.125, 0.0, -1.0, 0.125, 0.5, 0.0]
                .map(f64::to_bits)
                .to_vec()
        )
    );
    let AlphaDeclaration::Continuous {
        rule: ContinuousRule::Custom(table),
        examples,
        alphas,
        ..
    } = alpha.declaration.as_ref()
    else {
        panic!("custom declaration")
    };
    assert_eq!(table.iter().map(|&(key, _)| key).collect::<Vec<_>>(), calls);
    assert_eq!(examples.labels, ["0", "0.25", "0.5", "0.75", "1"]);
    assert_eq!(alphas, &[0, 0, 230, 230, 230]);
    let _ = alpha.clone().select([1, 0, 1]).unwrap();
    assert_eq!(calls.len(), 6);
    for source in [
        NumericValues::new("bad", [0.25, f64::NAN]),
        NumericValues::nullable("bad", [Some(0.25), None]),
    ] {
        let error = AlphaValues::continuous(source, (0.0, 1.0), |_| {
            calls.push(99);
            255
        })
        .unwrap_err();
        assert!(matches!(error, Error::Data(_)));
        assert!(error.to_string().contains("index 1"), "{error}");
    }
    assert!(
        AlphaValues::continuous(NumericValues::new("bad", [0.25]), (1.0, 1.0), |_| {
            calls.push(99);
            255
        })
        .is_err()
    );
    assert_eq!(calls.len(), 6);
    let mut empty_calls = Vec::new();
    let empty = AlphaValues::continuous(NumericValues::new("empty", []), (0.0, 1.0), |x| {
        empty_calls.push(x.to_bits());
        0
    })
    .unwrap();
    assert!(empty.is_empty());
    assert_eq!(
        empty_calls,
        [0.0_f64, 0.25, 0.5, 0.75, 1.0].map(f64::to_bits)
    );
    assert_eq!(empty.semantics(), AlphaSemantics::Continuous);
}

#[test]
fn categorical_alpha_owns_first_observed_raw_categories_and_rejects_nulls_before_callbacks() {
    let mut calls = Vec::new();
    let source = CategoryValues::new("method", ["A  B", "é", "A B", "e\u{301}", "A  B"]).unwrap();
    let alpha = AlphaValues::categorical(source, |raw| {
        calls.push(raw.to_owned());
        match raw {
            "A  B" => 0,
            "é" => 170,
            "A B" => 90,
            _ => 255,
        }
    })
    .unwrap();
    assert_eq!(calls, ["A  B", "é", "A B", "e\u{301}"]);
    assert_eq!(alpha.semantics(), AlphaSemantics::Categorical);
    assert_eq!(alpha.data.values, [0, 170, 90, 255, 0]);
    assert_eq!(
        alpha.data.members,
        AlphaMembers::Categorical(vec![0, 1, 2, 3, 0])
    );
    let AlphaDeclaration::Categorical(entries) = alpha.declaration.as_ref() else {
        panic!("categorical declaration")
    };
    assert_eq!(
        entries
            .iter()
            .map(|(raw, _)| raw.as_str())
            .collect::<Vec<_>>(),
        calls
    );
    let selected = alpha.select([3, 0, 3]).unwrap();
    assert_eq!(selected.data.values, [255, 0, 255]);
    assert_eq!(
        selected.data.members,
        AlphaMembers::Categorical(vec![3, 0, 3])
    );
    assert!(Arc::ptr_eq(&selected.declaration, &alpha.declaration));
    let error = AlphaValues::categorical(
        CategoryValues::nullable("bad", [Some("valid"), None]).unwrap(),
        |raw| {
            calls.push(raw.to_owned());
            123
        },
    )
    .unwrap_err();
    assert!(matches!(error, Error::Data(_)));
    assert!(error.to_string().contains("index 1"), "{error}");
    assert_eq!(calls.len(), 4);
    let empty = AlphaValues::categorical(
        CategoryValues::new("empty", std::iter::empty::<&str>()).unwrap(),
        |_| {
            calls.push("unexpected".into());
            0
        },
    )
    .unwrap();
    assert!(empty.is_empty());
    assert_eq!(empty.semantics(), AlphaSemantics::Categorical);
    assert_eq!(calls.len(), 4);
}

#[test]
fn thresholds_assign_equality_to_the_right_and_store_distinct_whole_set_boundary_labels() {
    let alpha = AlphaValues::threshold(
        NumericValues::new("signal", [-1.0, -0.0, 0.0, 0.5, 1.0, 2.0]),
        [0.0, 1.0],
        [0, 90, 255],
    )
    .unwrap();
    assert_eq!(alpha.semantics(), AlphaSemantics::Threshold);
    assert_eq!(alpha.data.values, [0, 90, 90, 90, 255, 255]);
    assert_eq!(
        alpha.data.members,
        AlphaMembers::Threshold(vec![
            ((-1.0_f64).to_bits(), 0),
            (0, 1),
            (0, 1),
            (0.5_f64.to_bits(), 1),
            (1.0_f64.to_bits(), 2),
            (2.0_f64.to_bits(), 2)
        ])
    );
    let AlphaDeclaration::Threshold {
        boundaries, labels, ..
    } = alpha.declaration.as_ref()
    else {
        panic!("threshold declaration")
    };
    assert_eq!(boundaries.members, [0, 1.0_f64.to_bits()]);
    assert_eq!(labels, &["< 0", "[0, 1)", ">= 1"]);
    let selected = alpha.select([4, 0, 4]).unwrap();
    assert_eq!(selected.data.values, [255, 0, 255]);
    assert!(Arc::ptr_eq(&selected.declaration, &alpha.declaration));
    let singleton =
        AlphaValues::threshold(NumericValues::new("signal", [1.0, -1.0]), [], [17]).unwrap();
    assert_eq!(singleton.data.values, [17, 17]);
    let AlphaDeclaration::Threshold {
        boundaries, labels, ..
    } = singleton.declaration.as_ref()
    else {
        panic!("single interval")
    };
    assert!(boundaries.members.is_empty());
    assert_eq!(labels, &["all values"]);
    let close = AlphaValues::threshold(
        NumericValues::new("signal", []),
        [1.0, 1.0_f64.next_up()],
        [0, 127, 255],
    )
    .unwrap();
    let AlphaDeclaration::Threshold {
        boundaries, labels, ..
    } = close.declaration.as_ref()
    else {
        panic!("close thresholds")
    };
    assert_eq!(boundaries.precision, 16);
    assert_eq!(
        boundaries.labels,
        ["1.0000000000000000e0", "1.0000000000000002e0"]
    );
    assert_eq!(labels[1], "[1.0000000000000000e0, 1.0000000000000002e0)");
    for boundaries in [
        vec![f64::NAN],
        vec![f64::INFINITY],
        vec![1.0, 0.0],
        vec![-0.0, 0.0],
    ] {
        assert!(matches!(
            AlphaValues::threshold(NumericValues::new("bad", []), boundaries, [0, 1, 2]),
            Err(Error::Data(_))
        ));
    }
    assert!(AlphaValues::threshold(NumericValues::new("bad", []), [0.0], [1]).is_err());
    assert!(AlphaValues::threshold(NumericValues::new("bad", []), [], []).is_err());
    let error = AlphaValues::threshold(
        NumericValues::nullable("bad", [Some(0.5), None]),
        [0.0],
        [0, 255],
    )
    .unwrap_err();
    assert!(matches!(error, Error::Data(_)));
    assert!(error.to_string().contains("index 1"), "{error}");
}

#[test]
fn selected_values_retain_complete_recipes_by_value_without_inventing_array_semantics() {
    fn owned<T: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync>() {}
    owned::<AlphaValues>();
    let raw = AlphaValues::new("confidence", [0, 128, 255].into_iter().filter(|_| true)).unwrap();
    assert_eq!(raw.name(), "confidence");
    assert_eq!(raw.len(), 3);
    assert_eq!(raw.get(0), Some(0));
    assert_eq!(raw.get(3), None);
    assert_eq!(raw.semantics(), AlphaSemantics::Undeclared);
    assert_eq!(raw.select([2, 0, 2]).unwrap().data.values, [255, 0, 255]);
    assert!(raw.select([3]).is_err());
    assert!(AlphaValues::new("empty", []).unwrap().is_empty());
    for name in ["", " leading", "trailing ", "control\n", "\u{0000}"] {
        assert!(matches!(AlphaValues::new(name, [255]), Err(Error::Data(_))));
    }
    let make = |special| {
        AlphaValues::continuous(
            NumericValues::new("confidence", [0.125, 0.5, 2.0]),
            (0.0, 1.0),
            |x| {
                if x.to_bits() == 0.125_f64.to_bits() {
                    special
                } else {
                    128
                }
            },
        )
        .unwrap()
    };
    let pooled = make(17);
    let selected = pooled.select([1, 2, 1]).unwrap();
    assert_eq!(selected.data.values, [128, 128, 128]);
    assert_eq!(
        selected.data.members,
        AlphaMembers::Numeric([0.5_f64, 2.0, 0.5].map(f64::to_bits).to_vec())
    );
    assert_eq!(selected, make(17).select([1, 2, 1]).unwrap());
    assert_ne!(selected, make(18).select([1, 2, 1]).unwrap());
    assert_ne!(selected, AlphaValues::new("confidence", [128; 3]).unwrap());
    let empty = pooled.select([]).unwrap();
    assert!(empty.is_empty());
    assert_eq!(empty.declaration, pooled.declaration);
    assert_eq!(pooled, pooled.clone());
    let linear = |domain| {
        AlphaValues::linear(NumericValues::new("confidence", [0.5]), domain, (128, 128)).unwrap()
    };
    assert_ne!(linear((0.0, 1.0)), linear((-1.0, 2.0)));
    let zero = |value| {
        AlphaValues::continuous(
            NumericValues::new("confidence", [value]),
            (-1.0, 1.0),
            |_| 0,
        )
        .unwrap()
    };
    assert_eq!(zero(-0.0), zero(0.0));
    let survivor = {
        let temporary = make(17);
        temporary.select([1]).unwrap()
    };
    assert_eq!(survivor, pooled.select([1]).unwrap());
}
