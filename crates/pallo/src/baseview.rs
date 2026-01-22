use std::sync::Arc;

use baseview::WindowHandle;
use baseview::{
    DropData, Event, EventStatus, MouseEvent, ScrollDelta, WindowEvent, WindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use keyboard_types::KeyState;
use nih_plug::editor::Editor;
use parking_lot::Mutex;
use ui::Modifiers;
use ui::UILike;

use crate::platform::{Platform, PlatformCommon};
use crate::{App, Component, ComponentId, Cx, IntPoint, Point, PointerId, UI, point, ui};

struct PalloWindowHandler<A: App> {
    ui: UI<A>,
    nih_ui_context: Arc<dyn nih_plug::prelude::GuiContext>,
    size: Arc<Mutex<IntPoint>>,
    mouse_pos: Point,
}

impl<A: App> PalloWindowHandler<A> {
    fn resize(&mut self) {
        let size = *self.size.lock();
        self.ui.on_event(ui::WindowEvent::Resized(size));
    }
}

impl From<crate::event::EventStatus> for baseview::EventStatus {
    fn from(val: crate::event::EventStatus) -> Self {
        match val {
            crate::EventStatus::Captured => EventStatus::Captured,
            crate::EventStatus::Ignored => EventStatus::Ignored,
        }
    }
}

impl<A: App> WindowHandler for PalloWindowHandler<A> {
    fn on_frame(&mut self, window: &mut baseview::Window) {
        self.ui.draw();

        if let Some(new_size) = self.ui.should_resize_to() {
            *self.size.lock() = new_size;
            if self.nih_ui_context.request_resize() {
                window.resize(baseview::Size::new(new_size.x as f64, new_size.y as f64));
                self.resize();
            }
        }
    }

    fn on_event(&mut self, window: &mut baseview::Window, event: Event) -> baseview::EventStatus {
        match event {
            Event::Mouse(event) => match event {
                MouseEvent::CursorMoved { position, modifiers: _ } => {
                    self.mouse_pos = point(position.x as f32, position.y as f32);
                    return self
                        .ui
                        .on_event(ui::WindowEvent::PointerMove { position: self.mouse_pos, id: PointerId::Mouse })
                        .into();
                }
                MouseEvent::ButtonPressed { button, modifiers: _ } => {
                    window.focus();
                    return self
                        .ui
                        .on_event(ui::WindowEvent::PointerDown {
                            position: self.mouse_pos,
                            button: match button {
                                baseview::MouseButton::Left => crate::MouseButton::Left,
                                baseview::MouseButton::Middle => crate::MouseButton::Middle,
                                baseview::MouseButton::Right => crate::MouseButton::Right,
                                _ => crate::MouseButton::Unknown,
                            },
                            id: PointerId::Mouse,
                        })
                        .into();
                }
                MouseEvent::ButtonReleased { button: _, modifiers: _ } => {
                    return self.ui.on_event(ui::WindowEvent::PointerUp { id: PointerId::Mouse }).into();
                }
                MouseEvent::WheelScrolled { delta: ScrollDelta::Pixels { x, y }, modifiers: _ } => {
                    return self.ui.on_event(ui::WindowEvent::MouseWheel(point(x, y))).into();
                }
                MouseEvent::DragEntered { position: _, modifiers: _, data: DropData::Files(files) } => {
                    if let crate::event::EventStatus::Captured = self.ui.on_event(ui::WindowEvent::FileHovered(
                        files.into_iter().filter_map(|f| f.to_str().map(|s| s.to_owned())).collect(),
                    )) {
                        return EventStatus::AcceptDrop(baseview::DropEffect::Copy);
                    }
                }
                MouseEvent::DragMoved { mut position, modifiers: _, data: DropData::Files(files) } => {
                    #[cfg(target_os = "macos")]
                    {
                        position.y = self.size.lock().y as f64 - position.y;
                    }

                    self.ui.on_event(ui::WindowEvent::PointerMove {
                        position: point(position.x as f32, position.y as f32),
                        id: PointerId::Mouse,
                    });
                    if let crate::event::EventStatus::Captured = self.ui.on_event(ui::WindowEvent::FileHovered(
                        files.into_iter().filter_map(|f| f.to_str().map(|s| s.to_owned())).collect(),
                    )) {
                        return EventStatus::AcceptDrop(baseview::DropEffect::Copy);
                    }
                }
                MouseEvent::DragLeft => {
                    if let crate::event::EventStatus::Captured = self.ui.on_event(ui::WindowEvent::FileDropCancelled) {
                        return EventStatus::AcceptDrop(baseview::DropEffect::Copy);
                    }
                }
                MouseEvent::DragDropped { position: _, modifiers: _, data: DropData::Files(files) } => {
                    if let crate::event::EventStatus::Captured =
                        self.ui.on_event(ui::WindowEvent::FileDropped(files.into_iter().map(|f| f.into()).collect()))
                    {
                        return EventStatus::AcceptDrop(baseview::DropEffect::Copy);
                    }
                }
                _ => {}
            },
            Event::Keyboard(event) => {
                self.ui.on_event(ui::WindowEvent::ModifiersChanged(Modifiers {
                    #[cfg(target_os = "windows")]
                    meta: event.modifiers.contains(keyboard_types::Modifiers::CONTROL),
                    #[cfg(target_os = "macos")]
                    meta: event.modifiers.contains(keyboard_types::Modifiers::META),
                    shift: event.modifiers.contains(keyboard_types::Modifiers::SHIFT),
                    alt: event.modifiers.contains(keyboard_types::Modifiers::ALT),
                    #[cfg(target_os = "macos")]
                    ctrl: event.modifiers.contains(keyboard_types::Modifiers::CONTROL),
                    #[cfg(target_os = "windows")]
                    ctrl: false,
                }));

                match event.state {
                    KeyState::Down => {
                        return self.ui.on_event(ui::WindowEvent::Keydown(event.key)).into();
                    }
                    KeyState::Up => {
                        return self.ui.on_event(ui::WindowEvent::Keyup(event.key)).into();
                    }
                }
            }
            Event::Window(event) => match event {
                WindowEvent::Focused => {
                    self.ui.on_event(ui::WindowEvent::FocusChanged(true));
                }
                WindowEvent::Unfocused => {
                    self.ui.on_event(ui::WindowEvent::FocusChanged(false));
                }
                _ => {}
            },
        }
        EventStatus::Ignored
    }
}

struct Instance {
    handle: WindowHandle,
}

impl Drop for Instance {
    fn drop(&mut self) {
        self.handle.close();
    }
}

unsafe impl Send for Instance {}

pub struct PalloEditor<A: App, R: Component<A>> {
    size: Arc<Mutex<IntPoint>>,
    create_root: Box<dyn Fn(&mut Cx<A>, ComponentId, Arc<dyn nih_plug::prelude::GuiContext>) -> R + Send>,
    init: A::AppInit,
}

impl<A: App, R: Component<A>> PalloEditor<A, R> {
    pub fn new(
        init: A::AppInit,
        create_root: impl Fn(&mut Cx<A>, ComponentId, Arc<dyn nih_plug::prelude::GuiContext>) -> R + Send + 'static,
    ) -> Self {
        let initial_size = A::get_initial_size(&init);
        Self { init, create_root: Box::new(create_root), size: Arc::new(Mutex::new(initial_size)) }
    }
}

impl<A: App, R: Component<A> + 'static> Editor for PalloEditor<A, R> {
    fn spawn(
        &self,
        parent: nih_plug::prelude::ParentWindowHandle,
        nih_ui_context: Arc<dyn nih_plug::prelude::GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        if let Some(platform) = Platform::new(parent) {
            let scale_factor = platform.get_scale_factor();

            let nih_ctx = nih_ui_context.clone();

            let ui = UI::new(self.init.clone(), platform, move |cx, root_id| {
                (self.create_root)(cx, root_id, nih_ctx.clone())
            });

            let draw_size = point(self.size().0 as f32, self.size().1 as f32);
            let options = WindowOpenOptions {
                scale: WindowScalePolicy::ScaleFactor(scale_factor as f64),
                size: baseview::Size { width: draw_size.x as f64, height: draw_size.y as f64 },
                title: "Plug-in".to_owned(),
            };

            let size = self.size.clone();
            let window_handle = baseview::Window::open_parented(&parent, options, move |_| {
                let mut handler = PalloWindowHandler { ui, nih_ui_context, size, mouse_pos: point(-1.0, -1.0) };
                handler.resize();
                handler
            });
            return Box::new(Instance { handle: window_handle });
        }
        panic!("invalid window handle");
    }

    fn size(&self) -> (u32, u32) {
        (*self.size.lock()).into()
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        // self.ui.on_event(ui::WindowEvent::ScaleFactorChanged(factor));
        true
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}

    fn param_values_changed(&self) {}
}
