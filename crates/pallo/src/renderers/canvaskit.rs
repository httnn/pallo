use crate::{BorderRadius, Color, Fill, IntPoint, Join, Point, RasterSurfaceType, Rect, rgba};
use js_sys::{Array, Float32Array, Object, Reflect, Uint16Array};
use rustc_hash::FxHashMap;
use wasm_bindgen::prelude::*;

use super::{Cap, FontVariable, ImageType};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["CanvasKit", "TextBlob"])]
    fn MakeFromText(text: String, font: &JsFont) -> JsTextBlob;

    #[wasm_bindgen(js_namespace = CanvasKit)]
    fn LTRBRect(left: f32, top: f32, right: f32, bottom: f32) -> JsRect;

    #[wasm_bindgen(js_name = Surface, js_namespace = CanvasKit)]
    type JsSurface;

    #[wasm_bindgen(js_namespace = CanvasKit)]
    fn MakeSurface(width: usize, height: usize) -> JsSurface;

    #[wasm_bindgen(method, js_class = JsSurface, js_namespace = CanvasKit)]
    fn getCanvas(this: &JsSurface) -> JsCanvas;

    #[wasm_bindgen(method, js_class = JsSurface, js_namespace = CanvasKit)]
    fn makeImageSnapshot(this: &JsSurface) -> JsImage;

    #[wasm_bindgen(js_name = Shader, js_namespace = CanvasKit)]
    type JsShader;

    #[wasm_bindgen(method, js_class = Shader, js_namespace = CanvasKit)]
    fn delete(this: &JsShader);

    #[wasm_bindgen(js_namespace = ["CanvasKit", "Shader"])]
    fn MakeLinearGradient(
        start: Array,
        end: Array,
        colors: Vec<Float32Array>,
        positions: Vec<f32>,
        tile_mode: &JsValue,
    ) -> JsShader;

    #[wasm_bindgen(js_name = Typeface, js_namespace = CanvasKit)]
    type JsTypeface;

    #[wasm_bindgen(method, js_class = Typeface, js_namespace = CanvasKit)]
    fn delete(this: &JsTypeface);

    #[wasm_bindgen(js_namespace = ["CanvasKit", "Typeface"])]
    fn MakeTypefaceFromData(text: &[u8]) -> JsTypeface;

    #[wasm_bindgen(js_name = Font, js_namespace = CanvasKit)]
    type JsFont;

    #[wasm_bindgen(constructor, js_class = Font, js_namespace = CanvasKit)]
    fn new() -> JsFont;

    #[wasm_bindgen(method, js_class = Font, js_namespace = CanvasKit)]
    fn clone(this: &JsFont) -> JsFont;

    #[wasm_bindgen(method, js_class = Font, js_namespace = CanvasKit)]
    fn setTypeface(this: &JsFont, typeface: &JsTypeface);

    #[wasm_bindgen(method, js_class = Font, js_namespace = CanvasKit)]
    fn setSubpixel(this: &JsFont, value: bool);

    #[wasm_bindgen(method, js_class = Font, js_namespace = CanvasKit)]
    fn setEdging(this: &JsFont, value: &JsValue);

    #[wasm_bindgen(method, js_class = Font, js_namespace = CanvasKit)]
    fn setHinting(this: &JsFont, value: &JsValue);

    #[wasm_bindgen(method, js_class = Font, js_namespace = CanvasKit)]
    fn setSize(this: &JsFont, size: f32);

    #[wasm_bindgen(method, js_class = Font, js_namespace = CanvasKit)]
    fn getSize(this: &JsFont) -> f32;

    #[wasm_bindgen(method, js_class = Font, js_namespace = CanvasKit)]
    fn getGlyphIDs(this: &JsFont, text: String) -> Uint16Array;

    #[wasm_bindgen(method, js_class = Font, js_namespace = CanvasKit)]
    fn getGlyphWidths(this: &JsFont, glyph_ids: &Uint16Array) -> Float32Array;

    #[wasm_bindgen(method, js_class = Font, js_namespace = CanvasKit)]
    fn delete(this: &JsFont);

    #[wasm_bindgen(js_name = TextBlob, js_namespace = CanvasKit)]
    type JsTextBlob;

    #[wasm_bindgen(constructor, js_class = TextBlob)]
    fn new(text: String, font: &JsFont) -> JsTextBlob;

    #[wasm_bindgen(method, js_class = TextBlob, js_namespace = CanvasKit)]
    fn delete(this: &JsTextBlob);

    #[wasm_bindgen(js_name = Image, js_namespace = CanvasKit)]
    type JsImage;

    #[wasm_bindgen(method, js_class = Image, js_namespace = CanvasKit)]
    fn width(this: &JsImage) -> f32;

    #[wasm_bindgen(method, js_class = Image, js_namespace = CanvasKit)]
    fn height(this: &JsImage) -> f32;

    #[wasm_bindgen(js_namespace = CanvasKit)]
    fn MakeImage(info: &JsValue, data: &[u8], bytes_per_row: usize) -> JsImage;

    #[wasm_bindgen(js_namespace = CanvasKit)]
    fn MakeImageFromEncoded(data: &[u8]) -> JsImage;

    #[wasm_bindgen(method, js_class = Image, js_namespace = CanvasKit)]
    fn delete(this: &JsImage);

    #[wasm_bindgen(js_name = Path, js_namespace = CanvasKit)]
    type JsPath;

    #[wasm_bindgen(constructor, js_class = Path, js_namespace = CanvasKit)]
    fn new() -> JsPath;

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn setFillType(this: &JsPath, fill_type: &JsValue) -> JsPath;

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn copy(this: &JsPath) -> JsPath;

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn moveTo(this: &JsPath, x: f32, y: f32);

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn lineTo(this: &JsPath, x: f32, y: f32);

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn conicTo(this: &JsPath, x1: f32, y1: f32, x2: f32, y2: f32, weight: f32);

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn quadTo(this: &JsPath, x1: f32, y1: f32, x2: f32, y2: f32);

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn arcToRotated(
        this: &JsPath,
        rx: f32,
        ry: f32,
        x_axis_rotate: f32,
        large_arc: bool,
        sweep: bool,
        endx: f32,
        endy: f32,
    );

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn addCircle(this: &JsPath, rx: f32, ry: f32, radius: f32, is_ccw: bool);

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn addRRect(this: &JsPath, rrect: Vec<f32>);

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn close(this: &JsPath);

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn cubicTo(this: &JsPath, x1: f32, y1: f32, x2: f32, y2: f32, pointx: f32, pointy: f32);

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn offset(this: &JsPath, x: f32, y: f32) -> JsPath;

    #[wasm_bindgen(method)]
    fn transform(this: &JsPath, matrix: &JsMatrix) -> JsPath;

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn reset(this: &JsPath);

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn delete(this: &JsPath);

    #[wasm_bindgen(method, js_class = Path, js_namespace = CanvasKit)]
    fn isDeleted(this: &JsPath) -> bool;

    #[wasm_bindgen(js_name = Matrix, js_namespace = CanvasKit)]
    pub type JsMatrix;

    #[wasm_bindgen(js_namespace = ["CanvasKit", "Matrix"])]
    pub fn scaled(x: f32, y: f32) -> JsMatrix;

    #[wasm_bindgen(js_name = Paint, js_namespace = CanvasKit)]
    pub type JsPaint;

    #[wasm_bindgen(constructor, js_class = Paint, js_namespace = CanvasKit)]
    fn new() -> JsPaint;

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn setBlendMode(this: &JsPaint, mode: &JsValue);

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn getStrokeWidth(this: &JsPaint) -> f32;

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn setStrokeWidth(this: &JsPaint, width: f32);

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn setAntiAlias(this: &JsPaint, value: bool);

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn setStyle(this: &JsPaint, style: &JsValue);

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn setColor(this: &JsPaint, color: &Array);

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn setColorFilter(this: &JsPaint, filter: &JsValue);

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn setMaskFilter(this: &JsPaint, mask_filter: &JsValue);

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn setStrokeCap(this: &JsPaint, cap: &JsValue);

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn setStrokeJoin(this: &JsPaint, cap: &JsValue);

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn setShader(this: &JsPaint, shader: &JsValue);

    #[wasm_bindgen(method, js_class = Paint, js_namespace = CanvasKit)]
    fn delete(this: &JsPaint);

    pub type JsRect;

    #[wasm_bindgen(js_name = ColorFilter, js_namespace = CanvasKit)]
    pub type JsColorFilter;

    #[wasm_bindgen(js_namespace = ["CanvasKit", "ColorFilter"])]
    fn MakeBlend(color: Array, mode: &JsValue) -> JsColorFilter;

    #[wasm_bindgen(method, js_class = ColorFilter, js_namespace = CanvasKit)]
    fn delete(this: &JsColorFilter);

    #[wasm_bindgen(js_name = MaskFilter, js_namespace = CanvasKit)]
    pub type JsMaskFilter;

    #[wasm_bindgen(js_namespace = ["CanvasKit", "MaskFilter"])]
    fn MakeBlur(blur_style: &JsValue, amount: f32, respect_ctm: bool) -> JsMaskFilter;

    #[wasm_bindgen(method, js_class = MaskFilter, js_namespace = CanvasKit)]
    fn delete(this: &JsMaskFilter);

    #[wasm_bindgen(js_name = ImageFilter, js_namespace = CanvasKit)]
    pub type JsImageFilter;

    #[wasm_bindgen(js_namespace = ["CanvasKit", "ImageFilter"], js_name = "MakeBlur")]
    fn ImageFilterMakeBlur(sigma_x: f32, sigma_y: f32, mode: &JsValue, input: &JsValue) -> JsImageFilter;

    #[wasm_bindgen(method, js_class = ImageFilter, js_namespace = CanvasKit)]
    fn delete(this: &JsImageFilter);

    #[wasm_bindgen(js_name = Canvas, js_namespace = CanvasKit)]
    pub type JsCanvas;

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn scale(this: &JsCanvas, x: f32, y: f32);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn translate(this: &JsCanvas, x: f32, y: f32);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn rotate(this: &JsCanvas, degrees: f32, rx: f32, ry: f32);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn save(this: &JsCanvas);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn restore(this: &JsCanvas);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn drawPath(this: &JsCanvas, path: &JsPath, paint: &JsPaint);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn drawImageRectOptions(
        this: &JsCanvas,
        image: &JsImage,
        src: JsRect,
        dest: JsRect,
        filter_mode: &JsValue,
        mipmap_mode: &JsValue,
    );

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn clipPath(this: &JsCanvas, path: &JsPath, op: &JsValue, antialias: bool);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn clipRect(this: &JsCanvas, rect: &JsRect, op: &JsValue, antialias: bool);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn clear(this: &JsCanvas, color: &Array);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn drawArc(this: &JsCanvas, bounds: &JsRect, start_angle: f32, sweep_angle: f32, use_center: bool, paint: &JsPaint);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn drawRect(this: &JsCanvas, rect: &JsRect, paint: &JsPaint);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn drawRRect(this: &JsCanvas, rrect: Vec<f32>, paint: &JsPaint);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn drawCircle(this: &JsCanvas, center_x: f32, center_y: f32, radius: f32, paint: &JsPaint);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn drawTextBlob(this: &JsCanvas, blob: &JsTextBlob, position_x: f32, position_y: f32, paint: &JsPaint);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn drawImage(this: &JsCanvas, image: &JsImage, position_x: f32, position_y: f32, paint: &JsPaint);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn saveLayer(this: &JsCanvas, paint: &JsValue, rect: &JsRect, backdrop: &JsImageFilter, flags: i32);

    #[wasm_bindgen(method, js_class = Canvas, js_namespace = CanvasKit)]
    fn writePixels(
        this: &JsCanvas,
        pixels: &[u8],
        src_width: usize,
        src_height: usize,
        dest_x: f32,
        dest_y: f32,
        alpha_type: &JsValue,
        color_type: &JsValue,
        color_space: &JsValue,
    );

    #[wasm_bindgen(thread_local_v2, js_name = SrcIn, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_SRC_IN: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Intersect, js_namespace = ["CanvasKit", "ClipOp"])]
    static CLIP_OP_INTERSECT: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Linear, js_namespace = ["CanvasKit", "FilterMode"])]
    static FILTER_MODE_LINEAR: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = None, js_namespace = ["CanvasKit", "MipmapMode"])]
    static MIPMAP_MODE_NONE: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Fill, js_namespace = ["CanvasKit", "PaintStyle"])]
    static PAINTSTYLE_FILL: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Stroke, js_namespace = ["CanvasKit", "PaintStyle"])]
    static PAINTSTYLE_STROKE: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Normal, js_namespace = ["CanvasKit", "BlurStyle"])]
    static BLURSTYLE_NORMAL: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Round, js_namespace = ["CanvasKit", "StrokeJoin"])]
    static STROKE_JOIN_ROUND: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Miter, js_namespace = ["CanvasKit", "StrokeJoin"])]
    static STROKE_JOIN_MITER: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Bevel, js_namespace = ["CanvasKit", "StrokeJoin"])]
    static STROKE_JOIN_BEVEL: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Butt, js_namespace = ["CanvasKit", "StrokeCap"])]
    static STROKE_CAP_BUTT: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Round, js_namespace = ["CanvasKit", "StrokeCap"])]
    static STROKE_CAP_ROUND: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Square, js_namespace = ["CanvasKit", "StrokeCap"])]
    static STROKE_CAP_SQUARE: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Clamp, js_namespace = ["CanvasKit", "TileMode"])]
    static TILEMODE_CLAMP: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = SubpixelAntiAlias, js_namespace = ["CanvasKit", "FontEdging"])]
    static FONT_EDGING_SUBPIXEL_AA: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Full, js_namespace = ["CanvasKit", "FontHinting"])]
    static FONT_HINTING_FULL: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Unpremul, js_namespace = ["CanvasKit", "AlphaType"])]
    static ALPHA_TYPE_UNPREMUL: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = SRGB, js_namespace = ["CanvasKit", "ColorSpace"])]
    static COLOR_SPACE_SRGB: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = RGBA_8888, js_namespace = ["CanvasKit", "ColorType"])]
    static COLOR_TYPE_RGBA_8888: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = EvenOdd, js_namespace = ["CanvasKit", "FillType"])]
    static FILL_TYPE_EVEN_ODD: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Clear, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_CLEAR: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Color, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_COLOR: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = ColorBurn, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_COLORBURN: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = ColorDodge, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_COLORDODGE: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Darken, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_DARKEN: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Difference, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_DIFFERENCE: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Dst, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_DST: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = DstATop, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_DSTATOP: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = DstIn, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_DSTIN: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = DstOut, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_DSTOUT: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = DstOver, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_DSTOVER: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Exclusion, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_EXCLUSION: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = HardLight, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_HARDLIGHT: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Hue, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_HUE: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Lighten, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_LIGHTEN: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Luminosity, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_LUMINOSITY: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Modulate, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_MODULATE: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Multiply, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_MULTIPLY: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Overlay, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_OVERLAY: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Plus, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_PLUS: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Saturation, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_SATURATION: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Screen, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_SCREEN: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = SoftLight, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_SOFTLIGHT: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Src, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_SRC: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = SrcATop, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_SRCATOP: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = SrcIn, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_SRCIN: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = SrcOut, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_SRCOUT: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = SrcOver, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_SRCOVER: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = Xor, js_namespace = ["CanvasKit", "BlendMode"])]
    static BLEND_MODE_XOR: JsValue;

    #[wasm_bindgen(thread_local_v2, js_name = SaveLayerF16ColorType, js_namespace = ["CanvasKit"])]
    static SAVE_LAYER_16_COLOR_TYPE: i32;

    #[wasm_bindgen(thread_local_v2, js_name = SaveLayerInitWithPrevious, js_namespace = ["CanvasKit"])]
    static SAVE_LAYER_INIT_WITH_PREVIOUS: i32;
}

