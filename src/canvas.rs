use egui::{Color32, Response, Sense, Stroke, Ui};

use crate::stroke::DrawStroke;

const DEFAULT_WHITE_BACKGROUND: Color32 = Color32::from_rgb(248, 246, 240);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct CanvasState {
    pub strokes: Vec<DrawStroke>,
    pub current_stroke: Option<DrawStroke>,
    pub current_color: Color32,
    pub current_width: f32,
    pub background: CanvasBackground,
    pub transparent_background_opacity: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            current_stroke: None,
            current_color: Color32::BLACK,
            current_width: 2.0,
            background: CanvasBackground::White,
            transparent_background_opacity: 0.0,
        }
    }
}

impl CanvasState {
    pub fn background_color(&self) -> Color32 {
        match self.background {
            CanvasBackground::White => DEFAULT_WHITE_BACKGROUND,
            CanvasBackground::Transparent => {
                let alpha = (self.transparent_background_opacity.clamp(0.0, 1.0) * 255.0) as u8;
                Color32::from_white_alpha(alpha)
            }
        }
    }

    pub fn clear(&mut self) {
        self.strokes.clear();
        self.current_stroke = None;
    }

    pub fn undo(&mut self) {
        self.strokes.pop();
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, Sense::drag());

        let rect = response.rect;
        painter.rect_filled(rect, 0.0, self.background_color());

        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut stroke = DrawStroke::new(self.current_color, self.current_width);
                stroke.points.push(pos);
                self.current_stroke = Some(stroke);
            }
        }

        if response.dragged() {
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

        if response.drag_stopped() {
            if let Some(stroke) = self.current_stroke.take() {
                if stroke.is_meaningful() {
                    self.strokes.push(stroke);
                }
            }
        }

        for stroke in &self.strokes {
            draw_stroke(&painter, stroke);
        }

        if let Some(stroke) = &self.current_stroke {
            draw_stroke(&painter, stroke);
        }

        response
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
