//! Fixed first-party opaque color arithmetic. T5d1 pins the numerical kernel
//! before T5d2/T5d3 route semantic palettes and gradients through it.

#![allow(
    dead_code,
    reason = "T5d1 numerical kernel is wired to active scale consumers in T5d2/T5d3"
)]

use crate::font::canonical::OperationGenerationResources;
use crate::Error;
use num_bigint::BigUint;
use std::cmp::Ordering;

const MATRIX_EDGE_SLACK: f64 = 1.0 / 4_194_304.0;

// The nearest encoded-sRGB byte transitions, in *linear* RGB. For boundary
// k, the encoded source is exactly (k+1/2)/255; the corresponding piecewise
// sRGB transfer is evaluated with 120, 300 and 480 decimal digits, then rounded
// nearest-even to binary64. All three independently selected binary64 tables
// agree. Big-endian u64 stream SHA-256:
// ace388c2947682af22573df8b12dde52e54c10fa706965d3011b159a5a7e340f.
// Using these half-channel thresholds avoids host powf in inverse OKLab output.
#[expect(
    clippy::unreadable_literal,
    reason = "pinned binary64 channel transitions are recorded verbatim"
)]
const ENCODED_HALF_LINEAR_BITS: [u64; 255] = [
    0x3f23e45677c176f7,
    0x3f3dd681b3a23272,
    0x3f48dd6c15b1d4b4,
    0x3f5167cba8c94818,
    0x3f5660e146b9a5d5,
    0x3f5b59f6e4aa0393,
    0x3f602986414d30a8,
    0x3f62a61110455f87,
    0x3f65229bdf3d8e66,
    0x3f679f26ae35bd45,
    0x3f6a1e5a03a8a4b5,
    0x3f6cbf734477e0e7,
    0x3f6f8680590912c1,
    0x3f713a0be3e98ad5,
    0x3f72c4665c6bc58b,
    0x3f74629793a399b0,
    0x3f7614e607554158,
    0x3f77db96ca0c99d9,
    0x3f79b6ed95fb6dba,
    0x3f7ba72cde4cb5ca,
    0x3f7dac95df183297,
    0x3f7fc768ac1bd733,
    0x3f80fbf21f2dc489,
    0x3f821f234061c55c,
    0x3f834d662df32ddc,
    0x3f8486d8e075e9d4,
    0x3f85cb98d9e2545f,
    0x3f871bc32a59ad46,
    0x3f887774749cc7c5,
    0x3f89dec8f23ba5bf,
    0x3f8b51dc7783fa2a,
    0x3f8cd0ca7733ec84,
    0x3f8e5bae05f5ea99,
    0x3f8ff2a1dda9da46,
    0x3f90cae0303fc81d,
    0x3f91a291cdf30864,
    0x3f928072a5a9656b,
    0x3f93648f6d71d8d4,
    0x3f944ef4b4ef6b8f,
    0x3f953faee688f616,
    0x3f9636ca4889ebee,
    0x3f973452fe3532a6,
    0x3f98385508caeef9,
    0x3f9942dc48821f75,
    0x3f9a53f47d76ca86,
    0x3f9b6ba9488d7613,
    0x3f9c8a062c4c9086,
    0x3f9daf168dac664f,
    0x3f9edae5b4de330d,
    0x3fa006bf67056983,
    0x3fa0a3767504c7e3,
    0x3fa1439d7f87bcfa,
    0x3fa1e739f4abdd1c,
    0x3fa28e5135e29dea,
    0x3fa338e8983f0647,
    0x3fa3e70564c063c2,
    0x3fa498acd89a2d57,
    0x3fa54de4257938cc,
    0x3fa606b071c66583,
    0x3fa6c316d8e6dd5e,
    0x3fa7831c6b7a0a43,
    0x3fa846c62f955cea,
    0x3fa90e1920fdffce,
    0x3fa9d91a31608f91,
    0x3faaa7ce4886f087,
    0x3fab7a3a448c57bf,
    0x3fac5062fa0f9c99,
    0x3fad2a4d3463e6bc,
    0x3fae07fdb5bfcb10,
    0x3faee979376ae979,
    0x3fafcec469ea1be6,
    0x3fb05bf1fa95233f,
    0x3fb0d26e3c54ebea,
    0x3fb14ad945d08395,
    0x3fb1c5355e946f1e,
    0x3fb24184ca308d86,
    0x3fb2bfc9c84a7aee,
    0x3fb3400694af6b4b,
    0x3fb3c23d67658241,
    0x3fb4467074bcad3f,
    0x3fb4cca1ed5f04c9,
    0x3fb554d3fe60b982,
    0x3fb5df08d14f9170,
    0x3fb66b428c41f9a1,
    0x3fb6f98351e5b03c,
    0x3fb789cd418e0ac2,
    0x3fb81c227741dc30,
    0x3fb8b0850bc8fe6b,
    0x3fb946f714b98253,
    0x3fb9df7aa4848998,
    0x3fba7a11ca82cd65,
    0x3fbb16be9300d4b9,
    0x3fbbb583074add2e,
    0x3fbc56612db878df,
    0x3fbcf95b09b7e3e6,
    0x3fbd9e729bd913de,
    0x3fbe45a9e1d883c9,
    0x3fbeef02d6a9be77,
    0x3fbf9a7f7281a9a9,
    0x3fc02410d57049f5,
    0x3fc07bf5b94e038a,
    0x3fc0d4ef5cf430b2,
    0x3fc12efeb7311b79,
    0x3fc18a24bd8bbe9d,
    0x3fc1e6626448412f,
    0x3fc243b89e6c58e7,
    0x3fc2a2285dc393e6,
    0x3fc301b292e38aa4,
    0x3fc362582d2ffac4,
    0x3fc3c41a1adecb7a,
    0x3fc426f948fbfc29,
    0x3fc48af6a36d7de4,
    0x3fc4f01314f6f85f,
    0x3fc5564f873d7af6,
    0x3fc5bdace2cb1a51,
    0x3fc6262c0f127b37,
    0x3fc68fcdf2724b0d,
    0x3fc6fa937238a68f,
    0x3fc7667d72a66f3d,
    0x3fc7d38cd6f28fe9,
    0x3fc841c2814d30ed,
    0x3fc8b11f52e2dc70,
    0x3fc921a42bdf9326,
    0x3fc99351eb71d1f9,
    0x3fca06296fcd88fb,
    0x3fca7a2b962f040b,
    0x3fcaef593addc582,
    0x3fcb65b3392f5350,
    0x3fcbdd3a6b89f6d8,
    0x3fcc55efab676fe3,
    0x3fcccfd3d1579b02,
    0x3fcd4ae7b5030ba9,
    0x3fcdc72c2d2d9a5b,
    0x3fce44a20fb8e723,
    0x3fcec34a31a6d0b4,
    0x3fcf4325671be069,
    0x3fcfc4348361ab75,
    0x3fd0233c2c7494c1,
    0x3fd064f8dca68078,
    0x3fd0a750baa9e48d,
    0x3fd0ea442e792155,
    0x3fd12dd39fa6c332,
    0x3fd171ff755e9545,
    0x3fd1b6c81666af86,
    0x3fd1fc2de920806a,
    0x3fd242315389d220,
    0x3fd288d2bb3dcb8f,
    0x3fd2d0128575ed1d,
    0x3fd317f1170b096c,
    0x3fd3606ed4763a09,
    0x3fd3a98c21d1d03f,
    0x3fd3f34962da4212,
    0x3fd43da6faef137c,
    0x3fd488a54d13bc06,
    0x3fd4d444bbf088ca,
    0x3fd52085a9d37af4,
    0x3fd56d6878b122d7,
    0x3fd5baed8a2577aa,
    0x3fd609153f74abf5,
    0x3fd657dff98bfecf,
    0x3fd6a74e190289f3,
    0x3fd6f75ffe1a0cbc,
    0x3fd7481608bfb425,
    0x3fd79970988cdfca,
    0x3fd7eb700cc7e40c,
    0x3fd83e14c464c956,
    0x3fd8915f1e0608a3,
    0x3fd8e54f77fd4542,
    0x3fd939e6304c03f5,
    0x3fd98f23a4a45f6c,
    0x3fd9e5083269ba32,
    0x3fda3b9436b16e15,
    0x3fda92c80e437919,
    0x3fdaeaa4159b27f7,
    0x3fdb4328a8e7be4a,
    0x3fdb9c56240d1c5a,
    0x3fdbf62ce2a462a8,
    0x3fdc50ad3ffc933a,
    0x3fdcabd7971b30b4,
    0x3fdd07ac42bcdb47,
    0x3fdd642b9d55eb84,
    0x3fddc15601130b20,
    0x3fde1f2bc7d9cba5,
    0x3fde7dad4b493b27,
    0x3fdedcdae4ba7707,
    0x3fdf3cb4ed413cbf,
    0x3fdf9d3bbdac78d2,
    0x3fdffe6fae86d3d6,
    0x3fe030288c0b9edc,
    0x3fe061702930bb93,
    0x3fe0930eda934ca2,
    0x3fe0c504cbf2cdca,
    0x3fe0f75228edec22,
    0x3fe129f71d02c75c,
    0x3fe15cf3d38f323d,
    0x3fe1904877d0f24b,
    0x3fe1c3f534e5fea4,
    0x3fe1f7fa35ccbe1b,
    0x3fe22c57a564448b,
    0x3fe2610dae6c8f65,
    0x3fe2961c7b86c188,
    0x3fe2cb8437355e5b,
    0x3fe301450bdc8432,
    0x3fe3375f23c225fb,
    0x3fe36dd2a90e443c,
    0x3fe3a49fc5cb2566,
    0x3fe3dbc6a3e58d79,
    0x3fe413476d2cf4fe,
    0x3fe44b224b53bf61,
    0x3fe4835767ef70a2,
    0x3fe4bbe6ec78e26b,
    0x3fe4f4d1024c7882,
    0x3fe52e15d2aa54a6,
    0x3fe567b586b689cd,
    0x3fe5a1b047794ed4,
    0x3fe5dc063ddf3090,
    0x3fe616b792b94357,
    0x3fe651c46ebd53f4,
    0x3fe68d2cfa861810,
    0x3fe6c8f15e935e0f,
    0x3fe70511c34a3c66,
    0x3fe7418e50f54067,
    0x3fe77e672fc49c8c,
    0x3fe7bb9c87ce563d,
    0x3fe7f92e810e7315,
    0x3fe8371d436725ab,
    0x3fe87568f6a0f9dc,
    0x3fe8b411c26b0098,
    0x3fe8f317ce5afb3e,
    0x3fe9327b41ed8674,
    0x3fe9723c44864498,
    0x3fe9b25afd7007af,
    0x3fe9f2d793dcfaee,
    0x3fea33b22ee6cbcc,
    0x3fea74eaf58ed2a5,
    0x3feab6820ebe3af1,
    0x3feaf877a1462b15,
    0x3feb3acbd3dfebbe,
    0x3feb7d7ecd2d0ee0,
    0x3febc090b3b79644,
    0x3fec0401adf219b8,
    0x3fec47d1e237ecd4,
    0x3fec8c0176cd4466,
    0x3fecd09091df5b70,
    0x3fed157f598497d1,
    0x3fed5acdf3bcae8a,
    0x3feda07c8670c7a5,
    0x3fede68b3773a1c6,
    0x3fee2cfa2c81b559,
    0x3fee73c98b41576d,
    0x3feebaf97942dc33,
    0x3fef028a1c00b929,
    0x3fef4a7b98dfa6eb,
    0x3fef92ce152ec2b7,
    0x3fefdb81b627af90,
];