pub struct Surface {
    surface: JsSurface,
    size: IntPoint,
    scaled_size: IntPoint,
}

impl RasterSurfaceType<Renderer> for Surface {
    fn new(size: IntPoint, scale_factor: f32) -> Self {
        let scaled_size = size.with_scale(scale_factor);
        Self { surface: MakeSurface(scaled_size.x as usize, scaled_size.y as usize), size, scaled_size }
    }

    fn get_canvas<'a>(&'a self) -> Canvas {
        Canvas::new(self.surface.getCanvas())
    }

    fn draw(&self, func: impl FnOnce(Canvas, Rect)) {
        (func)(self.get_canvas(), Rect::from_xywh(0.0, 0.0, self.scaled_size.x as f32, self.scaled_size.y as f32))
    }

    fn get_size(&self) -> IntPoint {
        self.size
    }
}

pub struct Font {
    font: JsFont,
}

impl Drop for Font {
    fn drop(&mut self) {
        self.font.delete();
    }
}

impl Clone for Font {
    fn clone(&self) -> Self {
        Self { font: self.font.clone() }
    }
}

impl super::FontType for Font {
    fn get_cap_height(&self) -> f32 {
        self.font.getSize() * 0.7
    }

    fn get_string_width(&self, str: &str) -> f32 {
        self.get_glyph_widths(str).into_iter().sum()
    }

