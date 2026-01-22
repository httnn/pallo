use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained, runtime::ProtocolObject,
};
use objc2_foundation::{NSArray, NSMutableArray, NSObject, NSObjectProtocol, NSString, NSTemporaryDirectory, NSURL};
use objc2_ui_kit::{UIDocumentPickerDelegate, UIDocumentPickerViewController, UIView};
use objc2_uniform_type_identifiers::UTType;
use std::{cell::RefCell, path::PathBuf, rc::Rc, str::FromStr};

pub struct Ivars {
    callback: Rc<RefCell<Box<dyn Fn(Vec<PathBuf>) + 'static>>>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[name = "TahtiDocumentPickerDelegate"]
    #[ivars = Ivars]
    pub struct TahtiDocumentPickerDelegate;

    unsafe impl NSObjectProtocol for TahtiDocumentPickerDelegate {}

    unsafe impl UIDocumentPickerDelegate for TahtiDocumentPickerDelegate {
        #[unsafe(method(documentPicker:didPickDocumentsAtURLs:))]
        fn did_pick_documents_at_urls(&self, _: &UIDocumentPickerViewController, urls: &NSArray<NSURL>) {
            self.ivars().callback.borrow()(
                urls.iter().filter_map(|url| url.path().map(|p| PathBuf::from(p.to_string()))).collect(),
            );
        }
    }
);

impl TahtiDocumentPickerDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(Ivars { callback: Rc::new(RefCell::new(Box::new(|_| {}))) });
        unsafe { msg_send![super(this), init] }
    }
}

pub fn open_file_opener(
    ui_view: &UIView,
    delegate: Retained<TahtiDocumentPickerDelegate>,
    extensions: Vec<String>,
    folder: bool,
    multi: bool,
    callback: impl Fn(Vec<PathBuf>) + Sized + 'static,
) {
    let mtm = MainThreadMarker::new().expect("must be on the main thread");

    *delegate.ivars().callback.borrow_mut() = Box::new(callback);

    let allowed_types: Retained<NSMutableArray<UTType>> = NSMutableArray::new();
    if folder && let Some(ut) = UTType::typeWithIdentifier(&NSString::from_str("public.folder")) {
        allowed_types.addObject(&ut);
    }
    for ext in extensions {
        if let Some(ut) = UTType::typeWithFilenameExtension(&NSString::from_str(&ext.to_string())) {
            allowed_types.addObject(&ut);
        }
    }

    let doc_picker = {
        let instance = UIDocumentPickerViewController::alloc(mtm);
        UIDocumentPickerViewController::initForOpeningContentTypes(instance, &allowed_types)
    };

    doc_picker.setAllowsMultipleSelection(multi);
    doc_picker.setDelegate(Some(&ProtocolObject::<dyn UIDocumentPickerDelegate>::from_retained(delegate)));

    ui_view
        .window()
        .and_then(|w| w.rootViewController())
        .map(|c| c.presentViewController_animated_completion(&doc_picker, true, None));
}

pub fn open_file_saver(ui_view: &UIView, filename: String, data: Vec<u8>) {
    let temp_dir = NSTemporaryDirectory();
    let mut path = PathBuf::from_str(&temp_dir.to_string()).unwrap();
    path.push(filename);
    let _ = std::fs::write(path.clone(), data);

    let urls: Retained<NSMutableArray<NSURL>> = NSMutableArray::new();
    let url = NSURL::fileURLWithPath(&NSString::from_str(path.to_str().unwrap()));
    urls.addObject(&url);

    let mtm = MainThreadMarker::new().expect("must be on the main thread");
    let doc_picker = {
        let instance = UIDocumentPickerViewController::alloc(mtm);
        UIDocumentPickerViewController::initForExportingURLs(instance, &urls)
    };

    ui_view
        .window()
        .and_then(|w| w.rootViewController())
        .map(|c| c.presentViewController_animated_completion(&doc_picker, true, None));
}
