use eframe::egui;

use crate::canvas::CanvasState;

#[derive(Debug, Default)]
pub struct AetherInkApp {
    canvas: CanvasState,
}

impl eframe::App for AetherInkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keyboard shortcuts
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z))) {
            self.canvas.undo();
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("AetherInk");

                ui.separator();

                ui.label("Color:");
                ui.color_edit_button_srgba(&mut self.canvas.current_color);

                ui.label("Width:");
                ui.add(egui::Slider::new(&mut self.canvas.current_width, 1.0..=20.0));

                ui.separator();

                if ui.button("Undo").clicked() {
                    self.canvas.undo();
                }

                if ui.button("Clear").clicked() {
                    self.canvas.clear();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Drag mouse to draw.");
            self.canvas.ui(ui);
        });

        ctx.request_repaint();
    }
}