    fn get_glyph_widths(&self, str: &str) -> Vec<f32> {
        let glyph_ids = self.font.getGlyphIDs(str.to_string());
        let glyph_widths = self.font.getGlyphWidths(&glyph_ids);
        glyph_widths.to_vec()
    }
}

pub struct TextBlob {
    blob: JsTextBlob,
}

impl Drop for TextBlob {
    fn drop(&mut self) {
        if !self.blob.is_null() {
            self.blob.delete();
        }
    }
}

impl super::TextBlobType<Renderer> for TextBlob {
    fn new(text: String, font: &Font) -> Option<Self> {
        Some(Self { blob: MakeFromText(text, &font.font) })
    }
}

pub struct Renderer {
    typefaces: FxHashMap<usize, JsTypeface>,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        for (_, typeface) in &self.typefaces {
            typeface.delete();
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self { typefaces: Default::default() }
    }
}

impl super::RendererType for Renderer {
    type Font = Font;
    type TextBlob = TextBlob;
    type Image = Image;
    type Path = Path;
    type Canvas<'a> = Canvas;
    type Surface = Surface;

    fn add_typeface(&mut self, id: impl Into<usize>, data: &[u8]) {
        self.typefaces.insert(id.into(), MakeTypefaceFromData(data));
    }

    fn create_font(&self, id: impl Into<usize>, font_size: f32, _variables: Vec<FontVariable>) -> Font {
        // TODO: implement variable font
        let font = JsFont::new();
        font.setTypeface(&self.typefaces[&id.into()]);
        font.setSubpixel(true);
        font.setEdging(&FONT_EDGING_SUBPIXEL_AA.with(JsValue::clone));
        font.setHinting(&FONT_HINTING_FULL.with(JsValue::clone));
        font.setSize(font_size);
        Font { font }
    }
}

