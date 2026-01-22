#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(target_os = "ios", path = "ios/mod.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
#[cfg_attr(target_family = "wasm", path = "web.rs")]
pub mod platform;

use crate::{Canvas, Later, WindowEvent};
use pallo_util::File;
pub use platform::*;
use std::{path::PathBuf, sync::Arc};

#[derive(Copy, Clone)]
pub enum InputType {
    Text,
    Number,
}

pub trait Clipboard {
    fn write_string(&mut self, text: impl Into<String>);
    fn write_data(&mut self, data: Vec<u8>);
    fn read_data(&self) -> Option<Vec<u8>>;
    fn read_string(&self) -> Option<String>;
    fn read_paths(&self) -> Option<Vec<PathBuf>>;
    fn read_audio(&self) -> Option<Vec<u8>>;
}

pub trait Frame {
    #[cfg(target_family = "wasm")]
    fn canvas(&mut self) -> Canvas;
    #[cfg(not(target_family = "wasm"))]
    fn canvas(&mut self) -> Canvas<'_>;
}

pub trait PlatformCommon {
    type Frame: Frame;
    fn open_url(&self, url: impl Into<String>);
    fn open_path_in_file_explorer(&self, path: PathBuf);
    fn file_open_dialog(&self, opts: FileOpenOptions);
    fn file_save_dialog(&self, options: FileSaveOptions);
    fn start_drag(&self, path: PathBuf);
    fn get_scale_factor(&self) -> f32;
    fn set_view_size(&mut self, size: (u32, u32));
    fn next_window_event(&mut self) -> Option<WindowEvent>;
    fn clipboard(&mut self) -> &mut impl Clipboard;
    fn documents_folder_path() -> Option<PathBuf>;
    fn open_prompt(
        &self,
        title: String,
        enter_text: String,
        value: String,
        input_type: InputType,
        result: &Later<String>,
    );
    fn new_frame(&mut self) -> Option<Self::Frame>;
    fn end_frame(&mut self, frame: Self::Frame);
}

pub struct FileOpenOptions {
    pub filetype_desc: String,
    pub extensions: Vec<String>,
    pub multi: bool,
    pub folder: bool,
    pub files: bool,
    pub result: Later<Vec<File>>,
}

pub struct FileSaveOptions {
    pub filename: String,
    pub filetype_desc: String,
    pub extension: String,
    pub mime_type: String,
    pub data: Arc<Vec<u8>>,
    pub result: Option<Later<PathBuf>>,
}
