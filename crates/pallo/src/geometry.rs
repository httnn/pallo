use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use crate::{App, Grid};

#[derive(Clone, Default, Copy, Debug, PartialEq)]
pub struct IntPoint {
    pub x: i32,
    pub y: i32,
}

impl IntPoint {
    pub fn with_scale(&self, s: f32) -> IntPoint {
        Self { x: (self.x as f32 * s) as i32, y: (self.y as f32 * s) as i32 }
    }

    pub fn to_float(self) -> Point {
        point(self.x as f32, self.y as f32)
    }
}

impl From<(i32, i32)> for IntPoint {
    fn from(value: (i32, i32)) -> Self {
        Self { x: value.0, y: value.1 }
    }
}

impl From<(u32, u32)> for IntPoint {
    fn from(value: (u32, u32)) -> Self {
        Self { x: value.0 as i32, y: value.1 as i32 }
    }
}

impl From<(usize, usize)> for IntPoint {
    fn from(value: (usize, usize)) -> Self {
        Self { x: value.0 as i32, y: value.1 as i32 }
    }
}

impl From<IntPoint> for (u32, u32) {
    fn from(val: IntPoint) -> Self {
        (val.x as u32, val.y as u32)
    }
}

impl From<IntPoint> for (usize, usize) {
    fn from(val: IntPoint) -> Self {
        (val.x as usize, val.y as usize)
    }
}

impl From<IntPoint> for (i32, i32) {
    fn from(val: IntPoint) -> Self {
        (val.x, val.y)
    }
}

pub const fn int_point(x: i32, y: i32) -> IntPoint {
    IntPoint { x, y }
}

#[derive(Default, Copy, PartialEq, Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Point {
    pub fn to_int(self) -> IntPoint {
        IntPoint { x: self.x as i32, y: self.y as i32 }
    }

    #[inline(always)]
    pub fn min(&self, other: Self) -> Self {
        Self { x: self.x.min(other.x), y: self.y.min(other.y) }
    }

    #[inline(always)]
    pub fn max(&self, other: Self) -> Self {
        Self { x: self.x.max(other.x), y: self.y.max(other.y) }
    }

    #[inline(always)]
    pub fn with_offset(&self, offset: impl Into<Point>) -> Self {
        let o: Point = offset.into();
        Self { x: self.x + o.x, y: self.y + o.y }
    }

    #[inline(always)]
    pub fn with_x_offset(&self, offset: f32) -> Self {
        Self { x: self.x + offset, y: self.y }
    }

    #[inline(always)]
    pub fn with_y_offset(&self, offset: f32) -> Self {
        Self { x: self.x, y: self.y + offset }
    }

    #[inline(always)]
    pub fn round(&self) -> Self {
        Self { x: self.x.round(), y: self.y.round() }
    }

    #[inline(always)]
    pub fn lerp(&self, other: Self, amount: f32) -> Self {
        Self { x: self.x + (other.x - self.x) * amount, y: self.y + (other.y - self.y) * amount }
    }

    #[inline(always)]
    pub fn lerp_point(&self, other: Self, amount: impl Into<Point>) -> Self {
        let amount: Point = amount.into();
        Self { x: self.x + (other.x - self.x) * amount.x, y: self.y + (other.y - self.y) * amount.y }
    }

    #[inline(always)]
    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<f32> for Point {
    type Output = Point;

    fn add(self, rhs: f32) -> Self::Output {
        Point::new(self.x + rhs, self.y + rhs)
    }
}

impl Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Point {
    type Output = Point;

