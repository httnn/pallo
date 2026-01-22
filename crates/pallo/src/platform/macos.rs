use std::{ffi::c_void, path::PathBuf, sync::Arc};

use block2::RcBlock;
use objc2::{AllocAnyThread, MainThreadMarker, Message, ffi, rc::Retained, runtime::ProtocolObject};
use objc2_app_kit::{
    NSApplication, NSDraggingItem, NSModalResponseOK, NSOpenPanel, NSPasteboard, NSPasteboardWriting, NSSavePanel,
    NSView, NSWorkspace,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSArray, NSData, NSFileManager, NSPoint, NSRect, NSSearchPathDirectory, NSString, NSURL};
use objc2_metal::{
    MTLCommandBuffer, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice, MTLDrawable, MTLPixelFormat, MTLTexture,
};
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use objc2_uniform_type_identifiers::UTType;
use pallo_util::File;
use skia_safe::{
    ColorType, Size, Surface,
    gpu::{self, DirectContext, SurfaceOrigin, backend_render_targets, direct_contexts, mtl},
    scalar,
};

use super::{Clipboard, PlatformCommon};
use crate::{
    Canvas, FileSaveOptions, Later, WindowEvent,
    platform::{FileOpenOptions, InputType},
};

pub struct Platform {
    metal_layer: Retained<CAMetalLayer>,
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    direct_context: DirectContext,
    ns_view: Retained<NSView>,
    clipboard: MacOsClipboard,
}

unsafe impl Send for Platform {}

pub struct Frame {
    drawable: Retained<ProtocolObject<dyn CAMetalDrawable>>,
    surface: Surface,
    autoreleasepool: *mut c_void,
}

impl super::Frame for Frame {
    fn canvas(&mut self) -> Canvas<'_> {
        Canvas::new(self.surface.canvas())
    }
}

pub struct MacOsClipboard;

impl Clipboard for MacOsClipboard {
    fn write_string(&mut self, text: impl Into<String>) {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();
        pasteboard.setString_forType(&NSString::from_str(&text.into()), &NSString::from_str("public.utf8-plain-text"));
    }

    fn write_data(&mut self, data: Vec<u8>) {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();
        pasteboard.setData_forType(Some(&NSData::from_vec(data)), &NSString::from_str("public.data"));
    }

    fn read_data(&self) -> Option<Vec<u8>> {
        let pasteboard = NSPasteboard::generalPasteboard();
        if let Some(types) = pasteboard.types() {
            let t = NSString::from_str("public.data");
            if types.containsObject(&t)
                && let Some(data) = pasteboard.dataForType(&t)
            {
                return Some(data.to_vec());
            }
        }
        None
    }

    fn read_string(&self) -> Option<String> {
        let pasteboard = NSPasteboard::generalPasteboard();
        if let Some(types) = pasteboard.types() {
            let t = NSString::from_str("public.utf8-plain-text");
            if types.containsObject(&t)
                && let Some(str) = pasteboard.stringForType(&t)
            {
                return Some(str.to_string());
            }
        }
        None
    }

    fn read_paths(&self) -> Option<Vec<PathBuf>> {
        NSPasteboard::generalPasteboard().pasteboardItems().and_then(|items| {
            let t = NSString::from_str("public.file-url");
            let mut out = vec![];
            for i in items {
                if let Some(str) = i.stringForType(&t) {
                    let url = NSURL::URLWithString(&str).unwrap();
                    out.push(PathBuf::from(url.path().unwrap().to_string()));
                }
            }
            if out.is_empty() { None } else { Some(out) }
        })
    }

    fn read_audio(&self) -> Option<Vec<u8>> {
        let pasteboard = NSPasteboard::generalPasteboard();
        if let Some(types) = pasteboard.types() {
            let t = NSString::from_str("com.apple.cocoa.pasteboard.sound");
            if types.containsObject(&t)
                && let Some(data) = pasteboard.dataForType(&t)
            {
                return Some(data.to_vec());
            }
        }
        None
    }
}

impl PlatformCommon for Platform {
    type Frame = Frame;

