use crate::{
    Canvas, File, FileOpenOptions, FileSaveOptions, PointerId, WindowEvent,
    platform::{InputType, platform::file_picker::open_file_opener},
    point,
};
use block2::RcBlock;
use file_picker::{TahtiDocumentPickerDelegate, open_file_saver};
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, ffi, msg_send, rc::Retained, runtime::ProtocolObject,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{
    NSData, NSDictionary, NSError, NSFileManager, NSObject, NSObjectProtocol, NSSearchPathDirectory, NSString, NSURL,
};
use objc2_metal::{
    MTLCommandBuffer, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice, MTLDrawable, MTLPixelFormat, MTLTexture,
};
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use objc2_ui_kit::{
    UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle, UIApplication, UIDragDropSession,
    UIDropInteraction, UIDropInteractionDelegate, UIDropOperation, UIDropProposal, UIDropSession, UIInteraction,
    UIPasteboard, UIResponderStandardEditActions, UITextField, UIView,
};
use objc2_uniform_type_identifiers::NSItemProviderUTType;
use parking_lot::Mutex;
use skia_safe::{
    ColorType, Size, Surface,
    gpu::{self, DirectContext, SurfaceOrigin, backend_render_targets, direct_contexts, mtl},
    scalar,
};
use std::collections::VecDeque;
use std::{ffi::c_void, path::PathBuf, ptr::NonNull, sync::Arc};

use super::{Clipboard, Later, PlatformCommon};

mod file_picker;

struct DragAndDropIvars {
    event_queue: Arc<Mutex<VecDeque<WindowEvent>>>,
    paths: Arc<Mutex<Vec<File>>>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[name = "TahtiDragAndDropDelegate"]
    #[ivars = DragAndDropIvars]
    struct TahtiDragAndDropDelegate;

    unsafe impl NSObjectProtocol for TahtiDragAndDropDelegate {}

    unsafe impl UIDropInteractionDelegate for TahtiDragAndDropDelegate {
        #[unsafe(method(dropInteraction:canHandleSession:))]
        unsafe fn can_handle_session(&self, _: &UIDropInteraction, _: &ProtocolObject<dyn UIDropSession>) -> bool {
            true
        }

        #[unsafe(method(dropInteraction:sessionDidUpdate:))]
        unsafe fn session_did_update(
            &self,
            interaction: &UIDropInteraction,
            session: &ProtocolObject<dyn UIDropSession>,
        ) -> *mut UIDropProposal {
            let mut queue = self.ivars().event_queue.lock();
            if let Some(view) = interaction.view() {
                let location = session.locationInView(&view);
                queue.push_back(WindowEvent::PointerMove {
                    id: PointerId::DragAndDrop,
                    position: point(location.x as f32, location.y as f32),
                });
            }

            queue.push_back(WindowEvent::FileHovered(
                session
                    .items()
                    .into_iter()
                    .map(|i| i.itemProvider().suggestedName().map(|n| n.to_string()).unwrap_or("".into()))
                    .collect(),
            ));
            let mtm = MainThreadMarker::new().unwrap();
            let instance = UIDropProposal::alloc(mtm);
            Retained::into_raw(UIDropProposal::initWithDropOperation(instance, UIDropOperation::Copy))
        }

        #[unsafe(method(dropInteraction:sessionDidExit:))]
        unsafe fn did_exit(&self, _: &UIDropInteraction, _: &ProtocolObject<dyn UIDropSession>) {
            let mut queue = self.ivars().event_queue.lock();
            queue.push_back(WindowEvent::FileDropCancelled);
            queue.push_back(WindowEvent::PointerUp { id: PointerId::DragAndDrop });
        }

        #[unsafe(method(dropInteraction:performDrop:))]
        unsafe fn perform_drop(&self, _: &UIDropInteraction, session: &ProtocolObject<dyn UIDropSession>) {
            unsafe {
                for item in session.items() {
                    let provider = item.itemProvider();
                    for i in provider.registeredContentTypes() {
                        let paths = self.ivars().paths.clone();
                        provider.loadFileRepresentationForTypeIdentifier_completionHandler(
                            &i.identifier(),
                            &RcBlock::new(move |url: *mut NSURL, _: *mut NSError| {
                                let url = &mut *url as &mut NSURL;
                                if let Some(path) = url.path().map(|p| p.to_string())
                                    && let Some(name) = url.lastPathComponent().map(|n| n.to_string())
                                    && let Ok(data) = std::fs::read(path)
                                {
                                    paths.lock().push(File::from_data(name, data));
                                }
                            }),
                        );
                    }
                }
            }
        }

        #[unsafe(method(dropInteraction:concludeDrop:))]
        unsafe fn conclude_drop(&self, _: &UIDropInteraction, _: &ProtocolObject<dyn UIDropSession>) {
            let mut queue = self.ivars().event_queue.lock();
            queue.push_back(WindowEvent::FileDropped(self.ivars().paths.lock().drain(..).collect()));
            queue.push_back(WindowEvent::PointerUp { id: PointerId::DragAndDrop });
        }

