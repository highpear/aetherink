mod app;
mod canvas;
mod stroke;
mod platform;

use app::AetherInkApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("AetherInk")
            .with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "AetherInk",
        options,
        Box::new(|_cc| Ok(Box::new(AetherInkApp::default()))),
    )
}