mod settings;
mod settings_window;
mod ui;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use arboard::{Clipboard, ImageData};
use chrono::Local;
use eframe::egui;
use rfd::FileDialog;

use self::settings::{AppSettings, OverlaySettings};
use self::ui::{
    clear_button, copy_image_button, drawing_mode_label, ink_visibility_label,
    keyboard_shortcut_pressed, redo_button, save_menu_button, show_pen_color_presets,
    show_pen_width_presets, top_bar_group_label, undo_button,
};
use crate::canvas::{CanvasBackground, CanvasState};
use crate::platform::{
    BackgroundCaptureAvailability, BackgroundCaptureController, ClickThroughController,
};
use crate::stroke::Tool;

const APP_SETTINGS_KEY: &str = "app_settings";
const CLICK_THROUGH_POLL_INTERVAL: Duration = Duration::from_millis(16);
const BACKGROUND_CAPTURE_DIALOG_DISMISS_DELAY: Duration = Duration::from_millis(400);
const SUCCESS_TOAST_DURATION: Duration = Duration::from_secs(3);
const ERROR_TOAST_DURATION: Duration = Duration::from_secs(5);
#[cfg(debug_assertions)]
const DEBUG_BACKGROUND_CAPTURE_ENV: &str = "AETHERINK_DEBUG_BACKGROUND_CAPTURE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExportStatusKind {
    Success,
    Error,
}

#[derive(Debug, Clone)]
struct ExportStatus {
    kind: ExportStatusKind,
    message: String,
    visible_until: Instant,
}

#[derive(Debug, Clone)]
enum PendingBackgroundCaptureTarget {
    PngExport { path: PathBuf },
    ClipboardCopy,
}

#[derive(Debug, Clone)]
struct PendingBackgroundCapture {
    target: PendingBackgroundCaptureTarget,
    capture_not_before: Instant,
    capture_ready: bool,
}

#[derive(Debug, Default)]
pub struct AetherInkApp {
    canvas: CanvasState,
    overlay: OverlaySettings,
    last_export_directory: Option<PathBuf>,
    quick_save_directory: Option<PathBuf>,
    is_settings_window_open: bool,
    copy_includes_screen_background: bool,
    export_status: Option<ExportStatus>,
    temporary_drawing_active: bool,
    click_through_controller: ClickThroughController,
    background_capture_controller: BackgroundCaptureController,
    background_capture_availability: BackgroundCaptureAvailability,
    pending_background_capture: Option<PendingBackgroundCapture>,
}

