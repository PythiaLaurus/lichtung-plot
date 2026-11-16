use super::*;

fn evaluate(value: f64) -> f64 {
    log10(value).unwrap()
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
