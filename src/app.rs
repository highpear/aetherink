mod settings;
mod settings_window;
mod ui;

use eframe::egui;

use self::settings::AppSettings;
use self::ui::{
    clear_button, drawing_mode_label, keyboard_shortcut_pressed, show_basic_pen_colors,
    undo_button,
};
use crate::canvas::CanvasState;
use crate::platform::ClickThroughController;

const APP_SETTINGS_KEY: &str = "app_settings";

#[derive(Debug, Default)]
pub struct AetherInkApp {
    canvas: CanvasState,
    is_settings_window_open: bool,
    drawing_enabled: bool,
    temporary_drawing_active: bool,
    always_on_top: bool,
    borderless_window: bool,
    click_through_mode: bool,
    transparent_window_background: bool,
    click_through_controller: ClickThroughController,
}

impl eframe::App for AetherInkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_overlay_state(ctx);

        // Keyboard shortcuts
        if keyboard_shortcut_pressed(ctx, egui::Key::Z, false) {
            self.canvas.undo();
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
                if self.drawing_enabled {
                    ui.label("Drag mouse to draw.");
                } else {
                    ui.label("Drawing paused. Enable Draw to edit the canvas.");
                }

                self.canvas.ui(ui, self.drawing_enabled);
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
        app.temporary_drawing_active = false;
        app.apply_pointer_passthrough(&cc.egui_ctx);

        app
    }

    fn apply_settings(&mut self, settings: AppSettings) {
        self.canvas.background = settings.background;
        self.canvas.transparent_background_opacity = settings.transparent_background_opacity;
        self.canvas.transparent_canvas_border_visibility =
            settings.transparent_canvas_border_visibility;
        self.drawing_enabled = settings.drawing_enabled;
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
            drawing_enabled: self.drawing_enabled,
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

    fn set_drawing_enabled(&mut self, enabled: bool) {
        self.drawing_enabled = enabled;

        if !enabled {
            self.temporary_drawing_active = false;
            self.canvas.stop_drawing();
        }
    }

    fn set_click_through_mode(&mut self, ctx: &egui::Context, enabled: bool) {
        self.click_through_mode =
            enabled && self.click_through_controller.supports_pointer_passthrough();
        self.temporary_drawing_active = false;
        self.apply_pointer_passthrough(ctx);
    }

    fn set_temporary_drawing_active(&mut self, ctx: &egui::Context, active: bool) {
        self.temporary_drawing_active = active && self.click_through_mode && self.drawing_enabled;

        if !self.temporary_drawing_active {
            self.canvas.stop_drawing();
        }

        self.apply_pointer_passthrough(ctx);
    }

    fn sync_overlay_state(&mut self, ctx: &egui::Context) {
        if self.click_through_controller.poll_overlay_toggle_shortcut() {
            self.set_click_through_mode(ctx, !self.click_through_mode);
        }

        let temporary_drawing_active = self.click_through_mode
            && self.drawing_enabled
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
            self.show_overlay_status(ui);
            self.show_settings_button(ui);
        });
    }

    fn show_window_drag_handle(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if !self.borderless_window {
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
        ui.label("Color:");
        ui.color_edit_button_srgba(&mut self.canvas.current_color);
        show_basic_pen_colors(ui, &mut self.canvas.current_color);

        ui.label("Width:");
        ui.add(egui::Slider::new(&mut self.canvas.current_width, 1.0..=20.0));
        ui.separator();
    }

    fn show_drawing_mode_toggle(&mut self, ui: &mut egui::Ui) {
        if ui
            .selectable_label(self.drawing_enabled, drawing_mode_label(self.drawing_enabled))
            .on_hover_text("Toggle whether mouse dragging draws on the canvas")
            .clicked()
        {
            self.set_drawing_enabled(!self.drawing_enabled);
        }

        ui.separator();
    }

    fn show_canvas_actions(&mut self, ui: &mut egui::Ui) {
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
            .on_hover_text("Remove all strokes from the canvas (Ctrl+Shift+C or Ctrl+Delete)")
            .clicked()
        {
            self.canvas.clear();
        }

        ui.separator();
    }

    fn show_always_on_top_toggle(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui.checkbox(&mut self.always_on_top, "Always on top").changed() {
            self.apply_always_on_top(ctx);
        }
    }

    fn show_overlay_status(&self, ui: &mut egui::Ui) {
        if !self.click_through_mode {
            return;
        }

        ui.separator();
        let temporary_drawing_label = self.click_through_controller.temporary_drawing_shortcut_label();
        let overlay_toggle_shortcut_label = "Ctrl+Shift+O";
        let click_through_status = if self.temporary_drawing_active {
            format!(
                "Release {} to return to click-through, or press {} to toggle overlay off.",
                temporary_drawing_label,
                overlay_toggle_shortcut_label
            )
        } else {
            format!(
                "Click-through active. Hold {} to draw or press {} to toggle overlay.",
                temporary_drawing_label,
                overlay_toggle_shortcut_label
            )
        };

        ui.label(click_through_status);
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
        self.click_through_mode && !self.temporary_drawing_active
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
