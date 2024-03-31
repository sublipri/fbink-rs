//! An incomplete thin wrapper around the raw bindings from [`fbink_sys`]
//! See the comments in `FBInk/fbink.h` for more usage instructions.
//! Comments are also auto-generated in [`fbink_sys`] but with broken formatting.
use crate::config::FbInkConfig;
use crate::dump::FbInkDump;
use crate::error::FbInkError;
use crate::state::{FbInkState, SunxiForceRotation};

use std::ffi::CString;
use std::mem::MaybeUninit;
use std::os::raw::c_int;

use fbink_sys as raw;
pub use fbink_sys::FBInkRect as FbInkRect;
use flagset::{flags, FlagSet};

// pub fn fbink_version() {}
//
// pub fn fbink_state_dump() {}
//

/// Open the framebuffer, returning the file descriptor. It's the caller's responsibility to call
/// fbink_close when finished with the FD. Use the [`FbInk`](crate::FbInk) wrapper to have this managed automatically.
pub fn fbink_open() -> Result<c_int, FbInkError> {
    match unsafe { raw::fbink_open() } {
        x if -x == libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("open".into())),
        x => Ok(x),
    }
}
pub fn fbink_close(fbfd: c_int) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_close(fbfd) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("close".into())),
        x => Err(FbInkError::Other(x)),
    }
}

pub fn fbink_init(fbfd: c_int, config: &FbInkConfig) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_init(fbfd, &(*config).into()) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("init".into())),
        libc::ENOSYS => Err(FbInkError::NotSupported(
            "Your device is not supported by FBInk".into(),
        )),
        x => Err(FbInkError::Other(x)),
    }
}

/// Return FBInk's current internal state
pub fn fbink_get_state(config: &FbInkConfig) -> FbInkState {
    let state = MaybeUninit::<raw::FBInkState>::zeroed();
    let mut state = unsafe { state.assume_init() };
    unsafe { raw::fbink_get_state(&(*config).into(), &mut state) };
    FbInkState::from_raw(state)
}

/// Re-initialize FBInk - MUST be called after certain config options have changed
pub fn fbink_reinit(fbfd: c_int, config: &FbInkConfig) -> ReinitResult {
    match unsafe { raw::fbink_reinit(fbfd, &(*config).into()) } {
        libc::EXIT_SUCCESS => Ok(None),
        x if -x == libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("reinit".into())),
        x if x > 256 => Ok(Some(FlagSet::new(x as u32).unwrap())),
        x => Err(FbInkError::Other(x)),
    }
}

pub type ReinitResult = Result<Option<FlagSet<ReinitChanges>>, FbInkError>;

flags! {
    pub enum ReinitChanges: u32 {
        BppChanged = raw::OK_BPP_CHANGE,
        RotationChanged = raw::OK_ROTA_CHANGE,
        LayoutChanged = raw::OK_LAYOUT_CHANGE,
        GrayscaleChanged = raw::OK_GRAYSCALE_CHANGE,
    }
}

/// Print text with the current configuration. Returns number of rows printed on success
pub fn fbink_print(fbfd: c_int, config: &FbInkConfig, msg: &str) -> Result<i32, FbInkError> {
    let c_string = CString::new(msg)?;
    let rv = unsafe { raw::fbink_print(fbfd, c_string.as_ptr(), &(*config).into()) };
    if rv > 0 {
        return Ok(rv);
    }
    match -rv {
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("print".into())),
        libc::EINVAL => Err(FbInkError::InvalidArgument("empty string".into())),
        libc::ENOSYS => Err(FbInkError::NotSupported("fixed-cell fonts".into())),
        _ => Err(FbInkError::Other(rv)),
    }
}

