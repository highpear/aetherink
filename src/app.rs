use eframe::egui;

use crate::canvas::{CanvasBackground, CanvasState};

#[derive(Debug, Default)]
pub struct AetherInkApp {
    canvas: CanvasState,
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

                ui.label("Width:");
                ui.add(egui::Slider::new(
                    &mut self.canvas.current_width,
                    1.0..=20.0,
                ));

                ui.separator();

                ui.label("Background:");
                egui::ComboBox::from_id_salt("canvas_background")
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

                if self.canvas.background == CanvasBackground::Transparent {
                    ui.label("Opacity:");
                    ui.add(
                        egui::Slider::new(
                            &mut self.canvas.transparent_background_opacity,
                            0.0..=1.0,
                        )
                        .show_value(true)
                        .fixed_decimals(2),
                    );
                }

                ui.separator();

                if ui.button("Undo").clicked() {
                    self.canvas.undo();
                }

                if ui.button("Clear").clicked() {
                    self.canvas.clear();
                }
            });
        });

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
