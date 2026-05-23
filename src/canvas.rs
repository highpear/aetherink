use std::path::Path;

mod eraser;
mod geometry;
mod pen;
mod raster;

use egui::{Color32, CursorIcon, Response, Sense, Shape, Stroke, Ui};
use image::RgbaImage;
use serde::{Deserialize, Serialize};

use self::eraser::erase_from_strokes;
use self::geometry::{
    canvas_image_size_from_rect, canvas_rect_to_screen_capture_rect, is_near_canvas_edge,
};
use self::pen::push_pen_point_if_needed;
use self::raster::{draw_screen_stroke_on_image, draw_stroke_on_image, rgba_from_color32};
use crate::stroke::{DrawStroke, Tool};

const DEFAULT_WHITE_BACKGROUND: Color32 = Color32::from_rgb(248, 246, 240);
const TRANSPARENT_CANVAS_BORDER: Color32 = Color32::from_gray(180);
const DEFAULT_ERASER_RADIUS: f32 = 8.0;
const PEN_CURSOR_MIN_RADIUS: f32 = 2.0;
const DISABLED_CURSOR_SIZE: f32 = 7.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasBackground {
    White,
    Transparent,
}

impl CanvasBackground {
    pub fn label(self) -> &'static str {
        match self {
            Self::White => "White",
            Self::Transparent => "Transparent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransparentCanvasBorderVisibility {
    Always,
    NearEdges,
}

impl TransparentCanvasBorderVisibility {
    pub fn label(self) -> &'static str {
        match self {
            Self::Always => "Always",
            Self::NearEdges => "Near edges",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasSettings {
    pub background: CanvasBackground,
    pub transparent_background_opacity: f32,
    pub transparent_canvas_border_visibility: TransparentCanvasBorderVisibility,
    #[serde(default = "default_ink_visible")]
    pub ink_visible: bool,
    pub default_pen_color: [u8; 4],
    pub default_pen_width: f32,
    pub eraser_radius: f32,
}

impl Default for CanvasSettings {
    fn default() -> Self {
        Self {
            background: CanvasBackground::White,
            transparent_background_opacity: 0.0,
            transparent_canvas_border_visibility: TransparentCanvasBorderVisibility::NearEdges,
            ink_visible: true,
            default_pen_color: egui::Color32::BLACK.to_array(),
            default_pen_width: 2.0,
            eraser_radius: DEFAULT_ERASER_RADIUS,
        }
    }
}

fn default_ink_visible() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ScreenCaptureRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct CanvasState {
    strokes: Vec<DrawStroke>,
    history: Vec<Vec<DrawStroke>>,
    redo_history: Vec<Vec<DrawStroke>>,
    last_canvas_rect: Option<egui::Rect>,
    current_stroke: Option<DrawStroke>,
    current_eraser_path: Vec<egui::Pos2>,
    current_color: Color32,
    current_width: f32,
    current_tool: Tool,
    eraser_radius: f32,
    settings: CanvasSettings,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            history: Vec::new(),
            redo_history: Vec::new(),
            last_canvas_rect: None,
            current_stroke: None,
            current_eraser_path: Vec::new(),
            current_color: Color32::BLACK,
            current_width: 2.0,
            current_tool: Tool::Pen,
            eraser_radius: DEFAULT_ERASER_RADIUS,
            settings: CanvasSettings::default(),
        }
    }
}

impl CanvasState {
    pub fn current_tool(&self) -> Tool {
        self.current_tool
    }

    pub fn set_current_tool(&mut self, tool: Tool) {
        if self.current_tool != tool {
            self.stop_drawing();
            self.current_tool = tool;
        }
    }

    pub fn current_color_mut(&mut self) -> &mut Color32 {
        &mut self.current_color
    }

    pub fn current_width_mut(&mut self) -> &mut f32 {
        &mut self.current_width
    }

    pub fn eraser_radius_mut(&mut self) -> &mut f32 {
        &mut self.eraser_radius
    }

    pub fn apply_settings(&mut self, settings: CanvasSettings) {
        self.current_color = egui::Color32::from_rgba_unmultiplied(
            settings.default_pen_color[0],
            settings.default_pen_color[1],
            settings.default_pen_color[2],
            settings.default_pen_color[3],
        );
        self.current_width = settings.default_pen_width;
        self.eraser_radius = settings.eraser_radius;
        self.settings = settings;
    }

    pub fn settings(&self) -> CanvasSettings {
        let mut settings = self.settings.clone();
        settings.default_pen_color = self.current_color.to_array();
        settings.default_pen_width = self.current_width;
        settings.eraser_radius = self.eraser_radius;
        settings
    }

    pub fn has_strokes(&self) -> bool {
        !self.strokes.is_empty()
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_history.is_empty()
    }

