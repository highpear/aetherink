use eframe::egui;

use crate::canvas::CanvasState;

#[derive(Debug, Default)]
pub struct AetherInkApp {
    canvas: CanvasState,
}

impl eframe::App for AetherInkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("AetherInk");

                if ui.button("Clear").clicked() {
                    self.canvas.clear();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Drag mouse to draw black lines.");
            self.canvas.ui(ui);
        });

        ctx.request_repaint();
    }
}