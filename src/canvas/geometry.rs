use crate::canvas::ScreenCaptureRect;

const CANVAS_BORDER_HOVER_THRESHOLD: f32 = 24.0;

pub(super) fn canvas_image_size_from_rect(canvas_rect: egui::Rect) -> (u32, u32) {
    (
        canvas_rect.width().round().max(1.0) as u32,
        canvas_rect.height().round().max(1.0) as u32,
    )
}

pub(super) fn canvas_rect_to_screen_capture_rect(
    canvas_rect: egui::Rect,
    viewport_inner_rect: egui::Rect,
    pixels_per_point: f32,
) -> Option<ScreenCaptureRect> {
    if !pixels_per_point.is_finite() || pixels_per_point <= 0.0 {
        return None;
    }

    let screen_min = viewport_inner_rect.min + canvas_rect.min.to_vec2();
    let screen_max = viewport_inner_rect.min + canvas_rect.max.to_vec2();
    let min_x = (screen_min.x * pixels_per_point).floor();
    let min_y = (screen_min.y * pixels_per_point).floor();
    let max_x = (screen_max.x * pixels_per_point).ceil();
    let max_y = (screen_max.y * pixels_per_point).ceil();

    if ![min_x, min_y, max_x, max_y]
        .iter()
        .all(|coordinate| coordinate.is_finite())
    {
        return None;
    }

    Some(ScreenCaptureRect {
        x: min_x as i32,
        y: min_y as i32,
        width: (max_x - min_x).max(1.0) as u32,
        height: (max_y - min_y).max(1.0) as u32,
    })
}

pub(super) fn is_near_canvas_edge(rect: egui::Rect, pointer_pos: egui::Pos2) -> bool {
    let distance_to_left = (pointer_pos.x - rect.left()).abs();
    let distance_to_right = (rect.right() - pointer_pos.x).abs();
    let distance_to_bottom = (rect.bottom() - pointer_pos.y).abs();

    distance_to_left <= CANVAS_BORDER_HOVER_THRESHOLD
        || distance_to_right <= CANVAS_BORDER_HOVER_THRESHOLD
        || distance_to_bottom <= CANVAS_BORDER_HOVER_THRESHOLD
}
