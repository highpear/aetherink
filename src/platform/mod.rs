#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub use windows::{BackgroundCaptureController, ClickThroughController};

#[cfg(target_os = "macos")]
pub use macos::{BackgroundCaptureController, ClickThroughController};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundCaptureAvailability {
    Available,
    PermissionRequired,
    Unsupported,
}

impl Default for BackgroundCaptureAvailability {
    fn default() -> Self {
        Self::Unsupported
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use image::RgbaImage;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use crate::canvas::ScreenCaptureRect;

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

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
#[derive(Debug, Default)]
pub struct BackgroundCaptureController;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
impl BackgroundCaptureController {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self
    }

    pub fn supports_background_capture(&self) -> bool {
        false
    }

    pub fn background_capture_availability(&self) -> BackgroundCaptureAvailability {
        BackgroundCaptureAvailability::Unsupported
    }

    pub fn request_background_capture_permission(&self) -> Result<bool, String> {
        Err(String::from(
            "Background capture is not implemented for this platform.",
        ))
    }

    pub fn capture_background(&self, _rect: ScreenCaptureRect) -> Result<RgbaImage, String> {
        Err(String::from(
            "Background capture is not implemented for this platform.",
        ))
    }
}
