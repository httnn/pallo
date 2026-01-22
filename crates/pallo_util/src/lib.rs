use std::{
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    sync::Arc,
};

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use serde::{Deserialize, Serialize};

#[cfg(not(target_family = "wasm"))]
pub fn log(input: impl Debug) {
    dbg!(input);
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
}

#[cfg(target_family = "wasm")]
pub fn log(input: impl Debug) {
    console_log(&format!("{:?}", input));
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum File {
    Path(std::path::PathBuf),
    Data { name: String, data: Arc<Vec<u8>> },
}

impl File {
    pub fn from_path_buf(path: std::path::PathBuf) -> Self {
        Self::Path(path)
    }

    pub fn from_data(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self::Data { name: name.into(), data: data.into() }
    }

    pub fn path(&self) -> Option<PathBuf> {
        match self {
            File::Path(path_buf) => Some(path_buf.clone()),
            File::Data { .. } => None,
        }
    }

    pub fn extension(&self) -> Option<String> {
        match self {
            File::Path(path) => path.extension().and_then(|e| e.to_str().map(|s| s.to_owned())),
            File::Data { name, .. } => name.split(".").last().map(|s| s.to_owned()),
        }
    }

    pub fn name(&self) -> Option<String> {
        match self {
            File::Path(path) => path.file_name().and_then(|e| e.to_str().map(|s| s.to_owned())),
            File::Data { name, .. } => Some(name.clone()),
        }
    }

    pub fn hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        match self {
            File::Path(path) => {
                std::fs::read(path).ok()?.hash(&mut hasher);
            }
            File::Data { data, .. } => {
                data.hash(&mut hasher);
            }
        };
        Some(hasher.finish())
    }

    pub fn size(&self) -> Option<u64> {
        match self {
            File::Path(path_buf) => std::fs::File::open(path_buf).ok()?.metadata().ok()?.len().into(),
            File::Data { data, .. } => Some(data.len() as u64),
        }
    }

    pub fn data(&self) -> Option<Arc<Vec<u8>>> {
        match self {
            // #[cfg(not(target_os = "ios"))]
            File::Path(path_buf) => std::fs::read(path_buf).ok().map(Arc::new),
            // #[cfg(target_os = "ios")]
            // File::Path(path) => {
            //     let url = objc2_foundation::NSURL::from_file_path(path)?;
            //     dbg!(unsafe { url.startAccessingSecurityScopedResource() });
            //     dbg!(&path);
            //     let data = std::fs::read(path).map(Arc::new).unwrap();
            //     unsafe { url.stopAccessingSecurityScopedResource() };
            //     data.into()
            // }
            File::Data { data, .. } => Some(data.clone()),
        }
    }
}

impl From<std::path::PathBuf> for File {
    fn from(value: std::path::PathBuf) -> Self {
        File::from_path_buf(value)
    }
}
