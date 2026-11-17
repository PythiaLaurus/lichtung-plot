use super::*;

#[derive(Clone, Copy, Debug)]
enum ImageKind {
    Raster,
    Svg,
}

fn png_pixel() -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[255, 0, 0, 255]).unwrap();
        writer.finish().unwrap();
    }
    bytes
}

fn sbix(png: &[u8]) -> Vec<u8> {
    let mut strike = Vec::new();
    strike.extend(16_u16.to_be_bytes()); // ppem
    strike.extend(72_u16.to_be_bytes()); // ppi
    let mut glyphs = Vec::new();
    // Four glyphs plus the terminal offset. B=1 has both outline and bitmap,
    // space=2 has neither, A=3 has only the bitmap.
    for painted in [false, true, false, true] {
        strike.extend(u32::try_from(24 + glyphs.len()).unwrap().to_be_bytes());
        if painted {
            glyphs.extend([0; 4]); // origin
            glyphs.extend(b"png ");
            glyphs.extend(png);
        }
    }
    strike.extend(u32::try_from(24 + glyphs.len()).unwrap().to_be_bytes());
    strike.extend(glyphs);
    let mut table = Vec::new();
    table.extend(1_u16.to_be_bytes());
    table.extend(1_u16.to_be_bytes()); // flags
    table.extend(1_u32.to_be_bytes()); // one strike
    table.extend(12_u32.to_be_bytes());
    table.extend(strike);
    table
}

fn svg() -> Vec<u8> {
    let document = br#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"><g id="glyph1"><rect width="1" height="1" fill="red"/></g><g id="glyph3"><rect width="1" height="1" fill="red"/></g></svg>"#;
    let mut table = Vec::new();
    table.extend(0_u16.to_be_bytes());
    table.extend(10_u32.to_be_bytes()); // document list
    table.extend(0_u32.to_be_bytes()); // reserved
    table.extend(2_u16.to_be_bytes());
    for glyph in [1_u16, 3] {
        table.extend(glyph.to_be_bytes());
        table.extend(glyph.to_be_bytes());
        table.extend(26_u32.to_be_bytes());
        table.extend(u32::try_from(document.len()).unwrap().to_be_bytes());
    }
    table.extend(document);
    table
}

fn image_font(kind: ImageKind, empty_placeholders: bool) -> Vec<u8> {
    let base = font("Image", &[' ', 'A', 'B'], Some((800, -200)), (800, -200));
    let raw = ttf_parser::RawFace::parse(&base, 0).unwrap();
    let mut tables: Vec<_> = raw
        .table_records
        .into_iter()
        .map(|record| {
            let tag = record.tag.0.to_be_bytes();
            let mut data = raw.table(record.tag).unwrap().to_vec();
            match &tag {
                b"cmap" => {
                    for group in data[28..].chunks_exact_mut(12) {
                        if group[..4] == u32::from('A').to_be_bytes() {
                            group[8..12].copy_from_slice(&3_u32.to_be_bytes());
                        }
                    }
                }
                b"maxp" => data[4..6].copy_from_slice(&4_u16.to_be_bytes()),
                b"hhea" => data[34..36].copy_from_slice(&4_u16.to_be_bytes()),
                b"hmtx" => data.extend([0x02, 0x58, 0, 0]),
                b"loca" => {
                    data.extend(17_u16.to_be_bytes());
                    if empty_placeholders {
                        data.fill(0);
                    }
                }
                b"glyf" if empty_placeholders => data.clear(),
                _ => {}
            }
            (tag, data)
        })
        .collect();
    tables.push(match kind {
        ImageKind::Raster => (*b"sbix", sbix(&png_pixel())),
        ImageKind::Svg => (*b"SVG ", svg()),
    });
    super::super::shaping_tests::sfnt(tables)
}

fn assert_image_payload(bytes: &[u8], kind: ImageKind) {
    let face = ttf_parser::Face::parse(bytes, 0).unwrap();
    let glyph = face.glyph_index('A').unwrap();
    assert_eq!(glyph.0, 3);
    assert!(face.glyph_bounding_box(glyph).is_none());
    match kind {
        ImageKind::Raster => {
            let image = face.glyph_raster_image(glyph, u16::MAX).unwrap();
            assert_eq!((image.width, image.height), (1, 1));
            let decoded = resvg::tiny_skia::Pixmap::decode_png(image.data).unwrap();
            assert_eq!((decoded.width(), decoded.height()), (1, 1));
            assert_eq!(decoded.pixels()[0].alpha(), 255);
        }
        ImageKind::Svg => {
            let document = face.glyph_svg_image(glyph).unwrap();
            let tree =
                resvg::usvg::Tree::from_data(document.data, &resvg::usvg::Options::default())
                    .unwrap();
            let mut pixel = resvg::tiny_skia::Pixmap::new(1, 1).unwrap();
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::identity(),
                &mut pixel.as_mut(),
            );
            assert_eq!(pixel.pixels()[0].alpha(), 255);
        }
    }
}