    pub fn is_drawing(&self) -> bool {
        self.current_stroke.is_some()
    }

    pub fn background(&self) -> CanvasBackground {
        self.settings.background
    }

    pub fn background_mut(&mut self) -> &mut CanvasBackground {
        &mut self.settings.background
    }

    pub fn background_color(&self) -> Color32 {
        match self.settings.background {
            CanvasBackground::White => DEFAULT_WHITE_BACKGROUND,
            CanvasBackground::Transparent => {
                let alpha =
                    (self.settings.transparent_background_opacity.clamp(0.0, 1.0) * 255.0) as u8;
                Color32::from_white_alpha(alpha)
            }
        }
    }

    pub fn ink_visible(&self) -> bool {
        self.settings.ink_visible
    }

    pub fn set_ink_visible(&mut self, visible: bool) {
        if self.settings.ink_visible != visible {
            self.stop_drawing();
            self.settings.ink_visible = visible;
        }
    }

    pub fn transparent_background_opacity_mut(&mut self) -> &mut f32 {
        &mut self.settings.transparent_background_opacity
    }

    pub fn transparent_canvas_border_visibility(&self) -> TransparentCanvasBorderVisibility {
        self.settings.transparent_canvas_border_visibility
    }

    pub fn transparent_canvas_border_visibility_mut(
        &mut self,
    ) -> &mut TransparentCanvasBorderVisibility {
        &mut self.settings.transparent_canvas_border_visibility
    }

    pub fn clear(&mut self) {
        if self.strokes.is_empty() {
            self.current_stroke = None;
            self.current_eraser_path.clear();
            return;
        }

        self.push_history_snapshot();
        self.strokes.clear();
        self.current_stroke = None;
        self.current_eraser_path.clear();
    }

    pub fn undo(&mut self) {
        self.stop_drawing();

        if let Some(previous_strokes) = self.history.pop() {
            self.redo_history.push(self.strokes.clone());
            self.strokes = previous_strokes;
        }
    }

    pub fn redo(&mut self) {
        self.stop_drawing();

        if let Some(next_strokes) = self.redo_history.pop() {
            self.history.push(self.strokes.clone());
            self.strokes = next_strokes;
        }
    }

    pub fn export_png(&self, path: &Path) -> Result<(), String> {
        let image = self.render_image()?;

        image
            .save(path)
            .map_err(|error| format!("Failed to save PNG: {error}"))
    }

    pub fn render_image(&self) -> Result<RgbaImage, String> {
        let (width, height) = self.canvas_image_size()?;
        let background =
            RgbaImage::from_pixel(width, height, rgba_from_color32(self.background_color()));

        self.render_image_over_background(background)
    }

    pub fn render_image_over_background(&self, background: RgbaImage) -> Result<RgbaImage, String> {
        let Some(canvas_rect) = self.last_canvas_rect else {
            return Err(String::from("The canvas size is not available yet."));
        };

        let expected_size = canvas_image_size_from_rect(canvas_rect);

        if background.dimensions() != expected_size {
            return Err(format!(
                "Background image size does not match the canvas: expected {}x{}, got {}x{}.",
                expected_size.0,
                expected_size.1,
                background.width(),
                background.height()
            ));
        }

        let mut image = background;

        if self.settings.ink_visible {
            for stroke in &self.strokes {
                draw_stroke_on_image(&mut image, stroke, canvas_rect.min);
            }
        }

        Ok(image)
    }

    #[allow(dead_code)]
    pub fn screen_capture_rect(&self, ctx: &egui::Context) -> Option<ScreenCaptureRect> {
        let canvas_rect = self.last_canvas_rect?;
        let viewport_inner_rect = ctx.input(|input| input.viewport().inner_rect)?;

        canvas_rect_to_screen_capture_rect(canvas_rect, viewport_inner_rect, ctx.pixels_per_point())
    }

    pub fn render_image_over_screen_background(
        &self,
        background: RgbaImage,
        ctx: &egui::Context,
    ) -> Result<RgbaImage, String> {
        let Some(canvas_rect) = self.last_canvas_rect else {
            return Err(String::from("The canvas size is not available yet."));
        };
        let Some(viewport_inner_rect) = ctx.input(|input| input.viewport().inner_rect) else {
            return Err(String::from(
                "The canvas screen position is not available yet.",
            ));
        };
        let pixels_per_point = ctx.pixels_per_point();
        let Some(capture_rect) =
            canvas_rect_to_screen_capture_rect(canvas_rect, viewport_inner_rect, pixels_per_point)
        else {
            return Err(String::from(
                "The canvas screen scale is not available yet.",
            ));
        };

        if background.dimensions() != (capture_rect.width, capture_rect.height) {
            return Err(format!(
                "Background image size does not match the screen capture: expected {}x{}, got {}x{}.",
                capture_rect.width,
                capture_rect.height,
                background.width(),
                background.height()
            ));
        }

        let mut image = background;

        if self.settings.ink_visible {
            for stroke in &self.strokes {
                draw_screen_stroke_on_image(
                    &mut image,
                    stroke,
                    viewport_inner_rect.min,
                    capture_rect,
                    pixels_per_point,
                );
            }
        }

        Ok(image)
    }

