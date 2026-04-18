use std::path::Path;

use egui::{Color32, CursorIcon, Response, Sense, Stroke, Ui};
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::stroke::{DrawStroke, Tool};

const DEFAULT_WHITE_BACKGROUND: Color32 = Color32::from_rgb(248, 246, 240);
const TRANSPARENT_CANVAS_BORDER: Color32 = Color32::from_gray(180);
const CANVAS_BORDER_HOVER_THRESHOLD: f32 = 24.0;
const DEFAULT_ERASER_RADIUS: f32 = 8.0;
const ERASER_SAMPLING_STEP: f32 = 2.0;
const PEN_CURSOR_MIN_RADIUS: f32 = 2.0;
const DISABLED_CURSOR_SIZE: f32 = 7.0;
const PEN_POINT_MIN_DISTANCE: f32 = 1.0;
const PEN_POINT_DISTANCE_PER_WIDTH: f32 = 0.35;
const PEN_POINT_MAX_DISTANCE: f32 = 4.0;
const PEN_DIRECTION_ALIGNMENT_THRESHOLD: f32 = 0.96;

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
            default_pen_color: egui::Color32::BLACK.to_array(),
            default_pen_width: 2.0,
            eraser_radius: DEFAULT_ERASER_RADIUS,
        }
    }
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
        let Some(canvas_rect) = self.last_canvas_rect else {
            return Err(String::from("The canvas size is not available yet."));
        };

        let width = canvas_rect.width().round().max(1.0) as u32;
        let height = canvas_rect.height().round().max(1.0) as u32;
        let mut image =
            RgbaImage::from_pixel(width, height, rgba_from_color32(self.background_color()));

        for stroke in &self.strokes {
            draw_stroke_on_image(&mut image, stroke, canvas_rect.min);
        }

        image
            .save(path)
            .map_err(|error| format!("Failed to save PNG: {error}"))
    }

    pub fn ui(&mut self, ui: &mut Ui, drawing_enabled: bool) -> Response {
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

        for stroke in &self.strokes {
            draw_stroke(&painter, stroke);
        }

        if let Some(stroke) = &self.current_stroke {
            draw_stroke(&painter, stroke);
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
    for points in stroke.points.windows(2) {
        painter.line_segment(
            [points[0], points[1]],
            Stroke::new(stroke.width, stroke.color),
        );
    }
}

fn draw_stroke_on_image(image: &mut RgbaImage, stroke: &DrawStroke, origin: egui::Pos2) {
    for points in stroke.points.windows(2) {
        let start = points[0] - origin.to_vec2();
        let end = points[1] - origin.to_vec2();
        draw_segment_on_image(image, start, end, stroke.width, stroke.color);
    }
}

fn draw_segment_on_image(
    image: &mut RgbaImage,
    start: egui::Pos2,
    end: egui::Pos2,
    width: f32,
    color: Color32,
) {
    let radius = (width * 0.5).max(0.5);
    let distance = start.distance(end);
    let step_distance = radius.max(0.75);
    let steps = (distance / step_distance).ceil().max(1.0) as usize;

    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let point = start.lerp(end, t);
        draw_filled_circle_on_image(image, point, radius, color);
    }
}

fn draw_filled_circle_on_image(
    image: &mut RgbaImage,
    center: egui::Pos2,
    radius: f32,
    color: Color32,
) {
    let min_x = (center.x - radius).floor().max(0.0) as i32;
    let max_x = (center.x + radius)
        .ceil()
        .min(image.width().saturating_sub(1) as f32) as i32;
    let min_y = (center.y - radius).floor().max(0.0) as i32;
    let max_y = (center.y + radius)
        .ceil()
        .min(image.height().saturating_sub(1) as f32) as i32;
    let radius_squared = radius * radius;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - center.x;
            let dy = y as f32 + 0.5 - center.y;

            if dx * dx + dy * dy <= radius_squared {
                blend_pixel(image.get_pixel_mut(x as u32, y as u32), color);
            }
        }
    }
}

fn blend_pixel(pixel: &mut Rgba<u8>, color: Color32) {
    let source = rgba_from_color32(color);
    let source_alpha = source[3] as f32 / 255.0;

    if source_alpha <= f32::EPSILON {
        return;
    }

    let destination = *pixel;
    let destination_alpha = destination[3] as f32 / 255.0;
    let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);

    if output_alpha <= f32::EPSILON {
        *pixel = Rgba([0, 0, 0, 0]);
        return;
    }

    let blend_channel = |source_channel: u8, destination_channel: u8| -> u8 {
        let source_value = source_channel as f32 / 255.0;
        let destination_value = destination_channel as f32 / 255.0;
        let output_value = (source_value * source_alpha
            + destination_value * destination_alpha * (1.0 - source_alpha))
            / output_alpha;

        (output_value * 255.0).round().clamp(0.0, 255.0) as u8
    };

    *pixel = Rgba([
        blend_channel(source[0], destination[0]),
        blend_channel(source[1], destination[1]),
        blend_channel(source[2], destination[2]),
        (output_alpha * 255.0).round().clamp(0.0, 255.0) as u8,
    ]);
}

