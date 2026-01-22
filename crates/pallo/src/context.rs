use rustc_hash::FxHashMap;
use std::{cell::RefCell, collections::VecDeque, ops::Deref, rc::Rc};
use web_time::Instant;

use crate::{
    Animations, AnyEvent, App, CanvasType, Component, Event, IntPoint, Modifiers, Overlay, Point,
    PointerId, PointerState, Property, PropertyId, RasterSurfaceType, Rect, Signal, SignalCx,
    Surface,
    component::{ComponentId, ComponentState, WeakComponentId},
    platform::Platform,
    renderers::{self, RendererType, renderer::Renderer},
    tree::{NodeId, Tree},
};

pub struct Cx<A: App> {
    pub(crate) tree: Tree<ComponentState<A>>,
    pub(crate) component_ids: Vec<ComponentId>,
    pub focused_component: Option<NodeId>,
    pub animations: Animations,
    pub(crate) pointer_state: FxHashMap<PointerId, PointerState<A>>,
    pub(crate) input: VecDeque<Event<A>>,
    pub app: A,
    pub frame_time_micros: u128,
    pub draw_time_micros: u128,
    pub update_time_micros: u128,
    pub(crate) backend: Renderer,
    pub mods: Modifiers,
    pub frame_delta_ms: f32,
    pub scale_factor: Signal<f32>,
    pub ui_scale: f32,
    pub(crate) resize: Option<IntPoint>,
    pub ui_bounds: Rect,
    pub(crate) overlays: Vec<Overlay<dyn Component<A>>>,
    signal_cx: SignalCx,
    pub num_clicks: usize,
    pub(crate) num_clicks_component: Option<NodeId>,
    pub(crate) previous_pointer_down_position: Point,
    pub(crate) previous_pointer_down_time: Instant,
    pub num_frames: u64,
    pub platform: Platform,
}

impl<A: App> Cx<A> {
    pub fn new(init: A::AppInit, platform: Platform) -> Self {
        let signal_cx = SignalCx::new();
        let app = A::new(&signal_cx, init);
        Self {
            tree: Default::default(),
            component_ids: vec![],
            focused_component: None,
            pointer_state: FxHashMap::default(),
            animations: Animations::default(),
            input: Default::default(),
            app,
            frame_time_micros: 0,
            draw_time_micros: 0,
            update_time_micros: 0,
            backend: Default::default(),
            mods: Default::default(),
            frame_delta_ms: 0.0,
            scale_factor: signal_cx.signal(1.0),
            resize: None,
            ui_bounds: Default::default(),
            overlays: vec![],
            signal_cx,
            num_frames: 0,
            ui_scale: 1.0,
            num_clicks: 0,
            num_clicks_component: None,
            previous_pointer_down_time: Instant::now(),
            previous_pointer_down_position: Point::new(0.0, 0.0),
            platform,
        }
    }

    pub fn add_font(&mut self, id: A::FontId, data: &[u8]) {
        self.backend.add_typeface(id, data)
    }

    pub fn send_event(&mut self, event: Event<A>) {
        self.input.push_back(event);
    }

    pub fn send_app_event(&mut self, event: A::Input) {
        self.input.push_back(Event::App(event));
    }

