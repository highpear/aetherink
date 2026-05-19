use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::canvas::CanvasSettings;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OverlaySettings {
    pub(crate) drawing_enabled: bool,
    pub(crate) always_on_top: bool,
    pub(crate) borderless_window: bool,
    pub(crate) click_through_mode: bool,
    pub(crate) transparent_window_background: bool,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            drawing_enabled: true,
            always_on_top: false,
            borderless_window: false,
            click_through_mode: false,
            transparent_window_background: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub(crate) struct AppSettings {
    pub(crate) canvas: CanvasSettings,
    pub(crate) overlay: OverlaySettings,
    pub(crate) last_export_directory: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_defaults_keep_editing_enabled_and_overlay_features_off() {
        let settings = OverlaySettings::default();

        assert!(settings.drawing_enabled);
        assert!(!settings.always_on_top);
        assert!(!settings.borderless_window);
        assert!(!settings.click_through_mode);
        assert!(!settings.transparent_window_background);
    }

    #[test]
    fn app_settings_default_has_no_export_directory() {
        let settings = AppSettings::default();

        assert!(settings.last_export_directory.is_none());
    }
}
