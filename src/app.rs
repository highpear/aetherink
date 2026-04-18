mod settings;
mod settings_window;
mod ui;

use std::path::PathBuf;
use std::time::Duration;

use chrono::Local;
use eframe::egui;
use rfd::FileDialog;

use self::settings::{AppSettings, OverlaySettings};
use self::ui::{
    clear_button, drawing_mode_label, keyboard_shortcut_pressed, redo_button, save_png_button,
    show_pen_color_presets, show_pen_width_presets, undo_button,
};
use crate::canvas::CanvasState;
use crate::platform::ClickThroughController;
use crate::stroke::Tool;

const APP_SETTINGS_KEY: &str = "app_settings";
const CLICK_THROUGH_POLL_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExportStatusKind {
    Success,
    Error,
}

#[derive(Debug, Clone)]
struct ExportStatus {
    kind: ExportStatusKind,
    message: String,
}

#[derive(Debug, Default)]
pub struct AetherInkApp {
    canvas: CanvasState,
    overlay: OverlaySettings,
    is_settings_window_open: bool,
    export_status: Option<ExportStatus>,
    temporary_drawing_active: bool,
    click_through_controller: ClickThroughController,
}

impl eframe::App for AetherInkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_overlay_state(ctx);

        if keyboard_shortcut_pressed(ctx, egui::Key::Z, false) {
            self.canvas.undo();
        }

        if keyboard_shortcut_pressed(ctx, egui::Key::Z, true)
            || keyboard_shortcut_pressed(ctx, egui::Key::Y, false)
        {
            self.canvas.redo();
        }

        if keyboard_shortcut_pressed(ctx, egui::Key::C, true)
            || keyboard_shortcut_pressed(ctx, egui::Key::Delete, false)
        {
            self.canvas.clear();
        }

        self.show_top_bar(ctx);
        self.show_settings_window(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(self.central_panel_fill_color()))
            .show(ctx, |ui| {
                if self.overlay.drawing_enabled {
                    ui.label("Drag mouse to draw.");
                } else {
                    ui.label("Drawing paused. Enable Draw to edit the canvas.");
                }

                self.canvas.ui(ui, self.overlay.drawing_enabled);
            });

        self.schedule_repaint(ctx);
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

        if let Some(storage) = cc.storage
            && let Some(settings) = eframe::get_value(storage, APP_SETTINGS_KEY)
        {
            app.apply_settings(settings);
        }

        app.apply_always_on_top(&cc.egui_ctx);
        app.apply_borderless_window(&cc.egui_ctx);
        app.overlay.click_through_mode = false;
        app.temporary_drawing_active = false;
        app.apply_pointer_passthrough(&cc.egui_ctx);

        app
    }

    fn apply_settings(&mut self, settings: AppSettings) {
        self.canvas.apply_settings(settings.canvas);
        self.overlay = settings.overlay;
    }

    fn collect_settings(&self) -> AppSettings {
        AppSettings {
            canvas: self.canvas.settings(),
            overlay: self.overlay.clone(),
        }
    }

    fn apply_always_on_top(&self, ctx: &egui::Context) {
        let window_level = if self.overlay.always_on_top {
            egui::viewport::WindowLevel::AlwaysOnTop
        } else {
            egui::viewport::WindowLevel::Normal
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(window_level));
    }

    fn apply_borderless_window(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(
            !self.overlay.borderless_window,
        ));
    }

    fn set_drawing_enabled(&mut self, enabled: bool) {
        self.overlay.drawing_enabled = enabled;

        if !enabled {
            self.temporary_drawing_active = false;
            self.canvas.stop_drawing();
        }
    }

    fn set_click_through_mode(&mut self, ctx: &egui::Context, enabled: bool) {
        self.overlay.click_through_mode =
            enabled && self.click_through_controller.supports_pointer_passthrough();
        self.temporary_drawing_active = false;
        self.apply_pointer_passthrough(ctx);
    }

    fn set_temporary_drawing_active(&mut self, ctx: &egui::Context, active: bool) {
        self.temporary_drawing_active =
            active && self.overlay.click_through_mode && self.overlay.drawing_enabled;

        if !self.temporary_drawing_active {
            self.canvas.stop_drawing();
        }

        self.apply_pointer_passthrough(ctx);
    }

    fn sync_overlay_state(&mut self, ctx: &egui::Context) {
        if self.click_through_controller.poll_overlay_toggle_shortcut() {
            self.set_click_through_mode(ctx, !self.overlay.click_through_mode);
        }

        let temporary_drawing_active = self.overlay.click_through_mode
            && self.overlay.drawing_enabled
            && self.click_through_controller.is_temporary_drawing_active();

        if temporary_drawing_active != self.temporary_drawing_active {
            self.set_temporary_drawing_active(ctx, temporary_drawing_active);
        }
    }

    fn show_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar")
            .frame(egui::Frame::NONE.fill(self.top_bar_fill_color()))
            .show(ctx, |ui| {
                self.show_top_bar_contents(ui, ctx);
            });
    }

    fn show_top_bar_contents(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            self.show_window_drag_handle(ui, ctx);
            self.show_pen_controls(ui);
            self.show_drawing_mode_toggle(ui);
            self.show_canvas_actions(ui);
            self.show_always_on_top_toggle(ui, ctx);
            self.show_export_status(ui);
            self.show_overlay_status(ui);
            self.show_settings_button(ui);
        });
    }

    fn show_window_drag_handle(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if !self.overlay.borderless_window {
            return;
        }

        ui.heading("AetherInk");
        ui.separator();

        let drag_response =
            ui.add(egui::Label::new("Drag window").sense(egui::Sense::click_and_drag()));

        if drag_response.drag_started() {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        ui.separator();
    }

    fn show_pen_controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Tool:");

        for tool in [Tool::Pen, Tool::Eraser] {
            if ui
                .selectable_label(self.canvas.current_tool() == tool, tool.label())
                .clicked()
            {
                self.canvas.set_current_tool(tool);
            }
        }

        ui.separator();

        if self.canvas.current_tool() == Tool::Eraser {
            ui.label("Size:");
            ui.add(egui::Slider::new(
                self.canvas.eraser_radius_mut(),
                2.0..=32.0,
            ));
            ui.separator();
            return;
        }

        ui.label("Color:");
        ui.color_edit_button_srgba(self.canvas.current_color_mut());
        show_pen_color_presets(ui, self.canvas.current_color_mut());

        ui.label("Width:");
        show_pen_width_presets(ui, self.canvas.current_width_mut());
        ui.add(egui::Slider::new(
            self.canvas.current_width_mut(),
            1.0..=20.0,
        ));
        ui.separator();
    }

    fn show_drawing_mode_toggle(&mut self, ui: &mut egui::Ui) {
        if ui
            .selectable_label(
                self.overlay.drawing_enabled,
                drawing_mode_label(self.overlay.drawing_enabled),
            )
            .on_hover_text("Toggle whether mouse dragging draws on the canvas")
            .clicked()
        {
            self.set_drawing_enabled(!self.overlay.drawing_enabled);
        }

        ui.separator();
    }

    fn show_canvas_actions(&mut self, ui: &mut egui::Ui) {
        let can_undo = self.canvas.can_undo();
        let can_redo = self.canvas.can_redo();
        let has_strokes = self.canvas.has_strokes();

        if ui
            .add_enabled(can_undo, undo_button())
            .on_hover_text("Remove the last stroke (Ctrl+Z)")
            .clicked()
        {
            self.canvas.undo();
        }

        if ui
            .add_enabled(can_redo, redo_button())
            .on_hover_text("Restore the last undone change (Ctrl+Shift+Z or Ctrl+Y)")
            .clicked()
        {
            self.canvas.redo();
        }

        if ui
            .add_enabled(has_strokes, clear_button())
            .on_hover_text("Remove all strokes from the canvas (Ctrl+Shift+C or Ctrl+Delete)")
            .clicked()
        {
            self.canvas.clear();
        }

        if ui
            .add_enabled(has_strokes, save_png_button())
            .on_hover_text(if has_strokes {
                "Choose where to save the current canvas as a PNG file"
            } else {
                "Draw something on the canvas before saving a PNG"
            })
            .clicked()
        {
            self.export_status = match self.save_canvas_png() {
                Ok(Some(path)) => Some(ExportStatus {
                    kind: ExportStatusKind::Success,
                    message: format!("Saved PNG: {}", path.display()),
                }),
                Ok(None) => self.export_status.take(),
                Err(error) => Some(ExportStatus {
                    kind: ExportStatusKind::Error,
                    message: error,
                }),
            };
        }

        ui.separator();
    }

    fn show_always_on_top_toggle(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui
            .checkbox(&mut self.overlay.always_on_top, "Always on top")
            .changed()
        {
            self.apply_always_on_top(ctx);
        }
    }

    fn show_overlay_status(&self, ui: &mut egui::Ui) {
        if !self.overlay.click_through_mode {
            return;
        }

        ui.separator();
        let temporary_drawing_label = self
            .click_through_controller
            .temporary_drawing_shortcut_label();
        let overlay_toggle_shortcut_label = "Ctrl+Shift+O";
        let click_through_status = if self.temporary_drawing_active {
            format!(
                "Release {} to return to click-through, or press {} to toggle overlay off.",
                temporary_drawing_label, overlay_toggle_shortcut_label
            )
        } else {
            format!(
                "Click-through active. Hold {} to draw or press {} to toggle overlay.",
                temporary_drawing_label, overlay_toggle_shortcut_label
            )
        };

        ui.label(click_through_status);
    }

    fn show_export_status(&self, ui: &mut egui::Ui) {
        let Some(status) = &self.export_status else {
            return;
        };

        ui.separator();
        let color = match status.kind {
            ExportStatusKind::Success => egui::Color32::from_rgb(36, 94, 62),
            ExportStatusKind::Error => egui::Color32::from_rgb(165, 36, 36),
        };
        ui.label(egui::RichText::new(&status.message).color(color));
    }

    fn show_settings_button(&mut self, ui: &mut egui::Ui) {
        ui.separator();

        if ui.button("Settings").clicked() {
            self.is_settings_window_open = true;
        }
    }

    fn apply_pointer_passthrough(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(
            self.effective_click_through_mode(),
        ));

        if !self.effective_click_through_mode() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
    }

    fn effective_click_through_mode(&self) -> bool {
        self.overlay.click_through_mode && !self.temporary_drawing_active
    }

    fn top_bar_fill_color(&self) -> egui::Color32 {
        if self.overlay.transparent_window_background {
            egui::Color32::from_rgba_unmultiplied(248, 246, 240, 168)
        } else {
            egui::Color32::from_rgba_unmultiplied(248, 246, 240, 245)
        }
    }

    fn central_panel_fill_color(&self) -> egui::Color32 {
        if self.overlay.transparent_window_background {
            egui::Color32::TRANSPARENT
        } else {
            self.canvas.background_color()
        }
    }

    fn schedule_repaint(&self, ctx: &egui::Context) {
        if self.overlay.click_through_mode {
            ctx.request_repaint_after(CLICK_THROUGH_POLL_INTERVAL);
        }
    }

    fn save_canvas_png(&mut self) -> Result<Option<PathBuf>, String> {
        self.canvas.stop_drawing();

        let Some(path) = FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name(&export_file_name())
            .save_file()
        else {
            return Ok(None);
        };

        self.canvas.export_png(&path)?;

        Ok(Some(path))
    }
}

fn export_file_name() -> String {
    let timestamp = Local::now();

    format!("aetherink-canvas-{}.png", timestamp.format("%Y%m%d-%H%M%S"))
}