    pub fn send_any_event<T: 'static>(&mut self, data: T) {
        self.input.push_back(Event::Any(AnyEvent(Box::new(data))));
    }

    pub(crate) fn add_child<T>(
        &mut self,
        parent_id: impl Into<NodeId>,
        add: impl FnOnce(&mut Cx<A>, ComponentId) -> T,
    ) -> T {
        let id = self.add_child_id(parent_id);
        (add)(self, id)
    }

    pub fn add_child_id(&mut self, parent_id: impl Into<NodeId>) -> ComponentId {
        let id = ComponentId(Rc::new(self.tree.add(parent_id.into())));
        self.component_ids.push(id.clone());
        id
    }

    pub(crate) fn is_visible(&self, id: impl Into<NodeId>) -> bool {
        let mut node_id = Some(id.into());
        while let Some(id) = node_id {
            if !self.tree.get(id).visible {
                return false;
            }
            node_id = self.tree.get_parent(id);
        }
        true
    }

    pub(crate) fn is_focused(&self, id: impl Into<NodeId>) -> bool {
        if let Some(h) = &self.focused_component {
            return id.into() == *h;
        }
        false
    }

    pub(crate) fn notify_size_changed(&mut self, id: impl Into<NodeId>) {
        let mut node_id = Some(id.into());
        while let Some(id) = node_id {
            self.tree.get_mut(id).needs_relayout = true;
            node_id = self.tree.get_parent(id);
        }
    }

    pub(crate) fn set_needs_relayout(&mut self, id: impl Into<NodeId>, value: bool) {
        self.tree.traverse_depth_mut(id.into(), |_id, state| {
            state.needs_relayout = value;
            true
        });
    }

    pub(crate) fn needs_relayout(&mut self, id: impl Into<NodeId>) -> bool {
        let id: NodeId = id.into();
        self.is_visible(id) && self.tree.get(id).needs_relayout
    }

    pub fn contains_child(&self, parent: impl Into<NodeId>, child: impl Into<NodeId>) -> bool {
        let parent: NodeId = parent.into();
        let mut child: Option<NodeId> = Some(child.into());
        while let Some(id) = child {
            if id == parent {
                return true;
            }
            child = self.tree.get_parent(id);
        }
        false
    }

    pub(crate) fn is_disabled(tree: &Tree<ComponentState<A>>, id: impl Into<NodeId>) -> bool {
        let mut node_id = Some(id.into());
        while let Some(id) = node_id {
            if tree.get(id).disabled {
                return true;
            }
            node_id = tree.get_parent(id);
        }
        false
    }

    pub fn get_hovered_id(&self, pointer_id: PointerId) -> Option<WeakComponentId> {
        self.pointer_state
            .get(&pointer_id)
            .and_then(|p| p.hovered_component.map(WeakComponentId))
    }

    pub fn get_focused_id(&self) -> Option<WeakComponentId> {
        self.focused_component.map(WeakComponentId)
    }

    pub(crate) fn is_hovered_any(&self, id: impl Into<NodeId>) -> bool {
        let id: NodeId = id.into();
        if let Some(p) = self
            .pointer_state
            .values()
            .find(|p| p.hovered_component == Some(id))
        {
            if p.pressed_component.is_some() {
                return p.pressed_component == Some(id);
            }
            return true;
        }
        false
    }

    pub(crate) fn is_hovered_ignoring_pressed_any(&self, id: impl Into<NodeId>) -> bool {
        let id: NodeId = id.into();
        self.pointer_state
            .values()
            .any(|p| p.hovered_component == Some(id))
    }

    pub(crate) fn is_pressed_any(&self, id: impl Into<NodeId>) -> bool {
        let id: NodeId = id.into();
        self.pointer_state
            .values()
            .any(|p| p.pressed_component == Some(id))
    }

    pub fn get_bounds(&self, id: impl Into<NodeId>) -> Rect {
        self.tree.get(id.into()).bounds
    }

    pub(crate) fn set_bounds(&mut self, id: impl Into<NodeId>, bounds: Rect) {
        self.tree.get_mut(id.into()).bounds = bounds;
    }

    pub(crate) fn set_visible(&mut self, c: impl Into<NodeId>, visible: bool) {
        self.tree.get_mut(c.into()).visible = visible;
    }

    pub(crate) fn set_disabled(&mut self, id: impl Into<NodeId>, disabled: bool) {
        self.tree.get_mut(id.into()).disabled = disabled;
    }

    pub(crate) fn set_interactive(&mut self, id: impl Into<NodeId>, interactive: bool) {
        let id: NodeId = id.into();
        self.set_hoverable(id, interactive);
        self.set_focusable(id, interactive);
    }

    pub(crate) fn set_hoverable(&mut self, id: impl Into<NodeId>, hoverable: bool) {
        self.tree.get_mut(id.into()).hoverable = hoverable;
    }

    pub(crate) fn set_clips_children(&mut self, id: impl Into<NodeId>, value: bool) {
        self.tree.get_mut(id.into()).clips_children = value;
    }

    pub(crate) fn set_focusable(&mut self, id: impl Into<NodeId>, focusable: bool) {
        self.tree.get_mut(id.into()).focusable = focusable;
    }

    pub fn get_component_state(&self, id: impl Into<NodeId>) -> &A::ComponentState {
        &self.tree.get(id.into()).app_state
    }

    pub fn get_component_state_mut(&mut self, id: impl Into<NodeId>) -> &mut A::ComponentState {
        &mut self.tree.get_mut(id.into()).app_state
    }

    pub fn find_component_id(
        &mut self,
        predicate: impl Fn(&A::ComponentState) -> bool,
    ) -> Option<WeakComponentId> {
        let mut out = None;
        self.tree
            .traverse_depth(self.tree.get_root_id(), |id, state| {
                if out.is_some() {
                    false
                } else if (predicate)(&state.app_state) {
                    out = Some(WeakComponentId(id));
                    false
                } else {
                    true
                }
            });
        out
    }

    pub(crate) fn set_focus(&mut self, id: Option<impl Into<NodeId>>) {
        if let Some(id) = id {
            self.focused_component = Some(id.into());
        } else {
            self.focused_component = None;
        }
        self.input.push_back(Event::FocusChanged(
            self.focused_component.map(WeakComponentId),
        ));
    }

    pub fn focus_next(&mut self) {
        let mut next = false;
        let mut node = None;
        self.tree
            .traverse_depth(self.tree.get_root_id(), |id, state| {
                if next
                    && self.is_visible(id)
                    && !state.disabled
                    && state.focusable
                    && node.is_none()
                {
                    node = Some(id);
                } else if let Some(focused_id) = self.focused_component
                    && focused_id == id
                {
                    next = true;
                }
                true
            });
        if let Some(node) = node {
            self.set_focus(Some(node));
        }
    }

    pub fn unfocus(&mut self) {
        self.set_focus(None as Option<NodeId>);
    }

    pub fn resize(&mut self, size: impl Into<IntPoint>) {
        self.resize = Some(size.into());
    }

    pub fn add_overlay<C: Component<A> + 'static>(
        &mut self,
        add_component: impl FnOnce(&mut Cx<A>, ComponentId) -> C,
    ) -> Overlay<C> {
        let id = ComponentId(Rc::new(self.tree.add(self.tree.get_root_id())));
        let component = Rc::new(RefCell::new((add_component)(self, id.clone())));
        self.component_ids.push(id);
        self.overlays.push(component.clone());
        component
    }

    pub fn move_to_front(&mut self, id: impl Into<NodeId>) {
        let id: NodeId = id.into();
        if let Some(parent_id) = self.tree.get_parent(id) {
            let children = self.tree.get_children_mut(parent_id);
            children.retain(|i| *i != id);
            children.push(id);
        }
    }

    pub fn main_pointer(&self) -> PointerState<A> {
        self.pointer_state
            .get(&PointerId::Mouse)
            .cloned()
            .unwrap_or_else(|| {
                self.pointer_state
                    .values()
                    .next()
                    .cloned()
                    .unwrap_or(PointerState::default())
            })
    }

    pub fn get_pointer(&self, id: PointerId) -> Option<&PointerState<A>> {
        self.pointer_state.get(&id)
    }

    pub fn pointer_if_hovered(&mut self, c: &ComponentId) -> Option<&mut PointerState<A>> {
        self.pointer_state.values_mut().find(|p| {
            if p.hovered_component == Some(c.into()) {
                if p.pressed_component.is_some() {
                    return p.pressed_component == Some(c.into());
                }
                return true;
            }
            false
        })
    }

    pub fn mock_modifiers(&mut self, mods: Modifiers) {
        let event: Event<A> = Event::ModifiersChanged(Modifiers {
            meta: self.mods.meta != mods.meta,
            shift: self.mods.shift != mods.shift,
            alt: self.mods.alt != mods.alt,
            ctrl: self.mods.ctrl != mods.ctrl,
        });
        self.mods = mods.clone();
        self.send_event(event);
    }

    pub fn set_root_property(&mut self, id: PropertyId, value: Property) {
        self.set_property(self.tree.get_root_id(), id, value);
    }

    pub fn set_property(&mut self, node_id: impl Into<NodeId>, id: PropertyId, value: Property) {
        let node_id = node_id.into();
        self.tree.get_mut(node_id).properties.set(id, value);
        self.tree.traverse_depth_mut(node_id, |_, state| {
            state.properties.set_dirty(id, true);
            true
        });
    }

    pub fn get_changed_property(
        &mut self,
        node_id: impl Into<NodeId>,
        id: PropertyId,
    ) -> Option<Property> {
        let node_id = node_id.into();
        if self.tree.get(node_id).properties.is_dirty(id) {
            self.tree.get_mut(node_id).properties.set_dirty(id, false);
            Some(self.get_property(node_id, id).clone())
        } else {
            None
        }
    }

    fn get_property(&self, node_id: impl Into<NodeId>, prop_id: PropertyId) -> &Property {
        let mut node_id = Some(node_id.into());
        while let Some(id) = node_id {
            if let Some(p) = self.tree.get(id).properties.get(prop_id) {
                return p;
            }
            node_id = self.tree.get_parent(id);
        }
        panic!()
    }
}

impl<A: App> Deref for Cx<A> {
    type Target = SignalCx;

    fn deref(&self) -> &Self::Target {
        &self.signal_cx
    }
}