/// Cartesian OKLab components. All RGB inputs are opaque and channel-exact;
/// opacity is selected by the independent mark/typography role after mapping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Oklab {
    pub(super) l: f64,
    pub(super) a: f64,
    pub(super) b: f64,
}

impl Oklab {
    pub(super) fn from_rgb(
        rgb: (u8, u8, u8),
        resources: &OperationGenerationResources,
    ) -> Result<Self, Error> {
        resources.claim_visits(3).map_err(work_error)?;
        let red = super::contrast::linear_channel(rgb.0);
        let green = super::contrast::linear_channel(rgb.1);
        let blue = super::contrast::linear_channel(rgb.2);
        // The declared left-associative binary64 evaluation is deliberately
        // not expressed through matrix multiplication, FMA or platform powf.
        let long = (0.412_221_470_8 * red + 0.536_332_536_3 * green) + 0.051_445_992_9 * blue;
        let medium = (0.211_903_498_2 * red + 0.680_699_545_1 * green) + 0.107_396_956_6 * blue;
        let short = (0.088_302_461_9 * red + 0.281_718_837_6 * green) + 0.629_978_700_5 * blue;
        let long = cube_root(long, resources)?;
        let medium = cube_root(medium, resources)?;
        let short = cube_root(short, resources)?;
        Ok(Self {
            l: (0.210_454_255_3 * long + 0.793_617_785_0 * medium) - 0.004_072_046_8 * short,
            a: (1.977_998_495_1 * long - 2.428_592_205_0 * medium) + 0.450_593_709_9 * short,
            b: (0.025_904_037_1 * long + 0.782_771_766_2 * medium) - 0.808_675_766_0 * short,
        })
    }

