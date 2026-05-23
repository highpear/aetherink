use egui::Color32;

use crate::stroke::DrawStroke;

const ERASER_SAMPLING_STEP: f32 = 2.0;

pub(super) fn erase_from_strokes(
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

pub(super) fn erase_from_stroke(
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

pub(super) fn sample_segment_points(
    start: egui::Pos2,
    end: egui::Pos2,
    step: f32,
) -> Vec<egui::Pos2> {
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

pub(super) fn point_is_inside_eraser_path(
    point: egui::Pos2,
    eraser_path: &[egui::Pos2],
    radius: f32,
) -> bool {
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

pub(super) fn distance_point_to_segment(
    point: egui::Pos2,
    start: egui::Pos2,
    end: egui::Pos2,
) -> f32 {
    let segment = end - start;
    let length_sq = segment.length_sq();

    if length_sq <= f32::EPSILON {
        return point.distance(start);
    }

    let projection = ((point - start).dot(segment) / length_sq).clamp(0.0, 1.0);
    let nearest = start + segment * projection;
    point.distance(nearest)
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
