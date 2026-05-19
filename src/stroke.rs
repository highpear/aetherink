use egui::{Color32, Pos2};

#[derive(Debug, Clone, PartialEq)]
pub struct DrawStroke {
    pub points: Vec<Pos2>,
    pub color: Color32,
    pub width: f32,
}

impl DrawStroke {
    pub fn new(color: Color32, width: f32) -> Self {
        Self {
            points: Vec::new(),
            color,
            width,
        }
    }

    pub fn is_meaningful(&self) -> bool {
        self.points.len() >= 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pen,
    Eraser,
}

impl Tool {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pen => "Pen",
            Self::Eraser => "Eraser",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroke_is_meaningful_only_after_two_points() {
        let mut stroke = DrawStroke::new(Color32::BLACK, 2.0);

        assert!(!stroke.is_meaningful());

        stroke.points.push(Pos2::new(0.0, 0.0));
        assert!(!stroke.is_meaningful());

        stroke.points.push(Pos2::new(1.0, 1.0));
        assert!(stroke.is_meaningful());
    }

    #[test]
    fn tool_labels_match_ui_text() {
        assert_eq!(Tool::Pen.label(), "Pen");
        assert_eq!(Tool::Eraser.label(), "Eraser");
    }
}
