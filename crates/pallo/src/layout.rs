use crate::{
    App, ComponentId, Cx, NodeIdLike, Svg,
    component::Component,
    geometry::{Margin, Rect},
};

enum Kind<'a, A: App> {
    Container,
    Component(&'a mut dyn Component<A>),
    Svg(&'a mut Svg),
    Rect(&'a mut Rect),
    ComponentId(&'a ComponentId),
    Fn(&'a mut dyn FnMut(&mut Cx<A>, Rect)),
    Empty,
}

pub struct Gap {
    before: f32,
    after: f32,
}

impl Gap {
    pub fn before(before: f32) -> Gap {
        Gap { before, after: 0.0 }
    }

    pub fn after(after: f32) -> Gap {
        Gap { before: 0.0, after }
    }

    pub fn both(v: f32) -> Gap {
        Gap { before: v, after: v }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Size {
    Pixels(f32),
    Fraction(f32),
}

#[derive(Copy, Clone)]
enum Direction {
    LeftRight,
    TopDown,
}

pub struct Grid<'a, A: App> {
    kind: Kind<'a, A>,
    size: Option<Size>,
    children: &'a mut [Grid<'a, A>],
    direction: Direction,
    margin: Margin,
    gap: Gap,
    child_gap: Option<Size>,
    check_visibility: bool,
    map_bounds: fn(Rect) -> Rect,
}

impl<A: App> Default for Grid<'_, A> {
    fn default() -> Self {
        Self {
            kind: Kind::Container,
            size: None,
            children: &mut [],
            direction: Direction::LeftRight,
            margin: Default::default(),
            gap: Gap { before: 0.0, after: 0.0 },
            child_gap: Default::default(),
            check_visibility: false,
            map_bounds: |r| r,
        }
    }
}

pub fn left_right<'a, A: App>(c: &'a mut [Grid<'a, A>]) -> Grid<'a, A> {
    Grid::container().left_right(c)
}

pub fn top_down<'a, A: App>(c: &'a mut [Grid<'a, A>]) -> Grid<'a, A> {
    Grid::container().top_down(c)
}

pub trait IntoGrid<'a, A: App> {
    fn grid(&'a mut self) -> Grid<'a, A>;
}

// Rust is dumb sometimes
pub trait IntoGridOwned<'a, A: App> {
    fn grid(self) -> Grid<'a, A>;
}

impl<'a, A: App> IntoGridOwned<'a, A> for f32 {
    fn grid(self) -> Grid<'a, A> {
        Grid::space(self)
    }
}

impl<'a, A: App> IntoGridOwned<'a, A> for i32 {
    fn grid(self) -> Grid<'a, A> {
        Grid::space(self as f32)
    }
}

impl<'a, A: App> IntoGridOwned<'a, A> for Size {
    fn grid(self) -> Grid<'a, A> {
        Grid::space(self)
    }
}

impl<'a, A: App, T: Component<A>> IntoGrid<'a, A> for T {
    fn grid(&'a mut self) -> Grid<'a, A> {
        Grid::component(self)
    }
}

impl<'a, A: App> IntoGrid<'a, A> for dyn Component<A> + 'static {
    fn grid(&'a mut self) -> Grid<'a, A> {
        Grid::component(self)
    }
}

impl<'a, A: App> IntoGridOwned<'a, A> for Option<Grid<'a, A>> {
    fn grid(self) -> Grid<'a, A> {
        match self {
            Some(g) => g,
            None => Grid::empty(),
        }
    }
}

pub trait Px {
    fn px(self) -> Size;
}

impl Px for f32 {
    fn px(self) -> Size {
        Size::Pixels(self)
    }
}

impl Px for i32 {
    fn px(self) -> Size {
        Size::Pixels(self as f32)
    }
}

pub trait Fr {
    fn fr(self) -> Size;
}

impl Fr for f32 {
    fn fr(self) -> Size {
        Size::Fraction(self)
    }
}

impl Fr for i32 {
    fn fr(self) -> Size {
        Size::Fraction(self as f32)
    }
}

impl From<f32> for Size {
    fn from(value: f32) -> Self {
        value.px()
    }
}

impl From<i32> for Size {
    fn from(value: i32) -> Self {
        (value as f32).px()
    }
}

impl<'a, A: App> Grid<'a, A> {
    fn container() -> Self {
        Grid { kind: Kind::Container, ..Default::default() }
    }

    pub fn svg(svg: &'a mut Svg) -> Self {
        Grid { kind: Kind::Svg(svg), ..Default::default() }
    }

    fn empty() -> Self {
        Grid { kind: Kind::Empty, ..Default::default() }
    }

    fn component(component: &'a mut dyn Component<A>) -> Self {
        Grid { kind: Kind::Component(component), ..Default::default() }
    }

    pub fn id(id: &'a ComponentId) -> Self {
        Grid { kind: Kind::ComponentId(id), ..Default::default() }
    }

    pub(crate) fn rect(rect: &'a mut Rect) -> Self {
        Grid { kind: Kind::Rect(rect), ..Default::default() }
    }