/// Refresh the screen at the given coordinates. If all arguments are 0, performs a full refresh
pub fn fbink_refresh(
    fbfd: c_int,
    config: &FbInkConfig,
    top: u32,
    left: u32,
    width: u32,
    height: u32,
) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_refresh(fbfd, top, left, width, height, &(*config).into()) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("refresh".into())),
        libc::ENOSYS => {
            let msg = "Refresh is only supported on eInk devices".into();
            Err(FbInkError::NotSupported(msg))
        }
        x => Err(FbInkError::Other(x)),
    }
}

/// Refresh the screen using a FbInkRect for coordinates
pub fn fbink_refresh_rect(
    fbfd: c_int,
    config: &FbInkConfig,
    rect: FbInkRect,
) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_refresh_rect(fbfd, &rect, &(*config).into()) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("refresh_rect".into())),
        libc::ENOSYS => {
            let msg = "Refresh is only supported on eInk devices".into();
            Err(FbInkError::NotSupported(msg))
        }
        x => Err(FbInkError::Other(x)),
    }
}

/// Refresh the screen using grid coordinates with the same positioning trickery as fbink_print
pub fn fbink_grid_refresh(
    fbfd: c_int,
    config: &FbInkConfig,
    cols: u16,
    rows: u16,
) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_grid_refresh(fbfd, cols, rows, &(*config).into()) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("grid_refresh".into())),
        libc::ENOSYS => {
            let msg = "Refresh is only supported on eInk devices".into();
            Err(FbInkError::NotSupported(msg))
        }
        x => Err(FbInkError::Other(x)),
    }
}

/// Clear a region of the screen
pub fn fbink_cls(
    fbfd: c_int,
    config: &FbInkConfig,
    rect: FbInkRect,
    no_rota: bool,
) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_cls(fbfd, &(*config).into(), &rect, no_rota) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("cls".into())),
        libc::ENOSYS => {
            let msg = "FBInk was built without drawing support".into();
            Err(FbInkError::NotSupported(msg))
        }
        x => Err(FbInkError::Other(x)),
    }
}

/// Clear the screen using grid coordinates with the same positioning trickery as fbink_print
pub fn fbink_grid_clear(
    fbfd: c_int,
    config: &FbInkConfig,
    cols: u16,
    rows: u16,
) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_grid_clear(fbfd, cols, rows, &(*config).into()) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("grid_clear".into())),
        libc::ENOSYS => {
            let msg = "FBInk was built without drawing support".into();
            Err(FbInkError::NotSupported(msg))
        }
        x => Err(FbInkError::Other(x)),
    }
}

/// Dump the contents of the framebuffer
pub fn fbink_dump(fbfd: c_int) -> Result<FbInkDump, FbInkError> {
    let mut dump = MaybeUninit::<raw::FBInkDump>::zeroed();
    let rv = unsafe { raw::fbink_dump(fbfd, dump.as_mut_ptr()) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(FbInkDump::new(unsafe { dump.assume_init() })),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("dump".into())),
        libc::ENOSYS => Err(FbInkError::NoImageSupport),
        x => Err(FbInkError::Other(x)),
    }
}

/// Dump the contents of a specific region of the framebuffer
pub fn fbink_region_dump(
    fbfd: c_int,
    config: &FbInkConfig,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
) -> Result<FbInkDump, FbInkError> {
    let mut dump = MaybeUninit::<raw::FBInkDump>::zeroed();
    let rv = unsafe {
        raw::fbink_region_dump(
            fbfd,
            x,
            y,
            width,
            height,
            &(*config).into(),
            dump.as_mut_ptr(),
        )
    };
    match -rv {
        libc::EXIT_SUCCESS => Ok(FbInkDump::new(unsafe { dump.assume_init() })),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("region_dump".into())),
        libc::EINVAL => Err(FbInkError::InvalidArgument("empty region".into())),
        libc::ENOSYS => Err(FbInkError::NoImageSupport),
        x => Err(FbInkError::Other(x)),
    }
}

