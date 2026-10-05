use super::*;

fn evaluate(value: f64) -> f64 {
    log10(value, &OperationGenerationResources::new()).unwrap()
}

#[test]
fn exact_powers_and_exceptional_values() {
    let mut power = 1.0;
    for exponent in 0..=22 {
        assert_eq!(evaluate(power).to_bits(), f64::from(exponent).to_bits());
        power *= 10.0;
    }
    for value in [f64::NAN, -1.0, f64::NEG_INFINITY] {
        assert!(evaluate(value).is_nan());
    }
    for value in [-0.0, 0.0] {
        assert_eq!(evaluate(value).to_bits(), f64::NEG_INFINITY.to_bits());
    }
    assert_eq!(evaluate(f64::INFINITY).to_bits(), f64::INFINITY.to_bits());
}

#[test]
fn mpfr_rounding_vectors_cover_subnormals_cancellation_and_platform_misses() {
    // MPFR 4.2.1 mpfr_log10 at precision 53, MPFR_RNDN. These inputs include
    // the first 24 platform-libm misses in a xorshift64 scan seeded 94104bdfbb328f09.
    for (input, expected) in [
        (0x0000_0000_0000_0001, 0xc074_34e6_420f_4374),
        (0x0000_0000_0000_0002, 0xc074_3015_3d3b_1b95),
        (0x000f_ffff_ffff_ffff, 0xc073_3a71_46f7_2a42),
        (0x0010_0000_0000_0000, 0xc073_3a71_46f7_2a42),
        (0x3fef_ffff_ffff_ffff, 0xbc8b_cb7b_1526_e50f),
        (0x3ff0_0000_0000_0001, 0x3c9b_cb7b_1526_e50d),
        (0x3ff0_0000_0000_0002, 0x3cab_cb7b_1526_e50c),
        (0x3fb9_9999_9999_999a, 0xbff0_0000_0000_0000),
        (0x4000_0000_0000_0000, 0x3fd3_4413_509f_79ff),
        (0x4023_ffff_ffff_ffff, 0x3fef_ffff_ffff_ffff),
        (0x4024_0000_0000_0001, 0x3ff0_0000_0000_0000),
        (0x7fef_ffff_ffff_ffff, 0x4073_4413_509f_79ff),
        (0x43f3_90bc_1f9a_1498, 0x4033_5a71_1203_fe03),
        (0x3feb_bcde_1def_697e, 0xbfaf_c8c1_acea_cb12),
        (0x3686_a18a_9b70_7136, 0xc046_a708_718a_d5d3),
        (0x447d_715a_68e5_097d, 0x4035_f063_85f4_90ef),
        (0x433d_c3c9_c2c1_99c3, 0x402f_d8a6_29b3_924d),
        (0x404e_c8fc_7581_70ad, 0x3ffc_a143_0fca_075f),
        (0x3ff9_8d34_b1b4_b24f, 0x3fca_05aa_1e62_b8d8),
        (0x100b_7bbd_580b_e707, 0xc06c_d4f6_39cf_9f53),
        (0x4b4d_3463_e168_289d, 0x404b_5fb6_a3c9_daf9),
        (0x434f_da5b_5636_b84f, 0x4030_40ed_35aa_b325),
        (0x3c96_f034_8a6c_30bb, 0xc030_1963_c2e9_887b),
        (0x4bc8_afc3_d47f_082b, 0x404c_8aa0_13af_9133),
        (0x3e89_47eb_62a9_a957, 0xc01a_e66a_ca05_09d7),
        (0x40bd_8909_6c3c_6d46, 0x400f_0755_a8f6_5e65),
        (0x4bec_6c50_5c1a_e3f1, 0x404c_df86_1fd5_9ae1),
        (0x3f77_d6a5_0fbe_576e, 0xc001_e172_9419_2243),
        (0x34cd_db9f_88b8_aac5, 0xc04a_ce84_defd_f1bd),
        (0x3fe2_aab3_b476_c54f, 0xbfcd_f655_56b8_765a),
        (0x3ff2_6c59_c566_dff4, 0x3faf_5ba7_71f9_cb15),
        (0x3fc6_e277_f51d_e1c1, 0xbfe7_ece0_e3cf_32db),
        (0x6d25_a637_ded5_10a5, 0x406b_38d5_140e_b6b9),
        (0x400f_1fe2_b172_4c36, 0x3fe2_e163_38ff_d08b),
        (0x3edf_69b6_4929_6b2e, 0xc014_8090_8892_23f9),
        (0x4068_b0e4_97b4_7633, 0x4002_5d72_73b9_5223),
    ] {
        assert_eq!(
            evaluate(f64::from_bits(input)).to_bits(),
            expected,
            "{input:016x}"
        );
    }
}

#[test]
fn outward_series_bounds_nest_and_ambiguous_rounding_requires_more_precision() {
    let resources = OperationGenerationResources::new();
    for (numerator, denominator) in [(1, 3), (1, 9), (17, 91)] {
        let (low, high) = ln_bounds(numerator, denominator, 32, &resources).unwrap();
        let (fine_low, fine_high) = ln_bounds(numerator, denominator, 128, &resources).unwrap();
        assert!((&low << 96) <= fine_low);
        assert!((&high << 96) >= fine_high);
        let unit = BigUint::from(1_u32) << 32_usize;
        assert_ne!(
            rounded_ratio(low, unit.clone(), &resources)
                .unwrap()
                .to_bits(),
            rounded_ratio(high, unit, &resources).unwrap().to_bits()
        );
        let unit = BigUint::from(1_u32) << 128_usize;
        assert_eq!(
            rounded_ratio(fine_low, unit.clone(), &resources)
                .unwrap()
                .to_bits(),
            rounded_ratio(fine_high, unit, &resources)
                .unwrap()
                .to_bits()
        );
    }
}