    fn documents_folder_path() -> Option<PathBuf> {
        let array = NSFileManager::defaultManager().URLsForDirectory_inDomains(
            NSSearchPathDirectory::DocumentDirectory,
            objc2_foundation::NSSearchPathDomainMask::UserDomainMask,
        );
        array.firstObject().and_then(|p| p.path()).map(|p| PathBuf::from(p.to_string()))
    }

    fn open_url(&self, url: impl Into<String>) {
        let _ = open::that(url.into());
    }

    fn file_open_dialog(&self, opts: FileOpenOptions) {
        let mtm = MainThreadMarker::new().expect("Should be called from main thread.");
        let panel = Arc::new(NSOpenPanel::new(mtm));
        panel.setCanChooseFiles(opts.files);
        panel.setCanChooseDirectories(opts.folder);
        panel.setAllowsMultipleSelection(opts.multi);
        panel.setTitle(Some(&NSString::from_str(&opts.filetype_desc)));
        panel.setAllowedContentTypes(&NSArray::from_retained_slice(
            opts.extensions
                .into_iter()
                .filter_map(|s| UTType::typeWithFilenameExtension(&NSString::from_str(&s)))
                .map(|s| s.retain())
                .collect::<Vec<Retained<UTType>>>()
                .as_slice(),
        ));
        let result_panel = panel.clone();
        panel.beginSheetModalForWindow_completionHandler(
            &self.ns_view.window().unwrap(),
            &RcBlock::new(move |response| {
                if response == NSModalResponseOK {
                    opts.result.set(
                        result_panel
                            .URLs()
                            .into_iter()
                            .filter_map(|u| u.path().map(|p| p.to_string()))
                            .map(|s| File::from_path_buf(PathBuf::from(s)))
                            .collect(),
                    );
                }
            }),
        );
    }

    fn file_save_dialog(&self, opts: FileSaveOptions) {
        let mtm = MainThreadMarker::new().expect("Should be called from main thread.");
        let panel = NSSavePanel::new(mtm);
        panel.setAllowedContentTypes(&NSArray::from_retained_slice(&[UTType::typeWithFilenameExtension(
            &NSString::from_str(&opts.extension),
        )
        .expect("Can't convert extension to UTType.")]));
        panel.setNameFieldStringValue(&NSString::from_str(&opts.filename));
        let result_panel = panel.clone();
        panel.beginSheetModalForWindow_completionHandler(
            &self.ns_view.window().unwrap(),
            &RcBlock::new(move |response| {
                if response == NSModalResponseOK
                    && let Some(url) = result_panel.URL()
                    && let Some(path) = url.path()
                {
                    let path = PathBuf::from(path.to_string());
                    let _ = std::fs::write(&path, &*opts.data);
                    if let Some(result) = &opts.result {
                        result.set(path);
                    }
                }
            }),
        );
    }

