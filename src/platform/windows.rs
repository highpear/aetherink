use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_SHIFT};

const OVERLAY_TOGGLE_SHORTCUT_KEY: i32 = 'O' as i32;
const TEMPORARY_DRAWING_KEY: i32 = VK_SHIFT as i32;

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
        let is_pressed = is_virtual_key_pressed(VK_CONTROL.into())
            && is_virtual_key_pressed(VK_SHIFT.into())
            && is_virtual_key_pressed(OVERLAY_TOGGLE_SHORTCUT_KEY);
        let was_pressed = self.overlay_toggle_shortcut_was_pressed;

        self.overlay_toggle_shortcut_was_pressed = is_pressed;

        is_pressed && !was_pressed
    }

    pub fn temporary_drawing_shortcut_label(&self) -> &'static str {
        "Shift"
    }

    pub fn is_temporary_drawing_active(&self) -> bool {
        is_virtual_key_pressed(TEMPORARY_DRAWING_KEY)
    }
}

fn is_virtual_key_pressed(virtual_key: i32) -> bool {
    unsafe { (GetAsyncKeyState(virtual_key) as u16 & 0x8000) != 0 }
}
