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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn near_edges_border_check_ignores_top_edge() {
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(100.0, 80.0));

        assert!(is_near_canvas_edge(rect, egui::pos2(12.0, 50.0)));
        assert!(is_near_canvas_edge(rect, egui::pos2(108.0, 50.0)));
        assert!(is_near_canvas_edge(rect, egui::pos2(50.0, 88.0)));
        assert!(!is_near_canvas_edge(rect, egui::pos2(50.0, 12.0)));
    }
}
