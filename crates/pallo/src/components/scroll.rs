use std::{cell::RefCell, marker::PhantomData, ops::Range};

use crate::*;

#[derive(Default)]
struct Memo<T> {
    value: RefCell<Option<T>>,
}

impl<T: Clone> Memo<T> {
    pub fn get(&self, set: impl FnOnce() -> T) -> T {
        let mut value = self.value.borrow_mut();
        if value.is_none() {
            *value = Some((set)());
        }
        unsafe { value.as_ref().unwrap_unchecked().clone() }
    }

    pub fn invalidate(&self) {
        *self.value.borrow_mut() = None;
    }
}

pub type ScrollbarDrawer<A> = fn(&mut Cx<A>, canvas: &mut Canvas, Rect, bool);

pub struct ScrollList<A: App, ItemID, C> {
    id: ComponentId,
    filtered_item_indexes: Vec<usize>,
    items: Vec<C>,
    item_ids: Vec<ItemID>,
    create_item: Box<dyn Fn(&mut Cx<A>, ComponentId, ItemID) -> C>,
    item_bounds: Vec<Rect>,
    content_height: f32,
    scroll_top: f32,
    scroll_top_on_mouse_down: f32,
    scrollbar_id: ComponentId,
    visible_items: Memo<Range<usize>>,
    scrollbar_bounds: Memo<Rect>,
    scrollbar_hovered: bool,
    dragging_scroll_handle: bool,
    scroll_velocity: f32,
    is_scrolling_with_cursor: bool,
    _p: PhantomData<A>,
    scroll_by_dragging: bool,
    draw_scrollbar: ScrollbarDrawer<A>,
    scrollbar_width: f32,
}

pub trait ScrollListItem<A: App> {
    #[allow(unused)]
    fn set_is_scrolling(&mut self, cx: &mut Cx<A>, is_scrolling_with_cursor: bool) {}
    fn get_shown(&self) -> bool {
        true
    }
}

impl<A: App, ItemId: PartialEq + Clone, C: Component<A> + ScrollListItem<A>> Component<A> for ScrollList<A, ItemId, C> {
    fn draw(&self, cx: &mut Cx<A>, canvas: &mut Canvas) {
        canvas.with_clip_rect(self.get_bounds(cx), |canvas| {
            for &i in &self.filtered_item_indexes[self.get_visible_items_range(cx)] {
                self.items[i].draw(cx, canvas);
            }

            if self.scrollbar_id.is_visible(cx) {
                let bounds = self.get_scrollbar_bounds(cx);
                (self.draw_scrollbar)(cx, canvas, bounds, self.scrollbar_hovered || self.dragging_scroll_handle);
            }
        });
    }

    fn event(&mut self, cx: &mut Cx<A>, event: &mut Event<A>) {
        let mut pass_to_items = true;
        match event {
            Event::Update => {
                if self.scroll_velocity.abs() > 0.001 {
                    self.scroll_to(cx, self.scroll_top - self.scroll_velocity);
                    self.scroll_velocity *= 0.995f32.powf(cx.frame_delta_ms);
                }

                self.relayout_if_necessary(cx);
            }
            Event::MouseWheel(delta) => {
                if self.get_bounds(cx).contains(&cx.main_pointer().position) {
                    self.scroll_to(cx, self.scroll_top - delta.y);
                    pass_to_items = false;
                }
            }
            Event::PointerDown(pointer) => {
                self.scrollbar_hovered = self.scrollbar_id.is_hovered(pointer);
                if self.scrollbar_hovered
                    || (self.is_visible(cx)
                        && self.scroll_by_dragging
                        && self.get_bounds(cx).contains(&pointer.position))
                {
                    self.scroll_velocity = 0.0;
                    let scrollbar_bounds = self.get_scrollbar_bounds(cx);
                    self.dragging_scroll_handle = scrollbar_bounds.contains(&pointer.position);

                    if !self.dragging_scroll_handle && self.scrollbar_hovered {
                        let scrollbar_area = self.scrollbar_id.get_bounds(cx);
                        let delta_ratio = (pointer.position.y - scrollbar_area.top() - scrollbar_bounds.height() * 0.5)
                            / scrollbar_area.height();
                        let delta = delta_ratio * self.content_height;
                        self.scroll_to(cx, delta);
                        self.dragging_scroll_handle = true;
                    }

                    self.scroll_top_on_mouse_down = self.scroll_top;

                    self.is_scrolling_with_cursor = !self.scrollbar_hovered;
                }
            }
            Event::PointerMove(pointer) => {
                if self.is_visible(cx) {
                    if self.is_scrolling_with_cursor && pointer.delta.len() > 7.0 {
                        for item in &mut self.items {
                            item.set_is_scrolling(cx, true);
                        }
                    }

                    if self.scrollbar_id.is_pressed(pointer) && self.dragging_scroll_handle {
                        let delta_ratio = pointer.delta.y / self.scrollbar_id.get_bounds(cx).height();
                        let delta = delta_ratio * self.content_height;
                        self.scroll_to(cx, self.scroll_top_on_mouse_down + delta);
                    } else if self.scroll_by_dragging
                        && pointer.pressed_component.is_some()
                        && self.get_bounds(cx).contains(&pointer.down_position)
                    {
                        self.scroll_to(cx, self.scroll_top_on_mouse_down - pointer.delta.y);
                    }
                }
            }
            Event::PointerUp(pointer) => {
                if pointer.is_pressed(&self.scrollbar_id) {
                    self.scrollbar_hovered = false;
                }
                if self.dragging_scroll_handle {
                    self.dragging_scroll_handle = false;
                }
                if self.is_scrolling_with_cursor {
                    if pointer.delta_sum.y.abs() > 5.0 {
                        self.scroll_velocity = pointer.velocity.y * 1.5;
                    }
                    self.is_scrolling_with_cursor = false;

                    for item in &mut self.items {
                        item.set_is_scrolling(cx, false);
                    }
                }
            }
            _ => {}
        }

        if pass_to_items {
            for &i in &self.filtered_item_indexes[self.get_visible_items_range(cx)] {
                self.items[i].event(cx, event);
            }
        }
    }