impl eframe::App for AetherInkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.capture_pending_background(ctx);
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

        if self.canvas.has_strokes() && keyboard_shortcut_pressed(ctx, egui::Key::S, false) {
            self.start_png_export();
        }

        if self.canvas.has_strokes() && keyboard_shortcut_pressed(ctx, egui::Key::S, true) {
            self.start_quick_png_export();
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

                if self.should_hide_canvas_for_background_capture() {
                    self.canvas
                        .ui_hidden_for_background_capture(ui, self.overlay.drawing_enabled);
                } else {
                    self.canvas.ui(ui, self.overlay.drawing_enabled);
                }
            });

        self.mark_pending_background_capture_ready(ctx);
        self.show_overlay_status_banner(ctx);
        self.show_export_toast(ctx);
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
        app.background_capture_controller = BackgroundCaptureController::new(cc);
        app.refresh_background_capture_availability();

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
        self.last_export_directory = settings.last_export_directory;
        self.quick_save_directory = settings.quick_save_directory;
        self.copy_includes_screen_background = settings.copy_includes_screen_background;
    }

    fn collect_settings(&self) -> AppSettings {
        AppSettings {
            canvas: self.canvas.settings(),
            overlay: self.overlay.clone(),
            last_export_directory: self.last_export_directory.clone(),
            quick_save_directory: self.quick_save_directory.clone(),
            copy_includes_screen_background: self.copy_includes_screen_background,
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
        self.overlay.click_through_mode = enabled && self.can_enable_click_through_mode();
        self.temporary_drawing_active = false;
        self.apply_pointer_passthrough(ctx);
    }

    fn can_enable_click_through_mode(&self) -> bool {
        self.click_through_controller.supports_pointer_passthrough()
            && self.click_through_controller.supports_shortcut_monitoring()
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
        // Poll every frame so the edge detector stays current, but only let the
        // global (system-wide) shortcut toggle click-through while it is on -
        // the only state where this window cannot receive key events. While
        // click-through is off, require the focused-window shortcut so
        // Ctrl+Shift+O pressed in another application cannot enable
        // passthrough behind the user's back.
        let global_toggle_pressed = self.click_through_controller.poll_overlay_toggle_shortcut();
        let toggle_pressed = if self.overlay.click_through_mode {
            global_toggle_pressed
        } else {
            keyboard_shortcut_pressed(ctx, egui::Key::O, true)
        };

        if toggle_pressed {
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
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            self.show_window_drag_handle(ui, ctx);
            self.show_drawing_group(ui);
            self.show_style_group(ui);
            self.show_export_group(ui);
            self.show_overlay_group(ui, ctx);
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

    fn show_drawing_group(&mut self, ui: &mut egui::Ui) {
        top_bar_group_label(ui, "Drawing");

        for tool in [Tool::Pen, Tool::Eraser] {
            if ui
                .selectable_label(self.canvas.current_tool() == tool, tool.label())
                .clicked()
            {
                self.canvas.set_current_tool(tool);
            }
        }

        self.show_drawing_mode_toggle(ui);
        self.show_ink_visibility_toggle(ui);

        let can_undo = self.canvas.can_undo();
        let can_redo = self.canvas.can_redo();

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
            .add_enabled(self.canvas.has_strokes(), clear_button())
            .on_hover_text("Remove all strokes from the canvas (Ctrl+Shift+C or Ctrl+Delete)")
            .clicked()
        {
            self.canvas.clear();
        }

        ui.separator();
    }

    fn show_style_group(&mut self, ui: &mut egui::Ui) {
        top_bar_group_label(ui, "Style");

        if self.canvas.current_tool() == Tool::Eraser {
            ui.label("Size");
            ui.add(egui::Slider::new(
                self.canvas.eraser_radius_mut(),
                2.0..=32.0,
            ));
            ui.separator();
            return;
        }

        show_pen_color_presets(ui, self.canvas.current_color_mut());

        ui.label("Width");
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
    }

    fn show_ink_visibility_toggle(&mut self, ui: &mut egui::Ui) {
        let ink_visible = self.canvas.ink_visible();

        if ui
            .selectable_label(ink_visible, ink_visibility_label(ink_visible))
            .on_hover_text("Show or hide ink without clearing strokes")
            .clicked()
        {
            self.canvas.set_ink_visible(!ink_visible);
        }
    }

    fn show_export_group(&mut self, ui: &mut egui::Ui) {
        top_bar_group_label(ui, "Export");

        let has_strokes = self.canvas.has_strokes();

        if ui
            .add_enabled(has_strokes, copy_image_button())
            .on_hover_text(if !has_strokes {
                "Draw something on the canvas before copying an image"
            } else if self.copy_should_include_screen_background() {
                "Copy the transparent canvas with the screen background behind it"
            } else {
                "Copy the current canvas image to the clipboard"
            })
            .clicked()
        {
            self.start_clipboard_image_copy();
        }

        let is_transparent_canvas = self.canvas.background() == CanvasBackground::Transparent;
        let background_capture_availability = self.background_capture_availability;
        let can_save_background_png = is_transparent_canvas
            && background_capture_availability != BackgroundCaptureAvailability::Unsupported;
        let can_open_save_menu = has_strokes || can_save_background_png;

        ui.add_enabled_ui(can_open_save_menu, |ui| {
            let (response, _) = egui::containers::menu::MenuButton::from_button(save_menu_button())
                .ui(ui, |ui| {
                    self.show_save_menu_contents(
                        ui,
                        has_strokes,
                        is_transparent_canvas,
                        background_capture_availability,
                        can_save_background_png,
                    );
                });

            response.on_hover_text(if can_open_save_menu {
                "Save the canvas or choose another PNG export option"
            } else {
                "Draw something on the canvas before saving a PNG"
            });
        });

        ui.separator();
    }

    fn show_save_menu_contents(
        &mut self,
        ui: &mut egui::Ui,
        has_strokes: bool,
        is_transparent_canvas: bool,
        background_capture_availability: BackgroundCaptureAvailability,
        can_save_background_png: bool,
    ) {
        if ui
            .add_enabled(has_strokes, egui::Button::new("Save PNG..."))
            .on_hover_text(if has_strokes {
                "Choose where to save the current canvas as a PNG file (Ctrl/Cmd+S)"
            } else {
                "Draw something on the canvas before saving a PNG"
            })
            .clicked()
        {
            ui.close();
            self.start_png_export();
        }

        let can_quick_save = has_strokes && self.quick_save_directory.is_some();

        if ui
            .add_enabled(can_quick_save, egui::Button::new("Quick Save"))
            .on_hover_text(if self.quick_save_directory.is_some() {
                "Save the current canvas to the quick save folder (Ctrl/Cmd+Shift+S)"
            } else {
                "Choose a quick save folder in Settings before using Quick Save"
            })
            .clicked()
        {
            ui.close();
            self.start_quick_png_export();
        }

        if ui
            .add_enabled(
                can_save_background_png,
                egui::Button::new("Save Background PNG"),
            )
            .on_hover_text(background_png_button_hover_text(
                is_transparent_canvas,
                background_capture_availability,
            ))
            .clicked()
        {
            ui.close();
            self.start_captured_background_png_export();
        }
    }

    fn show_overlay_group(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        top_bar_group_label(ui, "Overlay");
        self.show_always_on_top_toggle(ui, ctx);
        self.show_settings_button(ui);
    }

    fn show_always_on_top_toggle(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui
            .checkbox(&mut self.overlay.always_on_top, "Always on top")
            .changed()
        {
            self.apply_always_on_top(ctx);
        }
    }

    fn overlay_status_text(&self) -> Option<String> {
        if !self.overlay.click_through_mode {
            return None;
        }

        let temporary_drawing_label = self
            .click_through_controller
            .temporary_drawing_shortcut_label();
        let overlay_toggle_shortcut_label = "Ctrl+Shift+O";
        let status = if self.temporary_drawing_active {
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

        Some(status)
    }

    fn show_overlay_status_banner(&self, ctx: &egui::Context) {
        let Some(status) = self.overlay_status_text() else {
            return;
        };

        let (fill_color, stroke_color, text_color) = if self.temporary_drawing_active {
            (
                egui::Color32::from_rgba_unmultiplied(230, 247, 236, 232),
                egui::Color32::from_rgb(86, 162, 118),
                egui::Color32::from_rgb(36, 94, 62),
            )
        } else {
            (
                egui::Color32::from_rgba_unmultiplied(227, 236, 248, 232),
                egui::Color32::from_rgb(127, 146, 179),
                egui::Color32::from_rgb(34, 44, 66),
            )
        };

        egui::Area::new("overlay_status_banner".into())
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -16.0))
            .interactable(false)
            .show(ctx, |ui| {
                ui.set_max_width(640.0);
                egui::Frame::new()
                    .fill(fill_color)
                    .stroke(egui::Stroke::new(1.0, stroke_color))
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(status).color(text_color)).wrap(),
                        );
                    });
            });
    }

    fn show_export_toast(&mut self, ctx: &egui::Context) {
        let Some(status) = &self.export_status else {
            return;
        };

        if Instant::now() >= status.visible_until {
            self.export_status = None;
            return;
        }

        let (fill_color, stroke_color, text_color) = match status.kind {
            ExportStatusKind::Success => (
                egui::Color32::from_rgba_unmultiplied(230, 247, 236, 244),
                egui::Color32::from_rgb(86, 162, 118),
                egui::Color32::from_rgb(36, 94, 62),
            ),
            ExportStatusKind::Error => (
                egui::Color32::from_rgba_unmultiplied(252, 232, 232, 244),
                egui::Color32::from_rgb(220, 68, 68),
                egui::Color32::from_rgb(165, 36, 36),
            ),
        };

        egui::Area::new("export_toast".into())
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -16.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(fill_color)
                    .stroke(egui::Stroke::new(1.0, stroke_color))
                    .corner_radius(10.0)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(&status.message).color(text_color));
                    });
            });
    }

    fn show_settings_button(&mut self, ui: &mut egui::Ui) {
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
        if self.overlay.transparent_window_background
            || self.should_hide_canvas_for_background_capture()
        {
            egui::Color32::TRANSPARENT
        } else {
            self.canvas.background_color()
        }
    }

    fn schedule_repaint(&self, ctx: &egui::Context) {
        if self.overlay.click_through_mode {
            ctx.request_repaint_after(CLICK_THROUGH_POLL_INTERVAL);
        }

        if self.pending_background_capture.is_some() {
            let repaint_after = self
                .pending_background_capture
                .as_ref()
                .map(|pending_export| {
                    pending_export
                        .capture_not_before
                        .saturating_duration_since(Instant::now())
                        .min(CLICK_THROUGH_POLL_INTERVAL)
                })
                .unwrap_or(CLICK_THROUGH_POLL_INTERVAL);
            ctx.request_repaint_after(repaint_after);
        }

        if let Some(status) = &self.export_status {
            let remaining = status
                .visible_until
                .saturating_duration_since(Instant::now());
            ctx.request_repaint_after(remaining);
        }
    }

    fn save_canvas_png(&mut self) -> Result<Option<PathBuf>, String> {
        self.canvas.stop_drawing();

        let mut file_dialog = FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name(export_file_name());

        if let Some(directory) = &self.last_export_directory {
            file_dialog = file_dialog.set_directory(directory);
        }

        let Some(path) = file_dialog.save_file() else {
            return Ok(None);
        };
        let path = ensure_png_extension(&path);

        self.last_export_directory = path.parent().map(Path::to_path_buf);

        self.canvas.export_png(&path)?;

        Ok(Some(path))
    }

    fn begin_canvas_with_captured_background_png_export(&mut self) -> Result<bool, String> {
        let mut file_dialog = FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name(background_export_file_name());

        if let Some(directory) = &self.last_export_directory {
            file_dialog = file_dialog.set_directory(directory);
        }

        let Some(path) = file_dialog.save_file() else {
            return Ok(false);
        };
        let path = ensure_png_extension(&path);

        self.last_export_directory = path.parent().map(Path::to_path_buf);

        self.canvas.stop_drawing();
        self.pending_background_capture = Some(PendingBackgroundCapture {
            target: PendingBackgroundCaptureTarget::PngExport { path },
            capture_not_before: Instant::now() + BACKGROUND_CAPTURE_DIALOG_DISMISS_DELAY,
            capture_ready: false,
        });

        Ok(true)
    }

    fn begin_canvas_with_captured_background_clipboard_copy(&mut self) {
        self.canvas.stop_drawing();
        self.pending_background_capture = Some(PendingBackgroundCapture {
            target: PendingBackgroundCaptureTarget::ClipboardCopy,
            capture_not_before: Instant::now() + BACKGROUND_CAPTURE_DIALOG_DISMISS_DELAY,
            capture_ready: false,
        });
    }

    fn quick_save_canvas_png(&mut self) -> Result<PathBuf, String> {
        self.canvas.stop_drawing();

        let Some(directory) = &self.quick_save_directory else {
            return Err(String::from(
                "Choose a quick save folder in Settings before using Quick Save.",
            ));
        };

        if !directory.is_dir() {
            return Err(format!(
                "Quick save folder is not available: {}",
                directory.display()
            ));
        }

        let path = directory.join(export_file_name());
        self.canvas.export_png(&path)?;

        Ok(path)
    }

    fn copy_canvas_image_to_clipboard(&mut self) -> Result<(), String> {
        self.canvas.stop_drawing();

        let image = self.canvas.render_image()?;
        copy_image_to_clipboard(image)
    }

    fn copy_canvas_with_captured_background_to_clipboard(
        &mut self,
        ctx: &egui::Context,
    ) -> Result<(), String> {
        let image = self.render_canvas_with_captured_background(ctx, None)?;
        copy_image_to_clipboard(image)
    }

    fn copy_should_include_screen_background(&self) -> bool {
        self.copy_includes_screen_background
            && self.canvas.background() == CanvasBackground::Transparent
    }

    fn render_canvas_with_captured_background(
        &mut self,
        ctx: &egui::Context,
        debug_export_path: Option<&Path>,
    ) -> Result<image::RgbaImage, String> {
        self.canvas.stop_drawing();

        if !self
            .background_capture_controller
            .supports_background_capture()
        {
            return Err(String::from("Background capture is not available yet."));
        }

        let capture_rect = self
            .canvas
            .screen_capture_rect(ctx)
            .ok_or_else(|| String::from("The canvas screen position is not available yet."))?;
        let background = self
            .background_capture_controller
            .capture_background(capture_rect)?;

        #[cfg(debug_assertions)]
        if let Some(debug_export_path) = debug_export_path {
            save_debug_background_capture(&background, debug_export_path)?;
        }

        self.canvas
            .render_image_over_screen_background(background, ctx)
    }

    fn should_hide_canvas_for_background_capture(&self) -> bool {
        self.pending_background_capture.is_some()
    }
    fn mark_pending_background_capture_ready(&mut self, ctx: &egui::Context) {
        if let Some(pending_export) = &mut self.pending_background_capture
            && !pending_export.capture_ready
        {
            let now = Instant::now();
            if now < pending_export.capture_not_before {
                ctx.request_repaint_after(pending_export.capture_not_before - now);
                return;
            }

            pending_export.capture_ready = true;
            ctx.request_repaint();
        }
    }

    fn capture_pending_background(&mut self, ctx: &egui::Context) {
        let Some(pending_export) = &self.pending_background_capture else {
            return;
        };

        if !pending_export.capture_ready {
            return;
        }

        let target = pending_export.target.clone();
        self.pending_background_capture = None;

        self.export_status = match target {
            PendingBackgroundCaptureTarget::PngExport { path } => {
                match self.save_captured_background_png(ctx, &path) {
                    Ok(()) => Some(ExportStatus {
                        kind: ExportStatusKind::Success,
                        message: format!("Saved background PNG: {}", path.display()),
                        visible_until: Instant::now() + SUCCESS_TOAST_DURATION,
                    }),
                    Err(error) => Some(ExportStatus {
                        kind: ExportStatusKind::Error,
                        message: error,
                        visible_until: Instant::now() + ERROR_TOAST_DURATION,
                    }),
                }
            }
            PendingBackgroundCaptureTarget::ClipboardCopy => {
                match self.copy_canvas_with_captured_background_to_clipboard(ctx) {
                    Ok(()) => Some(ExportStatus {
                        kind: ExportStatusKind::Success,
                        message: String::from(
                            "Copied canvas image with screen background to clipboard.",
                        ),
                        visible_until: Instant::now() + SUCCESS_TOAST_DURATION,
                    }),
                    Err(error) => Some(ExportStatus {
                        kind: ExportStatusKind::Error,
                        message: error,
                        visible_until: Instant::now() + ERROR_TOAST_DURATION,
                    }),
                }
            }
        };
    }

    fn save_captured_background_png(
        &mut self,
        ctx: &egui::Context,
        path: &Path,
    ) -> Result<(), String> {
        let image = self.render_canvas_with_captured_background(ctx, Some(path))?;
        image
            .save(path)
            .map_err(|error| format!("Failed to save PNG: {error}"))
    }

    fn start_png_export(&mut self) {
        self.export_status = match self.save_canvas_png() {
            Ok(Some(path)) => Some(ExportStatus {
                kind: ExportStatusKind::Success,
                message: format!("Saved PNG: {}", path.display()),
                visible_until: Instant::now() + SUCCESS_TOAST_DURATION,
            }),
            Ok(None) => self.export_status.take(),
            Err(error) => Some(ExportStatus {
                kind: ExportStatusKind::Error,
                message: error,
                visible_until: Instant::now() + ERROR_TOAST_DURATION,
            }),
        };
    }

    fn start_clipboard_image_copy(&mut self) {
        if self.copy_should_include_screen_background() {
            self.start_clipboard_image_copy_with_screen_background();
            return;
        }

        self.export_status = match self.copy_canvas_image_to_clipboard() {
            Ok(()) => Some(ExportStatus {
                kind: ExportStatusKind::Success,
                message: String::from("Copied canvas image to clipboard."),
                visible_until: Instant::now() + SUCCESS_TOAST_DURATION,
            }),
            Err(error) => Some(ExportStatus {
                kind: ExportStatusKind::Error,
                message: error,
                visible_until: Instant::now() + ERROR_TOAST_DURATION,
            }),
        };
    }

    fn start_clipboard_image_copy_with_screen_background(&mut self) {
        self.refresh_background_capture_availability();

        if self.background_capture_availability == BackgroundCaptureAvailability::PermissionRequired
        {
            self.request_background_capture_permission();
            return;
        }

        if self.background_capture_availability == BackgroundCaptureAvailability::Unsupported {
            self.export_status = Some(ExportStatus {
                kind: ExportStatusKind::Error,
                message: String::from("Background copy is not available on this platform yet."),
                visible_until: Instant::now() + ERROR_TOAST_DURATION,
            });
            return;
        }

        self.begin_canvas_with_captured_background_clipboard_copy();
        // Drop any visible toast so it cannot appear in the captured screen area.
        self.export_status = None;
    }

    fn start_captured_background_png_export(&mut self) {
        self.refresh_background_capture_availability();

        if self.background_capture_availability == BackgroundCaptureAvailability::PermissionRequired
        {
            self.request_background_capture_permission();
            return;
        }

        self.export_status = match self.begin_canvas_with_captured_background_png_export() {
            // Drop any visible toast so it cannot appear in the captured screen area.
            Ok(true) => None,
            Ok(false) => self.export_status.take(),
            Err(error) => Some(ExportStatus {
                kind: ExportStatusKind::Error,
                message: error,
                visible_until: Instant::now() + ERROR_TOAST_DURATION,
            }),
        };
    }

    fn start_quick_png_export(&mut self) {
        self.export_status = match self.quick_save_canvas_png() {
            Ok(path) => Some(ExportStatus {
                kind: ExportStatusKind::Success,
                message: format!("Quick saved PNG: {}", path.display()),
                visible_until: Instant::now() + SUCCESS_TOAST_DURATION,
            }),
            Err(error) => Some(ExportStatus {
                kind: ExportStatusKind::Error,
                message: error,
                visible_until: Instant::now() + ERROR_TOAST_DURATION,
            }),
        };
    }

    fn refresh_background_capture_availability(&mut self) {
        self.background_capture_availability = self
            .background_capture_controller
            .background_capture_availability();
    }

    fn request_background_capture_permission(&mut self) {
        self.export_status = match self
            .background_capture_controller
            .request_background_capture_permission()
        {
            Ok(true) => {
                self.refresh_background_capture_availability();
                Some(ExportStatus {
                    kind: ExportStatusKind::Success,
                    message: String::from(
                        "Screen Recording permission is available. Restart AetherInk if background capture still fails.",
                    ),
                    visible_until: Instant::now() + SUCCESS_TOAST_DURATION,
                })
            }
            Ok(false) => {
                self.refresh_background_capture_availability();
                Some(ExportStatus {
                    kind: ExportStatusKind::Error,
                    message: String::from(
                        "Grant Screen Recording permission in System Settings, then restart AetherInk.",
                    ),
                    visible_until: Instant::now() + ERROR_TOAST_DURATION,
                })
            }
            Err(error) => Some(ExportStatus {
                kind: ExportStatusKind::Error,
                message: error,
                visible_until: Instant::now() + ERROR_TOAST_DURATION,
            }),
        };
    }
}