/// Like region_dump but takes a FbInkRect and doesn't apply any rotation/positioning tricks
pub fn fbink_rect_dump(fbfd: c_int, rect: FbInkRect) -> Result<FbInkDump, FbInkError> {
    let mut dump = MaybeUninit::<raw::FBInkDump>::zeroed();
    let rv = unsafe { raw::fbink_rect_dump(fbfd, &rect, dump.as_mut_ptr()) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(FbInkDump::new(unsafe { dump.assume_init() })),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("rect_dump".into())),
        libc::ENOSYS => Err(FbInkError::NoImageSupport),
        libc::EINVAL => Err(FbInkError::InvalidArgument("region out of bounds".into())),
        x => Err(FbInkError::Other(x)),
    }
}

/// Get the coordinates & dimensions of the last thing drawn on the framebuffer
pub fn fbink_get_last_rect(rotated: bool) -> FbInkRect {
    unsafe { raw::fbink_get_last_rect(rotated) }
}

/// Restore the contents of a dump back to the framebuffer
pub fn fbink_restore(
    fbfd: c_int,
    config: &FbInkConfig,
    dump: &FbInkDump,
) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_restore(fbfd, &(*config).into(), dump.raw_dump()) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("restore".into())),
        libc::ENOSYS => Err(FbInkError::NoImageSupport),
        libc::EINVAL => Err(FbInkError::InvalidArgument("no data".into())),
        x => Err(FbInkError::Other(x)),
    }
}

// Doesn't seem to work
// pub fn fbink_print_image<P: AsRef<Path>>(
//     fbfd: c_int,
//     config: &FbInkConfig,
//     path: P,
//     x_off: i16,
//     y_off: i16,
// ) -> Result<(), FbInkError> {
//     let filename = CString::new(path.as_ref().as_os_str().as_bytes())?.as_ptr();
//     let rv = unsafe { raw::fbink_print_image(fbfd, filename, x_off, y_off, &(*config).into()) };
//     match -rv {
//         libc::EXIT_SUCCESS => Ok(()),
//         libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("print_image".into())),
//         libc::ENOSYS => Err(FbInkError::NoImageSupport),
//         x => Err(FbInkError::Other(x)),
//     }
// }

/// Print raw scanlines on the screen (packed pixels).
pub fn fbink_print_raw_data(
    fbfd: c_int,
    config: &FbInkConfig,
    data: &[u8],
    w: i32,
    h: i32,
    x_off: i16,
    y_off: i16,
) -> Result<(), FbInkError> {
    let rv = unsafe {
        raw::fbink_print_raw_data(
            fbfd,
            data.as_ptr() as *mut std::os::raw::c_uchar,
            w,
            h,
            data.len(),
            x_off,
            y_off,
            &(*config).into(),
        )
    };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("print_raw_data".into())),
        libc::ENOSYS => Err(FbInkError::NoImageSupport),
        x => Err(FbInkError::Other(x)),
    }
}
// pub fn fbink_add_ot_font() {}
// pub fn fbink_add_ot_font_v2() {}
// pub fn fbink_free_ot_fonts() {}
// pub fn fbink_free_ot_fonts_v2() {}
// pub fn fbink_print_ot() {}
//
// pub fn fbink_printf() {}
//
// pub fn fbink_wait_for_submission() {}
pub fn fbink_wait_for_complete(fbfd: c_int, marker: u32) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_wait_for_complete(fbfd, marker) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::ENOSYS => Err(FbInkError::NotSupported(
            "wait_for_complete not supported on this device".into(),
        )),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("wait_for_complete".into())),
        x => Err(FbInkError::Other(x)),
    }
}
pub fn fbink_wait_for_any_complete(fbfd: c_int) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_wait_for_any_complete(fbfd) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::ENOSYS => Err(FbInkError::NotSupported(
            "wait_for_any_complete not supported on this device".into(),
        )),
        libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("wait_for_any_complete".into())),
        x => Err(FbInkError::Other(x)),
    }
}
// pub fn fbink_get_last_marker() {}
//
// pub fn fbink_update_verbosity() {}
// pub fn fbink_update_pen_colors() {}
// pub fn fbink_set_fg_pen_gray() {}
// pub fn fbink_set_bg_pen_gray() {}
// pub fn fbink_set_fg_pen_rgba() {}
// pub fn fbink_set_bg_pen_rgba() {}
//
// pub fn fbink_print_progress_bar() {}
// pub fn fbink_print_activity_bar() {}
//
pub fn fbink_free_dump_data(data: &mut raw::FBInkDump) -> Result<(), FbInkError> {
    let rv = unsafe { raw::fbink_free_dump_data(data) };
    match -rv {
        libc::EXIT_SUCCESS => Ok(()),
        libc::ENOSYS => Err(FbInkError::NoImageSupport),
        libc::EINVAL => Err(FbInkError::InvalidArgument("Dump was already freed".into())),
        x => Err(FbInkError::Other(x)),
    }
}
//
// pub fn fbink_button_scan() {}
// pub fn fbink_wait_for_usbms_processing() {}

