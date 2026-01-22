use std::cell::UnsafeCell;

use rustc_hash::FxHashMap;
use skia_safe::{
    ClipOp, Data, FontArguments, FontMgr, FourByteTag, ISize, ImageInfo, MaskFilter, Paint, PathDirection, RRect,
    SamplingOptions, Typeface,
    canvas::SaveLayerRec,
    color_filters,
    font_arguments::{VariationPosition, variation_position::Coordinate},
    gradient_shader::{GradientShaderColors, linear},
    image_filters::{self, CropRect},
    path::ArcSize,
    surfaces,
};

use crate::{Color, IntPoint, Point, Rect, point, renderers::ImageType, rgb};

use super::{BorderRadius, CanvasType, Cap, Fill, FontVariable, Join, RasterSurfaceType};

#[derive(Clone)]
pub struct Font {
    font: skia_safe::Font,
}

impl super::FontType for Font {
    fn get_cap_height(&self) -> f32 {
        self.font.metrics().1.cap_height
    }

    fn get_string_width(&self, str: &str) -> f32 {
        let (w, _) = self.font.measure_str(str, None);
        w
    }

    fn get_glyph_widths(&self, str: &str) -> Vec<f32> {
        let glyphs = self.font.str_to_glyphs_vec(str);
        let mut widths: Vec<skia_safe::scalar> = glyphs.iter().map(|_| 0.0).collect();
        self.font.get_widths(&glyphs, &mut widths);
        widths
    }
}

pub struct TextBlob {
    blob: skia_safe::TextBlob,
}

impl super::TextBlobType<Renderer> for TextBlob {
    fn new(text: String, font: &Font) -> Option<Self> {
        skia_safe::TextBlob::new(text.clone(), &font.font).map(|blob| Self { blob })
    }
}

#[derive(Default)]
pub struct Renderer {
    typefaces: FxHashMap<usize, Typeface>,
}

impl super::RendererType for Renderer {
    type Font = Font;
    type TextBlob = TextBlob;
    type Image = Image;
    type Path = Path;
    type Canvas<'a> = Canvas<'a>;
    type Surface = Surface;

    fn add_typeface(&mut self, id: impl Into<usize>, data: &[u8]) {
        let mgr = FontMgr::default();
        self.typefaces.insert(id.into(), mgr.new_from_data(&Data::new_copy(data), None).unwrap());
    }

    fn create_font(&self, id: impl Into<usize>, font_size: f32, variables: Vec<FontVariable>) -> Font {
        let mut coordinates: Vec<Coordinate> = vec![];
        for variable in variables {
            let mut chars = variable.axis.chars();
            coordinates.push(Coordinate {
                axis: FourByteTag::from_chars(
                    chars.next().unwrap(),
                    chars.next().unwrap(),
                    chars.next().unwrap(),
                    chars.next().unwrap(),
                ),
                value: variable.value,
            });
        }
        let font_args =
            FontArguments::default().set_variation_design_position(VariationPosition { coordinates: &coordinates });
        let typeface = &self.typefaces[&id.into()].clone_with_arguments(&font_args).unwrap();
        let mut font = skia_safe::Font::from_typeface(typeface, font_size);
        font.set_subpixel(true);
        font.set_edging(skia_safe::font::Edging::SubpixelAntiAlias);
        font.set_hinting(skia_safe::FontHinting::Full);
        Font { font }
    }
}

impl From<Cap> for skia_safe::PaintCap {
    fn from(value: Cap) -> Self {
        match value {
            Cap::Butt => skia_safe::PaintCap::Butt,
            Cap::Round => skia_safe::PaintCap::Round,
            Cap::Square => skia_safe::PaintCap::Square,
        }
    }
}

pub struct Image {
    image: skia_safe::Image,
}

impl super::ImageType for Image {
    fn from_data(data: &[u8], width: i32, height: i32) -> Option<Image> {
        skia_safe::images::raster_from_data(
            &ImageInfo::new(
                ISize::new(width, height),
                skia_safe::ColorType::RGBA8888,
                skia_safe::AlphaType::Unpremul,
                None,
            ),
            Data::new_copy(data),
            (width * 4) as usize,
        )
        .map(|image| Image { image })
    }