fn rgba_from_color32(color: Color32) -> Rgba<u8> {
    let [red, green, blue, alpha] = color.to_array();
    Rgba([red, green, blue, alpha])
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

fn push_pen_point_if_needed(points: &mut Vec<egui::Pos2>, pos: egui::Pos2, width: f32) {
    let min_distance = pen_point_min_distance(width);

    match points.len() {
        0 => {
            points.push(pos);
        }
        1 => {
            if points[0].distance(pos) >= min_distance {
                points.push(pos);
            }
        }
        _ => {
            let previous = points[points.len() - 2];
            let last = points[points.len() - 1];

            if should_replace_last_pen_point(previous, last, pos, min_distance) {
                if let Some(last_point) = points.last_mut() {
                    *last_point = pos;
                }
                return;
            }

            if last.distance(pos) >= min_distance {
                points.push(pos);
            }
        }
    }
}

fn pen_point_min_distance(width: f32) -> f32 {
    (PEN_POINT_MIN_DISTANCE + width * PEN_POINT_DISTANCE_PER_WIDTH).min(PEN_POINT_MAX_DISTANCE)
}

fn should_replace_last_pen_point(
    previous: egui::Pos2,
    last: egui::Pos2,
    pos: egui::Pos2,
    min_distance: f32,
) -> bool {
    let incoming = last - previous;
    let outgoing = pos - last;

    if incoming.length_sq() <= f32::EPSILON || outgoing.length_sq() <= f32::EPSILON {
        return false;
    }

    if outgoing.length() > min_distance * 1.5 {
        return false;
    }

    incoming.normalized().dot(outgoing.normalized()) >= PEN_DIRECTION_ALIGNMENT_THRESHOLD
}

fn erase_from_strokes(
    strokes: &[DrawStroke],
    eraser_path: &[egui::Pos2],
    eraser_radius: f32,
) -> Vec<DrawStroke> {
    let mut erased_strokes = Vec::new();

    for stroke in strokes {
        erased_strokes.extend(erase_from_stroke(stroke, eraser_path, eraser_radius));
    }

    erased_strokes
}

fn erase_from_stroke(
    stroke: &DrawStroke,
    eraser_path: &[egui::Pos2],
    eraser_radius: f32,
) -> Vec<DrawStroke> {
    if stroke.points.len() < 2 {
        return Vec::new();
    }

    let mut remaining_strokes = Vec::new();
    let effective_radius = eraser_radius + stroke.width * 0.5;
    let mut current_points = Vec::new();

    for segment in stroke.points.windows(2) {
        let sampled_points = sample_segment_points(segment[0], segment[1], ERASER_SAMPLING_STEP);

        for point in sampled_points {
            let is_erased = point_is_inside_eraser_path(point, eraser_path, effective_radius);

            if is_erased {
                finalize_stroke_fragment(
                    &mut remaining_strokes,
                    &mut current_points,
                    stroke.color,
                    stroke.width,
                );
            } else {
                push_point_if_needed(&mut current_points, point);
            }
        }
    }

    finalize_stroke_fragment(
        &mut remaining_strokes,
        &mut current_points,
        stroke.color,
        stroke.width,
    );

    remaining_strokes
}

fn sample_segment_points(start: egui::Pos2, end: egui::Pos2, step: f32) -> Vec<egui::Pos2> {
    let distance = start.distance(end);

    if distance <= step {
        return vec![start, end];
    }

    let segment = end - start;
    let sample_count = (distance / step).ceil() as usize;
    let mut points = Vec::with_capacity(sample_count + 1);

    for index in 0..=sample_count {
        let t = index as f32 / sample_count as f32;
        points.push(start + segment * t);
    }

    points
}

fn finalize_stroke_fragment(
    remaining_strokes: &mut Vec<DrawStroke>,
    current_points: &mut Vec<egui::Pos2>,
    color: Color32,
    width: f32,
) {
    if current_points.len() >= 2 {
        remaining_strokes.push(DrawStroke {
            points: std::mem::take(current_points),
            color,
            width,
        });
    } else {
        current_points.clear();
    }
}

fn point_is_inside_eraser_path(point: egui::Pos2, eraser_path: &[egui::Pos2], radius: f32) -> bool {
    if eraser_path.is_empty() {
        return false;
    }

    if eraser_path.len() == 1 {
        return point.distance(eraser_path[0]) <= radius;
    }

    eraser_path.windows(2).any(|eraser_segment| {
        distance_point_to_segment(point, eraser_segment[0], eraser_segment[1]) <= radius
    })
}

fn distance_point_to_segment(point: egui::Pos2, start: egui::Pos2, end: egui::Pos2) -> f32 {
    let segment = end - start;
    let length_sq = segment.length_sq();

    if length_sq <= f32::EPSILON {
        return point.distance(start);
    }

    let projection = ((point - start).dot(segment) / length_sq).clamp(0.0, 1.0);
    let nearest = start + segment * projection;
    point.distance(nearest)
}

fn is_near_canvas_edge(rect: egui::Rect, pointer_pos: egui::Pos2) -> bool {
    let distance_to_left = (pointer_pos.x - rect.left()).abs();
    let distance_to_right = (rect.right() - pointer_pos.x).abs();
    let distance_to_bottom = (rect.bottom() - pointer_pos.y).abs();

    distance_to_left <= CANVAS_BORDER_HOVER_THRESHOLD
        || distance_to_right <= CANVAS_BORDER_HOVER_THRESHOLD
        || distance_to_bottom <= CANVAS_BORDER_HOVER_THRESHOLD
}