fn export_file_name() -> String {
    let timestamp = Local::now();

    format!("aetherink-canvas-{}.png", timestamp.format("%Y%m%d-%H%M%S"))
}

fn background_export_file_name() -> String {
    let timestamp = Local::now();

    format!(
        "aetherink-background-{}.png",
        timestamp.format("%Y%m%d-%H%M%S")
    )
}

fn copy_image_to_clipboard(image: image::RgbaImage) -> Result<(), String> {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let bytes = image.into_raw();
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("Failed to open clipboard: {error}"))?;

    clipboard
        .set_image(ImageData {
            width,
            height,
            bytes: bytes.into(),
        })
        .map_err(|error| format!("Failed to copy image: {error}"))
}

fn background_png_button_hover_text(
    is_transparent_canvas: bool,
    background_capture_availability: BackgroundCaptureAvailability,
) -> &'static str {
    if !is_transparent_canvas {
        "Switch the canvas background to Transparent before saving a background PNG"
    } else if background_capture_availability == BackgroundCaptureAvailability::Unsupported {
        "Background PNG export is not available on this platform yet"
    } else if background_capture_availability == BackgroundCaptureAvailability::PermissionRequired {
        "Screen Recording permission is required for background PNG export"
    } else {
        "Save the transparent canvas area with the screen background behind it"
    }
}

