const CG_EVENT_SOURCE_STATE_COMBINED_SESSION_STATE: i32 = 0;
const MACOS_KEY_CODE_O: u16 = 0x1F;
const MACOS_KEY_CODE_LEFT_SHIFT: u16 = 0x38;
const MACOS_KEY_CODE_RIGHT_SHIFT: u16 = 0x3C;
const MACOS_KEY_CODE_LEFT_CONTROL: u16 = 0x3B;
const MACOS_KEY_CODE_RIGHT_CONTROL: u16 = 0x3E;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventSourceKeyState(state_id: i32, key: u16) -> bool;
}

#[derive(Debug, Default)]
pub struct ClickThroughController {
    overlay_toggle_shortcut_was_pressed: bool,
}

impl ClickThroughController {
    pub fn supports_pointer_passthrough(&self) -> bool {
        true
    }

    pub fn supports_shortcut_monitoring(&self) -> bool {
        true
    }

    pub fn poll_overlay_toggle_shortcut(&mut self) -> bool {
        let is_pressed = is_control_pressed()
            && is_shift_pressed()
            && is_key_pressed(MACOS_KEY_CODE_O);
        let was_pressed = self.overlay_toggle_shortcut_was_pressed;

        self.overlay_toggle_shortcut_was_pressed = is_pressed;

        is_pressed && !was_pressed
    }

    pub fn temporary_drawing_shortcut_label(&self) -> &'static str {
        "Shift"
    }

    pub fn is_temporary_drawing_active(&self) -> bool {
        is_shift_pressed()
    }
}

fn is_control_pressed() -> bool {
    is_key_pressed(MACOS_KEY_CODE_LEFT_CONTROL) || is_key_pressed(MACOS_KEY_CODE_RIGHT_CONTROL)
}

fn is_shift_pressed() -> bool {
    is_key_pressed(MACOS_KEY_CODE_LEFT_SHIFT) || is_key_pressed(MACOS_KEY_CODE_RIGHT_SHIFT)
}

fn is_key_pressed(key_code: u16) -> bool {
    unsafe { CGEventSourceKeyState(CG_EVENT_SOURCE_STATE_COMBINED_SESSION_STATE, key_code) }
}
