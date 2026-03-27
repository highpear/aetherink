#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub use windows::ClickThroughController;

#[cfg(not(target_os = "windows"))]
#[derive(Debug, Default)]
pub struct ClickThroughController;

#[cfg(not(target_os = "windows"))]
impl ClickThroughController {
    pub fn is_supported(&self) -> bool {
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
