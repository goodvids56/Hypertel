/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0.
 * If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIFont`.

use super::ui_graphics::UIGraphicsGetCurrentContext;
use crate::font::{Font, TextAlignment, WrapMode};
use crate::frameworks::core_graphics::cg_bitmap_context::CGBitmapContextDrawer;
use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::ns_string::{from_rust_string, get_static_str, to_rust_string};
use crate::frameworks::foundation::NSInteger;
use crate::objc::{autorelease, id, msg, nil, objc_classes, ClassExports, HostObject, NSZonePtr};
use crate::Environment;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Default)]
pub(super) struct State {
    fonts: HashMap<FontKind, Font>,
    sans_regular_ja: Option<Font>,
    sans_bold_ja: Option<Font>,
}
impl State {
    fn get_font_by_kind(&mut self, font_kind: FontKind) -> &Font {
        self.fonts
            .entry(font_kind)
            .or_insert_with(|| match font_kind {
                FontKind::MonoRegular => Font::mono_regular(),
                FontKind::MonoBold => Font::mono_bold(),
                FontKind::MonoBoldItalic => Font::mono_bold_italic(),
                FontKind::MonoItalic => Font::mono_italic(),
                FontKind::SansRegular => Font::sans_regular(),
                FontKind::SansBold => Font::sans_bold(),
                FontKind::SansBoldItalic => Font::sans_bold_italic(),
                FontKind::SansItalic => Font::sans_italic(),
                FontKind::SerifRegular => Font::serif_regular(),
                FontKind::SerifBold => Font::serif_bold(),
                FontKind::SerifBoldItalic => Font::serif_bold_italic(),
                FontKind::SerifItalic => Font::serif_italic(),
            })
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
enum FontKind {
    #[default]
    MonoRegular,
    MonoBold,
    MonoBoldItalic,
    MonoItalic,
    SansRegular,
    SansBold,
    SansBoldItalic,
    SansItalic,
    SerifRegular,
    SerifBold,
    SerifBoldItalic,
    SerifItalic,
}

#[derive(Default)]
struct UIFontHostObject {
    size: CGFloat,
    kind: FontKind,
}
impl HostObject for UIFontHostObject {}

pub type UILineBreakMode = NSInteger;
pub const UILineBreakModeWordWrap: UILineBreakMode = 0;
pub const UILineBreakModeCharacterWrap: UILineBreakMode = 1;
#[allow(dead_code)]
pub const UILineBreakModeClip: UILineBreakMode = 2;
#[allow(dead_code)]
pub const UILineBreakModeHeadTruncation: UILineBreakMode = 3;
pub const UILineBreakModeTailTruncation: UILineBreakMode = 4;
#[allow(dead_code)]
pub const UILineBreakModeMiddleTruncation: UILineBreakMode = 5;

pub type UITextAlignment = NSInteger;
pub const UITextAlignmentLeft: UITextAlignment = 0;
pub const UITextAlignmentCenter: UITextAlignment = 1;
pub const UITextAlignmentRight: UITextAlignment = 2;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);
@implementation UIFont: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = UIFontHostObject {
        size: 17.0,
        kind: FontKind::SansRegular,
    };
    env.objc.alloc_object(this, Box::new(host_object), &mut env.mem)
}

+ (CGFloat)systemFontSize {
    14.0
}

+ (CGFloat)smallSystemFontSize {
    12.0
}

+ (CGFloat)labelFontSize {
    17.0
}

+ (CGFloat)buttonFontSize {
    18.0
}

+ (id)systemFontOfSize:(CGFloat)size {
    let host_object = UIFontHostObject {
        size,
        kind: FontKind::SansRegular,
    };
    let new = env.objc.alloc_object(this, Box::new(host_object), &mut env.mem);
    autorelease(env, new)
}

+ (id)boldSystemFontOfSize:(CGFloat)size {
    let host_object = UIFontHostObject {
        size,
        kind: FontKind::SansBold,
    };
    let new = env.objc.alloc_object(this, Box::new(host_object), &mut env.mem);
    autorelease(env, new)
}

+ (id)italicSystemFontOfSize:(CGFloat)size {
    let host_object = UIFontHostObject {
        size,
        kind: FontKind::SansItalic,
    };
    let new = env.objc.alloc_object(this, Box::new(host_object), &mut env.mem);
    autorelease(env, new)
}

+ (id)fontWithName:(id)fontName size:(CGFloat)fontSize {
    if fontName == nil {
        let host_object = UIFontHostObject {
            kind: FontKind::SansRegular,
            size: fontSize,
        };
        let new = env.objc.alloc_object(this, Box::new(host_object), &mut env.mem);
        return autorelease(env, new);
    }
    let font_name = to_rust_string(env, fontName).to_string();
    let host_object = UIFontHostObject {
        kind: get_equivalent_font(&font_name).unwrap_or_else(|| {
            FontKind::SansRegular
        }),
        size: fontSize,
    };
    let new = env.objc.alloc_object(this, Box::new(host_object), &mut env.mem);
    autorelease(env, new)
}

- (id)initWithCoder:(id)coder {
    let key_name = get_static_str(env, "UIFontName");
    let mut font_name: id = nil;
    if msg![env; coder containsValueForKey:key_name] {
        font_name = msg![env; coder decodeObjectForKey:key_name];
    }

    let key_size = get_static_str(env, "UIFontPointSize");
    let mut font_size: f32 = 17.0;
    if msg![env; coder containsValueForKey:key_size] {
        font_size = msg![env; coder decodeFloatForKey:key_size];
    }

    let kind = if font_name != nil {
        let name_str = to_rust_string(env, font_name).to_string();
        get_equivalent_font(&name_str).unwrap_or(FontKind::SansRegular)
    } else {
        FontKind::SansRegular
    };
    env.objc.borrow_mut::<UIFontHostObject>(this).size = font_size as CGFloat;
    env.objc.borrow_mut::<UIFontHostObject>(this).kind = kind;

    this
}

- (CGFloat)pointSize {
    let host_object = env.objc.borrow::<UIFontHostObject>(this);
    host_object.size
}

- (CGFloat)ascender {
    let host_object = env.objc.borrow::<UIFontHostObject>(this);
    let font = env.framework_state.uikit.ui_font.get_font_by_kind(host_object.kind);
    font.ascent(host_object.size)
}

- (CGFloat)descender {
    let host_object = env.objc.borrow::<UIFontHostObject>(this);
    let font = env.framework_state.uikit.ui_font.get_font_by_kind(host_object.kind);
    font.descent(host_object.size)
}

- (CGFloat)leading {
    let host_object = env.objc.borrow::<UIFontHostObject>(this);
    let font = env.framework_state.uikit.ui_font.get_font_by_kind(host_object.kind);
    // This _mostly_ lines up with what is reported on actual devices. It
    // seems there's variance between what apple and rusttype report for
    // leading/descent values, which is probably to be expected.
    //
    // As for what the 1.575 is doing here, I don't know. It's probably not
    // the right value, it's just (close) to a linear regression of size/leading
    // for Liberation Sans, and it mostly makes it work.
    (font.ascent(host_object.size) - font.descent(host_object.size) + font.line_gap(host_object.size) + 1.575).round()
}

- (CGFloat)lineHeight {
    let ascender: CGFloat = msg![env; this ascender];
    let descender: CGFloat = msg![env; this descender];
    let leading: CGFloat = msg![env; this leading];
    ascender + leading - descender
}

- (id)fontWithSize:(CGFloat)size {
    let kind = env.objc.borrow::<UIFontHostObject>(this).kind;
    let host_object = UIFontHostObject { size, kind };
    let class_ptr = env.objc.get_known_class("UIFont", &mut env.mem);
    let new_font = env.objc.alloc_object(class_ptr, Box::new(host_object), &mut env.mem);
    autorelease(env, new_font)
}

- (id)fontName {
    let kind = env.objc.borrow::<UIFontHostObject>(this).kind;
    let name = font_kind_to_name(kind);
    let ns = from_rust_string(env, name.to_string());
    autorelease(env, ns)
}

- (id)familyName {
    let kind = env.objc.borrow::<UIFontHostObject>(this).kind;
    let family = font_kind_to_family(kind);
    let ns = from_rust_string(env, family.to_string());
    autorelease(env, ns)
}

@end

};

fn convert_line_break_mode(ui_mode: UILineBreakMode) -> WrapMode {
    match ui_mode {
        UILineBreakModeWordWrap => WrapMode::Word,
        UILineBreakModeCharacterWrap => WrapMode::Char,
        UILineBreakModeTailTruncation => WrapMode::Word,
        _ => WrapMode::Word,
    }
}

#[rustfmt::skip]
fn get_font<'a>(state: &'a mut State, kind: FontKind, text: &str) -> &'a Font {
    for c in text.chars() {
        let c = c as u32;
        if (0x3000..=0x30FF).contains(&c) || (0xFF00..=0xFFEF).contains(&c) ||
           (0x4e00..=0x9FA0).contains(&c) || (0x3400..=0x4DBF).contains(&c) {
            match kind {
                FontKind::MonoRegular |
                FontKind::MonoItalic | FontKind::SansRegular | FontKind::SansItalic | FontKind::SerifRegular | FontKind::SerifItalic => {
                    if state.sans_regular_ja.is_none() {
                        state.sans_regular_ja = Some(Font::sans_regular_ja());
                    }
                    return state.sans_regular_ja.as_ref().unwrap();
                },
                FontKind::MonoBold | FontKind::MonoBoldItalic |
                FontKind::SansBold | FontKind::SansBoldItalic | FontKind::SerifBold | FontKind::SerifBoldItalic => {
                    if state.sans_bold_ja.is_none() {
                        state.sans_bold_ja = Some(Font::sans_bold_ja());
                    }
                    return state.sans_bold_ja.as_ref().unwrap();
                },
            }
        }
    }

    state.get_font_by_kind(kind)
}

pub fn size_with_font(
    env: &mut Environment,
    font: id,
    text: &str,
    constrained: Option<(CGSize, UILineBreakMode)>,
) -> CGSize {
    let host_object = env.objc.borrow::<UIFontHostObject>(font);
    let font = get_font(
        &mut env.framework_state.uikit.ui_font,
        host_object.kind,
        text,
    );
    let wrap = constrained.map(|(size, ui_mode)| (size.width, convert_line_break_mode(ui_mode)));

    let (width, height) = font.calculate_text_size(host_object.size, text, wrap);
    CGSize { width, height }
}

pub fn break_lines_with_font<'a>(
    env: &mut Environment,
    font: id,
    text: &'a str,
    constrained: Option<(CGSize, UILineBreakMode)>,
) -> Vec<(f32, &'a str)> {
    let host_object = env.objc.borrow::<UIFontHostObject>(font);
    let font = get_font(
        &mut env.framework_state.uikit.ui_font,
        host_object.kind,
        text,
    );
    let wrap = constrained.map(|(size, ui_mode)| (size.width, convert_line_break_mode(ui_mode)));

    font.break_lines(host_object.size, text, wrap)
}

#[inline(always)]
pub fn draw_font_glyph(
    drawer: &mut CGBitmapContextDrawer,
    raster_glyph: crate::font::RasterGlyph,
    fill_color: (f32, f32, f32, f32),
    clip_x: Option<Range<f32>>,
    clip_y: Option<Range<f32>>,
) {
    let mut glyph_rect = {
        let (x, y) = raster_glyph.origin();
        let (width, height) = raster_glyph.dimensions();
        CGRect {
            origin: CGPoint { x, y },
            size: CGSize {
                width: width as f32,
                height: height as f32,
            },
        }
    };
    if let Some(clip_x) = clip_x {
        if glyph_rect.origin.x >= clip_x.end {
            return;
        }
        if glyph_rect.origin.x + glyph_rect.size.width > clip_x.end {
            glyph_rect.size.width = clip_x.end - glyph_rect.origin.x;
        }
    }
    if let Some(clip_y) = clip_y {
        if glyph_rect.origin.y >= clip_y.end {
            return;
        }
        if glyph_rect.origin.y + glyph_rect.size.height > clip_y.end {
            glyph_rect.size.height = clip_y.end - glyph_rect.origin.y;
        }
    }

    for ((x, y), (tex_x, tex_y)) in drawer.iter_transformed_pixels(glyph_rect) {
        let coverage = raster_glyph.pixel_at((
            (tex_x * glyph_rect.size.width - 0.5).round() as i32,
            (tex_y * glyph_rect.size.height - 0.5).round() as i32,
        ));
        let (r, g, b, a) = fill_color;
        let (r, g, b, a) = (r * coverage, g * coverage, b * coverage, a * coverage);
        drawer.put_pixel((x, y), (r, g, b, a), true);
    }
}

pub fn draw_at_point(
    env: &mut Environment,
    font: id,
    text: &str,
    point: CGPoint,
    width_and_line_break_mode: Option<(CGFloat, UILineBreakMode)>,
) -> CGSize {
    let context = UIGraphicsGetCurrentContext(env);
    let host_object = env.objc.borrow::<UIFontHostObject>(font);

    let font = get_font(
        &mut env.framework_state.uikit.ui_font,
        host_object.kind,
        text,
    );
    let width_and_line_break_mode =
        width_and_line_break_mode.map(|(width, ui_mode)| (width, convert_line_break_mode(ui_mode)));
    let clip_x = width_and_line_break_mode.map(|(width, _)| point.x..(point.x + width));
    let (width, height) =
        font.calculate_text_size(host_object.size, text, width_and_line_break_mode);
    let mut drawer = CGBitmapContextDrawer::new(&env.objc, &mut env.mem, context);
    let fill_color = drawer.rgb_fill_color();
    font.draw(
        host_object.size,
        text,
        (point.x, point.y),
        width_and_line_break_mode,
        TextAlignment::Left,
        |raster_glyph| draw_font_glyph(&mut drawer, raster_glyph, fill_color, clip_x.clone(), None),
    );
    CGSize { width, height }
}

pub fn draw_in_rect(
    env: &mut Environment,
    font: id,
    text: &str,
    rect: CGRect,
    line_break_mode: UILineBreakMode,
    alignment: UITextAlignment,
) -> CGSize {
    let context = UIGraphicsGetCurrentContext(env);
    let text_size = size_with_font(env, font, text, Some((rect.size, line_break_mode)));

    let host_object = env.objc.borrow::<UIFontHostObject>(font);
    let font = get_font(
        &mut env.framework_state.uikit.ui_font,
        host_object.kind,
        text,
    );
    let mut drawer = CGBitmapContextDrawer::new(&env.objc, &mut env.mem, context);
    let fill_color = drawer.rgb_fill_color();
    let (origin_x_offset, alignment) = match alignment {
        UITextAlignmentLeft => (0.0, TextAlignment::Left),
        UITextAlignmentCenter => (rect.size.width / 2.0, TextAlignment::Center),
        UITextAlignmentRight => (rect.size.width, TextAlignment::Right),
        _ => (0.0, TextAlignment::Left),
    };
    font.draw(
        host_object.size,
        text,
        (rect.origin.x + origin_x_offset, rect.origin.y),
        Some((rect.size.width, convert_line_break_mode(line_break_mode))),
        alignment,
        |raster_glyph| {
            draw_font_glyph(
                &mut drawer,
                raster_glyph,
                fill_color,
                Some(rect.origin.x..(rect.origin.x + rect.size.width)),
                Some(rect.origin.y..(rect.origin.y + rect.size.height)),
            )
        },
    );
    text_size
}

#[rustfmt::skip]
fn get_equivalent_font(system_font: &str) -> Option<FontKind> {
    match system_font {
        "Courier"                          => Some(FontKind::MonoRegular),
        "Courier-Bold"                     => Some(FontKind::MonoBold),
        "Courier-Oblique"                  => Some(FontKind::MonoItalic),
        "Courier-BoldOblique"              => Some(FontKind::MonoBoldItalic),
        "CourierNewPSMT"                   => Some(FontKind::MonoRegular),
        "CourierNewPS-BoldMT"              => Some(FontKind::MonoBold),
        "CourierNewPS-ItalicMT"            => Some(FontKind::MonoItalic),
        "CourierNewPS-BoldItalicMT"        => Some(FontKind::MonoBoldItalic),
        "ArialMT"                          => Some(FontKind::SansRegular),
        "Arial-BoldMT"                     => Some(FontKind::SansBold),
        "Arial-ItalicMT"                   => Some(FontKind::SansItalic),
        "Arial-BoldItalicMT"               => Some(FontKind::SansBoldItalic),
        "ArialRoundedMTBold"               => Some(FontKind::SansBold),
        "ArialUnicodeMS"                   => None,
        "Helvetica"                        => Some(FontKind::SansRegular),
        "Helvetica-Bold"                   => Some(FontKind::SansBold),
        "Helvetica-Oblique"                => Some(FontKind::SansItalic),
        "Helvetica-BoldOblique"            => Some(FontKind::SansBoldItalic),
        "Helvetica-Light"                  => Some(FontKind::SansRegular),
        "Helvetica-LightOblique"           => Some(FontKind::SansItalic),
        "Helvetica-Narrow"                 => Some(FontKind::SansRegular),
        "Helvetica-Narrow-Bold"            => Some(FontKind::SansBold),
        "Helvetica-Narrow-Oblique"         => Some(FontKind::SansItalic),
        "Helvetica-Narrow-BoldOblique"     => Some(FontKind::SansBoldItalic),
        "HelveticaNeue"                    => Some(FontKind::SansRegular),
        "HelveticaNeue-Bold"               => Some(FontKind::SansBold),
        "HelveticaNeue-Italic"             => Some(FontKind::SansItalic),
        "HelveticaNeue-BoldItalic"         => Some(FontKind::SansBoldItalic),
        "HelveticaNeue-Light"              => Some(FontKind::SansRegular),
        "HelveticaNeue-LightItalic"        => Some(FontKind::SansItalic),
        "HelveticaNeue-Medium"             => Some(FontKind::SansBold),
        "HelveticaNeue-UltraLight"         => Some(FontKind::SansRegular),
        "HelveticaNeue-UltraLightItalic"   => Some(FontKind::SansItalic),
        "HelveticaNeue-CondensedBold"      => Some(FontKind::SansBold),
        "HelveticaNeue-CondensedBlack"     => Some(FontKind::SansBold),
        "HelveticaNeue-Thin"               => Some(FontKind::SansRegular),
        "HelveticaNeue-ThinItalic"         => Some(FontKind::SansItalic),
        "Verdana"                          => Some(FontKind::SansRegular),
        "Verdana-Bold"                     => Some(FontKind::SansBold),
        "Verdana-Italic"                   => Some(FontKind::SansItalic),
        "Verdana-BoldItalic"               => Some(FontKind::SansBoldItalic),
        "TrebuchetMS"                      => Some(FontKind::SansRegular),
        "TrebuchetMS-Bold"                 => Some(FontKind::SansBold),
        "TrebuchetMS-Italic"               => Some(FontKind::SansItalic),
        "TrebuchetMS-BoldItalic"           => Some(FontKind::SansBoldItalic),
        "Futura-Medium"                    => Some(FontKind::SansRegular),
        "Futura-MediumItalic"              => Some(FontKind::SansItalic),
        "Futura-CondensedMedium"           => Some(FontKind::SansRegular),
        "Futura-CondensedExtraBold"        => Some(FontKind::SansBold),
        "GillSans"                         => Some(FontKind::SansRegular),
        "GillSans-Bold"                    => Some(FontKind::SansBold),
        "GillSans-Italic"                  => Some(FontKind::SansItalic),
        "GillSans-BoldItalic"              => Some(FontKind::SansBoldItalic),
        "GillSans-Light"                   => Some(FontKind::SansRegular),
        "GillSans-LightItalic"             => Some(FontKind::SansItalic),
        "Optima-Regular"                   => Some(FontKind::SansRegular),
        "Optima-Bold"                      => Some(FontKind::SansBold),
        "Optima-Italic"                    => Some(FontKind::SansItalic),
        "Optima-BoldItalic"                => Some(FontKind::SansBoldItalic),
        "Optima-ExtraBlack"                => Some(FontKind::SansBold),
        "TimesNewRomanPSMT"                => Some(FontKind::SerifRegular),
        "TimesNewRomanPS-BoldMT"           => Some(FontKind::SerifBold),
        "TimesNewRomanPS-ItalicMT"         => Some(FontKind::SerifItalic),
        "TimesNewRomanPS-BoldItalicMT"     => Some(FontKind::SerifBoldItalic),
        "Georgia"                          => Some(FontKind::SerifRegular),
        "Georgia-Bold"                     => Some(FontKind::SerifBold),
        "Georgia-Italic"                   => Some(FontKind::SerifItalic),
        "Georgia-BoldItalic"               => Some(FontKind::SerifBoldItalic),
        "Palatino-Roman"                   => Some(FontKind::SerifRegular),
        "Palatino-Bold"                    => Some(FontKind::SerifBold),
        "Palatino-Italic"                  => Some(FontKind::SerifItalic),
        "Palatino-BoldItalic"              => Some(FontKind::SerifBoldItalic),
        "Baskerville"                      => Some(FontKind::SerifRegular),
        "Baskerville-Bold"                 => Some(FontKind::SerifBold),
        "Baskerville-Italic"               => Some(FontKind::SerifItalic),
        "Baskerville-BoldItalic"           => Some(FontKind::SerifBoldItalic),
        "Baskerville-SemiBold"             => Some(FontKind::SerifBold),
        "Baskerville-SemiBoldItalic"       => Some(FontKind::SerifBoldItalic),
        "Didot"                            => Some(FontKind::SerifRegular),
        "Didot-Bold"                       => Some(FontKind::SerifBold),
        "Didot-Italic"                     => Some(FontKind::SerifItalic),
        "Cochin"                           => Some(FontKind::SerifRegular),
        "Cochin-Bold"                      => Some(FontKind::SerifBold),
        "Cochin-Italic"                    => Some(FontKind::SerifItalic),
        "Cochin-BoldItalic"                => Some(FontKind::SerifBoldItalic),
        "AmericanTypewriter"               => Some(FontKind::MonoRegular),
        "AmericanTypewriter-Bold"          => Some(FontKind::MonoBold),
        "AmericanTypewriter-Condensed"     => Some(FontKind::MonoRegular),
        "AmericanTypewriter-CondensedBold" => Some(FontKind::MonoBold),
        "AmericanTypewriter-CondensedLight"=> Some(FontKind::MonoRegular),
        "AmericanTypewriter-Light"         => Some(FontKind::MonoRegular),
        "MarkerFelt-Thin"                  => Some(FontKind::SansRegular),
        "MarkerFelt-Wide"                  => Some(FontKind::SansBold),
        "ChalkboardSE-Regular"             => Some(FontKind::SansRegular),
        "ChalkboardSE-Bold"                => Some(FontKind::SansBold),
        "ChalkboardSE-Light"               => Some(FontKind::SansRegular),
        "Chalkduster"                      => Some(FontKind::SansRegular),
        "BradleyHandITCTT-Bold"            => Some(FontKind::SansBold),
        "EuphemiaUCAS"                     => Some(FontKind::SansRegular),
        "EuphemiaUCAS-Bold"                => Some(FontKind::SansBold),
        "EuphemiaUCAS-Italic"              => Some(FontKind::SansItalic),
        "DBLCDTempBlack"                   => Some(FontKind::MonoBold),
        "Thonburi"                         => Some(FontKind::SansRegular),
        "Thonburi-Bold"                    => Some(FontKind::SansBold),
        "soopafre.ttf"                     => Some(FontKind::SansRegular),

        _ => None,
    }
}

fn font_kind_to_name(kind: FontKind) -> &'static str {
    match kind {
        FontKind::MonoRegular => "CourierNewPSMT",
        FontKind::MonoBold => "CourierNewPS-BoldMT",
        FontKind::MonoBoldItalic => "CourierNewPS-BoldItalicMT",
        FontKind::MonoItalic => "CourierNewPS-ItalicMT",
        FontKind::SansRegular => "Helvetica",
        FontKind::SansBold => "Helvetica-Bold",
        FontKind::SansBoldItalic => "Helvetica-BoldOblique",
        FontKind::SansItalic => "Helvetica-Oblique",
        FontKind::SerifRegular => "TimesNewRomanPSMT",
        FontKind::SerifBold => "TimesNewRomanPS-BoldMT",
        FontKind::SerifBoldItalic => "TimesNewRomanPS-BoldItalicMT",
        FontKind::SerifItalic => "TimesNewRomanPS-ItalicMT",
    }
}

fn font_kind_to_family(kind: FontKind) -> &'static str {
    match kind {
        FontKind::MonoRegular
        | FontKind::MonoBold
        | FontKind::MonoBoldItalic
        | FontKind::MonoItalic => "Courier New",
        FontKind::SansRegular
        | FontKind::SansBold
        | FontKind::SansBoldItalic
        | FontKind::SansItalic => "Helvetica",
        FontKind::SerifRegular
        | FontKind::SerifBold
        | FontKind::SerifBoldItalic
        | FontKind::SerifItalic => "Times New Roman",
    }
}