        #[unsafe(method(dropInteraction:sessionDidEnd:))]
        unsafe fn session_did_end(&self, _: &UIDropInteraction, _: &ProtocolObject<dyn UIDropSession>) {
            let mut queue = self.ivars().event_queue.lock();
            queue.push_back(WindowEvent::FileDropCancelled);
            queue.push_back(WindowEvent::PointerUp { id: PointerId::DragAndDrop });
        }
    }
);

impl TahtiDragAndDropDelegate {
    pub fn new(mtm: MainThreadMarker, event_queue: Arc<Mutex<VecDeque<WindowEvent>>>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(DragAndDropIvars { event_queue, paths: Arc::new(Mutex::new(vec![])) });
        unsafe { msg_send![super(this), init] }
    }
}

pub struct IOSClipboard;

impl Clipboard for IOSClipboard {
    fn write_string(&mut self, text: impl Into<String>) {
        unsafe {
            UIPasteboard::generalPasteboard().setString(Some(&NSString::from_str(&text.into())));
        }
    }
    fn read_string(&self) -> Option<String> {
        unsafe { UIPasteboard::generalPasteboard().string().map(|s| s.to_string()) }
    }

    fn read_paths(&self) -> Option<Vec<PathBuf>> {
        unsafe {
            let pasteboard = UIPasteboard::generalPasteboard();
            dbg!(pasteboard.URLs().map(|u| u.into_iter().filter_map(|url| url.path().map(|p| p.to_string()))));
        }
        None
    }

    fn read_audio(&self) -> Option<Vec<u8>> {
        let pasteboard = UIPasteboard::generalPasteboard();
        pasteboard.dataForPasteboardType(&NSString::from_str("public.audio")).map(|d| d.to_vec())
    }

    fn write_data(&mut self, data: Vec<u8>) {
        UIPasteboard::generalPasteboard()
            .setData_forPasteboardType(&NSData::from_vec(data), &NSString::from_str("public.data"));
    }

    fn read_data(&self) -> Option<Vec<u8>> {
        let pasteboard = UIPasteboard::generalPasteboard();
        pasteboard.dataForPasteboardType(&NSString::from_str("public.data")).map(|d| d.to_vec())
    }
}

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

pub struct Platform {
    metal_layer: Retained<CAMetalLayer>,
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    direct_context: DirectContext,
    view: Retained<UIView>,
    document_picker_delegate: Retained<TahtiDocumentPickerDelegate>,
    _drag_and_drop_delegate: Retained<TahtiDragAndDropDelegate>,
    event_queue: Arc<Mutex<VecDeque<WindowEvent>>>,
    clipboard: IOSClipboard,
}

impl PlatformCommon for Platform {
    type Frame = Frame;

    fn next_window_event(&mut self) -> Option<WindowEvent> {
        self.event_queue.lock().pop_front()
    }

    fn get_scale_factor(&self) -> f32 {
        self.view.window().and_then(|w| w.windowScene()).map(|w| w.screen().scale()).unwrap_or(1.0) as f32
    }

