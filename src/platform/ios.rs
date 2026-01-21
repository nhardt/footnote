use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use std::path::Path;
use std::path::PathBuf;
// Use AnyObject to satisfy the 'id' type requirement
use objc2::runtime::{AnyObject, NSObject};
use objc2_foundation::{NSArray, NSString, NSURL};
use objc2_ui_kit::{UIActivityViewController, UIApplication, UIViewController};

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