    fn mul(self, rhs: f32) -> Self::Output {
        Point::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Point> for Point {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Point::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Div<f32> for Point {
    type Output = Point;

    fn div(self, rhs: f32) -> Self::Output {
        Point::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}

impl From<(f32, f32)> for Point {
    fn from(val: (f32, f32)) -> Self {
        Point::new(val.0, val.1)
    }
}

impl From<f32> for Point {
    fn from(value: f32) -> Self {
        Point::new(value, value)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[inline]
pub fn point(x: f32, y: f32) -> Point {
    Point::new(x, y)
}

#[derive(Default, Copy, Clone)]
pub struct Margin {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) right: f32,
    pub(crate) bottom: f32,
}

impl Margin {
    pub fn even(value: f32) -> Self {
        Margin { left: value, top: value, right: value, bottom: value }
    }

    pub fn xy(x: f32, y: f32) -> Self {
        Margin { left: x, top: y, right: x, bottom: y }
    }

    pub fn left_right(value: f32) -> Self {
        Margin { left: value, top: 0.0, right: value, bottom: 0.0 }
    }

    pub fn top_bottom(value: f32) -> Self {
        Margin { left: 0.0, top: value, right: 0.0, bottom: value }
    }

    pub fn top(top: f32) -> Self {
        Margin { top, ..Default::default() }
    }
}

#[derive(Copy, Clone)]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Copy, Clone)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Top => Side::Bottom,
            Side::Right => Side::Left,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
        }
    }
}

pub struct Expansion {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl Expansion {
    #[inline(always)]
    pub fn x(amount: f32) -> Self {
        Self { top: 0.0, right: amount, bottom: 0.0, left: amount }
    }

    #[inline(always)]
    pub fn y(amount: f32) -> Self {
        Self { top: amount, right: 0.0, bottom: amount, left: 0.0 }
    }

    #[inline(always)]
    pub fn xy(x: f32, y: f32) -> Self {
        Self { top: y, right: x, bottom: y, left: x }
    }
}

impl From<f32> for Expansion {
    fn from(val: f32) -> Self {
        Expansion { top: val, right: val, bottom: val, left: val }
    }
}

#[derive(Default, Copy, PartialEq, Clone, Debug)]
pub struct Rect {
    pub a: Point,
    pub b: Point,
}

impl Rect {
    pub fn grid<A: App>(&mut self) -> Grid<'_, A> {
        Grid::rect(self)
    }

    pub fn from_size(w: f32, h: f32) -> Self {
        Self { a: point(0.0, 0.0), b: point(w, h) }
    }

    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { a: point(x, y), b: point(x + w, y + h) }
    }

    pub fn from_ab(a: Point, b: Point) -> Self {
        Self { a: a.min(b), b: a.max(b) }
    }

    #[inline(always)]
    pub fn left(&self) -> f32 {
        self.a.x
    }

    #[inline(always)]
    pub fn top(&self) -> f32 {
        self.a.y
    }

    #[inline(always)]
    pub fn right(&self) -> f32 {
        self.b.x
    }

    #[inline(always)]
    pub fn bottom(&self) -> f32 {
        self.b.y
    }

    #[inline(always)]
    pub fn height(&self) -> f32 {
        self.b.y - self.a.y
    }

    #[inline(always)]
    pub fn width(&self) -> f32 {
        self.b.x - self.a.x
    }

    #[inline(always)]
    pub fn aspect_ratio(&self) -> f32 {
        self.width() / self.height()
    }

    #[inline(always)]
    pub fn size(&self) -> Point {
        point(self.width(), self.height())
    }

    #[inline(always)]
    pub fn int_size(&self) -> IntPoint {
        int_point(self.width() as i32, self.height() as i32)
    }

    #[inline(always)]
    pub fn contains(&self, p: &Point) -> bool {
        p.x >= self.a.x && p.x <= self.b.x && p.y >= self.a.y && p.y <= self.b.y
    }

    #[inline(always)]
    pub fn overlaps(&self, other: Rect) -> bool {
        self.contains(&other.a) || self.contains(&other.b)
    }

    #[inline(always)]
    pub fn union(&self, other: Rect) -> Rect {
        Self { a: self.a.min(other.a), b: self.b.max(other.b) }
    }

    #[inline(always)]
    pub fn intersects(&self, other: Rect) -> bool {
        self.left() <= other.right()
            && other.left() <= self.right()
            && self.top() <= other.bottom()
            && other.top() <= self.bottom()
    }

    #[inline(always)]
    pub fn center(&self) -> Point {
        point((self.a.x + self.b.x) * 0.5, (self.a.y + self.b.y) * 0.5)
    }

    #[inline(always)]
    pub fn x_aligned_within(&self, other: Rect, align: Align) -> Rect {
        let left = match align {
            Align::Start => other.left(),
            Align::Center => other.center().x - self.width() * 0.5,
            Align::End => other.right() - self.width(),
        };
        Rect::from_xywh(left, self.top(), self.width(), self.height())
    }

    #[inline(always)]
    pub fn y_aligned_within(&self, other: Rect, align: Align) -> Rect {
        let top = match align {
            Align::Start => other.top(),
            Align::Center => other.center().y - self.height() * 0.5,
            Align::End => other.bottom() - self.height(),
        };
        Rect::from_xywh(self.left(), top, self.width(), self.height())
    }

    #[inline(always)]
    pub fn centered_within(&self, other: Rect) -> Rect {
        self.x_aligned_within(other, Align::Center).y_aligned_within(other, Align::Center)
    }

    #[inline(always)]
    pub fn remove_from(&mut self, amount: f32, side: Side) -> Rect {
        match side {
            Side::Top => {
                let new = self.with_bottom(self.a.y + amount);
                self.a.y += amount;
                new
            }
            Side::Right => {
                let new = self.with_left(self.b.x - amount);
                self.b.x -= amount;
                new
            }
            Side::Bottom => {
                let new = self.with_top(self.b.y - amount);
                self.b.y -= amount;
                new
            }
            Side::Left => {
                let new = self.with_right(self.a.x + amount);
                self.a.x += amount;
                new
            }
        }
    }

    #[inline(always)]
    pub fn with_margin(&self, margin: Margin) -> Rect {
        let mut new = *self;
        new.a.x += margin.left;
        new.a.y += margin.top;
        new.b.x -= margin.right;
        new.b.y -= margin.bottom;
        new
    }

    pub fn relative_point(&self, point: impl Into<Point>) -> Point {
        self.a.lerp_point(self.b, point)
    }

    #[inline(always)]
    pub fn edge_point(&self, x: Align, y: Align) -> Point {
        self.relative_point((
            match x {
                Align::Start => 0.0,
                Align::Center => 0.5,
                Align::End => 1.0,
            },
            match y {
                Align::Start => 0.0,
                Align::Center => 0.5,
                Align::End => 1.0,
            },
        ))
    }

    #[inline(always)]
    pub fn with_width(&self, width: f32) -> Self {
        Self { a: self.a, b: point(self.a.x + width, self.b.y) }
    }

    #[inline(always)]
    pub fn with_width_align(&self, width: f32, align: Align) -> Self {
        self.with_width(width).x_aligned_within(*self, align)
    }

    #[inline(always)]
    pub fn with_height(&self, height: f32) -> Self {
        Self { a: self.a, b: point(self.b.x, self.a.y + height) }
    }

    #[inline(always)]
    pub fn with_height_align(&self, width: f32, align: Align) -> Self {
        self.with_height(width).y_aligned_within(*self, align)
    }

    #[inline(always)]
    pub fn with_size(&self, size: impl Into<Point>) -> Self {
        let s: Point = size.into();
        Self { a: self.a, b: point(self.a.x + s.x, self.a.y + s.y) }
    }

    #[inline(always)]
    pub fn with_left(&self, left: f32) -> Self {
        let mut new = *self;
        new.a.x = left;
        new
    }

    #[inline(always)]
    pub fn with_proportional_left(&self, left: f32) -> Self {
        let mut new = *self;
        new.a.x += left * new.width();
        new
    }

    #[inline(always)]
    pub fn with_right(&self, right: f32) -> Self {
        let mut new = *self;
        new.b.x = right;
        new
    }

    #[inline(always)]
    pub fn with_right_keep_width(&self, right: f32) -> Self {
        let mut new = *self;
        new.b.x = right;
        new.a.x = new.b.x - self.width();
        new
    }

    #[inline(always)]
    pub fn with_bottom(&self, bottom: f32) -> Self {
        let mut new = *self;
        new.b.y = bottom;
        new
    }

    #[inline(always)]
    pub fn with_top(&self, top: f32) -> Self {
        let mut new = *self;
        new.a.y = top;
        new
    }

    #[inline(always)]
    pub fn with_topleft(&self, a: Point) -> Self {
        Self { a, b: self.b + (a - self.a) }
    }

    #[inline(always)]
    pub fn with_x_offset(&self, x: f32) -> Self {
        Self { a: point(self.a.x + x, self.a.y), b: point(self.b.x + x, self.b.y) }
    }

    #[inline(always)]
    pub fn with_y_offset(&self, y: f32) -> Self {
        Self { a: point(self.a.x, self.a.y + y), b: point(self.b.x, self.b.y + y) }
    }

    #[inline(always)]
    pub fn with_offset(&self, p: Point) -> Self {
        Self { a: point(self.a.x + p.x, self.a.y + p.y), b: point(self.b.x + p.x, self.b.y + p.y) }
    }

    #[inline(always)]
    pub fn with_expansion(&self, amount: impl Into<Expansion>) -> Self {
        let amount: Expansion = amount.into();
        let mut new = *self;
        new.a.x -= amount.left;
        new.a.y -= amount.top;
        new.b.x += amount.right;
        new.b.y += amount.bottom;
        new
    }

    #[inline(always)]
    pub fn with_scale(&self, scale: impl Into<Point>) -> Self {
        let scale: Point = scale.into();
        let mut new = *self;
        new.a = new.a * scale;
        new.b = new.b * scale;
        new
    }

    #[inline(always)]
    pub fn with_lerp(&self, other: &Self, amount: f32) -> Self {
        Self { a: self.a.lerp(other.a, amount), b: self.b.lerp(other.b, amount) }
    }

    #[inline(always)]
    pub fn with_aspect_ratio_keep_centered(&self, aspect_ratio: f32) -> Self {
        let width = self.b.x - self.a.x;
        let height = self.b.y - self.a.y;

        let original_aspect_ratio = width / height;

        let (new_width, new_height) = if aspect_ratio > original_aspect_ratio {
            (width, width / aspect_ratio)
        } else {
            (height * aspect_ratio, height)
        };

        let new_center_x = self.a.x + width * 0.5;
        let new_center_y = self.a.y + height * 0.5;

        let new_a = Point { x: new_center_x - new_width * 0.5, y: new_center_y - new_height * 0.5 };
        let new_b = Point { x: new_center_x + new_width * 0.5, y: new_center_y + new_height * 0.5 };

        Rect { a: new_a, b: new_b }
    }

    #[inline(always)]
    pub fn with_relative_offset(&self, offset: impl Into<Point>) -> Self {
        let offset: Point = offset.into();
        let a = self.a + offset * self.size();
        Rect { a, b: a + self.size() }
    }

    #[inline(always)]
    pub fn rounded(&self) -> Self {
        let mut out = *self;
        out.a.x = out.a.x.round();
        out.a.y = out.a.y.round();
        out.b.x = out.b.x.round();
        out.b.y = out.b.y.round();
        out
    }

    pub fn with_clamped(&self, within: Rect) -> Self {
        let size = self.size();
        let a = self.a.max(within.a);
        let out = Self { a, b: a + size };
        out.with_x_offset((within.right() - out.right()).min(0.0))
            .with_y_offset((within.bottom() - out.bottom()).min(0.0))
    }
}