#[test]
fn mpfr_digest_covers_every_binary_exponent_and_near_one_cancellation() {
    // Independent MPFR 4.2.1, precision 53 / MPFR_RNDN, in this exact input order.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut count = 0;
    let mut observe = |input: u64| {
        hash = (hash ^ input).wrapping_mul(0x100_0000_01b3);
        hash = (hash ^ evaluate(f64::from_bits(input)).to_bits()).wrapping_mul(0x100_0000_01b3);
        count += 1;
    };
    for exponent in -1074..=1023 {
        let bits = if exponent < -1022 {
            1_u64 << (exponent + 1074)
        } else {
            u64::try_from(exponent + 1023).unwrap() << 52
        };
        for input in [bits - 1, bits, bits + 1] {
            if input != 0 {
                observe(input);
            }
        }
    }
    let mut state = 0x9410_4bdf_bb32_8f09_u64;
    for _ in 0..2048 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let input = state & 0x7fff_ffff_ffff_ffff;
        if input != 0 && input < 0x7ff0_0000_0000_0000 {
            observe(input);
        }
    }
    for input in (1.0_f64.to_bits() - 256)..=(1.0_f64.to_bits() + 256) {
        observe(input);
    }
    assert_eq!(count, 8851);
    assert_eq!(hash, 0xc974_db04_4c3d_49e6);
}

#[test]
fn rational_rounding_resolves_even_ties_and_binade_carry() {
    for (numerator, denominator, expected) in [
        ((1_u64 << 53) + 1, 1_u64 << 53, 1.0),
        ((1_u64 << 53) + 3, 1_u64 << 53, 1.0_f64.next_up().next_up()),
        ((1_u64 << 54) - 1, 1_u64 << 53, 2.0),
        (1, 2, 0.5),
    ] {
        assert_eq!(
            rounded_ratio(
                numerator.into(),
                denominator.into(),
                &OperationGenerationResources::new()
            )
            .unwrap()
            .to_bits(),
            expected.to_bits()
        );
    }
}

#[test]
fn scratch_and_work_limits_refuse_before_the_guarded_series_iteration() {
    // At p=128: 8 outer temporaries + 7 series setup temporaries, 96 bytes each.
    // The first iteration admits 6 more before its first BigUint expression.
    for (bytes, completed) in [(2015, 0), (2016, 1)] {
        let resources = OperationGenerationResources::with_test_limits(1_000_000, bytes);
        SERIES_WORK.with(|work| work.set(0));
        assert!(log10(1.5, &resources)
            .unwrap_err()
            .to_string()
            .contains("log10 scratch admission"));
        assert_eq!(SERIES_WORK.with(std::cell::Cell::get), completed);
        assert_eq!(
            resources.byte_usage().unwrap(),
            1440 + completed as u64 * 576
        );
    }
    let measured = OperationGenerationResources::new();
    let expected = log10(1.5, &measured).unwrap();
    let required = measured.byte_usage().unwrap();
    for bytes in [required - 1, required, required + 1] {
        let resources = OperationGenerationResources::with_test_limits(1_000_000, bytes);
        let result = log10(1.5, &resources);
        if bytes < required {
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("log10 scratch admission"));
        } else {
            assert_eq!(result.unwrap().to_bits(), expected.to_bits());
        }
    }
    for remaining in [required - 1, required] {
        let resources = OperationGenerationResources::new();
        resources
            .claim_bytes(512 * 1024 * 1024 - remaining)
            .unwrap();
        assert_eq!(log10(1.5, &resources).is_ok(), remaining == required);
    }
    let resources = OperationGenerationResources::with_test_limits(26, 512 * 1024 * 1024);
    SERIES_WORK.with(|work| work.set(0));
    assert!(log10(1.5, &resources)
        .unwrap_err()
        .to_string()
        .contains("log10 work admission"));
    assert_eq!(SERIES_WORK.with(std::cell::Cell::get), 0);
    assert_eq!(resources.usage().unwrap().1, 26);
    assert!(admit(&OperationGenerationResources::new(), usize::MAX, u64::MAX).is_err());
}

#[test]
fn nice_ranges_and_prepared_forest_limits_propagate_root_refusal() {
    let resources = OperationGenerationResources::with_test_limits(0, 512 * 1024 * 1024);
    let spec = crate::layout::AxisSpec::default();
    for result in [
        crate::layout::continuous_auto_range((1.0, 2.0), &spec, &resources),
        crate::layout::anchored_auto_range((1.0, 2.0), &spec, 0.0, &resources),
    ] {
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("log10 work admission"));
    }
    let plot = crate::Plot::forest(
        crate::CategoryValues::new("term", ["a"]).unwrap(),
        crate::NumericValues::new("estimate", [1.5]),
        crate::Bounds::new(
            crate::NumericValues::new("lower", [1.0]),
            crate::NumericValues::new("upper", [2.0]),
        ),
        1.0,
    )
    .scale(crate::AxisScale::Log10)
    .x_limits(1.0, 2.0)
    .erase();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let before = resources.usage().unwrap().1;
    resources.set_test_limits(before + 3, 512 * 1024 * 1024);
    let result = crate::spec::PlotRenderContext::new(
        plot.data(),
        crate::layout::PanelBox::new(0.0, 400.0, 0.0, 300.0),
        &theme,
    )
    .with_prepared_axes(&plot);
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("log10 work admission"));
    assert_eq!(resources.usage().unwrap().1, before + 3);
}
