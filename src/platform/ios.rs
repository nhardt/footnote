use objc2::ffi::*;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::runtime::{AnyClass, Bool, Sel};
use objc2::runtime::{AnyObject, NSObject};
use objc2::{msg_send, sel};
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSArray, NSDictionary, NSString, NSURL};
use objc2_ui_kit::UIApplicationDelegate;
use objc2_ui_kit::{UIActivityViewController, UIApplication, UIViewController};
use std::ffi::c_void;
use std::ffi::CStr;
use std::path::Path;
use std::path::PathBuf;
use std::ptr;

pub const SHARE_SHEET_SUPPORTED: bool = true;

pub fn get_app_dir() -> anyhow::Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))
}
pub fn share_contact_file(file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let mtm = MainThreadMarker::new().ok_or("UIKit calls must be made from the main thread")?;

    unsafe {
        let path_str = file_path.to_str().ok_or("Invalid path")?;
        let ns_path = NSString::from_str(path_str);

        // 1. The File Item (for AirDrop, Files app, Email)
        let url = NSURL::fileURLWithPath(&ns_path);

        // 2. The Text Item (for Copy to Clipboard, Messages)
        // You can use the file content or just a descriptive string
        let content_text =
            std::fs::read_to_string(file_path).unwrap_or_else(|_| path_str.to_string());
        let ns_text = NSString::from_str(&content_text);

        // 3. Cast both to AnyObject and put in the NSArray
        let item_file = Retained::cast_unchecked::<AnyObject>(url);
        let item_text = Retained::cast_unchecked::<AnyObject>(ns_text);

        let items = NSArray::from_retained_slice(&[item_file, item_text]);

        let controller = UIActivityViewController::initWithActivityItems_applicationActivities(
            UIActivityViewController::alloc(mtm),
            &items,
            None,
        );

        // ... (rest of the presentation logic remains the same)
        let app = UIApplication::sharedApplication(mtm);
        let window = app.keyWindow().ok_or("No key window found")?;
        let root_vc = window
            .rootViewController()
            .ok_or("No root view controller")?;

        if let Some(popover) = controller.popoverPresentationController() {
            let view = root_vc.view().expect("View Controller must have a view");
            popover.setSourceView(Some(&view));
            popover.setSourceRect(view.bounds());
        }

        root_vc.presentViewController_animated_completion(&controller, true, None);
    }
    Ok(())
}

pub fn inject_open_url_handler() {
    let mtm = MainThreadMarker::new().expect("Must be on main thread");
    let app = UIApplication::sharedApplication(mtm);

    if let Some(delegate) = unsafe { app.delegate() } {
        // Cast to AnyObject to get access to .class()
        let obj = unsafe { &*(Retained::as_ptr(&delegate) as *const AnyObject) };
        let cls = obj.class();
        let sel = sel!(application:openURL:options:);
        let cls_ref: &AnyClass = obj.class();
        let cls_ptr = cls_ref as *const AnyClass as *mut AnyClass;

        if cls.responds_to(sel) {
            tracing::warn!("Delegate already implements openURL.");
        } else {
            unsafe {
                // Obj-C Type Encodings:
                // B is Bool, @ is object, : is selector
                // The signature is: Bool (self, _cmd, app, url, options)
                let types = CStr::from_bytes_with_nul(b"B@:@@@\0").unwrap();

                // cast a Rust extern "C" fn to Obj-C Imp
                let imp: objc2::runtime::Imp = std::mem::transmute(
                    open_url_callback as unsafe extern "C" fn(_, _, _, _, _) -> _,
                );

                let success = objc2::ffi::class_addMethod(cls_ptr, sel, imp, types.as_ptr());

                if success.as_bool() {
                    tracing::info!(
                        "Successfully injected application:openURL:options: into {:?}",
                        cls.name() // Use {:?} because cls.name() returns &CStr
                    );
                } else {
                    tracing::error!("Failed to inject method.");
                }
            }
        }
    }
}

extern "C" fn open_url_callback(
    this: &AnyObject,
    _sel: Sel,
    _app: *mut AnyObject,
    url: &NSURL,
    _options: &NSDictionary<NSString, AnyObject>,
) -> Bool {
    let path = url.path();
    tracing::info!("Successfully intercepted file open! Path: {:?}", path);

    // TODO: Send this path to your Dioxus state via a channel or global

    Bool::YES
}