    /// Convert fixed OKLab coordinates back to linear-sRGB without host
    /// exponentiation. Gamut reduction for generated out-of-range colors is
    /// owned by T5d1b; this branch accepts only in-gamut channel operands.
    pub(super) fn linear_rgb(self) -> Result<[f64; 3], Error> {
        if ![self.l, self.a, self.b].into_iter().all(f64::is_finite) {
            return Err(Error::Render("OKLab coordinates must be finite".into()));
        }
        let long = (self.l + 0.396_337_777_4 * self.a) + 0.215_803_757_3 * self.b;
        let medium = (self.l - 0.105_561_345_8 * self.a) - 0.063_854_172_8 * self.b;
        let short = (self.l - 0.089_484_177_5 * self.a) - 1.291_485_548_0 * self.b;
        let long = (long * long) * long;
        let medium = (medium * medium) * medium;
        let short = (short * short) * short;
        let result = [
            (4.076_741_662_1 * long - 3.307_711_591_3 * medium) + 0.230_969_929_2 * short,
            (-1.268_438_004_6 * long + 2.609_757_401_1 * medium) - 0.341_319_396_5 * short,
            (-0.004_196_086_3 * long - 0.703_418_614_7 * medium) + 1.707_614_701_0 * short,
        ];
        if !result.into_iter().all(f64::is_finite) {
            return Err(Error::Render(
                "OKLab inverse channels are not finite".into(),
            ));
        }
        Ok(result)
    }

