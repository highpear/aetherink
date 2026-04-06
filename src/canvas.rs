use egui::{Color32, Response, Sense, Stroke, Ui};
use serde::{Deserialize, Serialize};

use crate::stroke::{DrawStroke, Tool};

const DEFAULT_WHITE_BACKGROUND: Color32 = Color32::from_rgb(248, 246, 240);
const TRANSPARENT_CANVAS_BORDER: Color32 = Color32::from_gray(180);
const CANVAS_BORDER_HOVER_THRESHOLD: f32 = 24.0;
const DEFAULT_ERASER_RADIUS: f32 = 8.0;

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
            self.strokes = previous_strokes;
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, drawing_enabled: bool) -> Response {
        let available_size = ui.available_size();
        let sense = if drawing_enabled {
            Sense::drag()
        } else {
            Sense::hover()
        };
        let (response, painter) = ui.allocate_painter(available_size, sense);

        let rect = response.rect;
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

        if self.current_tool == Tool::Eraser {
            draw_eraser_preview(&painter, &self.current_eraser_path, response.hover_pos(), self.eraser_radius);
        }

        response
    }

    pub fn stop_drawing(&mut self) {
        self.finish_current_stroke();
    }

    fn finish_current_stroke(&mut self) {
        if let Some(stroke) = self.current_stroke.take() {
            if stroke.is_meaningful() {
                self.push_history_snapshot();
                self.strokes.push(stroke);
            }
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
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
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
        }

        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                match self.current_tool {
                    Tool::Pen => {
                        if let Some(stroke) = &mut self.current_stroke {
                            push_point_if_needed(&mut stroke.points, pos);
                        }
                    }
                    Tool::Eraser => {
                        push_point_if_needed(&mut self.current_eraser_path, pos);
                    }
                }
            }
        }

        if response.drag_stopped() {
            self.finish_current_stroke();
        }
    }

    fn push_history_snapshot(&mut self) {
        self.history.push(self.strokes.clone());
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
    let mut current_points = Vec::new();
    let effective_radius = eraser_radius + stroke.width * 0.5;

    for segment in stroke.points.windows(2) {
        let start = segment[0];
        let end = segment[1];
        let is_erased = segment_intersects_eraser_path(start, end, eraser_path, effective_radius);

        if is_erased {
            if current_points.len() >= 2 {
                remaining_strokes.push(DrawStroke {
                    points: current_points,
                    color: stroke.color,
                    width: stroke.width,
                });
            }

            current_points = Vec::new();
            continue;
        }

        if current_points.last().copied() != Some(start) {
            current_points.push(start);
        }
        current_points.push(end);
    }

    if current_points.len() >= 2 {
        remaining_strokes.push(DrawStroke {
            points: current_points,
            color: stroke.color,
            width: stroke.width,
        });
    }

    remaining_strokes
}

fn segment_intersects_eraser_path(
    segment_start: egui::Pos2,
    segment_end: egui::Pos2,
    eraser_path: &[egui::Pos2],
    radius: f32,
) -> bool {
    if eraser_path.is_empty() {
        return false;
    }

    if eraser_path.len() == 1 {
        return distance_point_to_segment(eraser_path[0], segment_start, segment_end) <= radius;
    }

    eraser_path.windows(2).any(|eraser_segment| {
        segment_distance(
            segment_start,
            segment_end,
            eraser_segment[0],
            eraser_segment[1],
        ) <= radius
    })
}

fn segment_distance(a1: egui::Pos2, a2: egui::Pos2, b1: egui::Pos2, b2: egui::Pos2) -> f32 {
    if segments_intersect(a1, a2, b1, b2) {
        return 0.0;
    }

    distance_point_to_segment(a1, b1, b2)
        .min(distance_point_to_segment(a2, b1, b2))
        .min(distance_point_to_segment(b1, a1, a2))
        .min(distance_point_to_segment(b2, a1, a2))
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

fn segments_intersect(a1: egui::Pos2, a2: egui::Pos2, b1: egui::Pos2, b2: egui::Pos2) -> bool {
    let a = a2 - a1;
    let b = b2 - b1;
    let start_offset = b1 - a1;
    let denominator = cross(a, b);

    if denominator.abs() <= f32::EPSILON {
        return false;
    }

    let t = cross(start_offset, b) / denominator;
    let u = cross(start_offset, a) / denominator;

    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

fn cross(a: egui::Vec2, b: egui::Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn is_near_canvas_edge(rect: egui::Rect, pointer_pos: egui::Pos2) -> bool {
    let distance_to_left = (pointer_pos.x - rect.left()).abs();
    let distance_to_right = (rect.right() - pointer_pos.x).abs();
    let distance_to_bottom = (rect.bottom() - pointer_pos.y).abs();

    distance_to_left <= CANVAS_BORDER_HOVER_THRESHOLD
        || distance_to_right <= CANVAS_BORDER_HOVER_THRESHOLD
        || distance_to_bottom <= CANVAS_BORDER_HOVER_THRESHOLD
}
