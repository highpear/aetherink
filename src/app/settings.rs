use serde::{Deserialize, Serialize};

use crate::canvas::{
    CanvasBackground, CanvasState, TransparentCanvasBorderVisibility,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AppSettings {
    pub(crate) background: CanvasBackground,
    pub(crate) transparent_background_opacity: f32,
    pub(crate) transparent_canvas_border_visibility: TransparentCanvasBorderVisibility,
    pub(crate) drawing_enabled: bool,
    pub(crate) always_on_top: bool,
    pub(crate) borderless_window: bool,
    pub(crate) click_through_mode: bool,
    pub(crate) transparent_window_background: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        let canvas = CanvasState::default();

        Self {
            background: canvas.background,
            transparent_background_opacity: canvas.transparent_background_opacity,
            transparent_canvas_border_visibility: canvas.transparent_canvas_border_visibility,
            drawing_enabled: true,
            always_on_top: false,
            borderless_window: false,
            click_through_mode: false,
            transparent_window_background: false,
        }
    }
}