    fn get_bounds(&self) -> Rect {
        let b = self.image.bounds();
        Rect { a: point(b.left as f32, b.top as f32), b: point(b.right as f32, b.bottom as f32) }
    }

    fn from_encoded(data: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        skia_safe::images::deferred_from_encoded_data(Data::new_copy(data), None).map(|image| Image { image })
    }
}

pub struct Path {
    path: skia_safe::Path,
}

impl Default for Path {
    fn default() -> Self {
        let mut path = skia_safe::Path::default();
        path.set_fill_type(skia_safe::PathFillType::Winding);
        Self { path }
    }
}

impl super::PathType for Path {
    fn move_to(&mut self, point: Point) -> &mut Self {
        self.path.move_to(point);
        self
    }

    fn line_to(&mut self, point: Point) -> &mut Self {
        self.path.line_to(point);
        self
    }

    fn conic_to(&mut self, p1: Point, p2: Point, weight: f32) -> &mut Self {
        self.path.conic_to(p1, p2, weight);
        self
    }

    fn quad_to(&mut self, p1: Point, p2: Point) -> &mut Self {
        self.path.quad_to(p1, p2);
        self
    }

    fn arc_to_rotated(&mut self, r: Point, x_axis_rotate: f32, large_arc: bool, sweep: bool, end: Point) -> &mut Self {
        self.path.arc_to_rotated(
            r,
            x_axis_rotate,
            if large_arc { ArcSize::Large } else { ArcSize::Small },
            if sweep { PathDirection::CW } else { PathDirection::CCW },
            end,
        );
        self
    }

    fn add_circle(&mut self, point: Point, radius: f32) -> &mut Self {
        self.path.add_circle(point, radius, None);
        self
    }

    fn add_rounded_rectangle(&mut self, rect: Rect, rounding: Point) -> &mut Self {
        self.path.add_round_rect(rect_to_rect(rect), (rounding.x, rounding.y), None);
        self
    }

    fn close(&mut self) {
        self.path.close();
    }

    fn cubic_to(&mut self, cp1: Point, cp2: Point, point: Point) -> &mut Self {
        self.path.cubic_to(cp1, cp2, point);
        self
    }

    fn with_offset(&self, value: Point) -> Self {
        Path { path: self.path.with_offset(value) }
    }

    fn with_scale(&mut self, value: Point) -> Self {
        Path { path: self.path.make_scale((value.x, value.y)) }
    }

    fn reset(&mut self) {
        self.path.reset();
    }

    fn fill_type_even_odd(&mut self) {
        self.path.set_fill_type(skia_safe::PathFillType::EvenOdd);
    }
}

pub struct Surface {
    surface: UnsafeCell<skia_safe::Surface>,
    size: IntPoint,
    scaled_size: IntPoint,
}

impl RasterSurfaceType<Renderer> for Surface {
    fn new(size: IntPoint, scale_factor: f32) -> Self {
        let scaled_size = size.with_scale(scale_factor);
        Self {
            size,
            scaled_size,
            surface: surfaces::raster(
                &ImageInfo::new(
                    ISize::new(scaled_size.x, scaled_size.y),
                    skia_safe::ColorType::RGBA8888,
                    skia_safe::AlphaType::Unpremul,
                    None,
                ),
                None,
                None,
            )
            .unwrap()
            .into(),
        }
    }

    fn get_canvas(&self) -> Canvas<'_> {
        Canvas::new(unsafe { (*self.surface.get()).canvas() })
    }

    fn draw(&self, func: impl FnOnce(Canvas, Rect)) {
        (func)(self.get_canvas(), Rect::from_xywh(0.0, 0.0, self.scaled_size.x as f32, self.scaled_size.y as f32))
    }

    fn get_size(&self) -> IntPoint {
        self.size
    }
}

pub struct Canvas<'a> {
    canvas: &'a skia_safe::Canvas,
    paint: skia_safe::Paint,
    alpha_mult: f32,
    scale_factor: f32,
}

impl<'a> Canvas<'a> {
    pub fn new(canvas: &'a skia_safe::Canvas) -> Self {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        Self { canvas, paint, alpha_mult: 1.0, scale_factor: 1.0 }
    }
}

