pub mod animation;

// #[cfg(any(target_os = "macos", target_os = "windows"))]
// pub mod baseview;

pub mod platform;

pub mod color;
pub mod component;
pub mod components;
pub mod context;
pub mod event;
pub mod geometry;
pub mod layer;
pub mod layout;
pub mod properties;
pub mod renderers;
pub mod signal;
pub mod svg;
mod tree;
pub mod ui;
pub mod utils;

pub use crate::{
    animation::*,
    color::*,
    component::*,
    components::{label::*, paragraph::*, scroll::*},
    context::*,
    event::*,
    geometry::*,
    layer::*,
    layout::*,
    properties::*,
    renderers::*,
    signal::*,
    svg::*,
    ui::*,
    utils::*,
};
pub use keyboard_types::Key;
pub use palette;
pub use pallo_macro::*;
pub use pallo_util::*;
pub use platform::{Clipboard, FileOpenOptions, FileSaveOptions, InputType, Platform, PlatformCommon};
pub use rustc_hash::FxHashMap;

#[cfg(target_family = "wasm")]
pub use platform::create_canvas;

#[cfg(target_family = "wasm")]
pub use js_sys;
#[cfg(target_family = "wasm")]
pub use wasm_bindgen;
#[cfg(target_family = "wasm")]
pub use wasm_bindgen_futures;
#[cfg(target_family = "wasm")]
pub use web_sys;
