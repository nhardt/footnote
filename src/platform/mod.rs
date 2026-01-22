use std::sync::OnceLock;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

//TODO: this can probably be wrapped up in SHARE_SHEET_SUPPORTED
static FILE_SENDER: OnceLock<UnboundedSender<String>> = OnceLock::new();
static RECEIVER_STORAGE: OnceLock<std::sync::Mutex<Option<UnboundedReceiver<String>>>> =
    OnceLock::new();

fn init_channel() {
    RECEIVER_STORAGE.get_or_init(|| {
        let (tx, rx) = mpsc::unbounded_channel();
        FILE_SENDER.set(tx).expect("sender already set");
        std::sync::Mutex::new(Some(rx))
    });
}

pub fn take_file_receiver() -> Option<UnboundedReceiver<String>> {
    init_channel();
    RECEIVER_STORAGE.get()?.lock().ok()?.take()
}

pub fn send_incoming_file(uri_or_path: String) {
    init_channel();
    if let Some(tx) = FILE_SENDER.get() {
        let _ = tx.send(uri_or_path);
    }
}

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::*;

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::*;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod stub;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use stub::*;