impl super::CanvasType<Renderer> for Canvas<'_> {
    fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
        self.scale(1.0);
    }

    fn scale(&mut self, mut factor: f32) -> &mut Self {
        factor *= self.scale_factor;
        self.canvas.reset_matrix();
        self.canvas.scale((factor, factor));
        self
    }

    fn with_tint(&mut self, color: Color, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.color(color);
        self.paint.set_color_filter(color_filters::blend(color, skia_safe::BlendMode::SrcIn));
        (cb)(self);
        self.paint.set_color_filter(None);
        self
    }

    fn draw_path(&mut self, path: &Path) -> &mut Self {
        self.canvas.draw_path(&path.path, &self.paint);
        self
    }

    fn draw_path_at(&mut self, path: &Path, bounds: Rect) -> &mut Self {
        self.canvas.save();
        self.canvas.translate((bounds.a.x, bounds.a.y));
        self.draw_path(path);
        self.canvas.restore();
        self
    }

    fn draw_image(&mut self, image: &Image, bounds: Rect) -> &mut Self {
        self.color(rgb(0x000000));
        self.canvas.draw_image_nine(
            &image.image,
            rect_to_irect(image.get_bounds()),
            rect_to_rect(bounds),
            skia_safe::FilterMode::Linear,
            Some(&self.paint),
        );
        self
    }

    fn with_blur(&mut self, amount: f32, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.paint.set_mask_filter(MaskFilter::blur(skia_safe::BlurStyle::Normal, amount, None));
        (cb)(self);
        self.paint.set_mask_filter(None);
        self
    }

    fn with_alpha(&mut self, alpha: f32, cb: impl FnOnce(&mut Self)) -> &mut Self {
        let prev_alpha = self.alpha_mult;
        self.alpha_mult *= alpha;
        (cb)(self);
        self.alpha_mult = prev_alpha;
        self
    }

    fn with_clip_path(&mut self, path: &Path, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.canvas.save();
        self.canvas.clip_path(&path.path, ClipOp::Intersect, true);
        (cb)(self);
        self.canvas.restore();
        self
    }

    fn with_clip_rect(&mut self, clip_rect: Rect, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.canvas.save();
        self.canvas.clip_rect(rect_to_rect(clip_rect), ClipOp::Intersect, true);
        (cb)(self);
        self.canvas.restore();
        self
    }

    fn fill(&mut self, fill: impl Into<Fill>) -> &mut Self {
        self.apply_fill(fill);
        self.paint.set_style(skia_safe::PaintStyle::Fill);
        self
    }

    fn stroke(&mut self, fill: impl Into<Fill>, width: f32) -> &mut Self {
        self.apply_fill(fill);
        self.paint.set_style(skia_safe::PaintStyle::Stroke);
        self.paint.set_stroke_width(width);
        self
    }

    fn clear(&mut self, color: Color) -> &mut Self {
        self.canvas.clear(color);
        self
    }

    fn color(&mut self, color: Color) -> &mut Self {
        self.paint.set_shader(None);
        self.paint.set_color(color.with_alpha_mul(self.alpha_mult));
        self
    }

    fn draw_arc(&mut self, bounds: Rect, start_angle: f32, sweep_angle: f32) -> &mut Self {
        let bounds = bounds.with_expansion(-self.paint.stroke_width() * 0.5);
        self.canvas.draw_arc(rect_to_rect(bounds), start_angle, sweep_angle, false, &self.paint);
        self
    }

    fn draw_rect(&mut self, rect: Rect) -> &mut Self {
        self.canvas.draw_rect(rect_to_rect(rect), &self.paint);
        self
    }

    fn draw_round_rect(&mut self, rect: Rect, radius: impl Into<BorderRadius>) -> &mut Self {
        let radius: BorderRadius = radius.into();
        self.canvas.draw_rrect(
            RRect::new_nine_patch(rect_to_rect(rect), radius.left, radius.top, radius.right, radius.bottom),
            &self.paint,
        );
        self
    }

    fn draw_circle(&mut self, center: impl Into<Point>, radius: f32) -> &mut Self {
        self.canvas.draw_circle(center.into(), radius, &self.paint);
        self
    }

    fn stroke_cap(&mut self, cap: Cap) -> &mut Self {
        self.paint.set_stroke_cap(cap.into());
        self
    }

    fn stroke_join(&mut self, join: Join) -> &mut Self {
        self.paint.set_stroke_join(match join {
            Join::Miter => skia_safe::PaintJoin::Miter,
            Join::Round => skia_safe::PaintJoin::Round,
            Join::Bevel => skia_safe::PaintJoin::Bevel,
        });
        self
    }

    fn draw_text(&mut self, blob: &TextBlob, position: Point) -> &mut Self {
        self.canvas.draw_text_blob(&blob.blob, position, &self.paint);
        self
    }

    fn draw_surface(&mut self, surface: &Surface, position: Point) -> &mut Self {
        self.canvas.save();
        self.canvas.scale((1.0 / self.scale_factor, 1.0 / self.scale_factor));
        unsafe {
            (*surface.surface.get()).draw(
                self.canvas,
                position * self.scale_factor,
                SamplingOptions::new(skia_safe::FilterMode::Linear, skia_safe::MipmapMode::None),
                None,
            );
        }
        self.canvas.restore();
        self
    }

    fn write_pixels(&mut self, size: IntPoint, offset: IntPoint, pixels: &[u8]) -> &mut Self {
        let _ = self.canvas.write_pixels(
            &ImageInfo::new(
                ISize::new(size.x, size.y),
                skia_safe::ColorType::RGBA8888,
                skia_safe::AlphaType::Unpremul,
                None,
            ),
            pixels,
            size.x as usize * 4,
            (offset.x, offset.y),
        );
        self
    }

    fn with_rotation(&mut self, degrees: f32, point: impl Into<Point>, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.canvas.save();
        let p: Point = point.into();
        self.canvas.rotate(degrees, Some(p.into()));
        (cb)(self);
        self.canvas.restore();
        self
    }

    fn with_scale(&mut self, scale: f32, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.canvas.save();
        self.scale(scale);
        (cb)(self);
        self.canvas.restore();
        self
    }

    fn with_blend_mode(&mut self, blend_mode: super::BlendMode, cb: impl FnOnce(&mut Self)) -> &mut Self {
        let prev_blend = self.paint.blend_mode_or(skia_safe::BlendMode::Src);
        self.paint.set_blend_mode(match blend_mode {
            super::BlendMode::Clear => skia_safe::BlendMode::Clear,
            super::BlendMode::Src => skia_safe::BlendMode::Src,
            super::BlendMode::Dst => skia_safe::BlendMode::Dst,
            super::BlendMode::SrcOver => skia_safe::BlendMode::SrcOver,
            super::BlendMode::DstOver => skia_safe::BlendMode::DstOver,
            super::BlendMode::SrcIn => skia_safe::BlendMode::SrcIn,
            super::BlendMode::DstIn => skia_safe::BlendMode::DstIn,
            super::BlendMode::SrcOut => skia_safe::BlendMode::SrcOut,
            super::BlendMode::DstOut => skia_safe::BlendMode::DstOut,
            super::BlendMode::SrcATop => skia_safe::BlendMode::SrcATop,
            super::BlendMode::DstATop => skia_safe::BlendMode::DstATop,
            super::BlendMode::Xor => skia_safe::BlendMode::Xor,
            super::BlendMode::Plus => skia_safe::BlendMode::Plus,
            super::BlendMode::Modulate => skia_safe::BlendMode::Modulate,
            super::BlendMode::Screen => skia_safe::BlendMode::Screen,
            super::BlendMode::Overlay => skia_safe::BlendMode::Overlay,
            super::BlendMode::Darken => skia_safe::BlendMode::Darken,
            super::BlendMode::Lighten => skia_safe::BlendMode::Lighten,
            super::BlendMode::ColorDodge => skia_safe::BlendMode::ColorDodge,
            super::BlendMode::ColorBurn => skia_safe::BlendMode::ColorBurn,
            super::BlendMode::HardLight => skia_safe::BlendMode::HardLight,
            super::BlendMode::SoftLight => skia_safe::BlendMode::SoftLight,
            super::BlendMode::Difference => skia_safe::BlendMode::Difference,
            super::BlendMode::Exclusion => skia_safe::BlendMode::Exclusion,
            super::BlendMode::Multiply => skia_safe::BlendMode::Multiply,
            super::BlendMode::Hue => skia_safe::BlendMode::Hue,
            super::BlendMode::Saturation => skia_safe::BlendMode::Saturation,
            super::BlendMode::Color => skia_safe::BlendMode::Color,
            super::BlendMode::Luminosity => skia_safe::BlendMode::Luminosity,
        });
        (cb)(self);
        self.paint.set_blend_mode(prev_blend);
        self
    }

    fn with_translation(&mut self, amount: impl Into<Point>, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.canvas.save();
        self.canvas.translate(amount.into());
        cb(self);
        self.canvas.restore();
        self
    }

    fn backdrop_filter(&mut self, bounds: Rect, amount: f32) -> &mut Self {
        self.canvas.save();
        self.canvas.save_layer(
            &SaveLayerRec::default().bounds(&rect_to_rect(bounds)).backdrop(
                &image_filters::blur(
                    (amount, amount),
                    Some(skia_safe::TileMode::Clamp),
                    None,
                    Some(CropRect::from(rect_to_rect(bounds))),
                )
                .unwrap(),
            ),
        );
        self.canvas.restore();
        self
    }

    fn save(&mut self) -> &mut Self {
        self.canvas.save();
        self
    }

    fn restore(&mut self) -> &mut Self {
        self.canvas.restore();
        self
    }

    fn translate(&mut self, point: impl Into<Point>) -> &mut Self {
        let p: Point = point.into();
        self.canvas.translate(p);
        self
    }

    fn scale_rel(&mut self, point: impl Into<Point>) -> &mut Self {
        let p: Point = point.into();
        self.canvas.scale((p.x, p.y));
        self
    }
}