    pub(super) fn opaque_rgb(
        self,
        resources: &OperationGenerationResources,
    ) -> Result<(u8, u8, u8), Error> {
        let [red, green, blue] = self.linear_rgb()?;
        Ok((
            encoded_channel(red, resources)?,
            encoded_channel(green, resources)?,
            encoded_channel(blue, resources)?,
        ))
    }

    fn in_gamut(self) -> Result<bool, Error> {
        Ok(self
            .linear_rgb()?
            .into_iter()
            .all(|channel| (-MATRIX_EDGE_SLACK..=1.0 + MATRIX_EDGE_SLACK).contains(&channel)))
    }

    /// Fixed 24-step chroma bisection at this source-derived lightness/hue.
    /// The first attempted color is used verbatim when already in gamut;
    /// otherwise every trial is admitted before inverse matrix evaluation.
    pub(super) fn reduced_rgb(
        self,
        resources: &OperationGenerationResources,
    ) -> Result<((u8, u8, u8), Self), Error> {
        if self.in_gamut()? {
            return Ok((self.opaque_rgb(resources)?, self));
        }
        let neutral = Self {
            a: 0.0,
            b: 0.0,
            ..self
        };
        if !neutral.in_gamut()? {
            return Err(Error::Render(
                "OKLab neutral lightness is outside the finite sRGB gamut".into(),
            ));
        }
        let (mut lower, mut upper) = (0.0_f64, 1.0_f64);
        for _ in 0..24 {
            resources.claim_visits(1).map_err(work_error)?;
            let midpoint = f64::midpoint(lower, upper);
            let candidate = Self {
                a: self.a * midpoint,
                b: self.b * midpoint,
                ..self
            };
            if candidate.in_gamut()? {
                lower = midpoint;
            } else {
                upper = midpoint;
            }
        }
        let selected = Self {
            a: self.a * lower,
            b: self.b * lower,
            ..self
        };
        Ok((selected.opaque_rgb(resources)?, selected))
    }
}

/// Interpolate final opaque data colors in Cartesian OKLab. Exact endpoints
/// retain the caller's RGB bytes and avoid an unnecessary matrix roundtrip.
pub(crate) fn interpolate(
    first: (u8, u8, u8),
    second: (u8, u8, u8),
    fraction: f64,
    resources: &OperationGenerationResources,
) -> Result<(u8, u8, u8), Error> {
    resources.claim_visits(1).map_err(work_error)?;
    if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
        return Err(Error::Data(
            "OKLab interpolation fraction must be finite in [0,1]".into(),
        ));
    }
    if fraction.to_bits() == 0.0_f64.to_bits() {
        return Ok(first);
    }
    if fraction.to_bits() == 1.0_f64.to_bits() {
        return Ok(second);
    }
    let first = Oklab::from_rgb(first, resources)?;
    let second = Oklab::from_rgb(second, resources)?;
    let interpolate = |low: f64, high: f64| low + (high - low) * fraction;
    Oklab {
        l: interpolate(first.l, second.l),
        a: interpolate(first.a, second.a),
        b: interpolate(first.b, second.b),
    }
    .reduced_rgb(resources)
    .map(|(rgb, _)| rgb)
}

/// Finite, prefix-stable maximin candidates for private qualitative and accent
/// palettes. Hues traverse a normalized rational square ring (no host trig);
/// lightness bands and chroma bands have a fixed order independent of the
/// eventual requested domain size. Each next color compares its final
/// quantized RGB identity and its actual reduced OKLab against prior colors.
const HUE_DIRECTIONS: usize = 64;
const LIGHTNESS_BANDS: [f64; 16] = [
    0.0, -0.05, 0.05, -0.10, 0.10, -0.15, 0.15, -0.20, 0.20, -0.25, 0.25, -0.30, 0.30, -0.35, 0.35,
    0.40,
];
const CHROMA_BANDS: [f64; 8] = [1.0, 0.75, 1.25, 0.5, 1.5, 0.25, 1.75, 0.0];

#[derive(Clone, Copy)]
struct PaletteCharacter {
    lightness: f64,
    chroma: f64,
}

struct CandidateChoice {
    distance: f64,
    index: usize,
    color: (u8, u8, u8),
    lab: Oklab,
}

