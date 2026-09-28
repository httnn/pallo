use crate::{App, Point, PointerId, UI, point, ui};
use baseview::dpi::{PhysicalSize, Size};
use baseview::{DropData, Event, EventStatus, MouseEvent, ScrollDelta, WindowContext, WindowEvent, WindowHandler};
use keyboard_types::KeyState;
use std::cell::RefCell;
use ui::Modifiers;

pub struct PalloWindowHandler<A: App> {
    ui: RefCell<UI<A>>,
    window_context: WindowContext,
    mouse_pos: RefCell<Point>,
}

impl<A: App> PalloWindowHandler<A> {
    pub fn new(ui: UI<A>, window_context: WindowContext) -> Self {
        Self { ui: RefCell::new(ui), window_context, mouse_pos: RefCell::new(Point::default()) }
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
    fn on_frame(&self) -> core::result::Result<(), baseview::HandlerError> {
        self.ui.borrow_mut().draw();

        if let Some(new_size) = self.ui.borrow_mut().should_resize_to() {
            let _ = self.window_context.resize(Size::Physical(PhysicalSize::new(new_size.x as u32, new_size.y as u32)));
        }
        Ok(())
    }

    fn on_event(&self, event: Event) -> baseview::EventStatus {
        let mut ui = self.ui.borrow_mut();
        match event {
            Event::Mouse(event) => match event {
                MouseEvent::CursorMoved { position, modifiers: _ } => {
                    *self.mouse_pos.borrow_mut() = point(position.x as f32, position.y as f32);
                    return ui
                        .on_event(ui::WindowEvent::PointerMove {
                            position: *self.mouse_pos.borrow(),
                            id: PointerId::Mouse,
                        })
                        .into();
                }
                MouseEvent::ButtonPressed { button, modifiers: _ } => {
                    let _ = self.window_context.focus();
                    return ui
                        .on_event(ui::WindowEvent::PointerDown {
                            position: *self.mouse_pos.borrow(),
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
                    return ui.on_event(ui::WindowEvent::PointerUp { id: PointerId::Mouse }).into();
                }
                MouseEvent::WheelScrolled { delta: ScrollDelta::Pixels { x, y }, modifiers: _ } => {
                    return ui.on_event(ui::WindowEvent::MouseWheel(point(x, y))).into();
                }
                MouseEvent::DragEntered { position: _, modifiers: _, data: DropData::Files(files) } => {
                    if let crate::event::EventStatus::Captured = ui.on_event(ui::WindowEvent::FileHovered(
                        files.into_iter().filter_map(|f| f.to_str().map(|s| s.to_owned())).collect(),
                    )) {
                        return EventStatus::AcceptDrop(baseview::DropEffect::Copy);
                    }
                }
                MouseEvent::DragMoved { mut position, modifiers: _, data: DropData::Files(files) } => {
                    #[cfg(target_os = "macos")]
                    {
                        position.y = self.window_context.size().physical.height as f64 - position.y;
                    }

                    ui.on_event(ui::WindowEvent::PointerMove {
                        position: point(position.x as f32, position.y as f32),
                        id: PointerId::Mouse,
                    });
                    if let crate::event::EventStatus::Captured = ui.on_event(ui::WindowEvent::FileHovered(
                        files.into_iter().filter_map(|f| f.to_str().map(|s| s.to_owned())).collect(),
                    )) {
                        return EventStatus::AcceptDrop(baseview::DropEffect::Copy);
                    }
                }
                MouseEvent::DragLeft => {
                    if let crate::event::EventStatus::Captured = ui.on_event(ui::WindowEvent::FileDropCancelled) {
                        return EventStatus::AcceptDrop(baseview::DropEffect::Copy);
                    }
                }
                MouseEvent::DragDropped { position: _, modifiers: _, data: DropData::Files(files) } => {
                    if let crate::event::EventStatus::Captured =
                        ui.on_event(ui::WindowEvent::FileDropped(files.into_iter().map(|f| f.into()).collect()))
                    {
                        return EventStatus::AcceptDrop(baseview::DropEffect::Copy);
                    }
                }
                _ => {}
            },
            Event::Keyboard(event) => {
                ui.on_event(ui::WindowEvent::ModifiersChanged(Modifiers {
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
                        return ui.on_event(ui::WindowEvent::Keydown(event.key)).into();
                    }
                    KeyState::Up => {
                        return ui.on_event(ui::WindowEvent::Keyup(event.key)).into();
                    }
                }
            }
            Event::Window(event) => match event {
                WindowEvent::Focused => {
                    ui.on_event(ui::WindowEvent::FocusChanged(true));
                }
                WindowEvent::Unfocused => {
                    ui.on_event(ui::WindowEvent::FocusChanged(false));
                }
                _ => {}
            },
            _ => {}
        }
        EventStatus::Ignored
    }

    fn resized(&self, new_size: baseview::WindowSize) -> core::result::Result<(), baseview::HandlerError> {
        self.ui
            .borrow_mut()
            .on_event(ui::WindowEvent::Resized((new_size.physical.width, new_size.physical.height).into()));
        Ok(())
    }
}
