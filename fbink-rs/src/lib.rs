pub use crate::config::FbInkConfig;
use crate::error::FbInkError;
pub use crate::state::{CanonicalRotation, FbInkState};

use std::ffi::CString;
use std::mem::MaybeUninit;

use dump::{dump_sunxi, FbInkDump};
use fbink_sys as raw;
pub use fbink_sys::FBInkRect as FbInkRect;
pub use image::ImageOutputFormat;

pub mod config;
pub mod dump;
pub mod error;
pub mod state;

#[derive(Debug)]
pub struct FbInk {
    pub config: FbInkConfig,
    fbfd: std::os::raw::c_int,
}

impl Drop for FbInk {
    fn drop(&mut self) {
        unsafe { raw::fbink_close(self.fbfd) };
    }
}

/// An incomplete rust interface to FBInk
/// See the definitions in `FBInk/fbink.h` for more usage instructions
impl FbInk {
    /// Open the framebuffer and initialize FBInk
    pub fn new(config: FbInkConfig) -> Result<Self, FbInkError> {
        let fbfd = match unsafe { raw::fbink_open() } {
            x if -x == libc::EXIT_FAILURE => return Err(FbInkError::ExitFailure("open".into())),
            x => x,
        };

        // returns negative error codes
        let rv = unsafe { raw::fbink_init(fbfd, &config.into()) };
        match -rv {
            libc::EXIT_SUCCESS => Ok(Self { config, fbfd }),
            libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("init".into())),
            libc::ENOSYS => Err(FbInkError::NotSupported(
                "Your device is not supported by FBInk".into(),
            )),
            x => Err(FbInkError::Other(x)),
        }
    }

    /// Return FBInk's current internal state
    pub fn state(&self) -> FbInkState {
        let state = MaybeUninit::<raw::FBInkState>::zeroed();
        let mut state = unsafe { state.assume_init() };
        unsafe { raw::fbink_get_state(&self.config.into(), &mut state) };
        FbInkState::from_raw(state)
    }

    /// Re-initialize FBInk - MUST be called after certain config options have changed
    /// See fbink.h for details.
    pub fn reinit(&self) -> Result<Option<FlagSet<ReinitChanges>>, FbInkError> {
        // TODO: It would be nice to handle this automatically.  We could make FbInkConfig
        // private and add setters for all the fields, having them call reinit when necessary.
        // To avoid multiple calls to reinit, we could set a needs_reinit field instead,
        // and then act on that in any methods that might require a reinit
        match unsafe { raw::fbink_reinit(self.fbfd, &self.config.into()) } {
            libc::EXIT_SUCCESS => Ok(None),
            x if -x == libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("reinit".into())),
            x if x > 256 => Ok(Some(FlagSet::new(x as u32).unwrap())),
            x => Err(FbInkError::Other(x)),
        }
    }

    /// Print text with the current configuration. Returns number of rows printed on success
    pub fn print(&self, msg: &str) -> Result<i32, FbInkError> {
        let c_string = CString::new(msg)?;
        // returns negative error codes
        let rv = unsafe { raw::fbink_print(self.fbfd, c_string.as_ptr(), &self.config.into()) };
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

    /// Print text at the given coordinates. Returns number of rows printed on success
    pub fn print_coords(&mut self, msg: &str, x: i16, y: i16) -> Result<i32, FbInkError> {
        self.config.hoffset = x;
        self.config.voffset = y;
        // ensure other options that affect positioning are disabled.
        self.config.row = 0;
        self.config.col = 0;
        self.config.is_centered = false;
        self.config.is_halfway = false;
        self.config.is_padded = false;
        self.config.is_rpadded = false;
        self.print(msg)
    }

    /// Dump the contents of the framebuffer
    pub fn dump(&self) -> Result<FbInkDump, FbInkError> {
        let mut dump = MaybeUninit::<raw::FBInkDump>::zeroed();
        let rv = unsafe { raw::fbink_dump(self.fbfd, dump.as_mut_ptr()) };
        match -rv {
            libc::EXIT_SUCCESS => Ok(FbInkDump::new(unsafe { dump.assume_init() })),
            libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("dump".into())),
            libc::ENOSYS => {
                let msg = "FBInk was built without image support".into();
                Err(FbInkError::NotSupported(msg))
            }
            x => Err(FbInkError::Other(x)),
        }
    }

    /// Dump the contents of a specific region of the framebuffer
    pub fn region_dump(
        &self,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> Result<FbInkDump, FbInkError> {
        let mut dump = MaybeUninit::<raw::FBInkDump>::zeroed();
        // returns negative error codes
        let rv = unsafe {
            raw::fbink_region_dump(
                self.fbfd,
                x,
                y,
                width,
                height,
                &self.config.into(),
                dump.as_mut_ptr(),
            )
        };
        match -rv {
            libc::EXIT_SUCCESS => Ok(FbInkDump::new(unsafe { dump.assume_init() })),
            libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("dump".into())),
            libc::EINVAL => Err(FbInkError::InvalidArgument("empty region".into())),
            libc::ENOSYS => {
                let msg = "FBInk was built without image support".into();
                Err(FbInkError::NotSupported(msg))
            }
            x => Err(FbInkError::Other(x)),
        }
    }

    /// Like region_dump but takes a FbInkRect and doesn't apply any rotation/positioning tricks
    pub fn rect_dump(&self, rect: FbInkRect) -> Result<FbInkDump, FbInkError> {
        let mut dump = MaybeUninit::<raw::FBInkDump>::zeroed();
        let rv = unsafe { raw::fbink_rect_dump(self.fbfd, &rect, dump.as_mut_ptr()) };
        match -rv {
            libc::EXIT_SUCCESS => Ok(FbInkDump::new(unsafe { dump.assume_init() })),
            libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("dump".into())),
            libc::ENOSYS => {
                let msg = "FBInk was built without image support".into();
                Err(FbInkError::NotSupported(msg))
            }
            libc::EINVAL => Err(FbInkError::InvalidArgument("region out of bounds".into())),
            x => Err(FbInkError::Other(x)),
        }
    }

    /// Get the coordinates & dimensions of the last thing drawn on the framebuffer
    pub fn get_last_rect(&self, rotated: bool) -> FbInkRect {
        unsafe { raw::fbink_get_last_rect(rotated)}
    }

    /// Restore the contents of a dump back to the framebuffer
    pub fn restore(&self, dump: &FbInkDump) -> Result<(), FbInkError> {
        let rv = unsafe { raw::fbink_restore(self.fbfd, &self.config.into(), dump.raw_dump()) };
        match -rv {
            libc::EXIT_SUCCESS => Ok(()),
            libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("restore".into())),
            libc::ENOSYS => {
                let msg = "FBInk was built without image support".into();
                Err(FbInkError::NotSupported(msg))
            }
            libc::EINVAL => Err(FbInkError::InvalidArgument("no data".into())),
            x => Err(FbInkError::Other(x)),
        }
    }

    /// Take a screenshot of the framebuffer. Returns the encoded image as bytes
    pub fn screenshot(&self, encoding: ImageOutputFormat) -> Result<Vec<u8>, FbInkError> {
        let state = self.state();
        if state.is_sunxi {
            Ok(dump_sunxi(Some(encoding), state.current_rota)?)
        } else {
            // On some devices the dump contains junk pixels outside the visible framebuffer
            let (width, height) = (state.view_width as u16, state.view_height as u16);
            let dump = self.region_dump(0, 0, width, height)?;
            Ok(dump.encode(encoding)?)
        }
    }
}

use flagset::{flags, FlagSet};

flags! {
    pub enum ReinitChanges: u32 {
        BppChanged = raw::OK_BPP_CHANGE,
        RotationChanged = raw::OK_ROTA_CHANGE,
        LayoutChanged = raw::OK_LAYOUT_CHANGE,
        GrayscaleChanged = raw::OK_GRAYSCALE_CHANGE,
    }
}
