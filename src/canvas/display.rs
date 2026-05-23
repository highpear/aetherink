use egui::{Shape, Stroke};

use crate::stroke::DrawStroke;

pub(super) fn draw_stroke(painter: &egui::Painter, stroke: &DrawStroke) {
    let line_stroke = Stroke::new(stroke.width, stroke.color);

    if stroke.points.len() == 2 {
        painter.line_segment([stroke.points[0], stroke.points[1]], line_stroke);
    } else if stroke.points.len() > 2 {
        painter.add(Shape::line(stroke.points.clone(), line_stroke));
    }
}
