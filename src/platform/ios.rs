use crate::platform::send_incoming_file;
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
        let obj = unsafe { &*(Retained::as_ptr(&delegate) as *const AnyObject) };
        let cls_ref: &AnyClass = obj.class();
        let cls_ptr = cls_ref as *const AnyClass as *mut AnyClass;

        let original_sel = sel!(application:openURL:options:);
        let swizzled_sel = sel!(rust_openURL:options:); // Our "backup" name

        unsafe {
            let types = CStr::from_bytes_with_nul(b"B@:@@@\0").unwrap();
            let imp: objc2::runtime::Imp =
                std::mem::transmute(open_url_callback as unsafe extern "C" fn(_, _, _, _, _) -> _);

            // 1. Add our Rust function under the NEW selector name
            let added = objc2::ffi::class_addMethod(cls_ptr, swizzled_sel, imp, types.as_ptr());

            if added.as_bool() {
                // 2. Get the Method objects for the original and our new one
                let original_method = objc2::ffi::class_getInstanceMethod(cls_ptr, original_sel);
                let swizzled_method = objc2::ffi::class_getInstanceMethod(cls_ptr, swizzled_sel);

                if !original_method.is_null() && !swizzled_method.is_null() {
                    // 3. Swap them!
                    // Now calling 'application:openURL:options:' triggers our Rust code
                    // And calling 'rust_openURL:options:' triggers the original Tao code
                    objc2::ffi::method_exchangeImplementations(
                        original_method as *mut _,
                        swizzled_method as *mut _,
                    );
                    tracing::info!("Successfully swizzled openURL into {:?}", cls_ref.name());
                }
            } else {
                tracing::error!("Could not add swizzled method (perhaps it already exists?)");
            }
        }
    }
}

extern "C" fn open_url_callback(
    this: &AnyObject,
    _sel: Sel,
    app: *mut AnyObject,
    url: &NSURL,
    options: &NSDictionary<NSString, AnyObject>,
) -> Bool {
    if let Some(path_str) = url.path() {
        let path: String = path_str.to_string();
        send_incoming_file(path);
    }

    // Call the original Tao implementation
    let swizzled_sel = sel!(rust_openURL:options:);
    unsafe {
        msg_send![this, performSelector: swizzled_sel, withObject: app, withObject: url, withObject: options]
    }
}