    fn canvas_image_size(&self) -> Result<(u32, u32), String> {
        let Some(canvas_rect) = self.last_canvas_rect else {
            return Err(String::from("The canvas size is not available yet."));
        };

        Ok(canvas_image_size_from_rect(canvas_rect))
    }

    pub fn ui(&mut self, ui: &mut Ui, drawing_enabled: bool) -> Response {
        self.ui_with_ink_visibility(ui, drawing_enabled, self.settings.ink_visible)
    }

    pub fn ui_without_ink(&mut self, ui: &mut Ui, drawing_enabled: bool) -> Response {
        self.ui_with_ink_visibility(ui, drawing_enabled, false)
    }

    fn ui_with_ink_visibility(
        &mut self,
        ui: &mut Ui,
        drawing_enabled: bool,
        ink_visible: bool,
    ) -> Response {
        let available_size = ui.available_size();
        let sense = if drawing_enabled {
            Sense::drag()
        } else {
            Sense::hover()
        };
        let (response, painter) = ui.allocate_painter(available_size, sense);
        let response = response.on_hover_cursor(CursorIcon::Crosshair);

        let rect = response.rect;
        self.last_canvas_rect = Some(rect);
        painter.rect_filled(rect, 0.0, self.background_color());
        let should_show_transparent_border = self.should_show_transparent_border(&response, rect);

        if should_show_transparent_border {
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0, TRANSPARENT_CANVAS_BORDER),
                egui::StrokeKind::Inside,
            );
        }

        if drawing_enabled {
            self.handle_pointer_input(&response);
        }

        if ink_visible {
            for stroke in &self.strokes {
                draw_stroke(&painter, stroke);
            }

            if let Some(stroke) = &self.current_stroke {
                draw_stroke(&painter, stroke);
            }
        }

        draw_cursor_indicator(
            &painter,
            &response,
            self.current_tool,
            drawing_enabled,
            self.current_width,
            &self.current_eraser_path,
            self.eraser_radius,
        );

        response
    }

    pub fn stop_drawing(&mut self) {
        self.finish_current_stroke();
    }

    fn finish_current_stroke(&mut self) {
        if let Some(stroke) = self.current_stroke.take()
            && stroke.is_meaningful()
        {
            self.push_history_snapshot();
            self.strokes.push(stroke);
        }

        if !self.current_eraser_path.is_empty() {
            let erased_strokes = erase_from_strokes(
                &self.strokes,
                &self.current_eraser_path,
                self.eraser_radius.max(1.0),
            );

            if erased_strokes != self.strokes {
                self.push_history_snapshot();
                self.strokes = erased_strokes;
            }

            self.current_eraser_path.clear();
        }
    }

    fn should_show_transparent_border(&self, response: &Response, rect: egui::Rect) -> bool {
        if self.settings.background != CanvasBackground::Transparent {
            return false;
        }

        match self.settings.transparent_canvas_border_visibility {
            TransparentCanvasBorderVisibility::Always => true,
            TransparentCanvasBorderVisibility::NearEdges => response
                .hover_pos()
                .is_some_and(|pointer_pos| is_near_canvas_edge(rect, pointer_pos)),
        }
    }

    fn handle_pointer_input(&mut self, response: &Response) {
        if response.drag_started()
            && let Some(pos) = response.interact_pointer_pos()
        {
            match self.current_tool {
                Tool::Pen => {
                    let mut stroke = DrawStroke::new(self.current_color, self.current_width);
                    stroke.points.push(pos);
                    self.current_stroke = Some(stroke);
                }
                Tool::Eraser => {
                    self.current_eraser_path.clear();
                    self.current_eraser_path.push(pos);
                }
            }
        }

        if response.dragged()
            && let Some(pos) = response.interact_pointer_pos()
        {
            match self.current_tool {
                Tool::Pen => {
                    if let Some(stroke) = &mut self.current_stroke {
                        push_pen_point_if_needed(&mut stroke.points, pos, stroke.width);
                    }
                }
                Tool::Eraser => {
                    push_point_if_needed(&mut self.current_eraser_path, pos);
                }
            }
        }

        if response.drag_stopped() {
            self.finish_current_stroke();
        }
    }

    fn push_history_snapshot(&mut self) {
        self.history.push(self.strokes.clone());
        self.redo_history.clear();
    }
}

