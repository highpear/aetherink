use std::ffi::{CString, c_char, c_void};
use std::path::PathBuf;

use image::RgbaImage;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use crate::canvas::ScreenCaptureRect;
use crate::platform::BackgroundCaptureAvailability;

type CFAllocatorRef = *const c_void;
type CFStringRef = *const c_void;
type CFURLRef = *const c_void;
type CGImageDestinationRef = *const c_void;
type CGImageRef = *const c_void;
type CGWindowID = u32;
type Id = *mut c_void;
type Sel = *const c_void;

const CG_EVENT_SOURCE_STATE_COMBINED_SESSION_STATE: i32 = 0;
const CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
const CF_URL_POSIX_PATH_STYLE: i32 = 0;
const CG_WINDOW_IMAGE_BOUNDS_IGNORE_FRAMING: u32 = 1;
const CG_WINDOW_LIST_OPTION_ON_SCREEN_BELOW_WINDOW: u32 = 1 << 2;
const MACOS_KEY_CODE_O: u16 = 0x1F;
const MACOS_KEY_CODE_LEFT_SHIFT: u16 = 0x38;
const MACOS_KEY_CODE_RIGHT_SHIFT: u16 = 0x3C;
const MACOS_KEY_CODE_LEFT_CONTROL: u16 = 0x3B;
const MACOS_KEY_CODE_RIGHT_CONTROL: u16 = 0x3E;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventSourceKeyState(state_id: i32, key: u16) -> bool;
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
    fn CGWindowListCreateImage(
        screen_bounds: CGRect,
        list_option: u32,
        window_id: CGWindowID,
        image_option: u32,
    ) -> CGImageRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(cf: *const c_void);
    fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        c_str: *const c_char,
        encoding: u32,
    ) -> CFStringRef;
    fn CFURLCreateWithFileSystemPath(
        allocator: CFAllocatorRef,
        file_path: CFStringRef,
        path_style: i32,
        is_directory: bool,
    ) -> CFURLRef;
}

#[link(name = "ImageIO", kind = "framework")]
unsafe extern "C" {
    fn CGImageDestinationCreateWithURL(
        url: CFURLRef,
        image_type: CFStringRef,
        count: usize,
        options: *const c_void,
    ) -> CGImageDestinationRef;
    fn CGImageDestinationAddImage(
        destination: CGImageDestinationRef,
        image: CGImageRef,
        properties: *const c_void,
    );
    fn CGImageDestinationFinalize(destination: CGImageDestinationRef) -> bool;
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

#[link(name = "objc", kind = "dylib")]
unsafe extern "C" {
    fn objc_msgSend();
    fn sel_registerName(name: *const c_char) -> Sel;
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
        let is_pressed =
            is_control_pressed() && is_shift_pressed() && is_key_pressed(MACOS_KEY_CODE_O);
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

#[derive(Debug, Default)]
pub struct BackgroundCaptureController {
    window_id: Option<CGWindowID>,
}

impl BackgroundCaptureController {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            window_id: window_id_from_creation_context(cc).ok(),
        }
    }

    pub fn supports_background_capture(&self) -> bool {
        self.window_id.is_some()
    }

    pub fn background_capture_availability(&self) -> BackgroundCaptureAvailability {
        if self.window_id.is_none() {
            BackgroundCaptureAvailability::Unsupported
        } else if has_screen_capture_access() {
            BackgroundCaptureAvailability::Available
        } else {
            BackgroundCaptureAvailability::PermissionRequired
        }
    }

    pub fn request_background_capture_permission(&self) -> Result<bool, String> {
        if self.window_id.is_none() {
            return Err(String::from(
                "The AetherInk window id is not available for background capture.",
            ));
        }

        if has_screen_capture_access() {
            return Ok(true);
        }

        Ok(unsafe { CGRequestScreenCaptureAccess() })
    }

    pub fn capture_background(&self, rect: ScreenCaptureRect) -> Result<RgbaImage, String> {
        let Some(window_id) = self.window_id else {
            return Err(String::from(
                "The AetherInk window id is not available for background capture.",
            ));
        };

        capture_screen_rect_below_window(rect, window_id)
    }
}

fn capture_screen_rect_below_window(
    rect: ScreenCaptureRect,
    window_id: CGWindowID,
) -> Result<RgbaImage, String> {
    if rect.width == 0 || rect.height == 0 {
        return Err(String::from("The background capture area is empty."));
    }

    ensure_screen_capture_access()?;

    let cg_image = create_image_below_window(rect, window_id)?;
    let path = temporary_capture_path();

    if let Err(error) = write_cg_image_to_png(cg_image, &path) {
        unsafe {
            CFRelease(cg_image.cast());
        }
        return Err(error);
    }

    let image = image::ImageReader::open(&path)
        .map_err(|error| format!("Failed to open captured background: {error}"))
        .and_then(|reader| {
            reader
                .decode()
                .map_err(|error| format!("Failed to decode captured background: {error}"))
        })
        .map(|image| image.to_rgba8());

    unsafe {
        CFRelease(cg_image.cast());
    }

    let _ = std::fs::remove_file(&path);

    image
}