    fn layout(&mut self, cx: &mut Cx<A>, mut bounds: Rect) {
        self.set_bounds(cx, bounds);
        self.scrollbar_id.set_bounds(cx, bounds.remove_from(self.scrollbar_width, Side::Right));
        self.update_item_bounds(cx);
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }
}

impl<A: App, ItemId: Clone + PartialEq, C: Component<A> + ScrollListItem<A>> ScrollList<A, ItemId, C> {
    pub fn new(
        cx: &mut Cx<A>,
        id: ComponentId,
        create_item: impl Fn(&mut Cx<A>, ComponentId, ItemId) -> C + 'static,
    ) -> Self {
        Self {
            filtered_item_indexes: vec![],
            items: vec![],
            item_bounds: vec![],
            item_ids: vec![],
            create_item: Box::new(create_item),
            content_height: 0.0,
            scroll_top: 0.0,
            scroll_top_on_mouse_down: 0.0,
            scrollbar_id: {
                let id = cx.add_child_id(&id);
                id.set_hoverable(cx, true);
                id
            },
            visible_items: Default::default(),
            scrollbar_bounds: Default::default(),
            id,
            scrollbar_hovered: false,
            dragging_scroll_handle: false,
            is_scrolling_with_cursor: false,
            scroll_velocity: 0.0,
            scroll_by_dragging: false,
            draw_scrollbar: A::draw_scrollbar,
            scrollbar_width: 8.0,
            _p: PhantomData,
        }
    }

    pub fn with_scrollbar_drawer(mut self, draw_scrollbar: ScrollbarDrawer<A>) -> Self {
        self.draw_scrollbar = draw_scrollbar;
        self
    }

    pub fn with_scrollbar_width(mut self, width: f32) -> Self {
        self.scrollbar_width = width;
        self
    }

    pub fn with_scroll_by_dragging(mut self) -> Self {
        self.scroll_by_dragging = true;
        self
    }

    pub fn set_scroll_by_dragging(&mut self, value: bool) {
        self.scroll_by_dragging = value;
    }

    pub fn is_scrolling(&self, cx: &mut Cx<A>) -> bool {
        self.dragging_scroll_handle || (self.is_scrolling_with_cursor && cx.main_pointer().delta.len() > 5.0)
    }

    pub fn scrollbar_id(&self) -> &ComponentId {
        &self.scrollbar_id
    }

    pub fn set_items_with_create_item(
        &mut self,
        cx: &mut Cx<A>,
        ids: impl IntoIterator<Item = ItemId>,
        create_item: impl Fn(&mut Cx<A>, ComponentId, ItemId) -> C + 'static,
    ) {
        self.create_item = Box::new(create_item);
        self.set_items(cx, ids);
    }

