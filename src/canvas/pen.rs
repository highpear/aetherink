const PEN_POINT_MIN_DISTANCE: f32 = 1.0;
const PEN_POINT_DISTANCE_PER_WIDTH: f32 = 0.35;
pub(super) const PEN_POINT_MAX_DISTANCE: f32 = 4.0;
const PEN_DIRECTION_ALIGNMENT_THRESHOLD: f32 = 0.96;

pub(super) fn push_pen_point_if_needed(points: &mut Vec<egui::Pos2>, pos: egui::Pos2, width: f32) {
    let min_distance = pen_point_min_distance(width);

    match points.len() {
        0 => {
            points.push(pos);
        }
        1 => {
            if points[0].distance(pos) >= min_distance {
                points.push(pos);
            }
        }
        _ => {
            let previous = points[points.len() - 2];
            let last = points[points.len() - 1];

            if should_replace_last_pen_point(previous, last, pos, min_distance) {
                if let Some(last_point) = points.last_mut() {
                    *last_point = pos;
                }
                return;
            }

            if last.distance(pos) >= min_distance {
                points.push(pos);
            }
        }
    }
}

pub(super) fn pen_point_min_distance(width: f32) -> f32 {
    (PEN_POINT_MIN_DISTANCE + width * PEN_POINT_DISTANCE_PER_WIDTH).min(PEN_POINT_MAX_DISTANCE)
}

fn should_replace_last_pen_point(
    previous: egui::Pos2,
    last: egui::Pos2,
    pos: egui::Pos2,
    min_distance: f32,
) -> bool {
    let incoming = last - previous;
    let outgoing = pos - last;

    if incoming.length_sq() <= f32::EPSILON || outgoing.length_sq() <= f32::EPSILON {
        return false;
    }

    if outgoing.length() > min_distance * 1.5 {
        return false;
    }

    incoming.normalized().dot(outgoing.normalized()) >= PEN_DIRECTION_ALIGNMENT_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pen_point_filter_replaces_close_collinear_point() {
        let mut points = vec![egui::pos2(0.0, 0.0), egui::pos2(2.0, 0.0)];

        push_pen_point_if_needed(&mut points, egui::pos2(2.5, 0.0), 2.0);

        assert_eq!(points, vec![egui::pos2(0.0, 0.0), egui::pos2(2.5, 0.0)]);
    }

    #[test]
    fn pen_point_filter_keeps_turning_points() {
        let mut points = vec![egui::pos2(0.0, 0.0), egui::pos2(2.0, 0.0)];

        push_pen_point_if_needed(&mut points, egui::pos2(2.0, 2.0), 2.0);

        assert_eq!(
            points,
            vec![
                egui::pos2(0.0, 0.0),
                egui::pos2(2.0, 0.0),
                egui::pos2(2.0, 2.0)
            ]
        );
    }

    #[test]
    fn pen_point_filter_adds_distant_collinear_points() {
        let mut points = vec![egui::pos2(0.0, 0.0), egui::pos2(2.0, 0.0)];

        push_pen_point_if_needed(&mut points, egui::pos2(5.0, 0.0), 2.0);

        assert_eq!(
            points,
            vec![
                egui::pos2(0.0, 0.0),
                egui::pos2(2.0, 0.0),
                egui::pos2(5.0, 0.0)
            ]
        );
    }

    #[test]
    fn pen_point_min_distance_grows_with_width_until_cap() {
        assert_eq!(pen_point_min_distance(1.0), 1.35);
        assert_eq!(pen_point_min_distance(4.0), 2.4);
        assert_eq!(pen_point_min_distance(20.0), PEN_POINT_MAX_DISTANCE);
    }
}
