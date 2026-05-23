use egui::Color32;
use image::{Rgba, RgbaImage};

use crate::canvas::ScreenCaptureRect;
use crate::stroke::DrawStroke;

pub(super) fn draw_stroke_on_image(image: &mut RgbaImage, stroke: &DrawStroke, origin: egui::Pos2) {
    for points in stroke.points.windows(2) {
        let start = points[0] - origin.to_vec2();
        let end = points[1] - origin.to_vec2();
        draw_segment_on_image(image, start, end, stroke.width, stroke.color);
    }
}

pub(super) fn draw_screen_stroke_on_image(
    image: &mut RgbaImage,
    stroke: &DrawStroke,
    viewport_inner_origin: egui::Pos2,
    capture_rect: ScreenCaptureRect,
    pixels_per_point: f32,
) {
    let capture_origin = egui::pos2(capture_rect.x as f32, capture_rect.y as f32);

    for points in stroke.points.windows(2) {
        let start = screen_point_to_capture_image_point(
            points[0],
            viewport_inner_origin,
            capture_origin,
            pixels_per_point,
        );
        let end = screen_point_to_capture_image_point(
            points[1],
            viewport_inner_origin,
            capture_origin,
            pixels_per_point,
        );

        draw_segment_on_image(
            image,
            start,
            end,
            stroke.width * pixels_per_point,
            stroke.color,
        );
    }
}

pub(super) fn screen_point_to_capture_image_point(
    point: egui::Pos2,
    viewport_inner_origin: egui::Pos2,
    capture_origin: egui::Pos2,
    pixels_per_point: f32,
) -> egui::Pos2 {
    let screen_point = viewport_inner_origin + point.to_vec2();

    egui::pos2(
        screen_point.x * pixels_per_point - capture_origin.x,
        screen_point.y * pixels_per_point - capture_origin.y,
    )
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

pub(super) fn rgba_from_color32(color: Color32) -> Rgba<u8> {
    let [red, green, blue, alpha] = color.to_array();
    Rgba([red, green, blue, alpha])
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
