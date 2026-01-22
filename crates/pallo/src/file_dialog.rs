use std::{cell::RefCell, sync::Arc};

use parking_lot::Mutex;

use crate::{
    App, Cx,
    platform::{OpenFile, PlatformCommon},
};

pub struct FileDialog<T> {
    meta: RefCell<Option<T>>,
    open_result: Arc<Mutex<Option<OpenFile>>>,
}

impl<T> Default for FileDialog<T> {
    fn default() -> Self {
        Self { meta: Default::default(), open_result: Default::default() }
    }
}

impl<T: std::fmt::Debug> FileDialog<T> {
    pub fn get_open_result(&mut self) -> Option<(T, OpenFile)> {
        if let Some(result) = self.open_result.lock().take() {
            let meta = self.meta.take().unwrap();
            Some((meta, result))
        } else {
            None
        }
    }
}

impl<T: std::fmt::Debug> FileDialog<T> {
    pub fn open_file<A: App>(
        &self,
        cx: &Cx<A>,
        filetype_desc: impl Into<String> + Send + 'static,
        extensions: &'static [impl ToString + Sync + 'static],
        meta: T,
    ) {
        *self.meta.borrow_mut() = Some(meta);
        cx.platform.open_file_open_dialog(filetype_desc, extensions, self.open_result.clone());
    }
}
