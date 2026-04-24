use eframe::egui;

use super::AetherInkApp;
use crate::canvas::{CanvasBackground, TransparentCanvasBorderVisibility};

impl AetherInkApp {
    pub(super) fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.is_settings_window_open {
            return;
        }

        let mut is_settings_window_open = self.is_settings_window_open;
        let mut drawing_enabled_changed = false;
        let mut always_on_top_changed = false;
        let mut borderless_window_changed = false;
        let mut click_through_mode_changed = false;
        let mut transparent_window_background_changed = false;
        let click_through_supported = self.click_through_controller.supports_pointer_passthrough();
        let click_through_shortcuts_supported =
            self.click_through_controller.supports_shortcut_monitoring();
        let click_through_can_be_enabled = self.can_enable_click_through_mode();
        let is_drawing = self.canvas.is_drawing();

        egui::Window::new("Settings")
            .open(&mut is_settings_window_open)
            .collapsible(false)
            .resizable(false)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.label("Canvas");

                ui.horizontal(|ui| {
                    ui.label("Background:");
                    egui::ComboBox::from_id_salt("settings_canvas_background")
                        .selected_text(self.canvas.background().label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                self.canvas.background_mut(),
                                CanvasBackground::White,
                                CanvasBackground::White.label(),
                            );
                            ui.selectable_value(
                                self.canvas.background_mut(),
                                CanvasBackground::Transparent,
                                CanvasBackground::Transparent.label(),
                            );
                        });
                });

                ui.add_enabled_ui(
                    self.canvas.background() == CanvasBackground::Transparent,
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Opacity:");
                            ui.add(
                                egui::Slider::new(
                                    self.canvas.transparent_background_opacity_mut(),
                                    0.0..=1.0,
                                )
                                .show_value(true)
                                .fixed_decimals(2),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Border:");
                            egui::ComboBox::from_id_salt("settings_canvas_border_visibility")
                                .selected_text(
                                    self.canvas.transparent_canvas_border_visibility().label(),
                                )
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        self.canvas.transparent_canvas_border_visibility_mut(),
                                        TransparentCanvasBorderVisibility::Always,
                                        TransparentCanvasBorderVisibility::Always.label(),
                                    );
                                    ui.selectable_value(
                                        self.canvas.transparent_canvas_border_visibility_mut(),
                                        TransparentCanvasBorderVisibility::NearEdges,
                                        TransparentCanvasBorderVisibility::NearEdges.label(),
                                    );
                                });
                        });
                    },
                );

                ui.separator();

                if ui
                    .checkbox(&mut self.overlay.drawing_enabled, "Enable drawing")
                    .changed()
                {
                    drawing_enabled_changed = true;
                }

                if ui
                    .checkbox(&mut self.overlay.always_on_top, "Always on top")
                    .changed()
                {
                    always_on_top_changed = true;
                }

                if ui
                    .checkbox(&mut self.overlay.borderless_window, "Borderless window")
                    .changed()
                {
                    borderless_window_changed = true;
                }

                if ui
                    .checkbox(
                        &mut self.overlay.transparent_window_background,
                        "Transparent window background",
                    )
                    .changed()
                {
                    transparent_window_background_changed = true;
                }

                if self.overlay.transparent_window_background {
                    ui.small("The window frame and panel background blend into the screen.");
                }

                ui.separator();
                ui.label("Overlay");

                ui.add_enabled_ui(click_through_can_be_enabled && !is_drawing, |ui| {
                    if ui
                        .checkbox(&mut self.overlay.click_through_mode, "Click-through mode")
                        .changed()
                    {
                        click_through_mode_changed = true;
                    }
                });

                if click_through_supported {
                    if click_through_shortcuts_supported {
                        ui.label("Overlay shortcut: Ctrl+Shift+O");
                        ui.label(format!(
                            "Hold {} to draw while click-through is enabled.",
                            self.click_through_controller.temporary_drawing_shortcut_label()
                        ));
                    } else {
                        ui.small(
                            "Pointer passthrough is available, but overlay shortcuts are not implemented on this platform yet.",
                        );
                        ui.small(
                            "Click-through stays disabled until a reliable way to return to the app is available.",
                        );
                    }

                    if click_through_can_be_enabled && is_drawing {
                        ui.small("Finish the current stroke before enabling click-through.");
                    } else if click_through_can_be_enabled && self.overlay.click_through_mode {
                        ui.small("Mouse input is passing through to the window behind AetherInk.");
                    }
                } else {
                    ui.small("Click-through mode is not available on this platform yet.");
                }
            });

        self.is_settings_window_open = is_settings_window_open;

        if drawing_enabled_changed {
            self.set_drawing_enabled(self.overlay.drawing_enabled);
        }

        if always_on_top_changed {
            self.apply_always_on_top(ctx);
        }

        if borderless_window_changed {
            self.apply_borderless_window(ctx);
        }

        if transparent_window_background_changed {
            ctx.request_repaint();
        }

        if click_through_mode_changed {
            self.set_click_through_mode(ctx, self.overlay.click_through_mode);

            if self.overlay.click_through_mode {
                self.is_settings_window_open = false;
            }
        }
    }
}