pub fn fbink_rota_canonical_to_native(rota: u8) -> Result<u32, FbInkError> {
    let rv = unsafe { raw::fbink_rota_canonical_to_native(rota) };
    // returns positive error codes, not the usual negative
    match rv as i32 {
        libc::ENOSYS => {
            let msg = "Canonical rotation is only supported on Kobo devices".into();
            Err(FbInkError::NotSupported(msg))
        }
        libc::ERANGE => Err(FbInkError::OutOfRange(format!(
            "{rv} is not a valid rotation"
        ))),
        0..=3 => Ok(rv),
        x => Err(FbInkError::Other(x)),
    }
}
pub fn fbink_rota_native_to_canonical(rota: u32) -> Result<u8, FbInkError> {
    // returns positive error codes, not the usual negative
    let rv = unsafe { raw::fbink_rota_native_to_canonical(rota) };
    match rv as i32 {
        libc::ENOSYS => {
            let msg = "Canonical rotation is only supported on Kobo devices".into();
            Err(FbInkError::NotSupported(msg))
        }
        libc::ERANGE => Err(FbInkError::OutOfRange(format!(
            "{rv} is not a valid rotation"
        ))),
        0..=3 => Ok(rv),
        x => Err(FbInkError::Other(x)),
    }
}

// pub fn fbink_invert_screen() {}
// pub fn fbink_get_fb_pointer() {}
// pub fn fbink_get_fb_info() {}
// pub fn fbink_set_fb_info() {}
//
// pub fn fbink_sunxi_toggle_ntx_pen_mode() {}

/// Control how fbink_init & fbink_reinit handle rotation on Sunxi SoCs
pub fn fbink_sunxi_ntx_enforce_rota(
    fbfd: i32,
    config: &FbInkConfig,
    mode: SunxiForceRotation,
) -> ReinitResult {
    let mode = mode.into();
    let mut rv = unsafe { raw::fbink_sunxi_ntx_enforce_rota(fbfd, mode, &(*config).into()) };
    if rv < 0 {
        rv = -rv
    }
    match rv {
        libc::EXIT_SUCCESS => Ok(None),
        libc::ENOSYS => Err(FbInkError::NotSupported(
            "Only supported on Kobos with Sunxi SoCs".into(),
        )),
        libc::EINVAL => Err(FbInkError::InvalidArgument(format!(
            "{mode} is not a valid mode"
        ))),
        libc::ENOTSUP => Err(FbInkError::NotSupported(format!("{mode} is not supported"))),
        x if x > 256 => Ok(Some(FlagSet::new(x as u32).unwrap())),
        _ => Err(FbInkError::Other(rv)),
    }
}
//
// pub fn fbink_mtk_set_swipe_data() {}
// pub fn fbink_mtk_set_halftone() {}
// pub fn fbink_mtk_toggle_auto_reagl() {}
// pub fn fbink_mtk_toggle_pen_mode() {}
