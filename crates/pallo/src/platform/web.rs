use crate::{
    App, Canvas, Component, ComponentId, Cx, EventStatus, File, IntPoint, JsCanvas, Later, Modifiers, MouseButton,
    PointerId, UI, WindowEvent,
    platform::{Clipboard, FileOpenOptions, FileSaveOptions, PlatformCommon},
    point,
};
use js_sys::Uint8Array;
use keyboard_types::Key;
use std::str::FromStr;
use std::{path::PathBuf, sync::Arc};
use wasm_bindgen::{JsValue, prelude::*};
use web_sys::window;

pub fn create_canvas<A: App, R: Component<A> + 'static>(
    init: A::AppInit,
    create_root: impl Fn(&mut Cx<A>, ComponentId) -> R + 'static,
) -> JsValue {
    let size = A::get_initial_size(&init);
    let ui = WebUI { ui: Box::new(UI::new(init, Platform::default(), &create_root)) };
    create_canvas_internal(ui, size.x as u32, size.y as u32)
}

fn convert_key(key: String) -> Key {
    Key::from_str(&key).unwrap_or(Key::Character(key))
}

impl<A: App> WebUIMethods for UI<A> {
    fn on_event_web(&mut self, event: WindowEvent) -> EventStatus {
        self.on_event(event)
    }

    fn get_view_web(&mut self) -> JsView {
        self.ui_context.platform.js_view.clone()
    }

    fn on_draw_web(&mut self) {
        self.draw();
    }

    fn set_frame(&mut self, frame: Frame) {
        self.ui_context.platform.set_frame(frame);
    }

    fn should_resize_to_web(&mut self) -> Option<IntPoint> {
        self.should_resize_to()
    }
}

trait WebUIMethods {
    fn on_event_web(&mut self, event: WindowEvent) -> EventStatus;
    fn on_draw_web(&mut self);
    fn get_view_web(&mut self) -> JsView;
    fn set_frame(&mut self, frame: Frame);
    fn should_resize_to_web(&mut self) -> Option<IntPoint>;
}

#[wasm_bindgen]
struct WebUI {
    ui: Box<dyn WebUIMethods>,
}

#[wasm_bindgen]
impl WebUI {
    pub fn on_resize(&mut self, width: usize, height: usize, scale_ratio: f32) {
        self.ui.on_event_web(WindowEvent::ScaleFactorChanged(scale_ratio));
        self.ui.on_event_web(WindowEvent::Resized((width as u32, height as u32).into()));
    }

    pub fn on_drag_over(&mut self, files: Vec<String>) {
        self.ui.on_event_web(WindowEvent::FileHovered(files));
    }

    pub fn on_drag_leave(&mut self) {
        self.ui.on_event_web(WindowEvent::FileDropCancelled);
    }

    pub fn on_file_dropped(&mut self, names: Vec<String>, files: Vec<Uint8Array>) {
        self.ui.on_event_web(WindowEvent::FileDropped(
            names
                .into_iter()
                .zip(files.into_iter())
                .map(|(name, data)| File::Data { name, data: data.to_vec().into() })
                .collect(),
        ));
    }

    pub fn get_view(&mut self) -> JsView {
        self.ui.get_view_web()
    }

    pub fn on_draw(&mut self, js_canvas: JsCanvas) {
        if let Some(point) = self.ui.should_resize_to_web() {
            self.ui.on_event_web(WindowEvent::Resized(point));
        }
        self.ui.set_frame(Frame { js_canvas: Some(js_canvas) });
        self.ui.on_draw_web();
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        self.ui.on_event_web(WindowEvent::PointerMove { position: point(x, y), id: PointerId::Mouse });
    }

    pub fn mouse_down(&mut self, x: f32, y: f32, context_menu: bool) {
        self.ui.on_event_web(WindowEvent::PointerDown {
            id: PointerId::Mouse,
            position: point(x, y),
            button: if context_menu {
                MouseButton::Right
            } else {
                MouseButton::Left
            },
        });
    }

    pub fn mouse_up(&mut self) {
        self.ui.on_event_web(WindowEvent::PointerUp { id: PointerId::Mouse });
    }

    pub fn mouse_wheel(&mut self, x: f32, y: f32) {
        self.ui.on_event_web(WindowEvent::MouseWheel(point(x, y)));
    }

    pub fn modifiers_changed(&mut self, meta: bool, shift: bool, alt: bool, ctrl: bool) {
        self.ui.on_event_web(WindowEvent::ModifiersChanged(Modifiers { ctrl, meta, shift, alt }));
    }

    pub fn key_down(&mut self, key: String) -> bool {
        matches!(self.ui.on_event_web(WindowEvent::Keydown(convert_key(key))), EventStatus::Captured)
    }

    pub fn key_up(&mut self, key: String) -> bool {
        matches!(self.ui.on_event_web(WindowEvent::Keyup(convert_key(key))), EventStatus::Captured)
    }

    pub fn focus(&mut self, focused: bool) {
        self.ui.on_event_web(WindowEvent::FocusChanged(focused));
    }
}

pub struct Frame {
    js_canvas: Option<JsCanvas>,
}

