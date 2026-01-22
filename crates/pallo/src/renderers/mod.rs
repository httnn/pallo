use crate::{Color, Gradient, IntPoint, Point, Rect};

#[cfg_attr(any(target_os = "macos", target_os = "windows", target_os = "ios"), path = "skia.rs")]
#[cfg_attr(target_family = "wasm", path = "canvaskit.rs")]
pub mod renderer;

pub use renderer::*;

pub struct BorderRadius {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl From<f32> for BorderRadius {
    fn from(val: f32) -> Self {
        BorderRadius { left: val, top: val, right: val, bottom: val }
    }
}

impl From<Point> for BorderRadius {
    fn from(val: Point) -> Self {
        BorderRadius { left: val.x, top: val.y, right: val.x, bottom: val.y }
    }
}

pub enum Fill {
    Color(Color),
    Gradient(Gradient),
}

#[allow(unused)]
#[derive(Clone)]
pub struct FontVariable {
    axis: &'static str,
    value: f32,
}

impl FontVariable {
    pub fn new(axis: &'static str, value: f32) -> Self {
        Self { axis, value }
    }
}

impl FontVariable {
    pub fn get_axis(&self) -> String {
        self.axis.into()
    }
}

pub enum Cap {
    Butt,
    Round,
    Square,
}

pub enum Join {
    Miter,
    Round,
    Bevel,
}

pub trait FontType {
    fn get_cap_height(&self) -> f32;
    fn get_string_width(&self, str: &str) -> f32;
    fn get_glyph_widths(&self, str: &str) -> Vec<f32>;
}

pub trait TextBlobType<B: RendererType> {
    fn new(text: String, font: &B::Font) -> Option<Self>
    where
        Self: Sized;
}

pub trait ImageType {
    fn from_encoded(data: &[u8]) -> Option<Self>
    where
        Self: Sized;
    fn from_data(data: &[u8], width: i32, height: i32) -> Option<Self>
    where
        Self: Sized;
    fn get_bounds(&self) -> Rect;
}

pub trait PathType {
    fn move_to(&mut self, point: Point) -> &mut Self;
    fn line_to(&mut self, point: Point) -> &mut Self;
    fn conic_to(&mut self, p1: Point, p2: Point, weight: f32) -> &mut Self;
    fn quad_to(&mut self, p1: Point, p2: Point) -> &mut Self;
    fn arc_to_rotated(&mut self, r: Point, x_axis_rotate: f32, large_arc: bool, sweep: bool, end: Point) -> &mut Self;
    fn add_circle(&mut self, point: Point, radius: f32) -> &mut Self;
    fn add_rounded_rectangle(&mut self, rect: Rect, rounding: Point) -> &mut Self;
    fn close(&mut self);
    fn cubic_to(&mut self, cp1: Point, cp2: Point, point: Point) -> &mut Self;
    fn with_offset(&self, value: Point) -> Self;
    fn with_scale(&mut self, value: Point) -> Self;
    fn fill_type_even_odd(&mut self);
    fn reset(&mut self);
}

pub enum BlendMode {
    Clear,
    Src,
    Dst,
    SrcOver,
    DstOver,
    SrcIn,
    DstIn,
    SrcOut,
    DstOut,
    SrcATop,
    DstATop,
    Xor,
    Plus,
    Modulate,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Multiply,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

pub trait CanvasType<B: RendererType> {
    fn set_scale_factor(&mut self, scale_factor: f32);
    fn scale(&mut self, factor: f32) -> &mut Self;
    fn with_tint(&mut self, color: Color, cb: impl FnOnce(&mut Self)) -> &mut Self;
    fn draw_path(&mut self, path: &B::Path) -> &mut Self;
    fn draw_path_at(&mut self, path: &B::Path, bounds: Rect) -> &mut Self;
    fn draw_image(&mut self, image: &B::Image, bounds: Rect) -> &mut Self;
    fn with_scale(&mut self, scale: f32, cb: impl FnOnce(&mut Self)) -> &mut Self;
    fn with_blur(&mut self, amount: f32, cb: impl FnOnce(&mut Self)) -> &mut Self;
    fn with_alpha(&mut self, alpha: f32, cb: impl FnOnce(&mut Self)) -> &mut Self;
    fn with_clip_path(&mut self, path: &B::Path, cb: impl FnOnce(&mut Self)) -> &mut Self;
    fn with_clip_rect(&mut self, clip_rect: Rect, cb: impl FnOnce(&mut Self)) -> &mut Self;
    fn with_rotation(&mut self, degrees: f32, point: impl Into<Point>, cb: impl FnOnce(&mut Self)) -> &mut Self;
    fn with_translation(&mut self, amount: impl Into<Point>, cb: impl FnOnce(&mut Self)) -> &mut Self;
    fn with_blend_mode(&mut self, blend_mode: BlendMode, cb: impl FnOnce(&mut Self)) -> &mut Self;
    fn fill(&mut self, fill: impl Into<Fill>) -> &mut Self;
    fn stroke(&mut self, fill: impl Into<Fill>, width: f32) -> &mut Self;
    fn clear(&mut self, color: Color) -> &mut Self;
    fn color(&mut self, color: Color) -> &mut Self;
    fn draw_arc(&mut self, bounds: Rect, start_angle: f32, sweep_angle: f32) -> &mut Self;
    fn draw_rect(&mut self, rect: Rect) -> &mut Self;
    fn draw_round_rect(&mut self, rect: Rect, radius: impl Into<BorderRadius>) -> &mut Self;
    fn draw_circle(&mut self, center: impl Into<Point>, radius: f32) -> &mut Self;
    fn stroke_cap(&mut self, cap: Cap) -> &mut Self;
    fn stroke_join(&mut self, join: Join) -> &mut Self;
    fn draw_text(&mut self, blob: &B::TextBlob, position: Point) -> &mut Self;
    fn draw_surface(&mut self, surface: &B::Surface, position: Point) -> &mut Self;
    fn write_pixels(&mut self, size: IntPoint, offset: IntPoint, pixels: &[u8]) -> &mut Self;
    fn backdrop_filter(&mut self, bounds: Rect, amount: f32) -> &mut Self;
    fn save(&mut self) -> &mut Self;
    fn restore(&mut self) -> &mut Self;
    fn translate(&mut self, point: impl Into<Point>) -> &mut Self;
    fn scale_rel(&mut self, point: impl Into<Point>) -> &mut Self;
}

pub trait RasterSurfaceType<B: RendererType> {
    fn new(size: IntPoint, scale_factor: f32) -> Self;
    fn get_canvas(&self) -> B::Canvas<'_>;
    fn draw(&self, func: impl FnOnce(Canvas, Rect));
    fn get_size(&self) -> IntPoint;
}

pub trait RendererType: Sized {
    type Font: FontType + Clone;
    type TextBlob: TextBlobType<Self>;
    type Image: ImageType;
    type Path: PathType;
    type Canvas<'a>: CanvasType<Self>;
    type Surface: RasterSurfaceType<Self>;
    fn add_typeface(&mut self, id: impl Into<usize>, data: &[u8]);
    fn create_font(&self, id: impl Into<usize>, font_size: f32, variables: Vec<FontVariable>) -> Self::Font;
}