fn candidate_better(distance: f64, index: usize, best: Option<&CandidateChoice>) -> bool {
    best.is_none_or(|choice| {
        distance.total_cmp(&choice.distance).is_gt()
            || distance.to_bits() == choice.distance.to_bits() && index < choice.index
    })
}

fn candidate_lab(character: PaletteCharacter, ordinal: usize) -> Oklab {
    let hue = ordinal % HUE_DIRECTIONS;
    let side = hue / 16;
    let step = f64::from(u32::try_from(hue % 16).expect("ring step is below 16")) / 16.0;
    let (dx, dy) = match side {
        0 => (1.0, -1.0 + 2.0 * step),
        1 => (1.0 - 2.0 * step, 1.0),
        2 => (-1.0, 1.0 - 2.0 * step),
        _ => (-1.0 + 2.0 * step, -1.0),
    };
    let length = (dx * dx + dy * dy).sqrt();
    let band = LIGHTNESS_BANDS[(ordinal / HUE_DIRECTIONS) % LIGHTNESS_BANDS.len()];
    let chroma_band = CHROMA_BANDS[ordinal / (HUE_DIRECTIONS * LIGHTNESS_BANDS.len())];
    let radius = character.chroma * chroma_band;
    Oklab {
        l: (character.lightness + band).clamp(0.05, 0.95),
        a: radius * dx / length,
        b: radius * dy / length,
    }
}

fn select_candidate(
    character: PaletteCharacter,
    colors: &[(u8, u8, u8)],
    labs: &[Oklab],
    candidates: usize,
    resources: &OperationGenerationResources,
) -> Result<((u8, u8, u8), Oklab), Error> {
    let mut best: Option<CandidateChoice> = None;
    for ordinal in 0..candidates {
        resources.claim_visits(1).map_err(work_error)?;
        let (rgb, _) = candidate_lab(character, ordinal).reduced_rgb(resources)?;
        let mut duplicate = false;
        for prior in colors {
            resources.claim_visits(1).map_err(work_error)?;
            if *prior == rgb {
                duplicate = true;
                break;
            }
        }
        if duplicate {
            continue;
        }
        let actual = Oklab::from_rgb(rgb, resources)?;
        let mut minimum = f64::INFINITY;
        for prior in labs {
            resources.claim_visits(1).map_err(work_error)?;
            let difference = (actual.l - prior.l) * (actual.l - prior.l)
                + (actual.a - prior.a) * (actual.a - prior.a)
                + (actual.b - prior.b) * (actual.b - prior.b);
            minimum = minimum.min(difference);
        }
        if candidate_better(minimum, ordinal, best.as_ref()) {
            best = Some(CandidateChoice {
                distance: minimum,
                index: ordinal,
                color: rgb,
                lab: actual,
            });
        }
    }
    let selected = best.ok_or_else(|| {
        Error::Data("extendable color palette exhausted its finite distinct candidate set".into())
    })?;
    Ok((selected.color, selected.lab))
}

pub(super) fn extend_palette(
    seeds: &[(u8, u8, u8)],
    requested: usize,
    resources: &OperationGenerationResources,
) -> Result<Vec<(u8, u8, u8)>, Error> {
    if seeds.is_empty() {
        return Err(Error::Data(
            "extendable color palette requires a nonempty prefix".into(),
        ));
    }
    if requested == 0 {
        return Ok(Vec::new());
    }
    if requested <= seeds.len() {
        resources
            .claim_slots(requested as u128)
            .map_err(work_error)?;
        let bytes = requested
            .checked_mul(std::mem::size_of::<(u8, u8, u8)>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| Error::Render("palette prefix storage overflowed".into()))?;
        resources
            .claim_bytes(bytes)
            .map_err(|error| Error::Render(format!("palette prefix storage admission: {error}")))?;
        let mut prefix = Vec::new();
        prefix
            .try_reserve_exact(requested)
            .map_err(|_| Error::Render("cannot reserve palette prefix".into()))?;
        prefix.extend_from_slice(&seeds[..requested]);
        return Ok(prefix);
    }
    let records = requested
        .checked_add(seeds.len())
        .ok_or_else(|| Error::Render("color palette capacity arithmetic overflowed".into()))?;
    resources.claim_slots(records as u128).map_err(work_error)?;
    let bytes = records
        .checked_mul(std::mem::size_of::<Oklab>() + 3)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| Error::Render("color palette storage arithmetic overflowed".into()))?;
    resources
        .claim_bytes(bytes)
        .map_err(|error| Error::Render(format!("color palette storage admission: {error}")))?;
    let mut result = Vec::new();
    let mut lab = Vec::new();
    result
        .try_reserve_exact(requested)
        .map_err(|_| Error::Render("cannot reserve resolved color palette".into()))?;
    lab.try_reserve_exact(records)
        .map_err(|_| Error::Render("cannot reserve resolved OKLab colors".into()))?;
    for &color in seeds {
        lab.push(Oklab::from_rgb(color, resources)?);
        if result.len() < requested {
            result.push(color);
        }
    }
    if result.len() == requested {
        return Ok(result);
    }
    let denominator = u32::try_from(seeds.len())
        .map_err(|_| Error::Render("palette seed count exceeds exact color arithmetic".into()))?;
    let (mut lightness, mut chroma) = (0.0_f64, 0.0_f64);
    for seed in &lab {
        resources.claim_visits(1).map_err(work_error)?;
        lightness += seed.l;
        chroma += (seed.a * seed.a + seed.b * seed.b).sqrt();
    }
    let character = PaletteCharacter {
        lightness: lightness / f64::from(denominator),
        chroma: (chroma / f64::from(denominator)).max(0.08),
    };
    let cap = HUE_DIRECTIONS * LIGHTNESS_BANDS.len() * CHROMA_BANDS.len();
    while result.len() < requested {
        let new_ordinal = result.len() - seeds.len();
        let candidates = 128_usize
            .saturating_add(new_ordinal.saturating_mul(32))
            .min(cap);
        let (rgb, actual) = select_candidate(character, &result, &lab, candidates, resources)?;
        result.push(rgb);
        lab.push(actual);
    }
    Ok(result)
}

