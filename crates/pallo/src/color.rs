use palette::{
    FromColor, Hsl, Hsla, IntoColor, Lighten, Mix, Okhsla, OklabHue, Oklaba, Saturate, Srgb, Srgba, WithAlpha, WithHue,
};

use crate::{Fill, Point};

#[derive(Clone, Debug, Default, Copy, PartialEq)]
pub struct Color {
    color: Srgba,
}

#[inline(always)]
pub fn rgb(hex: u32) -> Color {
    Color::from_rgb_hex(hex)
}

#[inline(always)]
pub fn rgba(hex: u32) -> Color {
    Color::from_rgba(hex)
}

pub fn hsl(h: f32, s: f32, l: f32) -> Color {
    Color { color: Hsl::new(h * 255.0, s, l).into_color() }
}

impl Color {
    #[inline(always)]
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Color { color: Srgba::new(r, g, b, 1.0) }
    }

    #[inline(always)]
    fn from_rgb_hex(hex: u32) -> Self {
        let color: Srgb = Srgb::from(hex).into();
        Color { color: Srgba::from_color(color) }
    }

    #[inline(always)]
    fn from_rgba(hex: u32) -> Self {
        Color { color: Srgba::from(hex).into() }
    }

    #[inline(always)]
    pub fn as_hex(&self) -> u32 {
        (((self.red() * 255.0) as u32 & 0xff) << 24)
            + (((self.green() * 255.0) as u32 & 0xff) << 16)
            + (((self.blue() * 255.0) as u32 & 0xff) << 8)
            + ((self.alpha() * 255.0) as u32 & 0xff)
    }

    #[inline(always)]
    pub fn with_lightness_oklab(self, lightness: f32) -> Self {
        let mut color_oklab = Oklaba::from_color(self.color);
        color_oklab.l = lightness;
        Color { color: Srgb::from_color(color_oklab).into() }
    }

    #[inline(always)]
    pub fn get_hue_okhsl(&self) -> f32 {
        Okhsla::from_color(self.color).hue.into_degrees()
    }

    #[inline(always)]
    pub fn with_hue_okhsl(self, hue: f32) -> Self {
        let color = Okhsla::from_color(self.color);
        Color { color: color.with_hue(OklabHue::from_degrees(hue)).into_color() }
    }

    #[inline(always)]
    pub fn get_saturation_okhsl(&self) -> f32 {
        Okhsla::from_color(self.color).saturation
    }

    #[inline(always)]
    pub fn with_saturation_okhsl(self, saturation: f32) -> Self {
        let mut color = Okhsla::from_color(self.color);
        color.saturation = saturation;
        Color { color: color.into_color() }
    }

    #[inline(always)]
    pub fn get_lightness_okhsl(&self) -> f32 {
        Okhsla::from_color(self.color).lightness
    }

    #[inline(always)]
    pub fn with_lightness_okhsl(self, lightness: f32) -> Self {
        let mut color = Okhsla::from_color(self.color);
        color.lightness = lightness;
        Color { color: color.into_color() }
    }

    #[inline(always)]
    pub fn with_brightness_mul(self, factor: f32) -> Self {
        let color = Hsla::from_color(self.color);
        Self { color: Srgba::from_color(color.lighten_fixed(color.lightness * factor)) }
    }

    #[inline(always)]
    pub fn with_saturation_mul(self, f: f32) -> Self {
        let color = Hsla::from_color(self.color);
        Self { color: Srgba::from_color(color.saturate_fixed(color.saturation * f)) }
    }

    #[inline(always)]
    pub fn get_alpha(&self) -> f32 {
        self.color.alpha
    }

    #[inline(always)]
    pub fn with_alpha(self, new_alpha: f32) -> Self {
        Self { color: self.color.with_alpha(new_alpha) }
    }

    #[inline(always)]
    pub fn with_alpha_mul(self, factor: f32) -> Self {
        Self { color: self.color.with_alpha(self.color.alpha * factor) }
    }

    #[inline(always)]
    pub fn with_alpha_add(self, factor: f32) -> Self {
        Self { color: self.color.with_alpha(self.color.alpha + factor) }
    }

    #[inline(always)]
    pub fn with_mix(self, other: Color, t: f32) -> Self {
        let t_clamped = t.clamp(0.0, 1.0);
        Self { color: self.color.mix(other.color, t_clamped) }
    }

    #[inline(always)]
    pub fn red(&self) -> f32 {
        self.color.red
    }

    #[inline(always)]
    pub fn green(&self) -> f32 {
        self.color.green
    }

    #[inline(always)]
    pub fn blue(&self) -> f32 {
        self.color.blue
    }

    #[inline(always)]
    pub fn alpha(&self) -> f32 {
        self.color.alpha
    }
}

#[derive(Default, Clone)]
pub struct Gradient {
    pub(crate) points: (Point, Point),
    pub(crate) colors: [Color; 4],
    pub(crate) positions: [f32; 4],
    pub(crate) num_positions: u8,
}

impl Gradient {
    pub fn two_points(points: (impl Into<Point>, impl Into<Point>), colors: (Color, Color)) -> Self {
        Self {
            points: (points.0.into(), points.1.into()),
            colors: [colors.0, colors.1, Default::default(), Default::default()],
            positions: [0.0, 1.0, 0.0, 0.0],
            num_positions: 2,
        }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(val: (u8, u8, u8)) -> Self {
        Color { color: Srgba::new(val.0 as f32 / 255.0, val.1 as f32 / 255.0, val.2 as f32 / 255.0, 1.0) }
    }
}

impl From<Color> for Fill {
    fn from(val: Color) -> Self {
        Fill::Color(val)
    }
}

impl From<Gradient> for Fill {
    fn from(val: Gradient) -> Self {
        Fill::Gradient(val)
    }
}
