use std::{hint::black_box, thread::sleep, time::Duration};

use crate::{point, rgb, Color, Point, Rect};

use super::{BorderRadius, CanvasType, Cap, Fill, FontVariable, Join, RasterSurfaceType};

#[derive(Clone)]
pub struct Font;

impl super::FontType for Font {
    fn get_cap_height(&self) -> f32 {
        12.0
    }

    fn get_string_width(&self, str: &str) -> f32 {
        (str.len() * 12) as f32
    }

    fn get_glyph_widths(&self, str: &str) -> Vec<f32> {
        (0..str.len()).map(|_| 12.0).collect()
    }
}

pub struct TextBlob;

impl super::TextBlobType<Backend> for TextBlob {
    fn new(_text: String, _font: &Font) -> Option<Self> {
        Some(Self)
    }
}

#[derive(Default)]
pub struct Backend {}

impl super::BackendType for Backend {
    type Font = Font;
    type TextBlob = TextBlob;
    type Image = Image;
    type Path = Path;
    type Canvas<'a> = Canvas;
    type Surface = Surface;

    fn add_typeface(&mut self, _id: impl Into<usize>, _data: &[u8]) {}

    fn create_font(&self, _id: impl Into<usize>, _font_size: f32, _variables: Vec<FontVariable>) -> Font {
        Font
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
    width: i32,
    height: i32,
}

impl super::ImageType for Image {
    fn from_data(_data: &[u8], width: i32, height: i32) -> Option<Image> {
        Some(Image { width, height })
    }

    fn get_bounds(&self) -> Rect {
        Rect::from_size(self.width as f32, self.height as f32)
    }
}

pub struct Path {
    path: skia_safe::Path,
}

impl Default for Path {
    fn default() -> Self {
        let mut path = skia_safe::Path::default();
        path.set_fill_type(skia_safe::PathFillType::EvenOdd);
        Self { path }
    }
}

#[allow(unused)]
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

    fn cubic_to(&mut self, cp1: Point, cp2: Point, point: Point) {
        self.path.cubic_to(cp1, cp2, point);
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
}

pub struct Surface {
    size: (usize, usize),
    scaled_size: (usize, usize),
}

impl RasterSurfaceType<Backend> for Surface {
    fn new(size: (usize, usize), scale_factor: f32) -> Self {
        let scaled_size = ((size.0 as f32 * scale_factor) as usize, (size.1 as f32 * scale_factor) as usize);
        Self { size, scaled_size }
    }

    fn get_canvas(&self) -> Canvas {
        Canvas::new()
    }

    fn draw(&self, func: impl FnOnce(Canvas, Rect)) {
        (func)(self.get_canvas(), Rect::from_xywh(0.0, 0.0, self.scaled_size.0 as f32, self.scaled_size.1 as f32))
    }

    fn get_size(&self) -> (usize, usize) {
        self.size
    }
}

pub struct Canvas {
    alpha_mult: f32,
    scale_factor: f32,
}

impl Canvas {
    pub fn new() -> Self {
        Self { alpha_mult: 1.0, scale_factor: 1.0 }
    }

    pub fn payload(&self) {
        sleep(Duration::from_micros(1));
    }
}

#[allow(unused)]
impl super::CanvasType<Backend> for Canvas {
    fn set_scale_factor(&mut self, scale_factor: f32) {
        black_box(Self::payload(&self));
        self.scale_factor = scale_factor;
        self.scale(1.0);
    }

    fn scale(&mut self, mut _factor: f32) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn with_tint(&mut self, _color: Color, _cb: impl FnOnce(&mut Self)) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn draw_path(&mut self, _path: &Path) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn draw_path_at(&mut self, _path: &Path, _bounds: Rect) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn draw_image(&mut self, _image: &Image, _bounds: Rect) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn with_blur(&mut self, _amount: f32, _cb: impl FnOnce(&mut Self)) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn with_alpha(&mut self, _alpha: f32, _cb: impl FnOnce(&mut Self)) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn with_clip_path(&mut self, _path: Path, _cb: impl FnOnce(&mut Self)) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn with_clip_rect(&mut self, _clip_rect: Rect, _cb: impl FnOnce(&mut Self)) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn fill(&mut self, fill: impl Into<Fill>) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn stroke(&mut self, fill: impl Into<Fill>, width: f32) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn clear(&mut self, color: Color) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn color(&mut self, color: Color) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn draw_arc(&mut self, bounds: Rect, start_angle: f32, sweep_angle: f32) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn draw_rect(&mut self, rect: Rect) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn draw_round_rect(&mut self, rect: Rect, radius: impl Into<BorderRadius>) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn draw_circle(&mut self, center: impl Into<Point>, radius: f32) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn stroke_cap(&mut self, cap: Cap) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn stroke_join(&mut self, join: Join) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn draw_text(&mut self, blob: &TextBlob, position: Point) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn draw_surface(&mut self, surface: &Surface, position: Point) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }

    fn write_pixels(&mut self, width: usize, height: usize, pixels: &[u8]) -> &mut Self {
        black_box(Self::payload(&self));
        self
    }
}

#[allow(unused)]
impl Canvas {
    fn apply_fill(&mut self, fill: impl Into<Fill>) {
        let fill: Fill = fill.into();
        match fill {
            Fill::Color(color) => {
                self.color(color);
            }
            Fill::Gradient(gradient) => {
                self.color(rgb(0));
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
