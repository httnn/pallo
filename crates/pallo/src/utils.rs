use parking_lot::Mutex;

use crate::{App, Canvas, CanvasType, Cx, RasterSurfaceType, Rect, Surface};
use std::{
    any::Any,
    collections::VecDeque,
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum PointerId {
    Mouse,
    DragAndDrop,
    Touch(usize),
}

pub struct Output<T> {
    queue: VecDeque<T>,
}

impl<T> Default for Output<T> {
    fn default() -> Self {
        Self { queue: Default::default() }
    }
}

impl<T> Output<T> {
    pub fn next_output(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    pub fn handle_outputs(&mut self, mut cb: impl FnMut(T)) {
        while let Some(output) = self.next_output() {
            (cb)(output);
        }
    }

    pub fn add_output(&mut self, event: T) {
        self.queue.push_back(event);

        #[cfg(debug_assertions)]
        assert!(self.queue.len() < 32, "Quite a few events in this output queue, are you clearing it?");
    }
}

pub fn exp_decay<A: App>(cx: &Cx<A>, value: &mut f32, decay_ms: f32, target: f32) {
    if decay_ms <= 0.0 {
        *value = target;
        return;
    }

    let decay_factor = (-cx.frame_delta_ms / decay_ms).exp();
    *value = target + (*value - target) * decay_factor;
}

pub struct CachedCanvas {
    surface: Surface,
    bounds: Rect,
    dirty: AtomicBool,
}

impl Default for CachedCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl CachedCanvas {
    pub fn new() -> Self {
        Self { surface: Surface::new((1, 1).into(), 1.0), dirty: AtomicBool::new(false), bounds: Rect::default() }
    }

    pub fn layout<A: App>(&mut self, cx: &mut Cx<A>, bounds: Rect) {
        self.surface = Surface::new(bounds.size().to_int(), cx.scale_factor.get_fast());
    }

    pub fn mark_dirty(&mut self) {
        self.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn draw<A: App>(&self, cx: &mut Cx<A>, canvas: &mut Canvas, draw: impl FnOnce(&mut Cx<A>, &mut Canvas)) {
        if self.dirty.load(std::sync::atomic::Ordering::Relaxed) {
            let mut canvas = self.surface.get_canvas();
            canvas.set_scale_factor(cx.scale_factor.get_fast());
            (draw)(cx, &mut canvas);
            self.dirty.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        canvas.draw_surface(&self.surface, self.bounds.relative_point((0.0, 0.0)));
    }
}

pub struct Later<T> {
    value: Arc<Mutex<Option<T>>>,
    context: Arc<Mutex<Option<Box<dyn Any + Send>>>>,
}

impl<T> Clone for Later<T> {
    fn clone(&self) -> Self {
        Self { value: self.value.clone(), context: self.context.clone() }
    }
}

impl<T> Default for Later<T> {
    fn default() -> Self {
        Self { value: Arc::new(Mutex::new(None)), context: Arc::new(Mutex::new(None)) }
    }
}

impl<T> Later<T> {
    pub fn set(&self, value: T) {
        *self.value.lock() = Some(value);
    }

    pub fn value(&self) -> Option<T> {
        self.value.lock().take()
    }

    pub fn set_context<M: Send + 'static>(&self, meta: M) {
        *self.context.lock() = Some(Box::new(meta));
    }

    pub fn take_context<M: 'static>(&self) -> Option<Box<M>> {
        self.context.lock().take().and_then(|v| v.downcast().ok())
    }
}