impl super::Frame for Frame {
    fn canvas(&mut self) -> Canvas {
        Canvas::new(self.js_canvas.take().expect("Canvas can only be taken once."))
    }
}

pub struct Platform {
    clipboard: WebClipboard,
    frame: Option<Frame>,
    js_view: JsView,
}

impl Default for Platform {
    fn default() -> Self {
        Self { clipboard: Default::default(), frame: Default::default(), js_view: JsView::new() }
    }
}

#[derive(Default)]
pub struct WebClipboard {
    data: Option<Vec<u8>>,
}

impl Clipboard for WebClipboard {
    fn write_string(&mut self, text: impl Into<String>) {
        let window = window().expect("should have a window in this context");
        let text: String = text.into();
        let _ = window.navigator().clipboard().write_text(&text);
    }

    fn write_data(&mut self, data: Vec<u8>) {
        // use js_sys::{Array, Uint8Array};
        // let window = window().expect("should have a window in this context");
        // let obj = js_sys::Object::new();
        // let _ = js_sys::Reflect::set(&obj, &"web application/octet-stream".into(), Blob::new(&*data).as_ref());
        // let _ = window
        //     .navigator()
        //     .clipboard()
        //     .write(&Array::of1(&web_sys::ClipboardItem::new_with_record_from_str_to_blob_promise(&obj).unwrap()));
        self.data = Some(data);
    }

    fn read_data(&self) -> Option<Vec<u8>> {
        self.data.clone()
    }

    fn read_string(&self) -> Option<String> {
        None
    }

    fn read_paths(&self) -> Option<Vec<PathBuf>> {
        None
    }

    fn read_audio(&self) -> Option<Vec<u8>> {
        None
    }
}

#[wasm_bindgen(module = "/src/platform/web_platform.js")]
extern "C" {
    fn create_canvas_internal(ui: WebUI, width: u32, height: u32) -> JsValue;

    fn save_file(filename: String, bytes: Vec<u8>, mime_type: String);
    fn get_file_input() -> web_sys::Element;
    fn trigger_file_input(closure: JsValue);

    type JsFile;

    #[wasm_bindgen(method)]
    fn get_name(this: &JsFile) -> String;

    #[wasm_bindgen(method)]
    fn get_data(this: &JsFile) -> Uint8Array;

    #[derive(Clone)]
    type JsView;

    #[wasm_bindgen(constructor)]
    fn new() -> JsView;

    #[wasm_bindgen(method)]
    fn resize(this: &JsView, widt: u32, height: u32);
}

impl Platform {
    pub fn set_frame(&mut self, frame: Frame) {
        self.frame = Some(frame);
    }
}

#[allow(unused)]
impl PlatformCommon for Platform {
    type Frame = Frame;

    fn get_scale_factor(&self) -> f32 {
        let window = web_sys::window().expect("should have a window in this context");
        window.device_pixel_ratio() as f32
    }

    fn clipboard(&mut self) -> &mut impl Clipboard {
        &mut self.clipboard
    }

    fn open_prompt(
        &self,
        title: String,
        enter_text: String,
        value: String,
        input_type: super::InputType,
        result: &Later<String>,
    ) {
        if let Ok(Some(value)) = window().unwrap().prompt_with_message_and_default(&title, &value) {
            result.set(value);
        }
    }

    fn documents_folder_path() -> Option<PathBuf> {
        None
    }

    fn open_path_in_file_explorer(&self, path: PathBuf) {}

    fn set_view_size(&mut self, size: (u32, u32)) {
        self.js_view.resize(size.0, size.1)
    }

    fn start_drag(&self, path: PathBuf) {}

    fn next_window_event(&mut self) -> Option<WindowEvent> {
        None
    }

    fn open_url(&self, url: impl Into<String>) {
        let window = web_sys::window().expect("should have a Window");
        let _ = window.open_with_url(&url.into());
    }

    fn file_open_dialog(&self, opts: FileOpenOptions) {
        let result = opts.result.clone();
        let input = get_file_input();
        input.set_attribute(
            "accept",
            &opts.extensions.into_iter().map(|s| format!(".{}", s.to_string())).collect::<Vec<String>>().join(","),
        );

        if opts.multi {
            input.set_attribute("multiple", "true");
        } else {
            input.remove_attribute("multiple");
        }

        if opts.folder {
            input.set_attribute("webkitdirectory", "true");
        } else {
            input.remove_attribute("webkitdirectory");
        }

        trigger_file_input(
            Closure::<dyn FnMut(Vec<JsFile>)>::new(move |files: Vec<JsFile>| {
                result.set(
                    files
                        .into_iter()
                        .map(|f| File::Data { name: f.get_name(), data: Arc::new(f.get_data().to_vec()) })
                        .collect(),
                );
            })
            .into_js_value(),
        );
    }

    fn file_save_dialog(&self, options: FileSaveOptions) {
        save_file(options.filename, options.data.to_vec(), options.mime_type);
    }

    fn new_frame(&mut self) -> Option<Self::Frame> {
        self.frame.take()
    }

    fn end_frame(&mut self, frame: Self::Frame) {}
}
