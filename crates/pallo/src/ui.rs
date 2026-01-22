use keyboard_types::Key;
use pallo_util::File;
use std::rc::Rc;
use web_time::Instant;

use crate::{
    Canvas, ComponentState, IntPoint, Overlay, PointerId, PointerState, SignalCx,
    component::{Component, ComponentId, WeakComponentId},
    context::Cx,
    event::{Event, EventStatus, MouseButton},
    geometry::{Point, Rect},
    platform::{Frame, Platform, PlatformCommon},
    point,
    renderers::CanvasType,
    rgb,
    tree::Tree,
};

#[derive(Default, Clone)]
pub struct Modifiers {
    pub meta: bool,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl Modifiers {
    pub fn with_meta(&self, meta: bool) -> Self {
        Self { meta, ..*self }
    }

    pub fn with_shift(&self, shift: bool) -> Self {
        Self { shift, ..*self }
    }

    pub fn with_alt(&self, alt: bool) -> Self {
        Self { alt, ..*self }
    }

    pub fn with_ctrl(&self, ctrl: bool) -> Self {
        Self { ctrl, ..*self }
    }
}

pub trait App: Sized + 'static {
    type Input: Clone;
    type FontId: Into<usize> + Default;
    type AppInit: Send + Sync + Clone;
    type ComponentState: Default;

    fn new(rt: &SignalCx, init: Self::AppInit) -> Self;
    fn get_ui_scale(&self, size: IntPoint) -> f32;
    fn get_initial_size(init: &Self::AppInit) -> IntPoint;
    fn default_font_weight() -> f32 {
        550.0
    }
    fn draw_scrollbar(_cx: &mut Cx<Self>, canvas: &mut Canvas, bounds: Rect, active: bool) {
        canvas
            .fill(rgb(0xffffff).with_alpha(if active { 0.8 } else { 0.5 }))
            .draw_round_rect(bounds, bounds.width() * 0.5);
    }
}

pub struct UI<A: App> {
    root: Box<dyn Component<A>>,
    pub(crate) ui_context: Cx<A>,
    last_frame_start: Instant,
    last_window_size: IntPoint,
    is_broadcasting: bool,
    overlays: Vec<Overlay<dyn Component<A>>>,
}

unsafe impl<A: App> Send for UI<A> {}
unsafe impl<A: App> Sync for UI<A> {}

impl<A: App> UI<A> {
    pub fn new<R: Component<A> + 'static>(
        init: A::AppInit,
        platform: Platform,
        create_root: impl Fn(&mut Cx<A>, ComponentId) -> R,
    ) -> Self {
        let mut ui_context = Cx::new(init, platform);
        let root_id = ComponentId(Rc::new(ui_context.tree.add(ui_context.tree.get_root_id())));
        ui_context.component_ids.push(root_id.clone());
        UI {
            root: Box::new((create_root)(&mut ui_context, root_id.clone())),
            ui_context,
            last_frame_start: Instant::now(),
            last_window_size: IntPoint::default(),
            is_broadcasting: false,
            overlays: vec![],
        }
    }
}

pub enum WindowEvent {
    Resized(IntPoint),
    PointerMove { position: Point, id: PointerId },
    PointerDown { position: Point, button: MouseButton, id: PointerId },
    PointerUp { id: PointerId },
    Keydown(Key),
    Keyup(Key),
    ScaleFactorChanged(f32),
    ModifiersChanged(Modifiers),
    FileHovered(Vec<String>),
    FileDropped(Vec<File>),
    FileDropCancelled,
    MouseWheel(Point),
    FocusChanged(bool),
}

impl<A: App> UI<A> {
    pub fn broadcast_event(&mut self, event: &mut Event<A>) {
        if !self.is_broadcasting {
            self.is_broadcasting = true;
            self.root.event(&mut self.ui_context, event);
            for overlay in self.overlays.iter() {
                overlay.borrow_mut().event(&mut self.ui_context, event);
            }
            self.is_broadcasting = false;
        }
    }