fn layered_color_font() -> Vec<u8> {
    let base = image_font(ImageKind::Raster, false);
    let raw = ttf_parser::RawFace::parse(&base, 0).unwrap();
    let mut tables: Vec<_> = raw
        .table_records
        .into_iter()
        .filter(|record| record.tag != ttf_parser::Tag::from_bytes(b"sbix"))
        .map(|record| {
            (
                record.tag.0.to_be_bytes(),
                raw.table(record.tag).unwrap().to_vec(),
            )
        })
        .collect();
    let mut palette = Vec::new();
    for value in [0_u16, 1, 1, 1] {
        palette.extend(value.to_be_bytes()); // v0, one entry/palette/color
    }
    palette.extend(14_u32.to_be_bytes());
    palette.extend(0_u16.to_be_bytes()); // first palette's color index
    palette.extend([0, 0, 255, 255]); // opaque red, BGRA
    let mut layers = Vec::new();
    layers.extend(0_u16.to_be_bytes()); // COLR v0
    layers.extend(2_u16.to_be_bytes()); // B=1 and A=3 have color definitions
    layers.extend(14_u32.to_be_bytes()); // base records
    layers.extend(26_u32.to_be_bytes()); // layer records
    layers.extend(1_u16.to_be_bytes());
    for glyph in [1_u16, 3] {
        for value in [glyph, 0, 1] {
            layers.extend(value.to_be_bytes()); // first layer 0, count 1
        }
    }
    layers.extend(1_u16.to_be_bytes()); // draw the real B outline
    layers.extend(0_u16.to_be_bytes()); // using palette entry 0
    tables.extend([(*b"CPAL", palette), (*b"COLR", layers)]);
    super::super::shaping_tests::sfnt(tables)
}

#[derive(Default)]
struct ColorPaintWitness {
    glyph: Option<ttf_parser::GlyphId>,
    color: Option<ttf_parser::RgbaColor>,
}

impl<'a> ttf_parser::colr::Painter<'a> for ColorPaintWitness {
    fn outline_glyph(&mut self, glyph: ttf_parser::GlyphId) {
        assert!(self.glyph.replace(glyph).is_none());
    }

    fn paint(&mut self, paint: ttf_parser::colr::Paint<'a>) {
        let ttf_parser::colr::Paint::Solid(color) = paint else {
            panic!("the generated v0 fixture has one solid layer")
        };
        assert!(self.color.replace(color).is_none());
    }

    // COLR v0 emits no transforms, clips or compositing operations.
    fn push_clip(&mut self) {}
    fn push_clip_box(&mut self, _: ttf_parser::colr::ClipBox) {}
    fn pop_clip(&mut self) {}
    fn push_layer(&mut self, _: ttf_parser::colr::CompositeMode) {}
    fn pop_layer(&mut self) {}
    fn push_transform(&mut self, _: ttf_parser::Transform) {}
    fn pop_transform(&mut self) {}
}

fn assert_color_payload(bytes: &[u8]) {
    let face = ttf_parser::Face::parse(bytes, 0).unwrap();
    let glyph = face.glyph_index('A').unwrap();
    assert_eq!(glyph.0, 3);
    assert!(face.glyph_bounding_box(glyph).is_none());
    assert!(face.glyph_raster_image(glyph, u16::MAX).is_none());
    assert!(face.glyph_svg_image(glyph).is_none());
    assert!(face.is_color_glyph(glyph));
    let mut witness = ColorPaintWitness::default();
    let color = ttf_parser::RgbaColor::new(255, 0, 0, 255);
    face.paint_color_glyph(glyph, 0, color, &mut witness)
        .unwrap();
    assert_eq!(witness.glyph, Some(ttf_parser::GlyphId(1)));
    assert_eq!(witness.color, Some(color));
    let ink = face.glyph_bounding_box(witness.glyph.unwrap()).unwrap();
    assert!(ink.x_max > ink.x_min && ink.y_max > ink.y_min);
}

#[test]
fn selected_color_layers_without_base_outline_fail_at_use() {
    let bytes = layered_color_font();
    assert_color_payload(&bytes);
    let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
    let b = face.glyph_index('B').unwrap();
    let space = face.glyph_index(' ').unwrap();
    assert!(face.glyph_bounding_box(b).is_some() && face.is_color_glyph(b));
    assert!(face.glyph_bounding_box(space).is_none() && !face.is_color_glyph(space));
    let fallback = font("Fallback", &['A'], Some((800, -200)), (800, -200));
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![bytes, fallback]);
    let error = plan(&environment, "A", &["Image", "Fallback"]).unwrap_err();
    let stack = canonical::named_stack(&["Image", "Fallback"]);
    let pair = environment
        .prepared_pair(&stack, RequestedWeight::Normal)
        .unwrap()
        .unwrap();
    assert_eq!(pair.provenance.candidates.len(), 2);
    assert!(matches!(error,
        TextPlanError::Font(FontError::UnsupportedGlyphPaint {
            identity, glyph_id: 3, paint: canonical::GlyphPaint::ColorLayers
        }) if identity == pair.primary));
    let mut outline = OutlineCount::default();
    assert!(
        matches!(environment.outline_glyph(pair.primary, 3, &mut outline),
        Err(FontError::UnsupportedGlyphPaint {
            identity, glyph_id: 3, paint: canonical::GlyphPaint::ColorLayers
        }) if identity == pair.primary)
    );
    assert_eq!(outline.0, 0);
    // A retained color-only glyph does not reject another selected outline or
    // a legitimate empty glyph, even after a failed request in this environment.
    for source in ["B", " "] {
        let measured = plan(&environment, source, &["Image", "Fallback"]).unwrap();
        assert_eq!(measured.lines[0].ink_quanta.is_some(), source == "B");
        check_output(&environment, &measured);
    }
}

