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
    let mut any_point_erased = false;

    for segment in stroke.points.windows(2) {
        let sampled_points = sample_segment_points(segment[0], segment[1], ERASER_SAMPLING_STEP);

        for point in sampled_points {
            let is_erased = point_is_inside_eraser_path(point, eraser_path, effective_radius);

            if is_erased {
                any_point_erased = true;
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

    // An untouched stroke must keep its original points; the sampled fragments
    // rebuilt above use resampled geometry, which would silently rewrite the
    // stroke and pollute the undo history on eraser drags that hit nothing.
    if !any_point_erased {
        return vec![stroke.clone()];
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

#[cfg(test)]
mod tests {
    use egui::Color32;

    use super::*;

    fn stroke(points: &[(f32, f32)]) -> DrawStroke {
        DrawStroke {
            points: points.iter().map(|(x, y)| egui::pos2(*x, *y)).collect(),
            color: Color32::BLACK,
            width: 2.0,
        }
    }

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
    fn sample_segment_points_includes_endpoints_and_even_spacing() {
        let points = sample_segment_points(egui::pos2(0.0, 0.0), egui::pos2(5.0, 0.0), 2.0);

        assert_eq!(points.len(), 4);
        assert_pos2_approx_eq(points[0], egui::pos2(0.0, 0.0));
        assert_pos2_approx_eq(points[1], egui::pos2(5.0 / 3.0, 0.0));
        assert_pos2_approx_eq(points[2], egui::pos2(10.0 / 3.0, 0.0));
        assert_pos2_approx_eq(points[3], egui::pos2(5.0, 0.0));
    }

    #[test]
    fn eraser_splits_stroke_around_erased_segment() {
        let source = stroke(&[(0.0, 0.0), (10.0, 0.0)]);
        let eraser_path = [egui::pos2(5.0, 0.0)];

        let remaining = erase_from_stroke(&source, &eraser_path, 1.0);

        assert_eq!(remaining.len(), 2);
        assert_eq!(
            remaining[0].points,
            vec![egui::pos2(0.0, 0.0), egui::pos2(2.0, 0.0)]
        );
        assert_eq!(
            remaining[1].points,
            vec![egui::pos2(8.0, 0.0), egui::pos2(10.0, 0.0)]
        );
    }

    #[test]
    fn eraser_keeps_stroke_unchanged_when_path_misses() {
        let source = stroke(&[(0.0, 0.0), (10.0, 0.0)]);
        let eraser_path = [egui::pos2(5.0, 10.0)];

        let remaining = erase_from_stroke(&source, &eraser_path, 1.0);

        assert_eq!(remaining, vec![source]);
    }

    #[test]
    fn eraser_removes_fully_covered_stroke() {
        let source = stroke(&[(0.0, 0.0), (10.0, 0.0)]);
        let eraser_path = [egui::pos2(5.0, 0.0)];

        let remaining = erase_from_stroke(&source, &eraser_path, 20.0);

        assert!(remaining.is_empty());
    }

    #[test]
    fn eraser_drops_non_meaningful_strokes() {
        let source = stroke(&[(0.0, 0.0)]);
        let eraser_path = [egui::pos2(0.0, 0.0)];

        let remaining = erase_from_stroke(&source, &eraser_path, 1.0);

        assert!(remaining.is_empty());
    }

    #[test]
    fn point_inside_eraser_path_checks_single_point_radius() {
        let eraser_path = [egui::pos2(5.0, 5.0)];

        assert!(point_is_inside_eraser_path(
            egui::pos2(6.0, 5.0),
            &eraser_path,
            1.0
        ));
        assert!(!point_is_inside_eraser_path(
            egui::pos2(6.1, 5.0),
            &eraser_path,
            1.0
        ));
    }

    #[test]
    fn point_inside_eraser_path_checks_segment_distance() {
        let eraser_path = [egui::pos2(0.0, 0.0), egui::pos2(10.0, 0.0)];

        assert!(point_is_inside_eraser_path(
            egui::pos2(5.0, 1.0),
            &eraser_path,
            1.0
        ));
        assert!(!point_is_inside_eraser_path(
            egui::pos2(5.0, 1.1),
            &eraser_path,
            1.0
        ));
    }

    #[test]
    fn distance_point_to_segment_clamps_projection_to_segment() {
        let start = egui::pos2(0.0, 0.0);
        let end = egui::pos2(10.0, 0.0);

        assert_eq!(
            distance_point_to_segment(egui::pos2(5.0, 3.0), start, end),
            3.0
        );
        assert_eq!(
            distance_point_to_segment(egui::pos2(-3.0, 4.0), start, end),
            5.0
        );
        assert_eq!(
            distance_point_to_segment(egui::pos2(13.0, 4.0), start, end),
            5.0
        );
    }
}
