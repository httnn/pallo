use crate::{Modifiers, Point, component::WeakComponentId, point, tree::NodeId, ui::App};
use keyboard_types::Key;
use pallo_util::File;
use std::{any::Any, marker::PhantomData};
use web_time::Instant;

pub struct AnyEvent(pub(crate) Box<dyn Any>);

impl AnyEvent {
    pub fn map<M, F>(&mut self, f: F)
    where
        M: Any + Send,
        F: FnOnce(&M),
    {
        if let Some(data) = self.0.as_ref().downcast_ref() {
            (f)(data);
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        if let Some(data) = self.0.as_ref().downcast_ref() {
            Some(data)
        } else {
            None
        }
    }
}

pub enum EventStatus {
    Captured,
    Ignored,
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    #[default]
    Unknown,
}

pub struct PointerState<A: App> {
    pub position: Point,
    pub down_position: Point,
    pub down_time: Option<Instant>,
    pub velocity: Point,
    pub delta: Point,
    pub delta_sum: Point,
    pub button: MouseButton,
    pub hovered_component: Option<NodeId>,
    pub pressed_component: Option<NodeId>,
    pub is_long_press: bool,
    pub _p: PhantomData<A>,
}

impl<A: App> Clone for PointerState<A> {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            down_position: self.down_position,
            down_time: self.down_time,
            velocity: self.velocity,
            delta: self.delta,
            delta_sum: self.delta_sum,
            button: self.button,
            hovered_component: self.hovered_component,
            pressed_component: self.pressed_component,
            is_long_press: self.is_long_press,
            _p: self._p,
        }
    }
}

impl<A: App> PointerState<A> {
    pub fn reset_delta(&mut self) {
        self.delta = point(0.0, 0.0);
        self.down_position = self.position;
    }

    pub fn is_pressed(&self, id: impl Into<NodeId>) -> bool {
        let id: NodeId = id.into();
        self.pressed_component.map(|p| id == p).unwrap_or(false)
    }

    pub fn is_hovered(&self, id: impl Into<NodeId>) -> bool {
        let id: NodeId = id.into();
        if let Some(p) = self.hovered_component {
            if self.pressed_component.is_some() {
                return self.pressed_component == Some(id);
            }
            return p == id;
        }
        false
    }

    pub fn is_hovered_ignoring_pressed(&self, id: impl Into<NodeId>) -> bool {
        let id: NodeId = id.into();
        self.hovered_component == Some(id)
    }
}

impl<A: App> Default for PointerState<A> {
    fn default() -> Self {
        Self {
            position: Default::default(),
            down_position: Default::default(),
            down_time: None,
            velocity: Default::default(),
            delta: Default::default(),
            delta_sum: Default::default(),
            button: Default::default(),
            hovered_component: Default::default(),
            pressed_component: Default::default(),
            is_long_press: false,
            _p: PhantomData,
        }
    }
}

pub enum Event<A: App> {
    Update,
    PointerDown(PointerState<A>),
    PointerUp(PointerState<A>),
    PointerMove(PointerState<A>),
    LongPress(PointerState<A>),
    App(A::Input),
    ModifiersChanged(Modifiers),
    MouseWheel(Point),
    FocusChanged(Option<WeakComponentId>),
    FileDropped(Vec<File>),
    FileHovered(Vec<String>),
    FileDropCancelled,
    Keydown { key: Key, captured: bool },
    Keyup(Key),
    WindowFocusChanged(bool),
    Any(AnyEvent),
}

impl<A: App> Event<A> {
    pub fn update(&self) -> bool {
        matches!(self, Self::Update)
    }
}