#[test]
fn selected_image_only_glyph_fails_at_use_without_font_fallback() {
    for (kind, placeholders) in [
        (ImageKind::Raster, false),
        (ImageKind::Raster, true),
        (ImageKind::Svg, false),
    ] {
        let bytes = image_font(kind, placeholders);
        assert_image_payload(&bytes, kind);
        let fallback = font("Fallback", &['A'], Some((800, -200)), (800, -200));
        let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![bytes, fallback]);
        let error = plan(&environment, "A", &["Image", "Fallback"]).unwrap_err();
        let stack = canonical::named_stack(&["Image", "Fallback"]);
        let pair = environment
            .prepared_pair(&stack, RequestedWeight::Normal)
            .unwrap()
            .unwrap();
        assert_eq!(pair.provenance.candidates.len(), 2);
        let expected = match kind {
            ImageKind::Raster => canonical::GlyphPaint::RasterImage,
            ImageKind::Svg => canonical::GlyphPaint::SvgImage,
        };
        assert!(matches!(error,
            TextPlanError::Font(FontError::UnsupportedGlyphPaint { identity, glyph_id: 3, paint })
            if identity == pair.primary && paint == expected));
        let mut outline = OutlineCount::default();
        assert!(
            matches!(environment.outline_glyph(pair.primary, 3, &mut outline),
            Err(FontError::UnsupportedGlyphPaint { identity, glyph_id: 3, paint })
            if identity == pair.primary && paint == expected)
        );
        assert_eq!(outline.0, 0);
    }
}

#[test]
fn outlines_and_defined_empty_glyphs_are_not_rejected_for_optional_images() {
    for kind in [ImageKind::Raster, ImageKind::Svg] {
        let bytes = image_font(kind, false);
        let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
        let b = face.glyph_index('B').unwrap();
        let space = face.glyph_index(' ').unwrap();
        assert!(face.glyph_bounding_box(b).is_some());
        assert!(face.glyph_bounding_box(space).is_none());
        match kind {
            ImageKind::Raster => {
                assert!(face.glyph_raster_image(b, u16::MAX).is_some());
                assert!(face.glyph_raster_image(space, u16::MAX).is_none());
            }
            ImageKind::Svg => {
                assert!(face.glyph_svg_image(b).is_some());
                assert!(face.glyph_svg_image(space).is_none());
            }
        }
        let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![bytes]);
        for source in ["B", " "] {
            let measured = plan(&environment, source, &["Image"]).unwrap();
            assert_eq!(measured.lines[0].ink_quanta.is_some(), source == "B");
            check_output(&environment, &measured);
            let mut outline = OutlineCount::default();
            let (_, ink) = environment
                .outline_glyph(
                    measured.lines[0].runs[0].face,
                    measured.lines[0].runs[0].glyphs[0].glyph_id,
                    &mut outline,
                )
                .unwrap();
            assert_eq!(ink.is_some(), source == "B");
            assert_eq!(outline.0 > 0, source == "B");
        }
    }
}

#[test]
fn defined_empty_nonspace_glyph_without_image_remains_supported() {
    let base = image_font(ImageKind::Raster, false);
    let raw = ttf_parser::RawFace::parse(&base, 0).unwrap();
    let tables = raw
        .table_records
        .into_iter()
        .filter(|record| record.tag != ttf_parser::Tag::from_bytes(b"sbix"))
        .map(|record| {
            (
                record.tag.0.to_be_bytes(),
                raw.table(record.tag).unwrap().to_vec(),
            )
        })
        .collect();
    let bytes = super::super::shaping_tests::sfnt(tables);
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![bytes]);
    let measured = plan(&environment, "A", &["Image"]).unwrap();
    assert_eq!(measured.lines[0].runs[0].glyphs[0].glyph_id, 3);
    assert!(measured.lines[0].ink_quanta.is_none());
    assert_eq!(measured.height_quanta, 10 * 1024);
    check_output(&environment, &measured);
}

#[derive(Default)]
struct OutlineCount(usize);
impl ttf_parser::OutlineBuilder for OutlineCount {
    fn move_to(&mut self, _: f32, _: f32) {
        self.0 += 1;
    }
    fn line_to(&mut self, _: f32, _: f32) {
        self.0 += 1;
    }
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 1;
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 1;
    }
    fn close(&mut self) {
        self.0 += 1;
    }
}
