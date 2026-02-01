// Platform-specific code for mobile (Android/iOS)
// Desktop uses footnote_core::platform directly

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::*;

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::*;

// Re-export core platform functions for all platforms
pub use footnote_core::platform::get_app_dir;
