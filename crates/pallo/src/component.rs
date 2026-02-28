use std::{cell::RefCell, rc::Rc};

use crate::{App, Canvas, Cx, Event, Grid, PointerState, Property, PropertyId, PropertyStore, Rect, tree::NodeId};

pub struct ComponentState<A: App> {
    pub(crate) visible: bool,
    pub(crate) focusable: bool,
    pub(crate) hoverable: bool,
    pub(crate) disabled: bool,
    pub(crate) bounds: Rect,
    pub(crate) clips_children: bool,
    pub(crate) needs_relayout: bool,
    pub(crate) app_state: A::ComponentState,
    pub(crate) properties: PropertyStore,
}

impl<A: App> Default for ComponentState<A> {
    fn default() -> Self {
        Self {
            visible: true,
            focusable: false,
            disabled: false,
            hoverable: false,
            clips_children: true,
            bounds: Rect::default(),
            needs_relayout: false,
            properties: PropertyStore::default(),
            app_state: A::ComponentState::default(),
        }
    }
}

macro_rules! component_methods {
    ($get_id:ident) => {
        #[inline]
        fn set_bounds(&self, cx: &mut Cx<A>, bounds: Rect) {
            cx.set_bounds(self.$get_id(), bounds);
        }

        #[inline]
        fn get_bounds(&self, cx: &Cx<A>) -> Rect {
            cx.get_bounds(self.$get_id())
        }

        #[inline]
        fn notify_size_changed(&self, cx: &mut Cx<A>) {
            cx.notify_size_changed(self.$get_id());
        }

        #[inline]
        fn set_disabled(&self, cx: &mut Cx<A>, disabled: bool) {
            cx.set_disabled(self.$get_id(), disabled);
        }

        #[inline]
        fn set_visible(&self, cx: &mut Cx<A>, visible: bool) {
            cx.set_visible(self.$get_id(), visible);
        }

        #[inline]
        fn focus(&self, cx: &mut Cx<A>) {
            cx.set_focus(Some(self.$get_id()));
        }

        #[inline]
        fn is_hovered(&self, pointer: &PointerState<A>) -> bool {
            pointer.is_hovered(self.$get_id())
        }

        #[inline]
        fn is_hovered_any(&self, cx: &Cx<A>) -> bool {
            cx.is_hovered_any(self.$get_id())
        }

        #[inline]
        fn is_pressed_any(&self, cx: &Cx<A>) -> bool {
            cx.is_pressed_any(self.$get_id())
        }

        #[inline]
        fn is_hovered_ignoring_pressed(&self, pointer: &PointerState<A>) -> bool {
            pointer.is_hovered_ignoring_pressed(self.$get_id())
        }

        #[inline]
        fn is_hovered_ignoring_pressed_any(&self, cx: &Cx<A>) -> bool {
            cx.is_hovered_ignoring_pressed_any(self.$get_id())
        }

        #[inline]
        fn is_visible(&self, cx: &Cx<A>) -> bool {
            cx.is_visible(self.$get_id())
        }

        #[inline]
        fn is_pressed(&self, pointer: &PointerState<A>) -> bool {
            pointer.is_pressed(self.$get_id())
        }

        #[inline]
        fn is_disabled(&self, cx: &Cx<A>) -> bool {
            Cx::is_disabled(&cx.tree, self.$get_id())
        }

        #[inline]
        fn is_focused(&self, cx: &Cx<A>) -> bool {
            cx.is_focused(self.$get_id())
        }

        #[inline]
        fn set_interactive(&self, cx: &mut Cx<A>, interactive: bool) {
            cx.set_interactive(self.$get_id(), interactive)
        }

        #[inline]
        fn set_hoverable(&self, cx: &mut Cx<A>, hoverable: bool) {
            cx.set_hoverable(self.$get_id(), hoverable);
        }

        #[inline]
        fn set_clips_children(&self, cx: &mut Cx<A>, value: bool) {
            cx.set_clips_children(self.$get_id(), value);
        }

        #[inline]
        fn state_mut<'a>(&'a self, cx: &'a mut Cx<A>) -> &'a mut A::ComponentState {
            cx.get_component_state_mut(self.$get_id())
        }

        #[inline]
        fn set_property(&self, cx: &mut Cx<A>, id: PropertyId, value: Property) {
            cx.set_property(self.$get_id(), id, value);
        }

        #[inline]
        fn get_changed_property(&self, cx: &mut Cx<A>, id: PropertyId) -> Option<Property> {
            cx.get_changed_property(self.$get_id(), id)
        }

        #[inline]
        fn move_to_front(&self, cx: &mut Cx<A>) {
            cx.move_to_front(self.$get_id());
        }

        #[inline]
        fn interactive(self, cx: &mut Cx<A>) -> Self
        where
            Self: Sized,
        {
            self.set_interactive(cx, true);
            self
        }

        #[inline]
        fn hoverable(self, cx: &mut Cx<A>) -> Self
        where
            Self: Sized,
        {
            self.set_hoverable(cx, true);
            self
        }

        #[inline]
        fn hidden(self, cx: &mut Cx<A>) -> Self
        where
            Self: Sized,
        {
            self.set_visible(cx, false);
            self
        }
    };
}

