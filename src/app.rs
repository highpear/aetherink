use eframe::egui;

use crate::canvas::{
    CanvasBackground, CanvasState, TransparentCanvasBorderVisibility,
};

const BASIC_PEN_COLORS: [(&str, egui::Color32); 5] = [
    ("Black", egui::Color32::BLACK),
    ("White", egui::Color32::WHITE),
    ("Red", egui::Color32::from_rgb(220, 38, 38)),
    ("Yellow", egui::Color32::from_rgb(234, 179, 8)),
    ("Blue", egui::Color32::from_rgb(37, 99, 235)),
];

#[derive(Debug, Default)]
pub struct AetherInkApp {
    canvas: CanvasState,
    is_settings_window_open: bool,
}

impl eframe::App for AetherInkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keyboard shortcuts
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Z,
            ))
        }) {
            self.canvas.undo();
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("AetherInk");

                ui.separator();

                ui.label("Color:");
                ui.color_edit_button_srgba(&mut self.canvas.current_color);
                show_basic_pen_colors(ui, &mut self.canvas.current_color);

                ui.label("Width:");
                ui.add(egui::Slider::new(
                    &mut self.canvas.current_width,
                    1.0..=20.0,
                ));

                ui.separator();

                if ui.button("Undo").clicked() {
                    self.canvas.undo();
                }

                if ui.button("Clear").clicked() {
                    self.canvas.clear();
                }

                ui.separator();

                if ui.button("Settings").clicked() {
                    self.is_settings_window_open = true;
                }
            });
        });

        self.show_settings_window(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(self.canvas.background_color()))
            .show(ctx, |ui| {
                ui.label("Drag mouse to draw.");
                self.canvas.ui(ui);
            });

        ctx.request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Color32::TRANSPARENT.to_normalized_gamma_f32()
    }
}

impl AetherInkApp {
    fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.is_settings_window_open {
            return;
        }

        egui::Window::new("Settings")
            .open(&mut self.is_settings_window_open)
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
            });
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