fn encoded_channel(value: f64, resources: &OperationGenerationResources) -> Result<u8, Error> {
    // The inverse coefficients use 10 decimal digits. A 2^-22 slack bounds
    // their finite binary64 roundtrip error at the two exact gamut edges;
    // genuinely out-of-gamut colors go to the explicit T5d1b chroma reducer.
    if !value.is_finite() || !(-MATRIX_EDGE_SLACK..=1.0 + MATRIX_EDGE_SLACK).contains(&value) {
        return Err(Error::Render(
            "OKLab inverse color lies outside the sRGB gamut".into(),
        ));
    }
    let value = value.clamp(0.0, 1.0);
    let (mut lower, mut upper) = (0_usize, ENCODED_HALF_LINEAR_BITS.len());
    while lower < upper {
        resources.claim_visits(1).map_err(work_error)?;
        let middle = lower + (upper - lower) / 2;
        let threshold = f64::from_bits(ENCODED_HALF_LINEAR_BITS[middle]);
        if value > threshold
            || value.to_bits() == threshold.to_bits() && (middle + 1).is_multiple_of(2)
        {
            lower = middle + 1;
        } else {
            upper = middle;
        }
    }
    u8::try_from(lower).map_err(|_| Error::Render("sRGB channel ordinal overflowed".into()))
}

/// Exact nearest-even binary64 cube root for the nonnegative LMS result of
/// one opaque RGB channel combination. Search the finite ordered binary64
/// bit domain, then compare the exact rational cube of the midpoint between
/// adjacent representable candidates. No host libm call, approximate fallback
/// or iteration-sensitive convergence threshold is involved.
fn cube_root(input: f64, resources: &OperationGenerationResources) -> Result<f64, Error> {
    if !input.is_finite() || !(0.0..=2.0).contains(&input) {
        return Err(Error::Render(
            "OKLab LMS component is outside its finite domain".into(),
        ));
    }
    if input == 0.0 {
        resources.claim_visits(1).map_err(work_error)?;
        return Ok(0.0);
    }
    let (mut lower, mut upper) = (0_u64, 2.0_f64.to_bits());
    while upper - lower > 1 {
        let midpoint = lower + (upper - lower) / 2;
        let (significand, exponent) = binary_rational(f64::from_bits(midpoint));
        if compare_cube(significand, exponent, input, resources)?.is_le() {
            lower = midpoint;
        } else {
            upper = midpoint;
        }
    }
    let (lo_sig, lo_exp) = binary_rational(f64::from_bits(lower));
    let (hi_sig, hi_exp) = binary_rational(f64::from_bits(upper));
    let common_exp = lo_exp.min(hi_exp);
    let middle_sig = (BigUint::from(lo_sig)
        << usize::try_from(lo_exp - common_exp).expect("lower exponent is above common exponent"))
        + (BigUint::from(hi_sig)
            << usize::try_from(hi_exp - common_exp)
                .expect("upper exponent is above common exponent"));
    let (target_sig, target_exp) = binary_rational(input);
    admit(resources)?;
    let cube = &middle_sig * &middle_sig * &middle_sig;
    let ordering = compare_rational(
        &cube,
        3 * (common_exp - 1),
        &BigUint::from(target_sig),
        target_exp,
    );
    Ok(f64::from_bits(
        if ordering.is_lt() || ordering.is_eq() && lower & 1 != 0 {
            upper
        } else {
            lower
        },
    ))
}

