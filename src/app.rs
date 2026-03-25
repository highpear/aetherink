use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::canvas::{
    CanvasBackground, CanvasState, TransparentCanvasBorderVisibility,
};
use crate::platform::ClickThroughController;

const BASIC_PEN_COLORS: [(&str, egui::Color32); 5] = [
    ("Black", egui::Color32::BLACK),
    ("White", egui::Color32::WHITE),
    ("Red", egui::Color32::from_rgb(220, 38, 38)),
    ("Yellow", egui::Color32::from_rgb(234, 179, 8)),
    ("Blue", egui::Color32::from_rgb(37, 99, 235)),
];
const APP_SETTINGS_KEY: &str = "app_settings";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct AppSettings {
    background: CanvasBackground,
    transparent_background_opacity: f32,
    transparent_canvas_border_visibility: TransparentCanvasBorderVisibility,
    always_on_top: bool,
    borderless_window: bool,
    click_through_mode: bool,
    transparent_window_background: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        let canvas = CanvasState::default();

        Self {
            background: canvas.background,
            transparent_background_opacity: canvas.transparent_background_opacity,
            transparent_canvas_border_visibility: canvas.transparent_canvas_border_visibility,
            always_on_top: false,
            borderless_window: false,
            click_through_mode: false,
            transparent_window_background: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct AetherInkApp {
    canvas: CanvasState,
    is_settings_window_open: bool,
    always_on_top: bool,
    borderless_window: bool,
    click_through_mode: bool,
    transparent_window_background: bool,
    click_through_controller: ClickThroughController,
}

impl eframe::App for AetherInkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.click_through_mode && self.click_through_controller.poll_restore_shortcut() {
            self.set_click_through_mode(ctx, false);
        }

        // Keyboard shortcuts
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Z,
            ))
        }) {
            self.canvas.undo();
        }

        egui::TopBottomPanel::top("top_bar")
            .frame(egui::Frame::NONE.fill(self.top_bar_fill_color()))
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.borderless_window {
                    ui.heading("AetherInk");
                    ui.separator();

                    let drag_response = ui.add(
                        egui::Label::new("Drag window").sense(egui::Sense::click_and_drag()),
                    );

                    if drag_response.drag_started() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }

                    ui.separator();
                }

                ui.label("Color:");
                ui.color_edit_button_srgba(&mut self.canvas.current_color);
                show_basic_pen_colors(ui, &mut self.canvas.current_color);

                ui.label("Width:");
                ui.add(egui::Slider::new(
                    &mut self.canvas.current_width,
                    1.0..=20.0,
                ));

                ui.separator();

                let has_strokes = self.canvas.has_strokes();

                if ui
                    .add_enabled(has_strokes, undo_button())
                    .on_hover_text("Remove the last stroke (Ctrl+Z)")
                    .clicked()
                {
                    self.canvas.undo();
                }

                if ui
                    .add_enabled(has_strokes, clear_button())
                    .on_hover_text("Remove all strokes from the canvas")
                    .clicked()
                {
                    self.canvas.clear();
                }

                ui.separator();

                if ui
                    .checkbox(&mut self.always_on_top, "Always on top")
                    .changed()
                {
                    self.apply_always_on_top(ctx);
                }

                if self.click_through_mode {
                    ui.separator();
                    ui.label(format!(
                        "Click-through active. Press {} to restore.",
                        self.click_through_controller.restore_shortcut_label()
                    ));
                }

                ui.separator();

                if ui.button("Settings").clicked() {
                    self.is_settings_window_open = true;
                }
            });
        });

        self.show_settings_window(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(self.central_panel_fill_color()))
            .show(ctx, |ui| {
                ui.label("Drag mouse to draw.");
                self.canvas.ui(ui);
            });

        ctx.request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_SETTINGS_KEY, &self.collect_settings());
    }
}