fn ensure_png_extension(path: &Path) -> PathBuf {
    let has_png_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"));

    if has_png_extension {
        path.to_path_buf()
    } else {
        path.with_extension("png")
    }
}

#[cfg(debug_assertions)]
fn save_debug_background_capture(
    background: &image::RgbaImage,
    export_path: &Path,
) -> Result<(), String> {
    if std::env::var_os(DEBUG_BACKGROUND_CAPTURE_ENV).is_none() {
        return Ok(());
    }

    let path = debug_background_capture_path(export_path);
    background
        .save(&path)
        .map_err(|error| format!("Failed to save debug background capture: {error}"))
}

#[cfg(debug_assertions)]
fn debug_background_capture_path(export_path: &Path) -> PathBuf {
    let file_name = export_path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .map(|file_stem| format!("{file_stem}-raw-background.png"))
        .unwrap_or_else(|| String::from("aetherink-raw-background.png"));

    export_path
        .parent()
        .map(|parent| parent.join(&file_name))
        .unwrap_or_else(|| PathBuf::from(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_png_extension_adds_missing_extension() {
        assert_eq!(
            ensure_png_extension(Path::new("drawing")),
            PathBuf::from("drawing.png")
        );
    }

    #[test]
    fn ensure_png_extension_replaces_non_png_extension() {
        assert_eq!(
            ensure_png_extension(Path::new("drawing.jpeg")),
            PathBuf::from("drawing.png")
        );
    }

    #[test]
    fn ensure_png_extension_keeps_png_extension_case_insensitively() {
        assert_eq!(
            ensure_png_extension(Path::new("drawing.PNG")),
            PathBuf::from("drawing.PNG")
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    fn debug_background_capture_path_uses_export_directory_and_stem() {
        assert_eq!(
            debug_background_capture_path(Path::new("exports/aetherink-background-20260521.png")),
            PathBuf::from("exports/aetherink-background-20260521-raw-background.png")
        );
    }

    #[test]
    fn background_png_hover_text_requires_transparent_canvas() {
        assert_eq!(
            background_png_button_hover_text(false, BackgroundCaptureAvailability::Available),
            "Switch the canvas background to Transparent before saving a background PNG"
        );
    }

    #[test]
    fn background_png_hover_text_reports_unsupported_platform() {
        assert_eq!(
            background_png_button_hover_text(true, BackgroundCaptureAvailability::Unsupported),
            "Background PNG export is not available on this platform yet"
        );
    }

    #[test]
    fn background_png_hover_text_reports_missing_permission() {
        assert_eq!(
            background_png_button_hover_text(
                true,
                BackgroundCaptureAvailability::PermissionRequired
            ),
            "Screen Recording permission is required for background PNG export"
        );
    }

    #[test]
    fn background_png_hover_text_describes_available_export() {
        assert_eq!(
            background_png_button_hover_text(true, BackgroundCaptureAvailability::Available),
            "Save the transparent canvas area with the screen background behind it"
        );
    }
}