    fn start_drag(&self, path: PathBuf) {
        if let Some(path) = path.to_str() {
            unsafe {
                let dragging_item = {
                    let pasteboard_item = NSURL::fileURLWithPath(&NSString::from_str(path));

                    let item = NSDraggingItem::alloc();
                    let item = NSDraggingItem::initWithPasteboardWriter(
                        item,
                        &ProtocolObject::<dyn NSPasteboardWriting>::from_retained(pasteboard_item),
                    );

                    let icon = NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(path));
                    let icon_size = icon.size();
                    let dragging_frame = NSRect::new(NSPoint::new(0.0, 0.0), icon_size);

                    item.setDraggingFrame_contents(dragging_frame, Some(&Retained::from(&*icon)));
                    item
                };

                let mtm = MainThreadMarker::new().expect("must be on the main thread");
                let current_event = NSApplication::sharedApplication(mtm).currentEvent().unwrap();

                let array: Retained<NSArray<NSDraggingItem>> = NSArray::arrayWithObject(&dragging_item);
                self.ns_view.beginDraggingSessionWithItems_event_source(
                    &array,
                    &current_event,
                    std::mem::transmute(&*self.ns_view),
                );
            }
        }
    }

    fn get_scale_factor(&self) -> f32 {
        self.ns_view.window().map(|w| w.backingScaleFactor() as f32).unwrap_or(1.0)
    }

    fn set_view_size(&mut self, size: (u32, u32)) {
        let scale_factor = self.get_scale_factor() as f64;
        self.metal_layer.setDrawableSize(CGSize::new(scale_factor * size.0 as f64, scale_factor * size.1 as f64));
        self.metal_layer.setBounds(CGRect::new(CGPoint::ZERO, CGSize::new(size.0 as f64, size.1 as f64)));
        self.metal_layer.setPosition(CGPoint::new(size.0 as f64 * 0.5, size.1 as f64 * 0.5));
    }

    fn next_window_event(&mut self) -> Option<WindowEvent> {
        None
    }

    fn clipboard(&mut self) -> &mut impl Clipboard {
        &mut self.clipboard
    }

    fn open_path_in_file_explorer(&self, path: PathBuf) {
        std::process::Command::new("open").arg("-R").arg(path.into_os_string()).spawn().unwrap();
    }

    fn open_prompt(&self, _: String, _: String, _: String, _: InputType, _: &Later<String>) {}

    fn new_frame(&mut self) -> Option<Frame> {
        let autoreleasepool = unsafe { ffi::objc_autoreleasePoolPush() };
        if let Some(drawable) = self.metal_layer.nextDrawable() {
            let drawable_size = {
                let size = self.metal_layer.drawableSize();
                Size::new(size.width as scalar, size.height as scalar)
            };

            let texture_info = unsafe {
                mtl::TextureInfo::new(
                    Retained::<ProtocolObject<dyn MTLTexture>>::as_ptr(&drawable.texture()) as mtl::Handle
                )
            };

            let backend_render_target = backend_render_targets::make_mtl(
                (drawable_size.width as i32, drawable_size.height as i32),
                &texture_info,
            );

            gpu::surfaces::wrap_backend_render_target(
                &mut self.direct_context,
                &backend_render_target,
                SurfaceOrigin::TopLeft,
                ColorType::BGRA8888,
                None,
                None,
            )
            .map(|surface| Frame { autoreleasepool, surface, drawable })
        } else {
            None
        }
    }

    fn end_frame(&mut self, frame: Frame) {
        self.direct_context.flush_and_submit();

        drop(frame.surface);
        let command_buffer = self.command_queue.commandBuffer().unwrap();

        command_buffer.presentDrawable(&ProtocolObject::<dyn MTLDrawable>::from_retained(frame.drawable));
        command_buffer.commit();
        self.metal_layer.setNeedsDisplay();

        unsafe {
            ffi::objc_autoreleasePoolPop(frame.autoreleasepool);
        }
    }
}

#[allow(unused)]
impl Platform {
    pub fn new_from_window_handle(handle: *mut c_void) -> Self {
        let view: Retained<NSView> = Retained::from(unsafe { &*(handle as *mut NSView) });
        let device = MTLCreateSystemDefaultDevice().unwrap();

        let metal_layer = {
            let layer = CAMetalLayer::new();
            layer.setDevice(Some(&device));
            layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            layer.setPresentsWithTransaction(false);
            layer.setFramebufferOnly(false);

            if let Some(view_layer) = view.layer() {
                view_layer.addSublayer(&layer);
            } else {
                view.setWantsLayer(true);
                view.setLayer(Some(&layer));
            }

            layer
        };

        let command_queue = device.newCommandQueue().unwrap();

        let backend = unsafe {
            mtl::BackendContext::new(
                Retained::<ProtocolObject<dyn MTLDevice>>::as_ptr(&device) as mtl::Handle,
                Retained::<ProtocolObject<dyn MTLCommandQueue>>::as_ptr(&command_queue) as mtl::Handle,
            )
        };
        Self {
            metal_layer,
            direct_context: direct_contexts::make_metal(&backend, None).unwrap(),
            command_queue,
            ns_view: view,
            clipboard: MacOsClipboard,
        }
    }
}