pub struct Image {
    image: JsImage,
}

impl Drop for Image {
    fn drop(&mut self) {
        self.image.delete();
    }
}

impl super::ImageType for Image {
    fn from_data(data: &[u8], width: i32, height: i32) -> Option<Image> {
        let image_info = Object::new();
        Reflect::set(&image_info, &"width".into(), &width.into()).unwrap();
        Reflect::set(&image_info, &"height".into(), &height.into()).unwrap();
        Reflect::set(&image_info, &"alphaType".into(), &ALPHA_TYPE_UNPREMUL.with(JsValue::clone)).unwrap();
        Reflect::set(&image_info, &"colorSpace".into(), &COLOR_SPACE_SRGB.with(JsValue::clone)).unwrap();
        Reflect::set(&image_info, &"colorType".into(), &COLOR_TYPE_RGBA_8888.with(JsValue::clone)).unwrap();
        Some(Self { image: MakeImage(&image_info.into(), data, (width * 4) as usize) })
    }

    fn from_encoded(data: &[u8]) -> Option<Image> {
        Some(Self { image: MakeImageFromEncoded(data) })
    }

    fn get_bounds(&self) -> Rect {
        Rect::from_xywh(0.0, 0.0, self.image.width(), self.image.height())
    }
}

fn make_rounded_rect(rect: Rect, radius: BorderRadius) -> Vec<f32> {
    vec![
        rect.a.x,
        rect.a.y,
        rect.b.x,
        rect.b.y,
        radius.left,
        radius.left,
        radius.top,
        radius.top,
        radius.right,
        radius.right,
        radius.bottom,
        radius.bottom,
    ]
}