    pub fn set_items(&mut self, cx: &mut Cx<A>, ids: impl IntoIterator<Item = ItemId>) {
        let mut new_items = vec![];
        let mut new_item_ids = vec![];
        for item_id in ids {
            new_item_ids.push(item_id.clone());
            if let Some(idx) = self.item_ids.iter().position(|id| *id == item_id) {
                new_items.push(self.items.remove(idx));
                self.item_ids.remove(idx);
            } else {
                new_items.push(cx.add_child(&self.id, |cx, id| (self.create_item)(cx, id, item_id)));
            }
        }
        self.items = new_items;
        self.item_ids = new_item_ids;
        self.scrollbar_id = cx.add_child_id(self.id()).interactive(cx);
        self.update_item_bounds(cx);
    }

    pub fn get_items(&mut self) -> impl Iterator<Item = &mut C> {
        self.items.iter_mut()
    }

    pub fn get_filtered_items(&mut self) -> impl Iterator<Item = &mut C> {
        self.items
            .iter_mut()
            .enumerate()
            .filter_map(|(i, item)| self.filtered_item_indexes.contains(&i).then_some(item))
    }

    pub fn items_mut(&mut self) -> &mut Vec<C> {
        &mut self.items
    }

    pub fn items(&self) -> &Vec<C> {
        &self.items
    }

    fn update_item_bounds(&mut self, cx: &mut Cx<A>) {
        self.filtered_item_indexes =
            self.items.iter().enumerate().filter_map(|(i, item)| item.get_shown().then_some(i)).collect();

        let mut bounds = self.get_bounds(cx);
        if self.content_height > bounds.height() {
            bounds.remove_from(self.scrollbar_width + 2.0, Side::Right);
        }
        let mut y = bounds.top();
        self.item_bounds = self
            .filtered_item_indexes
            .iter()
            .map(|i| {
                let height = self.items[*i]
                    .get_preferred_size(cx, bounds)
                    .1
                    .unwrap_or_else(|| panic!("Each scroll list item must declare its own height!"));
                let bounds = Rect::from_xywh(bounds.left(), y, bounds.width(), height);
                y += height;
                bounds
            })
            .collect();
        self.content_height = self.item_bounds.iter().map(|b| b.height()).sum::<f32>().max(bounds.height());
        self.scrollbar_id.set_visible(cx, self.content_height > bounds.height());
        self.scroll_to(cx, self.scroll_top);
    }

    fn get_scrollbar_bounds(&self, cx: &mut Cx<A>) -> Rect {
        self.scrollbar_bounds.get(|| {
            let content_bounds = self.get_bounds(cx);
            let scrollbar_area = self.scrollbar_id.get_bounds(cx);

            let scrollbar_height = scrollbar_area.height() * content_bounds.height() / self.content_height;
            let scrollbar_top = self.scroll_top * content_bounds.height() / self.content_height;
            scrollbar_area.with_y_offset(scrollbar_top).with_height(scrollbar_height)
        })
    }

    fn get_visible_items_range(&self, cx: &mut Cx<A>) -> Range<usize> {
        self.visible_items.get(|| {
            let content_bounds = self.get_bounds(cx);
            let viewport_top = content_bounds.top();
            let viewport_height = content_bounds.height();
            let mut first = 0;
            let mut last = 0;
            for (i, bounds) in self.item_bounds.iter().enumerate() {
                if bounds.bottom() >= viewport_top + self.scroll_top {
                    first = i;
                    break;
                }
            }
            for (i, bounds) in self.item_bounds.iter().enumerate().rev() {
                if bounds.top() <= viewport_top + self.scroll_top + viewport_height {
                    last = i + 1;
                    break;
                }
            }
            first..last
        })
    }

    fn scroll_to(&mut self, cx: &mut Cx<A>, top: f32) {
        self.scroll_top = top.clamp(0.0, self.content_height - self.get_bounds(cx).height());
        self.visible_items.invalidate();
        self.scrollbar_bounds.invalidate();

        let range = self.get_visible_items_range(cx);

        for (i, item) in self.items.iter_mut().enumerate() {
            if let Some(filtered_idx) = self.filtered_item_indexes.iter().position(|j| *j == i) {
                if range.contains(&filtered_idx) {
                    let bounds = self.item_bounds[filtered_idx];
                    item.layout(cx, bounds.with_top(bounds.top() - self.scroll_top).with_height(bounds.height()));
                    item.set_visible(cx, true);
                } else {
                    item.set_visible(cx, false);
                }
            }
        }
    }
}