fn draw_stroke(painter: &egui::Painter, stroke: &DrawStroke) {
    let line_stroke = Stroke::new(stroke.width, stroke.color);

    if stroke.points.len() == 2 {
        painter.line_segment([stroke.points[0], stroke.points[1]], line_stroke);
    } else if stroke.points.len() > 2 {
        painter.add(Shape::line(stroke.points.clone(), line_stroke));
    }
}

fn draw_eraser_preview(
    painter: &egui::Painter,
    path: &[egui::Pos2],
    hover_pos: Option<egui::Pos2>,
    radius: f32,
) {
    let preview_color = egui::Color32::from_rgba_unmultiplied(190, 56, 56, 140);

    if path.is_empty() {
        if let Some(pointer_pos) = hover_pos {
            painter.circle_stroke(pointer_pos, radius, Stroke::new(1.0, preview_color));
        }
        return;
    }

    for point in path {
        painter.circle_stroke(*point, radius, Stroke::new(1.0, preview_color));
    }

    for points in path.windows(2) {
        painter.line_segment(
            [points[0], points[1]],
            Stroke::new(radius * 2.0, preview_color),
        );
    }

    if let Some(last_point) = path.last() {
        draw_crosshair(
            painter,
            *last_point,
            radius * 0.45,
            Stroke::new(1.0, preview_color),
        );
    }
}

fn draw_cursor_indicator(
    painter: &egui::Painter,
    response: &Response,
    current_tool: Tool,
    drawing_enabled: bool,
    pen_width: f32,
    eraser_path: &[egui::Pos2],
    eraser_radius: f32,
) {
    if current_tool == Tool::Eraser {
        draw_eraser_preview(painter, eraser_path, response.hover_pos(), eraser_radius);
        return;
    }

    let Some(pointer_pos) = response.hover_pos() else {
        return;
    };

    if !response.rect.contains(pointer_pos) {
        return;
    }

    if drawing_enabled {
        draw_pen_cursor(painter, pointer_pos, pen_width);
    } else {
        draw_disabled_cursor(painter, pointer_pos);
    }
}

fn draw_pen_cursor(painter: &egui::Painter, pointer_pos: egui::Pos2, pen_width: f32) {
    let radius = (pen_width * 0.5).max(PEN_CURSOR_MIN_RADIUS);
    let outer_stroke = Stroke::new(1.5, Color32::WHITE);
    let inner_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(24, 24, 24, 220));

    painter.circle_stroke(pointer_pos, radius + 1.0, outer_stroke);
    painter.circle_stroke(pointer_pos, radius, inner_stroke);
    painter.circle_filled(
        pointer_pos,
        1.2,
        Color32::from_rgba_unmultiplied(24, 24, 24, 220),
    );
}

fn draw_disabled_cursor(painter: &egui::Painter, pointer_pos: egui::Pos2) {
    let stroke = Stroke::new(1.5, Color32::from_rgba_unmultiplied(120, 120, 120, 220));
    draw_crosshair(painter, pointer_pos, DISABLED_CURSOR_SIZE, stroke);
    painter.circle_stroke(
        pointer_pos,
        DISABLED_CURSOR_SIZE + 2.0,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 200)),
    );
}