pub trait Component<A: App> {
    fn layout(&mut self, cx: &mut Cx<A>, bounds: Rect);
    fn id(&self) -> &ComponentId;

    #[allow(unused_variables)]
    fn draw_children(&self, cx: &mut Cx<A>, canvas: &mut Canvas) {}

    fn draw(&self, cx: &mut Cx<A>, canvas: &mut Canvas) {
        self.draw_children(cx, canvas);
    }

    fn relayout(&mut self, cx: &mut Cx<A>) {
        self.layout(cx, self.get_bounds(cx));
    }

    #[allow(unused_variables)]
    fn event_children(&mut self, cx: &mut Cx<A>, event: &mut Event<A>) {}

    #[allow(unused_variables)]
    fn event(&mut self, cx: &mut Cx<A>, event: &mut Event<A>) {
        self.event_children(cx, event);
    }

    #[allow(unused_variables)]
    fn get_preferred_size(&mut self, cx: &mut Cx<A>, parent_bounds: Rect) -> (Option<f32>, Option<f32>) {
        (None, None)
    }

    fn relayout_if_necessary(&mut self, cx: &mut Cx<A>) {
        self.relayout_if_necessary_with_parent(cx, self.id().weak());
    }

    fn relayout_if_necessary_with_parent(&mut self, cx: &mut Cx<A>, id: WeakComponentId) {
        if cx.needs_relayout(id) {
            self.relayout(cx);
            cx.set_needs_relayout(id, false);
        }
    }

    component_methods!(id);
}

pub type Overlay<T> = Rc<RefCell<T>>;

pub trait NodeIdLike<A: App> {
    fn node_id(&self) -> NodeId;

    fn add_child<T>(&self, cx: &mut Cx<A>, add_child: impl FnOnce(&mut Cx<A>, ComponentId) -> T) -> T {
        cx.add_child(self.node_id(), add_child)
    }

    component_methods!(node_id);
}

#[derive(Clone, PartialEq)]
pub struct ComponentId(pub(crate) Rc<NodeId>);

impl ComponentId {
    pub fn weak(&self) -> WeakComponentId {
        WeakComponentId(*self.0)
    }

    pub fn grid<A: App>(&self) -> Grid<'_, A> {
        Grid::id(self)
    }
}

impl<A: App> NodeIdLike<A> for ComponentId {
    fn node_id(&self) -> NodeId {
        *self.0
    }
}

impl PartialEq<NodeId> for ComponentId {
    fn eq(&self, other: &NodeId) -> bool {
        *self.0 == *other
    }
}

impl From<&ComponentId> for NodeId {
    fn from(val: &ComponentId) -> Self {
        *val.0
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Copy)]
pub struct WeakComponentId(pub(crate) NodeId);

impl<A: App> NodeIdLike<A> for WeakComponentId {
    fn node_id(&self) -> NodeId {
        self.0
    }
}

impl From<WeakComponentId> for NodeId {
    fn from(val: WeakComponentId) -> Self {
        val.0
    }
}
