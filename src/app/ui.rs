use eframe::egui;

const BASIC_PEN_COLORS: [(&str, egui::Color32); 5] = [
    ("Black", egui::Color32::BLACK),
    ("White", egui::Color32::WHITE),
    ("Red", egui::Color32::from_rgb(220, 38, 38)),
    ("Yellow", egui::Color32::from_rgb(234, 179, 8)),
    ("Blue", egui::Color32::from_rgb(37, 99, 235)),
];

pub(crate) fn drawing_mode_label(drawing_enabled: bool) -> &'static str {
    if drawing_enabled {
        "Draw: On"
    } else {
        "Draw: Off"
    }
}

pub(crate) fn keyboard_shortcut_pressed(
    ctx: &egui::Context,
    key: egui::Key,
    require_shift: bool,
) -> bool {
    ctx.input_mut(|input| {
        let modifiers = input.modifiers;
        let command_pressed = modifiers.command || modifiers.ctrl;
        let shift_matches = modifiers.shift == require_shift;

        if command_pressed && shift_matches && input.key_pressed(key) {
            input.consume_key(modifiers, key)
        } else {
            false
        }
    })
}

pub(crate) fn show_basic_pen_colors(ui: &mut egui::Ui, current_color: &mut egui::Color32) {
    ui.horizontal(|ui| {
        for (label, color) in BASIC_PEN_COLORS {
            let is_selected = *current_color == color;
            let stroke_color = if is_selected {
                egui::Color32::from_rgb(30, 30, 30)
            } else {
                egui::Color32::from_gray(120)
            };

            let response = ui
                .add(
                    egui::Button::new("")
                        .min_size(egui::vec2(18.0, 18.0))
                        .fill(color)
                        .stroke(egui::Stroke::new(1.0, stroke_color))
                        .corner_radius(9.0),
                )
                .on_hover_text(label);

            if response.clicked() {
                *current_color = color;
            }
        }
    });
}

pub(crate) fn undo_button() -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new("Undo")
            .strong()
            .color(egui::Color32::from_rgb(34, 44, 66)),
    )
    .fill(egui::Color32::from_rgb(227, 236, 248))
    .stroke(egui::Stroke::new(
        1.0,
        egui::Color32::from_rgb(127, 146, 179),
    ))
    .corner_radius(6.0)
    .min_size(egui::vec2(62.0, 28.0))
}

pub(crate) fn clear_button() -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new("Clear")
            .strong()
            .color(egui::Color32::from_rgb(122, 32, 32)),
    )
    .fill(egui::Color32::from_rgb(252, 231, 231))
    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 38, 38)))
    .corner_radius(6.0)
    .min_size(egui::vec2(68.0, 28.0))
}