fn compare_cube(
    significand: u64,
    exponent: i32,
    input: f64,
    resources: &OperationGenerationResources,
) -> Result<Ordering, Error> {
    let (target_sig, target_exp) = binary_rational(input);
    admit(resources)?;
    let value = BigUint::from(significand);
    Ok(compare_rational(
        &(&value * &value * &value),
        3 * exponent,
        &BigUint::from(target_sig),
        target_exp,
    ))
}

fn binary_rational(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), exponent - 1023 - 52)
    }
}

fn compare_rational(left: &BigUint, left_exp: i32, right: &BigUint, right_exp: i32) -> Ordering {
    if left_exp >= right_exp {
        (left << usize::try_from(left_exp - right_exp).expect("nonnegative exponent difference"))
            .cmp(right)
    } else {
        left.cmp(
            &(right
                << usize::try_from(right_exp - left_exp).expect("nonnegative exponent difference")),
        )
    }
}

fn admit(resources: &OperationGenerationResources) -> Result<(), Error> {
    resources.claim_visits(1).map_err(work_error)?;
    // Comparing three 53-bit binary significands with a binary64 target can
    // align at most 3*1074+160 bits. Charge five 4096-bit scratch operands
    // before the exact products and shifts, against the one root ledger.
    resources
        .claim_bytes(5 * 512)
        .map_err(|error| Error::Render(format!("OKLab cube-root scratch admission: {error}")))
}

