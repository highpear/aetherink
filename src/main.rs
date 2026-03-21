mod app;
mod canvas;
mod platform;
mod stroke;

use app::AetherInkApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("AetherInk")
            .with_inner_size([1000.0, 700.0])
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "AetherInk",
        options,
        Box::new(|cc| Ok(Box::new(AetherInkApp::new(cc)))),
    )
}