fn draw_crosshair(painter: &egui::Painter, center: egui::Pos2, radius: f32, stroke: Stroke) {
    painter.line_segment(
        [
            center + egui::vec2(-radius, 0.0),
            center + egui::vec2(radius, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + egui::vec2(0.0, -radius),
            center + egui::vec2(0.0, radius),
        ],
        stroke,
    );
}

fn push_point_if_needed(points: &mut Vec<egui::Pos2>, pos: egui::Pos2) {
    let should_push = match points.last() {
        Some(last) => last.distance(pos) > 0.5,
        None => true,
    };

    if should_push {
        points.push(pos);
    }
}

#[cfg(test)]
mod tests {
    use super::eraser::{
        distance_point_to_segment, erase_from_stroke, point_is_inside_eraser_path,
        sample_segment_points,
    };
    use super::geometry::{canvas_rect_to_screen_capture_rect, is_near_canvas_edge};
    use super::pen::{PEN_POINT_MAX_DISTANCE, pen_point_min_distance, push_pen_point_if_needed};
    use super::raster::screen_point_to_capture_image_point;
    use super::*;
    use image::Rgba;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn stroke(points: &[(f32, f32)]) -> DrawStroke {
        DrawStroke {
            points: points.iter().map(|(x, y)| egui::pos2(*x, *y)).collect(),
            color: Color32::BLACK,
            width: 2.0,
        }
    }

    fn temp_png_path(name: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "aetherink-{name}-{}-{unique_suffix}.png",
            std::process::id()
        ))
    }

    fn assert_pos2_approx_eq(actual: egui::Pos2, expected: egui::Pos2) {
        const TOLERANCE: f32 = 0.000_001;

        assert!(
            (actual.x - expected.x).abs() <= TOLERANCE,
            "x mismatch: actual={}, expected={}",
            actual.x,
            expected.x
        );
        assert!(
            (actual.y - expected.y).abs() <= TOLERANCE,
            "y mismatch: actual={}, expected={}",
            actual.y,
            expected.y
        );
    }

    #[test]
    fn clear_records_undo_snapshot() {
        let original_stroke = stroke(&[(0.0, 0.0), (10.0, 0.0)]);
        let mut canvas = CanvasState {
            strokes: vec![original_stroke.clone()],
            ..Default::default()
        };

        canvas.clear();

        assert!(canvas.strokes.is_empty());
        assert!(canvas.can_undo());

        canvas.undo();

        assert_eq!(canvas.strokes, vec![original_stroke]);
    }

    #[test]
    fn redo_restores_cleared_canvas() {
        let mut canvas = CanvasState {
            strokes: vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])],
            ..Default::default()
        };

        canvas.clear();
        canvas.undo();
        canvas.redo();

        assert!(canvas.strokes.is_empty());
        assert!(canvas.can_undo());
        assert!(!canvas.can_redo());
    }

    #[test]
    fn clear_empty_canvas_does_not_create_undo_history() {
        let mut canvas = CanvasState::default();

        canvas.clear();

        assert!(canvas.strokes.is_empty());
        assert!(!canvas.can_undo());
        assert!(!canvas.can_redo());
    }

    #[test]
    fn clear_after_undo_clears_redo_history() {
        let mut canvas = CanvasState {
            strokes: vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])],
            ..Default::default()
        };

        canvas.clear();
        canvas.undo();
        assert!(canvas.can_redo());

        canvas.clear();

        assert!(!canvas.can_redo());
    }

    #[test]
    fn new_stroke_clears_redo_history() {
        let mut canvas = CanvasState {
            strokes: vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])],
            ..Default::default()
        };

        canvas.clear();
        canvas.undo();
        assert!(canvas.can_redo());

        canvas.current_stroke = Some(stroke(&[(5.0, 5.0), (15.0, 5.0)]));
        canvas.stop_drawing();

        assert!(!canvas.can_redo());
        assert_eq!(canvas.strokes.len(), 2);
    }

    #[test]
    fn transparent_background_opacity_controls_alpha() {
        let mut canvas = CanvasState::default();
        *canvas.background_mut() = CanvasBackground::Transparent;
        *canvas.transparent_background_opacity_mut() = 0.5;

        assert_eq!(canvas.background_color(), Color32::from_white_alpha(127));
    }

    #[test]
    fn transparent_background_opacity_is_clamped() {
        let mut canvas = CanvasState::default();
        *canvas.background_mut() = CanvasBackground::Transparent;

        *canvas.transparent_background_opacity_mut() = -1.0;
        assert_eq!(canvas.background_color(), Color32::from_white_alpha(0));

        *canvas.transparent_background_opacity_mut() = 2.0;
        assert_eq!(canvas.background_color(), Color32::from_white_alpha(255));
    }

    #[test]
    fn settings_capture_current_drawing_defaults() {
        let mut canvas = CanvasState::default();
        *canvas.current_color_mut() = Color32::from_rgb(220, 38, 38);
        *canvas.current_width_mut() = 8.0;
        *canvas.eraser_radius_mut() = 14.0;

        let settings = canvas.settings();

        assert_eq!(
            settings.default_pen_color,
            Color32::from_rgb(220, 38, 38).to_array()
        );
        assert_eq!(settings.default_pen_width, 8.0);
        assert_eq!(settings.eraser_radius, 14.0);
    }

    #[test]
    fn apply_settings_restores_canvas_preferences() {
        let settings = CanvasSettings {
            background: CanvasBackground::Transparent,
            transparent_background_opacity: 0.35,
            transparent_canvas_border_visibility: TransparentCanvasBorderVisibility::Always,
            ink_visible: false,
            default_pen_color: Color32::from_rgb(37, 99, 235).to_array(),
            default_pen_width: 12.0,
            eraser_radius: 20.0,
        };
        let mut canvas = CanvasState::default();

        canvas.apply_settings(settings);

        assert_eq!(canvas.background(), CanvasBackground::Transparent);
        assert_eq!(*canvas.transparent_background_opacity_mut(), 0.35);
        assert_eq!(
            canvas.transparent_canvas_border_visibility(),
            TransparentCanvasBorderVisibility::Always
        );
        assert!(!canvas.ink_visible());
        assert_eq!(*canvas.current_color_mut(), Color32::from_rgb(37, 99, 235));
        assert_eq!(*canvas.current_width_mut(), 12.0);
        assert_eq!(*canvas.eraser_radius_mut(), 20.0);
    }

    #[test]
    fn ink_visibility_is_saved_in_settings() {
        let mut canvas = CanvasState::default();

        canvas.set_ink_visible(false);

        assert!(!canvas.settings().ink_visible);
    }

    #[test]
    fn hidden_ink_is_omitted_from_rendered_image() {
        let mut canvas = CanvasState {
            strokes: vec![stroke(&[(1.0, 1.0), (3.0, 1.0)])],
            last_canvas_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(4.0, 4.0),
            )),
            ..Default::default()
        };
        canvas.set_ink_visible(false);

        let image = canvas.render_image().expect("image render should succeed");

        assert_eq!(image.get_pixel(2, 1).0, [248, 246, 240, 255]);
    }

    #[test]
    fn render_image_over_background_preserves_background_and_draws_visible_ink() {
        let canvas = CanvasState {
            strokes: vec![stroke(&[(1.0, 1.0), (3.0, 1.0)])],
            last_canvas_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(4.0, 4.0),
            )),
            ..Default::default()
        };
        let background = RgbaImage::from_pixel(4, 4, Rgba([10, 20, 30, 255]));

        let image = canvas
            .render_image_over_background(background)
            .expect("background composition should succeed");

        assert_eq!(image.get_pixel(3, 3).0, [10, 20, 30, 255]);
        assert_eq!(image.get_pixel(2, 1).0, [0, 0, 0, 255]);
    }

    #[test]
    fn render_image_over_background_respects_hidden_ink() {
        let mut canvas = CanvasState {
            strokes: vec![stroke(&[(1.0, 1.0), (3.0, 1.0)])],
            last_canvas_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(4.0, 4.0),
            )),
            ..Default::default()
        };
        canvas.set_ink_visible(false);
        let background = RgbaImage::from_pixel(4, 4, Rgba([10, 20, 30, 255]));

        let image = canvas
            .render_image_over_background(background)
            .expect("background composition should succeed");

        assert_eq!(image.get_pixel(2, 1).0, [10, 20, 30, 255]);
    }

    #[test]
    fn render_image_over_background_requires_matching_size() {
        let canvas = CanvasState {
            last_canvas_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(4.0, 4.0),
            )),
            ..Default::default()
        };
        let background = RgbaImage::from_pixel(3, 4, Rgba([10, 20, 30, 255]));

        let error = canvas
            .render_image_over_background(background)
            .expect_err("mismatched background size should fail");

        assert_eq!(
            error,
            "Background image size does not match the canvas: expected 4x4, got 3x4."
        );
    }

    #[test]
    fn screen_point_to_capture_image_point_accounts_for_scale_and_outward_rounding() {
        let point = screen_point_to_capture_image_point(
            egui::pos2(10.25, 20.25),
            egui::pos2(-4.5, 5.25),
            egui::pos2(8.0, 38.0),
            1.5,
        );

        assert_pos2_approx_eq(point, egui::pos2(0.625, 0.25));
    }

    #[test]
    fn screen_point_to_capture_image_point_maps_canvas_end_into_capture_space() {
        let point = screen_point_to_capture_image_point(
            egui::pos2(30.5, 40.5),
            egui::pos2(-4.5, 5.25),
            egui::pos2(8.0, 38.0),
            1.5,
        );

        assert_pos2_approx_eq(point, egui::pos2(31.0, 30.625));
    }

    #[test]
    fn canvas_screen_capture_rect_maps_canvas_points_to_screen_pixels() {
        let canvas_rect =
            egui::Rect::from_min_max(egui::pos2(10.0, 24.0), egui::pos2(310.0, 224.0));
        let viewport_inner_rect =
            egui::Rect::from_min_size(egui::pos2(100.0, 50.0), egui::vec2(800.0, 600.0));

        let screen_rect = canvas_rect_to_screen_capture_rect(canvas_rect, viewport_inner_rect, 2.0)
            .expect("valid viewport and scale should map to a screen rect");

        assert_eq!(
            screen_rect,
            ScreenCaptureRect {
                x: 220,
                y: 148,
                width: 600,
                height: 400,
            }
        );
    }

    #[test]
    fn canvas_screen_capture_rect_rounds_outward() {
        let canvas_rect =
            egui::Rect::from_min_max(egui::pos2(10.25, 20.25), egui::pos2(30.5, 40.5));
        let viewport_inner_rect =
            egui::Rect::from_min_size(egui::pos2(-4.5, 5.25), egui::vec2(800.0, 600.0));

        let screen_rect = canvas_rect_to_screen_capture_rect(canvas_rect, viewport_inner_rect, 1.5)
            .expect("fractional coordinates should map to a screen rect");

        assert_eq!(
            screen_rect,
            ScreenCaptureRect {
                x: 8,
                y: 38,
                width: 31,
                height: 31,
            }
        );
    }

    #[test]
    fn canvas_screen_capture_rect_rejects_invalid_scale() {
        let canvas_rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
        let viewport_inner_rect =
            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0));

        assert!(
            canvas_rect_to_screen_capture_rect(canvas_rect, viewport_inner_rect, 0.0).is_none()
        );
    }

    #[test]
    fn switching_tool_finishes_current_stroke() {
        let mut canvas = CanvasState {
            current_stroke: Some(stroke(&[(0.0, 0.0), (10.0, 0.0)])),
            ..Default::default()
        };

        canvas.set_current_tool(Tool::Eraser);

        assert_eq!(canvas.current_tool(), Tool::Eraser);
        assert_eq!(canvas.strokes, vec![stroke(&[(0.0, 0.0), (10.0, 0.0)])]);
        assert!(canvas.current_stroke.is_none());
        assert!(canvas.can_undo());
    }

    #[test]
    fn pen_point_filter_replaces_close_collinear_point() {
        let mut points = vec![egui::pos2(0.0, 0.0), egui::pos2(2.0, 0.0)];

        push_pen_point_if_needed(&mut points, egui::pos2(2.5, 0.0), 2.0);

        assert_eq!(points, vec![egui::pos2(0.0, 0.0), egui::pos2(2.5, 0.0)]);
    }

    #[test]
    fn pen_point_filter_keeps_turning_points() {
        let mut points = vec![egui::pos2(0.0, 0.0), egui::pos2(2.0, 0.0)];

        push_pen_point_if_needed(&mut points, egui::pos2(2.0, 2.0), 2.0);

        assert_eq!(
            points,
            vec![
                egui::pos2(0.0, 0.0),
                egui::pos2(2.0, 0.0),
                egui::pos2(2.0, 2.0)
            ]
        );
    }

    #[test]
    fn pen_point_filter_adds_distant_collinear_points() {
        let mut points = vec![egui::pos2(0.0, 0.0), egui::pos2(2.0, 0.0)];

        push_pen_point_if_needed(&mut points, egui::pos2(5.0, 0.0), 2.0);

        assert_eq!(
            points,
            vec![
                egui::pos2(0.0, 0.0),
                egui::pos2(2.0, 0.0),
                egui::pos2(5.0, 0.0)
            ]
        );
    }

    #[test]
    fn pen_point_min_distance_grows_with_width_until_cap() {
        assert_eq!(pen_point_min_distance(1.0), 1.35);
        assert_eq!(pen_point_min_distance(4.0), 2.4);
        assert_eq!(pen_point_min_distance(20.0), PEN_POINT_MAX_DISTANCE);
    }

    #[test]
    fn sample_segment_points_includes_endpoints_and_even_spacing() {
        let points = sample_segment_points(egui::pos2(0.0, 0.0), egui::pos2(5.0, 0.0), 2.0);

        assert_eq!(points.len(), 4);
        assert_pos2_approx_eq(points[0], egui::pos2(0.0, 0.0));
        assert_pos2_approx_eq(points[1], egui::pos2(5.0 / 3.0, 0.0));
        assert_pos2_approx_eq(points[2], egui::pos2(10.0 / 3.0, 0.0));
        assert_pos2_approx_eq(points[3], egui::pos2(5.0, 0.0));
    }

    #[test]
    fn eraser_splits_stroke_around_erased_segment() {
        let source = stroke(&[(0.0, 0.0), (10.0, 0.0)]);
        let eraser_path = [egui::pos2(5.0, 0.0)];

        let remaining = erase_from_stroke(&source, &eraser_path, 1.0);

        assert_eq!(remaining.len(), 2);
        assert_eq!(
            remaining[0].points,
            vec![egui::pos2(0.0, 0.0), egui::pos2(2.0, 0.0)]
        );
        assert_eq!(
            remaining[1].points,
            vec![egui::pos2(8.0, 0.0), egui::pos2(10.0, 0.0)]
        );
    }

    #[test]
    fn eraser_keeps_stroke_when_path_misses() {
        let source = stroke(&[(0.0, 0.0), (10.0, 0.0)]);
        let eraser_path = [egui::pos2(5.0, 10.0)];

        let remaining = erase_from_stroke(&source, &eraser_path, 1.0);

        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].color, source.color);
        assert_eq!(remaining[0].width, source.width);
        assert_eq!(remaining[0].points.first(), source.points.first());
        assert_eq!(remaining[0].points.last(), source.points.last());
    }

    #[test]
    fn eraser_removes_fully_covered_stroke() {
        let source = stroke(&[(0.0, 0.0), (10.0, 0.0)]);
        let eraser_path = [egui::pos2(5.0, 0.0)];

        let remaining = erase_from_stroke(&source, &eraser_path, 20.0);

        assert!(remaining.is_empty());
    }

    #[test]
    fn eraser_drops_non_meaningful_strokes() {
        let source = stroke(&[(0.0, 0.0)]);
        let eraser_path = [egui::pos2(0.0, 0.0)];

        let remaining = erase_from_stroke(&source, &eraser_path, 1.0);

        assert!(remaining.is_empty());
    }

    #[test]
    fn point_inside_eraser_path_checks_single_point_radius() {
        let eraser_path = [egui::pos2(5.0, 5.0)];

        assert!(point_is_inside_eraser_path(
            egui::pos2(6.0, 5.0),
            &eraser_path,
            1.0
        ));
        assert!(!point_is_inside_eraser_path(
            egui::pos2(6.1, 5.0),
            &eraser_path,
            1.0
        ));
    }

    #[test]
    fn point_inside_eraser_path_checks_segment_distance() {
        let eraser_path = [egui::pos2(0.0, 0.0), egui::pos2(10.0, 0.0)];

        assert!(point_is_inside_eraser_path(
            egui::pos2(5.0, 1.0),
            &eraser_path,
            1.0
        ));
        assert!(!point_is_inside_eraser_path(
            egui::pos2(5.0, 1.1),
            &eraser_path,
            1.0
        ));
    }

    #[test]
    fn distance_point_to_segment_clamps_projection_to_segment() {
        let start = egui::pos2(0.0, 0.0);
        let end = egui::pos2(10.0, 0.0);

        assert_eq!(
            distance_point_to_segment(egui::pos2(5.0, 3.0), start, end),
            3.0
        );
        assert_eq!(
            distance_point_to_segment(egui::pos2(-3.0, 4.0), start, end),
            5.0
        );
        assert_eq!(
            distance_point_to_segment(egui::pos2(13.0, 4.0), start, end),
            5.0
        );
    }

    #[test]
    fn export_png_writes_canvas_background_and_strokes() {
        let path = temp_png_path("stroke-export");
        let canvas = CanvasState {
            strokes: vec![stroke(&[(1.0, 1.0), (3.0, 1.0)])],
            last_canvas_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(4.0, 4.0),
            )),
            ..Default::default()
        };

        canvas.export_png(&path).expect("PNG export should succeed");
        let image = image::ImageReader::open(&path)
            .expect("exported PNG should open")
            .decode()
            .expect("exported PNG should decode")
            .to_rgba8();
        let _ = std::fs::remove_file(&path);

        assert_eq!(image.dimensions(), (4, 4));
        assert_eq!(image.get_pixel(3, 3).0, [248, 246, 240, 255]);
        assert_eq!(image.get_pixel(2, 1).0, [0, 0, 0, 255]);
    }

    #[test]
    fn export_png_preserves_transparent_background() {
        let path = temp_png_path("transparent-export");
        let mut canvas = CanvasState {
            last_canvas_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(2.0, 2.0),
            )),
            ..Default::default()
        };
        *canvas.background_mut() = CanvasBackground::Transparent;
        *canvas.transparent_background_opacity_mut() = 0.0;

        canvas.export_png(&path).expect("PNG export should succeed");
        let image = image::ImageReader::open(&path)
            .expect("exported PNG should open")
            .decode()
            .expect("exported PNG should decode")
            .to_rgba8();
        let _ = std::fs::remove_file(&path);

        assert_eq!(image.dimensions(), (2, 2));
        assert_eq!(image.get_pixel(0, 0).0, [0, 0, 0, 0]);
    }

    #[test]
    fn export_png_requires_canvas_rect() {
        let path = temp_png_path("missing-rect-export");
        let canvas = CanvasState::default();

        let error = canvas
            .export_png(&path)
            .expect_err("PNG export should fail before the canvas is laid out");

        assert_eq!(error, "The canvas size is not available yet.");
    }

    #[test]
    fn near_edges_border_check_ignores_top_edge() {
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(100.0, 80.0));

        assert!(is_near_canvas_edge(rect, egui::pos2(12.0, 50.0)));
        assert!(is_near_canvas_edge(rect, egui::pos2(108.0, 50.0)));
        assert!(is_near_canvas_edge(rect, egui::pos2(50.0, 88.0)));
        assert!(!is_near_canvas_edge(rect, egui::pos2(50.0, 12.0)));
    }
}