fn ensure_screen_capture_access() -> Result<(), String> {
    if has_screen_capture_access() {
        return Ok(());
    }

    Err(String::from(
        "macOS Screen Recording permission is required to capture windows behind AetherInk.",
    ))
}

fn has_screen_capture_access() -> bool {
    unsafe { CGPreflightScreenCaptureAccess() }
}

fn window_id_from_creation_context(cc: &eframe::CreationContext<'_>) -> Result<CGWindowID, String> {
    let window_handle = cc
        .window_handle()
        .map_err(|error| format!("Failed to read the native window handle: {error}"))?;

    let RawWindowHandle::AppKit(handle) = window_handle.as_raw() else {
        return Err(String::from(
            "The native window handle is not an AppKit window.",
        ));
    };

    let view = handle.ns_view.as_ptr();
    let window = objc_call_id(view, objc_selector("window")?);
    if window.is_null() {
        return Err(String::from("Failed to find the AetherInk macOS window."));
    }

    let window_number = objc_call_isize(window, objc_selector("windowNumber")?);

    if window_number <= 0 {
        return Err(String::from("The AetherInk macOS window id is invalid."));
    }

    Ok(window_number as CGWindowID)
}

fn create_image_below_window(
    rect: ScreenCaptureRect,
    window_id: CGWindowID,
) -> Result<CGImageRef, String> {
    let bounds = CGRect {
        origin: CGPoint {
            x: rect.x.into(),
            y: rect.y.into(),
        },
        size: CGSize {
            width: rect.width.into(),
            height: rect.height.into(),
        },
    };

    let image = unsafe {
        CGWindowListCreateImage(
            bounds,
            CG_WINDOW_LIST_OPTION_ON_SCREEN_BELOW_WINDOW,
            window_id,
            CG_WINDOW_IMAGE_BOUNDS_IGNORE_FRAMING,
        )
    };

    if image.is_null() {
        Err(String::from(
            "macOS returned no background image. Check Screen Recording permission.",
        ))
    } else {
        Ok(image)
    }
}

fn write_cg_image_to_png(image: CGImageRef, path: &PathBuf) -> Result<(), String> {
    let path_string = cf_string_from_str(&path.display().to_string())?;
    let image_type = cf_string_from_str("public.png")?;
    let url = unsafe {
        CFURLCreateWithFileSystemPath(
            std::ptr::null(),
            path_string,
            CF_URL_POSIX_PATH_STYLE,
            false,
        )
    };

    if url.is_null() {
        unsafe {
            CFRelease(path_string.cast());
            CFRelease(image_type.cast());
        }
        return Err(String::from(
            "Failed to create a file URL for background capture.",
        ));
    }

    let destination =
        unsafe { CGImageDestinationCreateWithURL(url, image_type, 1, std::ptr::null()) };

    if destination.is_null() {
        unsafe {
            CFRelease(url.cast());
            CFRelease(path_string.cast());
            CFRelease(image_type.cast());
        }
        return Err(String::from(
            "Failed to create a background PNG destination.",
        ));
    }

    unsafe {
        CGImageDestinationAddImage(destination, image, std::ptr::null());
    }
    let finalized = unsafe { CGImageDestinationFinalize(destination) };

    unsafe {
        CFRelease(destination.cast());
        CFRelease(url.cast());
        CFRelease(path_string.cast());
        CFRelease(image_type.cast());
    }

    if finalized {
        Ok(())
    } else {
        Err(String::from("Failed to write the captured background PNG."))
    }
}

fn temporary_capture_path() -> PathBuf {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S%.3f");
    std::env::temp_dir().join(format!("aetherink-background-capture-{timestamp}.png"))
}

fn cf_string_from_str(value: &str) -> Result<CFStringRef, String> {
    let c_string = CString::new(value).map_err(|_| String::from("String contains a null byte."))?;
    let string = unsafe {
        CFStringCreateWithCString(std::ptr::null(), c_string.as_ptr(), CF_STRING_ENCODING_UTF8)
    };

    if string.is_null() {
        Err(String::from("Failed to create a CoreFoundation string."))
    } else {
        Ok(string)
    }
}

fn objc_selector(name: &str) -> Result<Sel, String> {
    let name =
        CString::new(name).map_err(|_| String::from("Selector name contains a null byte."))?;
    let selector = unsafe { sel_registerName(name.as_ptr()) };

    if selector.is_null() {
        Err(String::from("Failed to register an Objective-C selector."))
    } else {
        Ok(selector)
    }
}

fn objc_call_id(receiver: Id, selector: Sel) -> Id {
    let msg_send: extern "C" fn(Id, Sel) -> Id =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    msg_send(receiver, selector)
}

fn objc_call_isize(receiver: Id, selector: Sel) -> isize {
    let msg_send: extern "C" fn(Id, Sel) -> isize =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    msg_send(receiver, selector)
}