    pub fn draw(&mut self) {
        let start = Instant::now();

        let scale_factor = self.ui_context.platform.get_scale_factor();
        if scale_factor != self.ui_context.scale_factor.get_fast() {
            self.on_event(WindowEvent::Resized(self.last_window_size));
        }

        self.ui_context.frame_delta_ms = (start - self.last_frame_start).as_millis() as f32;
        self.ui_context.num_frames += 1;
        self.last_frame_start = start;

        for overlay in self.ui_context.overlays.drain(..) {
            self.overlays.push(overlay);
        }

        while let Some(event) = self.ui_context.platform.next_window_event() {
            self.on_event(event);
        }

        // check for long presses
        #[cfg(target_os = "ios")]
        {
            let mut long_press = None;
            for pointer in self.ui_context.pointer_state.values_mut() {
                if !pointer.is_long_press
                    && pointer.down_time.is_some_and(|t| (start - t).as_millis() > 400)
                    && pointer.delta_sum.len() < 1.0
                {
                    long_press = Some(pointer.clone());
                    pointer.is_long_press = true;
                    break;
                }
            }
            if let Some(pointer) = long_press {
                self.broadcast_event(&mut Event::LongPress(pointer));
            }
        }

        // send update event
        {
            let start = Instant::now();
            self.broadcast_event(&mut Event::Update);
            let duration = (Instant::now() - start).as_micros();
            self.ui_context.update_time_micros = duration;
        }

        // check if focused component is still visible
        if let Some(focused) = self.ui_context.focused_component
            && !self.ui_context.is_visible(focused)
        {
            self.ui_context.focused_component = None;
            let event = &mut Event::FocusChanged(self.ui_context.focused_component.map(WeakComponentId));
            self.broadcast_event(event);
        }

        // handle and broadcast input events
        while let Some(mut e) = self.ui_context.input.pop_front() {
            self.broadcast_event(&mut e);
        }

        // garbage collect removed components
        self.ui_context.component_ids.retain(|id| {
            if Rc::strong_count(&id.0) > 1 {
                return true;
            }
            self.ui_context.tree.remove(*id.0);
            if Some(id.into()) == self.ui_context.focused_component {
                self.ui_context.focused_component = None;
                // let event = &mut Event::FocusChanged(None);
                // self.broadcast_event(event);
            }
            for pointer in self.ui_context.pointer_state.values_mut() {
                if Some(id.into()) == pointer.hovered_component {
                    pointer.hovered_component = None;
                }
            }
            false
        });

        // advance all animations
        self.ui_context.animations.tick(self.ui_context.frame_delta_ms);

        // draw
        if let Some(mut frame) = self.ui_context.platform.new_frame() {
            let mut canvas = frame.canvas();
            canvas.set_scale_factor(self.ui_context.scale_factor.get_fast());
            canvas.scale(self.ui_context.ui_scale);
            {
                let start = Instant::now();
                self.root.draw(&mut self.ui_context, &mut canvas);
                for overlay in self.overlays.iter().rev() {
                    overlay.borrow().draw(&mut self.ui_context, &mut canvas);
                }
                self.ui_context.draw_time_micros = (Instant::now() - start).as_micros();
            }
            self.ui_context.platform.end_frame(frame);
        }

        // calculate cpu time
        self.ui_context.frame_time_micros = (Instant::now() - start).as_micros();
    }

    fn update_hovered_component(tree: &mut Tree<ComponentState<A>>, pointer: &mut PointerState<A>) {
        let mut hovered_component = None;
        tree.traverse_depth(tree.get_root_id(), |id, state| {
            let contains_point = state.bounds.contains(&pointer.position);
            if state.visible && state.hoverable && !Cx::is_disabled(tree, id) && contains_point {
                hovered_component = Some(id);
            }
            state.visible && (!state.clips_children || contains_point)
        });
        pointer.hovered_component = hovered_component;
    }

    pub fn should_resize_to(&mut self) -> Option<IntPoint> {
        self.ui_context.resize.take()
    }