    pub fn func(func: &'a mut dyn FnMut(&mut Cx<A>, Rect)) -> Self {
        Grid { kind: Kind::Fn(func), ..Default::default() }
    }

    fn space(size: impl Into<Size>) -> Self {
        Grid { kind: Kind::Container, size: Some(size.into()), ..Default::default() }
    }

    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    pub fn left_right(mut self, c: &'a mut [Grid<'a, A>]) -> Self {
        self.children = c;
        self.direction = Direction::LeftRight;
        self
    }

    pub fn top_down(mut self, c: &'a mut [Grid<'a, A>]) -> Self {
        self.children = c;
        self.direction = Direction::TopDown;
        self
    }

    pub fn child_gap(mut self, gap: impl Into<Size>) -> Self {
        self.child_gap = Some(gap.into());
        self
    }

    pub fn gap(mut self, gap: impl Into<Gap>) -> Self {
        self.gap = gap.into();
        self
    }

    pub fn map_bounds(mut self, map: fn(Rect) -> Rect) -> Self {
        self.map_bounds = map;
        self
    }

    pub fn respect_visibility(mut self) -> Self {
        self.check_visibility = true;
        self
    }

    fn is_skipped(&self, cx: &mut Cx<A>) -> bool {
        match &self.kind {
            Kind::Empty => true,
            Kind::Component(c) => self.check_visibility && !cx.is_visible(c.id()),
            _ => false,
        }
    }

    fn get_size(&mut self, cx: &mut Cx<A>, bounds: Rect, direction: Direction) -> Size {
        if self.check_visibility
            && let Kind::Component(c) = &self.kind
            && !cx.is_visible(c.id())
        {
            return 0.px();
        }

        self.size.unwrap_or_else(|| {
            if let Kind::Component(c) = &mut self.kind {
                let preferred_size = c.get_preferred_size(cx, bounds);
                match direction {
                    Direction::LeftRight => preferred_size.0,
                    Direction::TopDown => preferred_size.1,
                }
                .map(|v| v.px())
                .unwrap_or(1.fr())
            } else {
                1.fr()
            }
        })
    }

    #[inline]
    pub fn layout(&mut self, cx: &mut Cx<A>, bounds: Rect) -> f32 {
        let bounds = (self.map_bounds)(bounds.with_margin(self.margin));

        match &mut self.kind {
            Kind::Container => {}
            Kind::Component(comp) => comp.layout(cx, bounds),
            Kind::Rect(rect) => **rect = bounds,
            Kind::ComponentId(id) => id.set_bounds(cx, bounds),
            Kind::Fn(func) => (func)(cx, bounds),
            Kind::Empty => {}
            Kind::Svg(svg) => svg.set_bounds(cx, bounds),
        }

        let fraction_size = {
            let num_gaps = self.children.iter().filter(|c| !c.is_skipped(cx)).count().saturating_sub(1) as f32;
            let mut fraction_sum: f32 = if let Some(Size::Fraction(fr)) = self.child_gap {
                fr * num_gaps
            } else {
                0.0
            };
            let mut fractionable_size = match self.direction {
                Direction::LeftRight => bounds.width(),
                Direction::TopDown => bounds.height(),
            };
            if let Some(Size::Pixels(px)) = self.child_gap {
                fractionable_size -= px * num_gaps;
            }
            for child in self.children.iter_mut() {
                if child.is_skipped(cx) {
                    continue;
                }
                fractionable_size -= child.gap.after + child.gap.before;
                match child.get_size(cx, bounds.with_margin(child.margin), self.direction) {
                    Size::Pixels(px) => fractionable_size -= px,
                    Size::Fraction(fr) => fraction_sum += fr,
                }
            }
            fractionable_size / fraction_sum
        };

        let mut position = match self.direction {
            Direction::LeftRight => bounds.left(),
            Direction::TopDown => bounds.top(),
        };
        let num_children = self.children.len();
        for (i, child) in self.children.iter_mut().enumerate() {
            if child.is_skipped(cx) {
                continue;
            }
            position += child.gap.before;
            let size = match child.get_size(cx, bounds.with_margin(child.margin), self.direction) {
                Size::Pixels(px) => px,
                Size::Fraction(fr) => fraction_size * fr,
            };
            let child_bounds = match self.direction {
                Direction::LeftRight => Rect::from_xywh(position, bounds.top(), size, bounds.height()),
                Direction::TopDown => Rect::from_xywh(bounds.left(), position, bounds.width(), size),
            };
            child.layout(cx, child_bounds);
            position += size + child.gap.after;

            if i != num_children - 1
                && let Some(gap) = &self.child_gap
            {
                let gap_size = match gap {
                    Size::Pixels(px) => *px,
                    Size::Fraction(fr) => fraction_size * fr,
                };
                position += gap_size;
            }
        }

        match self.direction {
            Direction::LeftRight => position - bounds.left() + self.margin.left + self.margin.right,
            Direction::TopDown => position - bounds.top() + self.margin.top + self.margin.bottom,
        }
    }
}
