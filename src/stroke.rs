use egui::{Color32, Pos2};

#[derive(Debug, Clone)]
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