fn work_error(error: crate::font::canonical::TextGenerationClaimError) -> Error {
    Error::Render(format!("OKLab cube-root work admission: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_channel_thresholds_match_the_pinned_independent_byte_stream() {
        use sha2::{Digest, Sha256};
        let bytes = ENCODED_HALF_LINEAR_BITS
            .iter()
            .flat_map(|bits| bits.to_be_bytes())
            .collect::<Vec<_>>();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            "ace388c2947682af22573df8b12dde52e54c10fa706965d3011b159a5a7e340f"
        );
        assert!(ENCODED_HALF_LINEAR_BITS
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
        let resources = OperationGenerationResources::new();
        for index in [0_usize, 1, 127, 254] {
            let midpoint = f64::from_bits(ENCODED_HALF_LINEAR_BITS[index]);
            let lower = u8::try_from(index).unwrap();
            assert_eq!(
                encoded_channel(midpoint.next_down(), &resources).unwrap(),
                lower
            );
            assert_eq!(
                encoded_channel(midpoint, &resources).unwrap(),
                if index.is_multiple_of(2) {
                    lower
                } else {
                    lower + 1
                }
            );
            assert_eq!(
                encoded_channel(midpoint.next_up(), &resources).unwrap(),
                lower + 1
            );
        }
    }

    #[test]
    fn opaque_channels_round_trip_without_an_srgb_libm_call() {
        let resources = OperationGenerationResources::new();
        for channel in 0_u8..=u8::MAX {
            for rgb in [
                (channel, channel, channel),
                (channel, 0, 0),
                (0, channel, 0),
                (0, 0, channel),
            ] {
                let lab = Oklab::from_rgb(rgb, &resources).unwrap();
                assert_eq!(
                    lab.opaque_rgb(&resources)
                        .unwrap_or_else(|error| panic!("{rgb:?}: {error}; {:?}", lab.linear_rgb())),
                    rgb
                );
            }
        }
        for (rgb, expected) in [
            ((0, 0, 0), (0.0, 0.0, 0.0)),
            ((255, 255, 255), (1.0, 0.0, 0.0)),
            (
                (255, 0, 0),
                (0.627_955_360_6, 0.224_863_061_1, 0.125_846_298_5),
            ),
            (
                (0, 255, 0),
                (0.866_439_611_5, -0.233_887_574_2, 0.179_498_479_9),
            ),
            (
                (0, 0, 255),
                (0.452_013_718_4, -0.032_456_984_2, -0.311_528_147_7),
            ),
        ] {
            let actual = Oklab::from_rgb(rgb, &resources).unwrap();
            let tolerance = if rgb == (255, 255, 255) { 1e-7 } else { 1e-9 };
            assert!(
                (actual.l - expected.0).abs() < tolerance,
                "{rgb:?}: {actual:?} vs {expected:?}"
            );
            assert!((actual.a - expected.1).abs() < tolerance);
            assert!((actual.b - expected.2).abs() < tolerance);
        }
    }

    #[test]
    #[expect(
        clippy::unreadable_literal,
        reason = "independently computed reference bit patterns are verbatim"
    )]
    fn cube_root_rounds_exact_binary_rationals_without_host_math() {
        let resources = OperationGenerationResources::new();
        for input in [0.0, f64::from_bits(1), 0.125, 1.0, 2.0] {
            let actual = cube_root(input, &resources).unwrap();
            assert!(actual.is_finite() && actual >= 0.0);
            if input.to_bits() == 0.125_f64.to_bits() {
                assert_eq!(actual.to_bits(), 0.5_f64.to_bits());
            }
            if input.to_bits() == 1.0_f64.to_bits() {
                assert_eq!(actual.to_bits(), 1.0_f64.to_bits());
            }
        }
        // Decimal roots of each *exact binary64 input*, independently
        // evaluated at 120, 240 and 480 decimal digits, agree on these bits.
        for (source, expected) in [
            (0x3fe0000000000000, 0x3fe965fea53d6e3d),
            (0x3fc0000000000001, 0x3fe0000000000000),
            (0x3fbfffffffffffff, 0x3fe0000000000000),
            (0x3fefffffffffffff, 0x3ff0000000000000),
            (0x3ff0000000000001, 0x3ff0000000000000),
            (0x0000000000000001, 0x2990000000000000),
            (0x01a56e1fc2f8f359, 0x2b2bff2ee48e0530),
            (0x3f69a5c37387b719, 0x3fc2b9ac34fcfa33),
        ] {
            assert_eq!(
                cube_root(f64::from_bits(source), &resources)
                    .unwrap()
                    .to_bits(),
                expected,
                "{source:016x}"
            );
        }
    }

    #[test]
    fn color_math_refuses_before_unadmitted_work_or_a_non_gamut_output() {
        for (visits, bytes, role) in [
            (0, u64::MAX, "work admission"),
            (1_000, 0, "scratch admission"),
        ] {
            let resources = OperationGenerationResources::with_test_limits(visits, bytes);
            let error = Oklab::from_rgb((240, 70, 50), &resources).unwrap_err();
            assert!(error.to_string().contains(role), "{error}");
        }
        let resources = OperationGenerationResources::new();
        let outside = Oklab {
            l: 0.5,
            a: 2.0,
            b: 2.0,
        };
        assert!(outside
            .opaque_rgb(&resources)
            .unwrap_err()
            .to_string()
            .contains("outside the sRGB gamut"));
    }

    #[test]
    fn independent_oklab_midpoints_preserve_endpoints_and_reduce_chroma_only_when_needed() {
        let resources = OperationGenerationResources::new();
        for (first, second, expected) in [
            ((255, 0, 0), (0, 0, 255), (140, 83, 162)),
            ((255, 0, 0), (0, 255, 0), (208, 168, 0)),
            ((0, 0, 0), (255, 255, 255), (99, 99, 99)),
        ] {
            assert_eq!(interpolate(first, second, 0.0, &resources).unwrap(), first);
            assert_eq!(interpolate(first, second, 1.0, &resources).unwrap(), second);
            assert_eq!(
                interpolate(first, second, 0.5, &resources).unwrap(),
                expected
            );
        }
        let lab = Oklab::from_rgb((255, 0, 0), &resources).unwrap();
        let outside = Oklab {
            a: lab.a * 2.0,
            b: lab.b * 2.0,
            ..lab
        };
        assert!(outside.opaque_rgb(&resources).is_err());
        let (rgb, reduced) = outside.reduced_rgb(&resources).unwrap();
        assert!(reduced.in_gamut().unwrap());
        assert!(reduced.a.abs() < outside.a.abs() && reduced.b.abs() < outside.b.abs());
        assert_eq!(reduced.opaque_rgb(&resources).unwrap(), rgb);
        assert!(interpolate((0, 0, 0), (255, 255, 255), f64::NAN, &resources).is_err());
    }

    #[test]
    fn candidate_order_is_prefix_stable_and_never_cycles_a_seed() {
        let seeds = [(0, 114, 178), (230, 159, 0)];
        let short = extend_palette(&seeds, 3, &OperationGenerationResources::new()).unwrap();
        let extended = extend_palette(&seeds, 5, &OperationGenerationResources::new()).unwrap();
        assert_eq!(short.as_slice(), &extended[..3]);
        assert_eq!(&extended[..2], &seeds);
        assert_eq!(
            &extended[2..],
            &[(192, 83, 122), (76, 145, 54), (157, 138, 238)]
        );
        assert!(extended
            .iter()
            .enumerate()
            .all(|(index, color)| !extended[..index].contains(color)));
        assert!(extend_palette(&[], 1, &OperationGenerationResources::new()).is_err());
        let first = CandidateChoice {
            distance: 0.25,
            index: 5,
            color: (0, 0, 0),
            lab: Oklab {
                l: 0.0,
                a: 0.0,
                b: 0.0,
            },
        };
        assert!(candidate_better(0.25, 4, Some(&first)));
        assert!(!candidate_better(0.25, 6, Some(&first)));
        assert!(candidate_better(0.5, 6, Some(&first)));
        assert!(!candidate_better(0.125, 4, Some(&first)));
    }
}
