#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub use windows::ClickThroughController;

#[cfg(target_os = "macos")]
pub use macos::ClickThroughController;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
#[derive(Debug, Default)]
pub struct ClickThroughController;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
impl ClickThroughController {
    pub fn supports_pointer_passthrough(&self) -> bool {
        false
    }

    pub fn supports_shortcut_monitoring(&self) -> bool {
        false
    }

    pub fn poll_overlay_toggle_shortcut(&mut self) -> bool {
        false
    }

    pub fn temporary_drawing_shortcut_label(&self) -> &'static str {
        ""
    }

    pub fn is_temporary_drawing_active(&self) -> bool {
        false
    }
}