    pub fn on_event(&mut self, event: WindowEvent) -> EventStatus {
        match event {
            WindowEvent::Resized(size) => {
                self.last_window_size = size;
                let cx = &mut self.ui_context;
                cx.platform.set_view_size(size.into());
                let bounds = Rect::from_size(size.x as f32, size.y as f32);

                let scale = cx.app.get_ui_scale(size);
                cx.ui_bounds = bounds.with_scale(1.0 / scale);
                cx.ui_scale = scale;

                let scale_factor_changed = cx.scale_factor.set_if_changed(cx.platform.get_scale_factor());
                if scale_factor_changed || self.root.get_bounds(cx) != cx.ui_bounds {
                    self.root.layout(cx, cx.ui_bounds);
                }
            }
            WindowEvent::PointerMove { mut position, id } => {
                let cx = &mut self.ui_context;
                position = position / cx.ui_scale;

                let state = cx.pointer_state.entry(id).or_default();
                state.velocity = position - state.position;
                state.position = position;
                state.delta = state.position - state.down_position;
                state.delta_sum += state.delta;

                Self::update_hovered_component(&mut cx.tree, state);

                let state = self.ui_context.pointer_state[&id].clone();
                self.broadcast_event(&mut Event::PointerMove(state));
            }
            WindowEvent::PointerDown { mut position, button, id } => {
                let cx = &mut self.ui_context;
                position = position / cx.ui_scale;

                // update pointer state and hovered component
                let state = cx.pointer_state.entry(id).or_default();
                state.delta = point(0.0, 0.0);
                state.delta_sum = point(0.0, 0.0);
                state.button = button;
                state.position = position;
                state.down_position = state.position;
                state.down_time = Some(Instant::now());
                Self::update_hovered_component(&mut cx.tree, state);

                if let Some(hovered) = state.hovered_component {
                    state.pressed_component = Some(hovered);
                }

                // count number of clicks
                cx.num_clicks += 1;
                let now = Instant::now();
                let mouse_down_delta_ms = (now - cx.previous_pointer_down_time).as_millis();
                cx.previous_pointer_down_time = now;
                let movement_since_last_down = (cx.previous_pointer_down_position - position).len();
                cx.previous_pointer_down_position = position;
                if mouse_down_delta_ms > 300
                    || movement_since_last_down > 10.0
                    || state.hovered_component != cx.num_clicks_component
                {
                    cx.num_clicks = 1;
                }
                cx.num_clicks_component = state.hovered_component;

                let state = state.clone();

                // update focused component
                let mut focused = None;
                cx.tree.traverse_depth(cx.tree.get_root_id(), |id, s| {
                    if s.focusable && state.is_hovered(id) {
                        focused = Some(id);
                    }
                    true
                });
                if focused != cx.focused_component {
                    cx.focused_component = focused;
                    let event = &mut Event::FocusChanged(cx.focused_component.map(WeakComponentId));
                    self.broadcast_event(event);
                }

                // broadcast event
                self.broadcast_event(&mut Event::PointerDown(state));
            }
            WindowEvent::PointerUp { id } => {
                if let Some(state) = self.ui_context.pointer_state.get(&id).cloned() {
                    self.broadcast_event(&mut Event::PointerUp(state));
                }

                let cx = &mut self.ui_context;
                if let Some(state) = cx.pointer_state.get_mut(&id) {
                    state.pressed_component = None;
                    state.is_long_press = false;
                    state.down_time = None;
                    Self::update_hovered_component(&mut cx.tree, state);
                }

                if let PointerId::Touch(_) = id {
                    self.ui_context.pointer_state.remove(&id);
                }
            }
            WindowEvent::ScaleFactorChanged(scale_factor) => {
                self.ui_context.scale_factor.set(scale_factor);
            }
            WindowEvent::ModifiersChanged(mods) => {
                let cx = &mut self.ui_context;
                let mut event = Event::ModifiersChanged(Modifiers {
                    meta: cx.mods.meta != mods.meta,
                    shift: cx.mods.shift != mods.shift,
                    alt: cx.mods.alt != mods.alt,
                    ctrl: cx.mods.ctrl != mods.ctrl,
                });
                cx.mods.alt = mods.alt;
                cx.mods.meta = mods.meta;
                cx.mods.shift = mods.shift;
                cx.mods.ctrl = mods.ctrl;
                self.broadcast_event(&mut event);
            }
            WindowEvent::MouseWheel(delta) => {
                self.broadcast_event(&mut Event::MouseWheel(delta));
            }
            WindowEvent::FileHovered(path) => {
                self.broadcast_event(&mut Event::FileHovered(path));
                return EventStatus::Captured;
            }
            WindowEvent::FileDropped(path) => {
                self.broadcast_event(&mut Event::FileDropped(path));
                return EventStatus::Captured;
            }
            WindowEvent::FileDropCancelled => self.broadcast_event(&mut Event::FileDropCancelled),
            WindowEvent::Keydown(key) => {
                let mut event = Event::Keydown { key, captured: false };
                self.broadcast_event(&mut event);
                if let Event::Keydown { captured: true, .. } = event {
                    return EventStatus::Captured;
                }
            }
            WindowEvent::Keyup(key) => self.broadcast_event(&mut Event::Keyup(key)),
            WindowEvent::FocusChanged(is_focused) => {
                self.broadcast_event(&mut Event::WindowFocusChanged(is_focused));
                if !is_focused && self.ui_context.focused_component.is_some() {
                    self.ui_context.focused_component = None;
                    self.broadcast_event(&mut Event::FocusChanged(None));
                }
                if !is_focused {
                    self.ui_context.mods.alt = false;
                    self.ui_context.mods.meta = false;
                    self.ui_context.mods.shift = false;
                }
            }
        }
        EventStatus::Ignored
    }
}