    fn set_view_size(&mut self, size: (u32, u32)) {
        let scale_factor = self.get_scale_factor() as f64;
        let new_size = CGSize::new(scale_factor * size.0 as f64, scale_factor * size.1 as f64);
        self.metal_layer.setDrawableSize(new_size);
        let new_frame = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(size.0 as f64, size.1 as f64));
        self.metal_layer.setFrame(new_frame);
    }

    fn open_path_in_file_explorer(&self, path: PathBuf) {
        if let Some(mut url) = path.to_str().map(|s| s.to_string().replace("file://", "shareddocuments://")) {
            if !url.starts_with("shareddocuments://") {
                url = "shareddocuments://".to_owned() + &url;
            }
            let mtm = MainThreadMarker::new().expect("must be on the main thread");
            unsafe {
                UIApplication::sharedApplication(mtm).openURL_options_completionHandler(
                    &NSURL::URLWithString(&NSString::from_str(&url)).unwrap(),
                    &NSDictionary::new(),
                    None,
                );
            }
        }
    }

    fn file_open_dialog(&self, opts: FileOpenOptions) {
        open_file_opener(
            &self.view,
            self.document_picker_delegate.clone(),
            opts.extensions,
            opts.folder,
            opts.multi,
            move |paths| {
                opts.result.set(paths.into_iter().map(File::from_path_buf).collect());
            },
        );
    }

    fn file_save_dialog(&self, options: FileSaveOptions) {
        open_file_saver(&self.view, options.filename, options.data.to_vec());
    }

    fn start_drag(&self, _path: PathBuf) {}

    fn open_url(&self, url: impl Into<String>) {
        let url: String = url.into();
        unsafe {
            if let Some(url) = NSURL::URLWithString(&NSString::from_str(&url)) {
                let mtm = MainThreadMarker::new().expect("must be on the main thread");
                UIApplication::sharedApplication(mtm).openURL_options_completionHandler(
                    &url,
                    &NSDictionary::new(),
                    None,
                );
            }
        }
    }

    fn clipboard(&mut self) -> &mut impl super::Clipboard {
        &mut self.clipboard
    }

    fn documents_folder_path() -> Option<PathBuf> {
        let array = NSFileManager::defaultManager().URLsForDirectory_inDomains(
            NSSearchPathDirectory::DocumentDirectory,
            objc2_foundation::NSSearchPathDomainMask::UserDomainMask,
        );
        array.firstObject().and_then(|p| p.path()).map(|p| PathBuf::from(p.to_string()))
    }

    fn open_prompt(
        &self,
        title: String,
        enter_text: String,
        value: String,
        _input_type: InputType,
        result: &Later<String>,
    ) {
        let mtm = MainThreadMarker::new().expect("Not on main thread.");
        let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
            Some(&NSString::from_str(&title)),
            None,
            UIAlertControllerStyle::Alert,
            mtm,
        );

        unsafe {
            alert.addTextFieldWithConfigurationHandler(Some(&RcBlock::new(move |view: NonNull<UITextField>| {
                view.as_ref().setText(Some(&NSString::from_str(&value)));
                // TODO: fix
                // view.as_ref().setKeyboardType(match input_type {
                //     InputType::Text => UIKeyboardType::Default,
                //     InputType::Number => UIKeyboardType::NumberPad,
                // });
            })))
        };

        let block_alert = alert.clone();
        let cb_result = result.clone();
        let block = RcBlock::new(move |_: NonNull<UIAlertAction>| {
            if let Some(text) = block_alert.textFields().and_then(|f| f.firstObject()).and_then(|o| o.text()) {
                cb_result.set(text.to_string());
            }
        });
        let action = UIAlertAction::actionWithTitle_style_handler(
            Some(&NSString::from_str(&enter_text)),
            UIAlertActionStyle::Default,
            Some(&block),
            mtm,
        );
        alert.addAction(&action);

        let completion_alert = alert.clone();
        if let Some(root_view_controller) = self.view.window().and_then(|w| w.rootViewController()) {
            unsafe {
                root_view_controller.presentViewController_animated_completion(
                    &alert,
                    true,
                    Some(&RcBlock::new(move || {
                        completion_alert.textFields().and_then(|f| f.firstObject()).map(|o| o.selectAll(None));
                    })),
                )
            };
        }
    }

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
        //self.metal_layer.setNeedsDisplay();

        unsafe {
            ffi::objc_autoreleasePoolPop(frame.autoreleasepool);
        }
    }
}

impl Platform {
    pub fn new_from_window_handle(handle: *mut c_void) -> Self {
        let view: Retained<UIView> = Retained::from(unsafe { &*(handle as *mut UIView) });
        let device = MTLCreateSystemDefaultDevice().expect("Could not create Metal device.");

        let event_queue = Arc::new(Mutex::new(VecDeque::new()));

        let drag_and_drop_delegate = {
            let mtm = MainThreadMarker::new().unwrap();
            let delegate = TahtiDragAndDropDelegate::new(mtm, event_queue.clone());
            let instance = UIDropInteraction::alloc(mtm);
            let interaction = UIDropInteraction::initWithDelegate(
                instance,
                &ProtocolObject::<dyn UIDropInteractionDelegate>::from_retained(delegate.clone()),
            );
            view.addInteraction(&ProtocolObject::<dyn UIInteraction>::from_retained(interaction));
            delegate
        };

        let metal_layer = {
            let layer = CAMetalLayer::new();
            layer.setDevice(Some(&device));
            layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            layer.setPresentsWithTransaction(false);
            layer.setFramebufferOnly(false);
            view.layer().addSublayer(&layer);
            layer
        };

        let command_queue = device.newCommandQueue().expect("Could not create Metal command queue");

        let backend = unsafe {
            mtl::BackendContext::new(
                Retained::<ProtocolObject<dyn MTLDevice>>::as_ptr(&device) as mtl::Handle,
                Retained::<ProtocolObject<dyn MTLCommandQueue>>::as_ptr(&command_queue) as mtl::Handle,
            )
        };
        Self {
            event_queue,
            metal_layer,
            _drag_and_drop_delegate: drag_and_drop_delegate,
            direct_context: direct_contexts::make_metal(&backend, None)
                .expect("Could not create metal direct context."),
            command_queue,
            document_picker_delegate: TahtiDocumentPickerDelegate::new(
                MainThreadMarker::new().expect("must be on the main thread"),
            ),
            view,
            clipboard: IOSClipboard,
        }
    }
}
