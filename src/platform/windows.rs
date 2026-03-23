use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_SHIFT,
};

const RESTORE_SHORTCUT_KEY: i32 = 'X' as i32;

#[derive(Debug, Default)]
pub struct ClickThroughController {
    restore_shortcut_was_pressed: bool,
}

impl ClickThroughController {
    pub fn is_supported(&self) -> bool {
        true
    }

    pub fn restore_shortcut_label(&self) -> &'static str {
        "Ctrl+Shift+X"
    }

    pub fn poll_restore_shortcut(&mut self) -> bool {
        let is_pressed = is_virtual_key_pressed(VK_CONTROL.into())
            && is_virtual_key_pressed(VK_SHIFT.into())
            && is_virtual_key_pressed(RESTORE_SHORTCUT_KEY);
        let was_pressed = self.restore_shortcut_was_pressed;

        self.restore_shortcut_was_pressed = is_pressed;

        is_pressed && !was_pressed
    }
}

fn is_virtual_key_pressed(virtual_key: i32) -> bool {
    unsafe { (GetAsyncKeyState(virtual_key) as u16 & 0x8000) != 0 }
}
