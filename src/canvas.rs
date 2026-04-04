use egui::{Color32, Response, Sense, Stroke, Ui};
use serde::{Deserialize, Serialize};

use crate::stroke::DrawStroke;

const DEFAULT_WHITE_BACKGROUND: Color32 = Color32::from_rgb(248, 246, 240);
const TRANSPARENT_CANVAS_BORDER: Color32 = Color32::from_gray(180);
const CANVAS_BORDER_HOVER_THRESHOLD: f32 = 24.0;

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
}

impl Default for CanvasSettings {
    fn default() -> Self {
        Self {
            background: CanvasBackground::White,
            transparent_background_opacity: 0.0,
            transparent_canvas_border_visibility: TransparentCanvasBorderVisibility::NearEdges,
        }
    }
}

#[derive(Debug)]
pub struct CanvasState {
    strokes: Vec<DrawStroke>,
    current_stroke: Option<DrawStroke>,
    current_color: Color32,
    current_width: f32,
    settings: CanvasSettings,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            current_stroke: None,
            current_color: Color32::BLACK,
            current_width: 2.0,
            settings: CanvasSettings::default(),
        }
    }
}

impl CanvasState {
    pub fn current_color_mut(&mut self) -> &mut Color32 {
        &mut self.current_color
    }

    pub fn current_width_mut(&mut self) -> &mut f32 {
        &mut self.current_width
    }

    pub fn apply_settings(&mut self, settings: CanvasSettings) {
        self.settings = settings;
    }

    pub fn settings(&self) -> CanvasSettings {
        self.settings.clone()
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
        self.strokes.clear();
        self.current_stroke = None;
    }

    pub fn undo(&mut self) {
        self.strokes.pop();
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

        if drawing_enabled && response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut stroke = DrawStroke::new(self.current_color, self.current_width);
                stroke.points.push(pos);
                self.current_stroke = Some(stroke);
            }
        }

        if drawing_enabled && response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                if let Some(stroke) = &mut self.current_stroke {
                    let should_push = match stroke.points.last() {
                        Some(last) => last.distance(pos) > 0.5,
                        None => true,
                    };

                    if should_push {
                        stroke.points.push(pos);
                    }
                }
            }
        }

        if drawing_enabled && response.drag_stopped() {
            self.finish_current_stroke();
        }

        for stroke in &self.strokes {
            draw_stroke(&painter, stroke);
        }

        if let Some(stroke) = &self.current_stroke {
            draw_stroke(&painter, stroke);
        }

        response
    }

    pub fn stop_drawing(&mut self) {
        self.finish_current_stroke();
    }

    fn finish_current_stroke(&mut self) {
        if let Some(stroke) = self.current_stroke.take() {
            if stroke.is_meaningful() {
                self.strokes.push(stroke);
            }
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
}

fn draw_stroke(painter: &egui::Painter, stroke: &DrawStroke) {
    for points in stroke.points.windows(2) {
        painter.line_segment(
            [points[0], points[1]],
            Stroke::new(stroke.width, stroke.color),
        );
    }
}

fn is_near_canvas_edge(rect: egui::Rect, pointer_pos: egui::Pos2) -> bool {
    let distance_to_left = (pointer_pos.x - rect.left()).abs();
    let distance_to_right = (rect.right() - pointer_pos.x).abs();
    let distance_to_bottom = (rect.bottom() - pointer_pos.y).abs();

    distance_to_left <= CANVAS_BORDER_HOVER_THRESHOLD
        || distance_to_right <= CANVAS_BORDER_HOVER_THRESHOLD
        || distance_to_bottom <= CANVAS_BORDER_HOVER_THRESHOLD
}
