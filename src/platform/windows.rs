use std::mem;
use std::ptr;

use image::RgbaImage;
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CAPTUREBLT, CreateCompatibleBitmap,
    CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, RGBQUAD,
    ReleaseDC, SRCCOPY, SelectObject,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_SHIFT};

use crate::canvas::ScreenCaptureRect;

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

#[derive(Debug, Default)]
pub struct BackgroundCaptureController;

impl BackgroundCaptureController {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self
    }

    pub fn supports_background_capture(&self) -> bool {
        true
    }

    pub fn capture_background(&self, rect: ScreenCaptureRect) -> Result<RgbaImage, String> {
        capture_screen_rect(rect)
    }
}

fn capture_screen_rect(rect: ScreenCaptureRect) -> Result<RgbaImage, String> {
    let width = i32::try_from(rect.width)
        .map_err(|_| String::from("Background capture width is too large."))?;
    let height = i32::try_from(rect.height)
        .map_err(|_| String::from("Background capture height is too large."))?;
    let byte_len = rect
        .width
        .checked_mul(rect.height)
        .and_then(|pixel_count| pixel_count.checked_mul(4))
        .ok_or_else(|| String::from("Background capture buffer is too large."))?;
    let mut pixels = vec![0; byte_len as usize];

    let screen_dc = unsafe { GetDC(ptr::null_mut()) };
    if screen_dc.is_null() {
        return Err(String::from("Failed to get the screen device context."));
    }

    let result = capture_screen_rect_with_dc(screen_dc, rect, width, height, &mut pixels);

    unsafe {
        ReleaseDC(ptr::null_mut(), screen_dc);
    }

    result?;

    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    RgbaImage::from_raw(rect.width, rect.height, pixels)
        .ok_or_else(|| String::from("Failed to create the captured background image."))
}

fn capture_screen_rect_with_dc(
    screen_dc: windows_sys::Win32::Graphics::Gdi::HDC,
    rect: ScreenCaptureRect,
    width: i32,
    height: i32,
    pixels: &mut [u8],
) -> Result<(), String> {
    let memory_dc = unsafe { CreateCompatibleDC(screen_dc) };
    if memory_dc.is_null() {
        return Err(String::from(
            "Failed to create the background capture device context.",
        ));
    }

    let bitmap = unsafe { CreateCompatibleBitmap(screen_dc, width, height) };
    if bitmap.is_null() {
        unsafe {
            DeleteDC(memory_dc);
        }
        return Err(String::from(
            "Failed to create the background capture bitmap.",
        ));
    }

    let previous_object = unsafe { SelectObject(memory_dc, bitmap) };
    if previous_object.is_null() {
        unsafe {
            DeleteObject(bitmap);
            DeleteDC(memory_dc);
        }
        return Err(String::from(
            "Failed to select the background capture bitmap.",
        ));
    }

    let bitblt_ok = unsafe {
        BitBlt(
            memory_dc,
            0,
            0,
            width,
            height,
            screen_dc,
            rect.x,
            rect.y,
            SRCCOPY | CAPTUREBLT,
        )
    } != 0;

    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: pixels.len() as u32,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0,
        }],
    };

    let get_bits_ok = bitblt_ok
        && unsafe {
            GetDIBits(
                memory_dc,
                bitmap,
                0,
                height as u32,
                pixels.as_mut_ptr().cast(),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            )
        } != 0;

    unsafe {
        SelectObject(memory_dc, previous_object);
        DeleteObject(bitmap);
        DeleteDC(memory_dc);
    }

    if !bitblt_ok {
        return Err(String::from("Failed to copy the background pixels."));
    }

    if !get_bits_ok {
        return Err(String::from("Failed to read the background pixels."));
    }

    Ok(())
}
