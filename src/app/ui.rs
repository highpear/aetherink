use eframe::egui;

const PEN_PRESET_COLORS: [(&str, egui::Color32); 6] = [
    ("Black", egui::Color32::BLACK),
    ("Red", egui::Color32::from_rgb(220, 38, 38)),
    ("Blue", egui::Color32::from_rgb(37, 99, 235)),
    ("Green", egui::Color32::from_rgb(22, 163, 74)),
    ("Yellow", egui::Color32::from_rgb(234, 179, 8)),
    ("White", egui::Color32::WHITE),
];
const PEN_WIDTH_PRESETS: [f32; 5] = [1.0, 2.0, 4.0, 8.0, 12.0];

pub(crate) fn drawing_mode_label(drawing_enabled: bool) -> &'static str {
    if drawing_enabled {
        "Draw: On"
    } else {
        "Draw: Off"
    }
}

pub(crate) fn ink_visibility_label(ink_visible: bool) -> &'static str {
    if ink_visible { "Ink: On" } else { "Ink: Off" }
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

pub(crate) fn show_pen_color_presets(ui: &mut egui::Ui, current_color: &mut egui::Color32) {
    ui.horizontal(|ui| {
        for (label, color) in PEN_PRESET_COLORS {
            let is_selected = *current_color == color;
            let stroke_color = if is_selected {
                egui::Color32::from_rgb(30, 30, 30)
            } else {
                egui::Color32::from_gray(120)
            };
            let check_color = if color == egui::Color32::WHITE || color == egui::Color32::YELLOW {
                egui::Color32::BLACK
            } else {
                egui::Color32::WHITE
            };

            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
            let response = response.on_hover_text(label);
            let painter = ui.painter();

            painter.rect_filled(rect, 9.0, color);
            painter.rect_stroke(
                rect,
                9.0,
                egui::Stroke::new(1.0, stroke_color),
                egui::StrokeKind::Outside,
            );

            if is_selected {
                let check_stroke = egui::Stroke::new(2.0, check_color);
                let first = egui::pos2(rect.left() + 4.5, rect.center().y);
                let middle = egui::pos2(rect.left() + 7.5, rect.bottom() - 5.0);
                let last = egui::pos2(rect.right() - 4.0, rect.top() + 5.0);

                painter.line_segment([first, middle], check_stroke);
                painter.line_segment([middle, last], check_stroke);
            }

            if response.clicked() {
                *current_color = color;
            }
        }
    });
}

pub(crate) fn top_bar_group_label(ui: &mut egui::Ui, label: &str) {
    ui.label(egui::RichText::new(label).strong());
}

pub(crate) fn show_pen_width_presets(ui: &mut egui::Ui, current_width: &mut f32) {
    ui.horizontal(|ui| {
        for width in PEN_WIDTH_PRESETS {
            let is_selected = (*current_width - width).abs() < f32::EPSILON;

            if ui
                .selectable_label(is_selected, width_preset_label(width))
                .on_hover_text(format!("Set pen width to {}", width))
                .clicked()
            {
                *current_width = width;
            }
        }
    });
}

fn width_preset_label(width: f32) -> &'static str {
    match width as i32 {
        1 => "XS",
        2 => "S",
        4 => "M",
        8 => "L",
        12 => "XL",
        _ => "?",
    }
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

pub(crate) fn redo_button() -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new("Redo")
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

pub(crate) fn save_menu_button() -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new("Save")
            .strong()
            .color(egui::Color32::from_rgb(32, 78, 54)),
    )
    .fill(egui::Color32::from_rgb(231, 248, 237))
    .stroke(egui::Stroke::new(
        1.0,
        egui::Color32::from_rgb(86, 162, 118),
    ))
    .corner_radius(6.0)
    .min_size(egui::vec2(72.0, 28.0))
}

pub(crate) fn copy_image_button() -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new("Copy")
            .strong()
            .color(egui::Color32::from_rgb(32, 78, 54)),
    )
    .fill(egui::Color32::from_rgb(231, 248, 237))
    .stroke(egui::Stroke::new(
        1.0,
        egui::Color32::from_rgb(86, 162, 118),
    ))
    .corner_radius(6.0)
    .min_size(egui::vec2(72.0, 28.0))
}