pub struct Path {
    path: JsPath,
}

impl Drop for Path {
    fn drop(&mut self) {
        if !self.path.isDeleted() {
            self.path.delete();
        }
    }
}

impl Default for Path {
    fn default() -> Self {
        Self { path: JsPath::new() }
    }
}

impl super::PathType for Path {
    fn fill_type_even_odd(&mut self) {
        self.path.setFillType(&FILL_TYPE_EVEN_ODD.with(JsValue::clone));
    }

    fn move_to(&mut self, point: Point) -> &mut Self {
        self.path.moveTo(point.x, point.y);
        self
    }

    fn line_to(&mut self, point: Point) -> &mut Self {
        self.path.lineTo(point.x, point.y);
        self
    }

    fn conic_to(&mut self, p1: Point, p2: Point, weight: f32) -> &mut Self {
        self.path.conicTo(p1.x, p1.y, p2.x, p2.y, weight);
        self
    }

    fn quad_to(&mut self, p1: Point, p2: Point) -> &mut Self {
        self.path.quadTo(p1.x, p1.y, p2.x, p2.y);
        self
    }

    fn arc_to_rotated(&mut self, r: Point, x_axis_rotate: f32, large_arc: bool, sweep: bool, end: Point) -> &mut Self {
        self.path.arcToRotated(r.x, r.y, x_axis_rotate, !large_arc, !sweep, end.x, end.y);
        self
    }

    fn add_circle(&mut self, point: Point, radius: f32) -> &mut Self {
        self.path.addCircle(point.x, point.y, radius, false);
        self
    }

    fn add_rounded_rectangle(&mut self, rect: Rect, rounding: Point) -> &mut Self {
        self.path.addRRect(make_rounded_rect(rect, rounding.into()));
        self
    }

    fn close(&mut self) {
        self.path.close();
    }

    fn cubic_to(&mut self, cp1: Point, cp2: Point, point: Point) -> &mut Self {
        self.path.cubicTo(cp1.x, cp1.y, cp2.x, cp2.y, point.x, point.y);
        self
    }

    fn with_offset(&self, value: Point) -> Self {
        Self { path: self.path.copy().offset(value.x, value.y) }
    }

    fn with_scale(&mut self, value: Point) -> Self {
        let matrix = scaled(value.x, value.y);
        let path = self.path.transform(&matrix);
        Self { path }
    }

