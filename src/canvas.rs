use egui::{Color32, Response, Sense, Stroke, Ui};

use crate::stroke::DrawStroke;

#[derive(Debug, Default)]
pub struct CanvasState {
    pub strokes: Vec<DrawStroke>,
    pub current_stroke: Option<DrawStroke>,
}

impl CanvasState {
    pub fn clear(&mut self) {
        self.strokes.clear();
        self.current_stroke = None;
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, Sense::drag());

        let rect = response.rect;
        painter.rect_filled(rect, 0.0, Color32::WHITE);

        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut stroke = DrawStroke::new(Color32::BLACK, 2.0);
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