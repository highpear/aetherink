use egui::{Color32, Response, Stroke};

use crate::stroke::Tool;

const PEN_CURSOR_MIN_RADIUS: f32 = 2.0;
const DISABLED_CURSOR_SIZE: f32 = 7.0;

pub(super) fn draw_cursor_indicator(
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