impl AetherInkApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();

        if let Some(storage) = cc.storage {
            if let Some(settings) = eframe::get_value(storage, APP_SETTINGS_KEY) {
                app.apply_settings(settings);
            }
        }

        app.apply_always_on_top(&cc.egui_ctx);
        app.apply_borderless_window(&cc.egui_ctx);
        app.click_through_mode = false;
        app.apply_click_through_mode(&cc.egui_ctx);

        app
    }

    fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.is_settings_window_open {
            return;
        }

        let mut is_settings_window_open = self.is_settings_window_open;
        let mut always_on_top_changed = false;
        let mut borderless_window_changed = false;
        let mut click_through_mode_changed = false;
        let mut transparent_window_background_changed = false;
        let click_through_supported = self.click_through_controller.is_supported();
        let is_drawing = self.canvas.current_stroke.is_some();

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
                        .selected_text(self.canvas.background.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.canvas.background,
                                CanvasBackground::White,
                                CanvasBackground::White.label(),
                            );
                            ui.selectable_value(
                                &mut self.canvas.background,
                                CanvasBackground::Transparent,
                                CanvasBackground::Transparent.label(),
                            );
                        });
                });

                ui.add_enabled_ui(
                    self.canvas.background == CanvasBackground::Transparent,
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Opacity:");
                            ui.add(
                                egui::Slider::new(
                                    &mut self.canvas.transparent_background_opacity,
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
                                    self.canvas
                                        .transparent_canvas_border_visibility
                                        .label(),
                                )
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.canvas.transparent_canvas_border_visibility,
                                        TransparentCanvasBorderVisibility::Always,
                                        TransparentCanvasBorderVisibility::Always.label(),
                                    );
                                    ui.selectable_value(
                                        &mut self.canvas.transparent_canvas_border_visibility,
                                        TransparentCanvasBorderVisibility::NearEdges,
                                        TransparentCanvasBorderVisibility::NearEdges.label(),
                                    );
                                });
                        });
                    },
                );

                ui.separator();

                if ui
                    .checkbox(&mut self.always_on_top, "Always on top")
                    .changed()
                {
                    always_on_top_changed = true;
                }

                if ui
                    .checkbox(&mut self.borderless_window, "Borderless window")
                    .changed()
                {
                    borderless_window_changed = true;
                }

                if ui
                    .checkbox(
                        &mut self.transparent_window_background,
                        "Transparent window background",
                    )
                    .changed()
                {
                    transparent_window_background_changed = true;
                }

                if self.transparent_window_background {
                    ui.small("The window frame and panel background blend into the screen.");
                }

                ui.separator();
                ui.label("Overlay");

                ui.add_enabled_ui(click_through_supported && !is_drawing, |ui| {
                    if ui
                        .checkbox(&mut self.click_through_mode, "Click-through mode")
                        .changed()
                    {
                        click_through_mode_changed = true;
                    }
                });

                if click_through_supported {
                    ui.label(format!(
                        "Restore shortcut: {}",
                        self.click_through_controller.restore_shortcut_label()
                    ));

                    if is_drawing {
                        ui.small("Finish the current stroke before enabling click-through.");
                    } else if self.click_through_mode {
                        ui.small("Mouse input is passing through to the window behind AetherInk.");
                    }
                } else {
                    ui.small("Click-through mode is currently available on Windows only.");
                }
            });

        self.is_settings_window_open = is_settings_window_open;

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
            self.set_click_through_mode(ctx, self.click_through_mode);

            if self.click_through_mode {
                self.is_settings_window_open = false;
            }
        }
    }

    fn apply_settings(&mut self, settings: AppSettings) {
        self.canvas.background = settings.background;
        self.canvas.transparent_background_opacity = settings.transparent_background_opacity;
        self.canvas.transparent_canvas_border_visibility =
            settings.transparent_canvas_border_visibility;
        self.always_on_top = settings.always_on_top;
        self.borderless_window = settings.borderless_window;
        self.click_through_mode = settings.click_through_mode;
        self.transparent_window_background = settings.transparent_window_background;
    }

    fn collect_settings(&self) -> AppSettings {
        AppSettings {
            background: self.canvas.background,
            transparent_background_opacity: self.canvas.transparent_background_opacity,
            transparent_canvas_border_visibility: self.canvas.transparent_canvas_border_visibility,
            always_on_top: self.always_on_top,
            borderless_window: self.borderless_window,
            click_through_mode: self.click_through_mode,
            transparent_window_background: self.transparent_window_background,
        }
    }

    fn apply_always_on_top(&self, ctx: &egui::Context) {
        let window_level = if self.always_on_top {
            egui::viewport::WindowLevel::AlwaysOnTop
        } else {
            egui::viewport::WindowLevel::Normal
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(window_level));
    }

    fn apply_borderless_window(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(
            !self.borderless_window,
        ));
    }

    fn set_click_through_mode(&mut self, ctx: &egui::Context, enabled: bool) {
        self.click_through_mode = enabled && self.click_through_controller.is_supported();
        self.apply_click_through_mode(ctx);
    }

    fn apply_click_through_mode(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(
            self.click_through_mode,
        ));

        if !self.click_through_mode {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
    }

    fn top_bar_fill_color(&self) -> egui::Color32 {
        if self.transparent_window_background {
            egui::Color32::from_rgba_unmultiplied(248, 246, 240, 168)
        } else {
            egui::Color32::from_rgba_unmultiplied(248, 246, 240, 245)
        }
    }

    fn central_panel_fill_color(&self) -> egui::Color32 {
        if self.transparent_window_background {
            egui::Color32::TRANSPARENT
        } else {
            self.canvas.background_color()
        }
    }
}

fn show_basic_pen_colors(ui: &mut egui::Ui, current_color: &mut egui::Color32) {
    ui.horizontal(|ui| {
        for (label, color) in BASIC_PEN_COLORS {
            let is_selected = *current_color == color;
            let stroke_color = if is_selected {
                egui::Color32::from_rgb(30, 30, 30)
            } else {
                egui::Color32::from_gray(120)
            };

            let response = ui
                .add(
                    egui::Button::new("")
                        .min_size(egui::vec2(18.0, 18.0))
                        .fill(color)
                        .stroke(egui::Stroke::new(1.0, stroke_color))
                        .corner_radius(9.0),
                )
                .on_hover_text(label);

            if response.clicked() {
                *current_color = color;
            }
        }
    });
}

fn undo_button() -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new("Undo")
            .strong()
            .color(egui::Color32::from_rgb(34, 44, 66)),
    )
    .fill(egui::Color32::from_rgb(227, 236, 248))
    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(127, 146, 179)))
    .corner_radius(6.0)
    .min_size(egui::vec2(62.0, 28.0))
}

fn clear_button() -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new("Clear")
            .strong()
            .color(egui::Color32::from_rgb(122, 32, 32)),
    )
    .fill(egui::Color32::from_rgb(252, 231, 231))
    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 38, 38)))
    .corner_radius(6.0)
    .min_size(egui::vec2(68.0, 28.0))
}