    fn reset(&mut self) {
        self.path.reset();
    }
}

fn to_skia_rect(rect: Rect) -> JsRect {
    LTRBRect(rect.a.x, rect.a.y, rect.b.x, rect.b.y)
}

fn to_skia_color(color: Color) -> Array {
    Array::of4(&color.red().into(), &color.green().into(), &color.blue().into(), &color.alpha().into())
}

fn to_skia_color_f32_array(color: Color) -> Float32Array {
    let arr = Float32Array::new_with_length(4);
    arr.set_index(0, color.red());
    arr.set_index(1, color.green());
    arr.set_index(2, color.blue());
    arr.set_index(3, color.alpha());
    arr
}

fn to_skia_point(point: Point) -> Array {
    Array::of2(&point.x.into(), &point.y.into())
}

#[wasm_bindgen]
pub struct Canvas {
    canvas: JsCanvas,
    paint: JsPaint,
    scale_factor: f32,
    alpha_mul: f32,
    prev_scale: f32,
    blend_mode: JsValue,
}

impl Drop for Canvas {
    fn drop(&mut self) {
        self.paint.delete();
    }
}

#[wasm_bindgen]
impl Canvas {
    pub fn new(canvas: JsCanvas) -> Self {
        let paint = JsPaint::new();
        paint.setAntiAlias(true);
        Self {
            canvas,
            paint,
            scale_factor: 1.0,
            alpha_mul: 1.0,
            prev_scale: 1.0,
            blend_mode: BLEND_MODE_SRCATOP.with(JsValue::clone),
        }
    }
}

impl super::CanvasType<Renderer> for Canvas {
    fn set_scale_factor(&mut self, scale_factor: f32) {
        self.canvas.restore();
        self.canvas.save();
        self.scale_factor = scale_factor;
        self.scale(1.0);
    }

    fn scale(&mut self, mut factor: f32) -> &mut Self {
        self.canvas.scale(1.0 / self.prev_scale, 1.0 / self.prev_scale);
        factor *= self.scale_factor;
        self.canvas.scale(factor, factor);
        self.prev_scale = factor;
        self
    }

    fn with_tint(&mut self, color: Color, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.color(color);
        let filter = MakeBlend(to_skia_color(color), &BLEND_MODE_SRC_IN.with(JsValue::clone));
        self.paint.setColorFilter(&filter);
        (cb)(self);
        self.paint.setColorFilter(&JsValue::null());
        filter.delete();
        self
    }

    fn with_blend_mode(&mut self, blend_mode: super::BlendMode, cb: impl FnOnce(&mut Self)) -> &mut Self {
        let prev_blend = self.blend_mode.clone();
        self.blend_mode = match blend_mode {
            super::BlendMode::Clear => &BLEND_MODE_CLEAR,
            super::BlendMode::Src => &BLEND_MODE_SRC,
            super::BlendMode::Dst => &BLEND_MODE_DST,
            super::BlendMode::SrcOver => &BLEND_MODE_SRCOVER,
            super::BlendMode::DstOver => &BLEND_MODE_DSTOVER,
            super::BlendMode::SrcIn => &BLEND_MODE_SRCIN,
            super::BlendMode::DstIn => &BLEND_MODE_DSTIN,
            super::BlendMode::SrcOut => &BLEND_MODE_SRCOUT,
            super::BlendMode::DstOut => &BLEND_MODE_DSTOUT,
            super::BlendMode::SrcATop => &BLEND_MODE_SRCATOP,
            super::BlendMode::DstATop => &BLEND_MODE_DSTATOP,
            super::BlendMode::Xor => &BLEND_MODE_XOR,
            super::BlendMode::Plus => &BLEND_MODE_PLUS,
            super::BlendMode::Modulate => &BLEND_MODE_MODULATE,
            super::BlendMode::Screen => &BLEND_MODE_SCREEN,
            super::BlendMode::Overlay => &BLEND_MODE_OVERLAY,
            super::BlendMode::Darken => &BLEND_MODE_DARKEN,
            super::BlendMode::Lighten => &BLEND_MODE_LIGHTEN,
            super::BlendMode::ColorDodge => &BLEND_MODE_COLORDODGE,
            super::BlendMode::ColorBurn => &BLEND_MODE_COLORBURN,
            super::BlendMode::HardLight => &BLEND_MODE_HARDLIGHT,
            super::BlendMode::SoftLight => &BLEND_MODE_SOFTLIGHT,
            super::BlendMode::Difference => &BLEND_MODE_DIFFERENCE,
            super::BlendMode::Exclusion => &BLEND_MODE_EXCLUSION,
            super::BlendMode::Multiply => &BLEND_MODE_MULTIPLY,
            super::BlendMode::Hue => &BLEND_MODE_HUE,
            super::BlendMode::Saturation => &BLEND_MODE_SATURATION,
            super::BlendMode::Color => &BLEND_MODE_COLOR,
            super::BlendMode::Luminosity => &BLEND_MODE_LUMINOSITY,
        }
        .with(JsValue::clone);
        self.paint.setBlendMode(&self.blend_mode);
        (cb)(self);
        self.paint.setBlendMode(&prev_blend);
        self.blend_mode = prev_blend;
        self
    }