impl Canvas<'_> {
    fn apply_fill(&mut self, fill: impl Into<Fill>) {
        let fill: Fill = fill.into();
        match fill {
            Fill::Color(color) => {
                self.color(color);
            }
            Fill::Gradient(gradient) => {
                self.color(rgb(0));
                self.paint.set_shader(linear(
                    gradient.points,
                    GradientShaderColors::Colors(&gradient.colors.map(|c| c.into())[..gradient.num_positions as usize]),
                    Some(&gradient.positions.map(|p| p)[..gradient.num_positions as usize]),
                    skia_safe::TileMode::Clamp,
                    None,
                    None,
                ));
            }
        }
    }
}

impl From<Point> for skia_safe::Point {
    fn from(val: Point) -> Self {
        skia_safe::Point { x: val.x, y: val.y }
    }
}

impl From<Point> for skia_safe::Size {
    fn from(val: Point) -> Self {
        skia_safe::Size { width: val.x, height: val.y }
    }
}

impl From<&skia_safe::Rect> for Rect {
    fn from(value: &skia_safe::Rect) -> Self {
        Rect { a: point(value.left, value.top), b: point(value.bottom, value.right) }
    }
}

impl From<Rect> for skia_safe::Rect {
    fn from(value: Rect) -> Self {
        skia_safe::Rect::new(value.left(), value.top(), value.right(), value.bottom())
    }
}

pub fn rect_to_rect(rect: Rect) -> skia_safe::Rect {
    skia_safe::Rect::new(rect.left(), rect.top(), rect.right(), rect.bottom())
}

pub fn rect_to_irect(rect: Rect) -> skia_safe::IRect {
    skia_safe::IRect::new(rect.left() as i32, rect.top() as i32, rect.right() as i32, rect.bottom() as i32)
}

impl From<crate::color::Color> for skia_safe::Color {
    fn from(val: crate::color::Color) -> Self {
        skia_safe::Color::from_argb(
            (val.alpha() * 255.0) as u8,
            (val.red() * 255.0) as u8,
            (val.green() * 255.0) as u8,
            (val.blue() * 255.0) as u8,
        )
    }
}

impl From<crate::color::Color> for skia_safe::Color4f {
    fn from(val: crate::color::Color) -> Self {
        skia_safe::Color4f::new(val.red(), val.green(), val.blue(), val.alpha())
    }
}