    fn draw_path(&mut self, path: &Path) -> &mut Self {
        self.draw_path_at(&path, Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
        self
    }

    fn draw_path_at(&mut self, path: &Path, bounds: Rect) -> &mut Self {
        self.canvas.save();
        self.canvas.translate(bounds.a.x, bounds.a.y);
        self.canvas.drawPath(&path.path, &self.paint);
        self.canvas.restore();
        self
    }

    fn draw_image(&mut self, image: &Image, bounds: Rect) -> &mut Self {
        self.canvas.drawImageRectOptions(
            &image.image,
            to_skia_rect(image.get_bounds()),
            to_skia_rect(bounds),
            &FILTER_MODE_LINEAR.with(JsValue::clone),
            &MIPMAP_MODE_NONE.with(JsValue::clone),
        );
        self
    }

    fn with_blur(&mut self, amount: f32, cb: impl FnOnce(&mut Self)) -> &mut Self {
        let filter = MakeBlur(&BLURSTYLE_NORMAL.with(JsValue::clone), amount, true);
        self.paint.setMaskFilter(&filter);
        (cb)(self);
        self.paint.setMaskFilter(&JsValue::null());
        if !filter.is_null() {
            filter.delete();
        }
        self
    }

    fn with_alpha(&mut self, alpha: f32, cb: impl FnOnce(&mut Self)) -> &mut Self {
        let prev_alpha = self.alpha_mul;
        self.alpha_mul *= alpha;
        (cb)(self);
        self.alpha_mul = prev_alpha;
        self
    }

    fn with_clip_path(&mut self, path: &Path, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.canvas.save();
        self.canvas.clipPath(&path.path, &CLIP_OP_INTERSECT.with(JsValue::clone), true);
        (cb)(self);
        self.canvas.restore();
        self
    }

    fn with_clip_rect(&mut self, clip_rect: Rect, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.canvas.save();
        self.canvas.clipRect(&to_skia_rect(clip_rect), &CLIP_OP_INTERSECT.with(JsValue::clone), true);
        (cb)(self);
        self.canvas.restore();
        self
    }

    fn fill(&mut self, fill: impl Into<Fill>) -> &mut Self {
        let fill: Fill = fill.into();
        self.paint.setStyle(&PAINTSTYLE_FILL.with(JsValue::clone));
        match fill {
            Fill::Color(color) => {
                self.color(color);
            }
            Fill::Gradient(gradient) => {
                self.color(rgba(0x000000ff));
                let colors =
                    gradient.colors.map(|c| to_skia_color_f32_array(c))[..gradient.num_positions as usize].to_vec();
                let shader = MakeLinearGradient(
                    to_skia_point(gradient.points.0),
                    to_skia_point(gradient.points.1),
                    colors,
                    gradient.positions[..gradient.num_positions as usize].to_vec(),
                    &TILEMODE_CLAMP.with(JsValue::clone),
                );
                self.paint.setShader(&shader);
                shader.delete();
            }
        }
        self
    }

    fn stroke(&mut self, fill: impl Into<Fill>, width: f32) -> &mut Self {
        let fill: Fill = fill.into();
        self.paint.setStyle(&PAINTSTYLE_STROKE.with(JsValue::clone));
        match fill {
            Fill::Color(color) => {
                self.paint.setStrokeWidth(width);
                self.color(color);
            }
            Fill::Gradient(gradient) => {
                self.color(rgba(0x000000ff));
                let colors =
                    gradient.colors.map(|c| to_skia_color_f32_array(c))[..gradient.num_positions as usize].to_vec();
                let shader = MakeLinearGradient(
                    to_skia_point(gradient.points.0),
                    to_skia_point(gradient.points.1),
                    colors,
                    gradient.positions[..gradient.num_positions as usize].to_vec(),
                    &TILEMODE_CLAMP.with(JsValue::clone),
                );
                self.paint.setShader(&shader);
                shader.delete();
            }
        }
        self
    }

    fn clear(&mut self, color: Color) -> &mut Self {
        self.canvas.clear(&to_skia_color(color));
        self
    }

    fn color(&mut self, color: Color) -> &mut Self {
        self.paint.setShader(&JsValue::null());
        let skia_color = to_skia_color(color.with_alpha_mul(self.alpha_mul));
        self.paint.setColor(&skia_color);
        self
    }

    fn draw_arc(&mut self, bounds: Rect, start_angle: f32, sweep_angle: f32) -> &mut Self {
        let w = self.paint.getStrokeWidth() * 0.5;
        let r = bounds.with_expansion(-w);
        self.canvas.drawArc(&to_skia_rect(r), start_angle, sweep_angle, false, &self.paint);
        self
    }

    fn draw_rect(&mut self, rect: Rect) -> &mut Self {
        self.canvas.drawRect(&to_skia_rect(rect), &self.paint);
        self
    }

    fn draw_round_rect(&mut self, rect: Rect, radius: impl Into<BorderRadius>) -> &mut Self {
        self.canvas.drawRRect(make_rounded_rect(rect, radius.into()), &self.paint);
        self
    }

    fn draw_circle(&mut self, center: impl Into<Point>, radius: f32) -> &mut Self {
        let center: Point = center.into();
        self.canvas.drawCircle(center.x, center.y, radius, &self.paint);
        self
    }

    fn stroke_cap(&mut self, cap: Cap) -> &mut Self {
        self.paint.setStrokeCap(&match cap {
            Cap::Butt => STROKE_CAP_BUTT.with(JsValue::clone),
            Cap::Round => STROKE_CAP_ROUND.with(JsValue::clone),
            Cap::Square => STROKE_CAP_SQUARE.with(JsValue::clone),
        });
        self
    }

    fn stroke_join(&mut self, join: Join) -> &mut Self {
        self.paint.setStrokeJoin(&match join {
            Join::Miter => STROKE_JOIN_MITER.with(JsValue::clone),
            Join::Round => STROKE_JOIN_ROUND.with(JsValue::clone),
            Join::Bevel => STROKE_JOIN_BEVEL.with(JsValue::clone),
        });
        self
    }

    fn draw_text(&mut self, blob: &TextBlob, position: Point) -> &mut Self {
        self.canvas.drawTextBlob(&blob.blob, position.x, position.y, &self.paint);
        self
    }

    fn draw_surface(&mut self, surface: &Surface, position: Point) -> &mut Self {
        self.canvas.save();
        self.canvas.scale(1.0 / self.scale_factor, 1.0 / self.scale_factor);
        let image = surface.surface.makeImageSnapshot();
        self.canvas.drawImage(&image, position.x * self.scale_factor, position.y * self.scale_factor, &self.paint);
        image.delete();
        self.canvas.restore();
        self
    }

    fn write_pixels(&mut self, size: IntPoint, offset: IntPoint, pixels: &[u8]) -> &mut Self {
        self.canvas.writePixels(
            pixels,
            size.x as usize,
            size.y as usize,
            offset.x as f32,
            offset.y as f32,
            &ALPHA_TYPE_UNPREMUL.with(JsValue::clone),
            &COLOR_TYPE_RGBA_8888.with(JsValue::clone),
            &COLOR_SPACE_SRGB.with(JsValue::clone),
        );
        self
    }

    fn with_rotation(&mut self, degrees: f32, point: impl Into<Point>, cb: impl FnOnce(&mut Self)) -> &mut Self {
        self.canvas.save();
        let p: Point = point.into();
        self.canvas.rotate(degrees, p.x, p.y);
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

    fn with_translation(&mut self, amount: impl Into<Point>, cb: impl FnOnce(&mut Self)) -> &mut Self {
        let p: Point = amount.into();
        self.canvas.save();
        self.canvas.translate(p.x, p.y);
        cb(self);
        self.canvas.restore();
        self
    }

    fn backdrop_filter(&mut self, bounds: Rect, amount: f32) -> &mut Self {
        self.canvas.save();
        self.canvas.clipRect(&to_skia_rect(bounds), &CLIP_OP_INTERSECT.with(JsValue::clone), true);
        let filter = ImageFilterMakeBlur(amount, amount, &TILEMODE_CLAMP.with(JsValue::clone), &JsValue::NULL);
        self.canvas.saveLayer(&JsValue::NULL, &to_skia_rect(bounds), &filter, 0);
        if !filter.is_null() {
            filter.delete();
        }
        self.canvas.restore();
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

    fn scale_rel(&mut self, point: impl Into<Point>) -> &mut Self {
        let p: Point = point.into();
        self.canvas.scale(p.x, p.y);
        self
    }

    fn translate(&mut self, point: impl Into<Point>) -> &mut Self {
        let p: Point = point.into();
        self.canvas.translate(p.x, p.y);
        self
    }
}
