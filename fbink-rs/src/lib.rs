pub use crate::config::FbInkConfig;
use crate::error::FbInkError;
pub use crate::state::{CanonicalRotation, FbInkState};
use crate::thin::*;

use dump::{dump_sunxi, FbInkDump};
pub use fbink_sys::FBInkRect as FbInkRect;
pub use image::ImageOutputFormat;
use state::SunxiForceRotation;

pub mod config;
pub mod dump;
pub mod error;
pub mod state;
pub mod thin;

/// An incomplete attempt at a more ergonomic Rust interface to FBInk. It wraps the functions
/// from [`crate::thin`] to avoid having to pass the fd and config every function call, and
/// provides a few convenience methods.
#[derive(Debug)]
pub struct FbInk {
    pub config: FbInkConfig,
    fbfd: std::os::raw::c_int,
}

impl Drop for FbInk {
    fn drop(&mut self) {
        fbink_close(self.fbfd).unwrap();
    }
}

impl FbInk {
    /// Open the framebuffer and initialize FBInk
    pub fn new(config: FbInkConfig) -> Result<Self, FbInkError> {
        let fbfd = fbink_open()?;
        fbink_init(fbfd, &config)?;
        Ok(Self { config, fbfd })
    }

    /// Return FBInk's current internal state
    pub fn state(&self) -> FbInkState {
        fbink_get_state(&self.config)
    }

    /// Re-initialize FBInk - MUST be called after certain config options have changed
    /// See fbink.h for details.
    pub fn reinit(&self) -> ReinitResult {
        // TODO: It would be nice to handle this automatically.  We could make FbInkConfig
        // private and add setters for all the fields, having them call reinit when necessary.
        // To avoid multiple calls to reinit, we could set a needs_reinit field instead,
        // and then act on that in any methods that might require a reinit
        fbink_reinit(self.fbfd, &self.config)
    }

    /// Print text with the current configuration. Returns number of rows printed on success
    pub fn print(&self, msg: &str) -> Result<i32, FbInkError> {
        fbink_print(self.fbfd, &self.config, msg)
    }

    /// Print text at the given coordinates. Returns number of rows printed on success
    pub fn print_coords(&self, msg: &str, x: i16, y: i16) -> Result<i32, FbInkError> {
        let mut config = self.config;
        config.hoffset = x;
        config.voffset = y;
        // ensure other options that affect positioning are disabled.
        config.row = 0;
        config.col = 0;
        config.is_centered = false;
        config.is_halfway = false;
        config.is_padded = false;
        config.is_rpadded = false;
        fbink_print(self.fbfd, &config, msg)
    }

    /// Refresh the screen at the given coordinates. If all arguments are 0, performs a full refresh
    pub fn refresh(&self, top: u32, left: u32, width: u32, height: u32) -> Result<(), FbInkError> {
        fbink_refresh(self.fbfd, &self.config, top, left, width, height)
    }

    /// Refresh the screen using a FbInkRect for coordinates
    pub fn refresh_rect(&self, rect: FbInkRect) -> Result<(), FbInkError> {
        fbink_refresh_rect(self.fbfd, &self.config, rect)
    }

    /// Refresh the screen using grid coordinates with the same positioning trickery as fbink_print
    pub fn grid_refresh(&self, cols: u16, rows: u16) -> Result<(), FbInkError> {
        fbink_grid_refresh(self.fbfd, &self.config, cols, rows)
    }

    /// Clear the entire screen using the background pen color
    pub fn cls(&self) -> Result<(), FbInkError> {
        fbink_cls(self.fbfd, &self.config, Default::default(), false)
    }

    /// Clear a specific region of the screen using the background pen color
    pub fn cls_rect(&self, rect: FbInkRect, no_rota: bool) -> Result<(), FbInkError> {
        fbink_cls(self.fbfd, &self.config, rect, no_rota)
    }

    /// Clear the screen using grid coordinates with the same positioning trickery as fbink_print
    pub fn grid_clear(&self, cols: u16, rows: u16) -> Result<(), FbInkError> {
        fbink_grid_clear(self.fbfd, &self.config, cols, rows)
    }

    /// Dump the contents of the framebuffer
    pub fn dump(&self) -> Result<FbInkDump, FbInkError> {
        fbink_dump(self.fbfd)
    }

    /// Dump the contents of a specific region of the framebuffer
    pub fn region_dump(
        &self,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> Result<FbInkDump, FbInkError> {
        fbink_region_dump(self.fbfd, &self.config, x, y, width, height)
    }

    /// Like region_dump but takes a FbInkRect and doesn't apply any rotation/positioning tricks
    pub fn rect_dump(&self, rect: FbInkRect) -> Result<FbInkDump, FbInkError> {
        fbink_rect_dump(self.fbfd, rect)
    }

    /// Get the coordinates & dimensions of the last thing drawn on the framebuffer
    pub fn get_last_rect(&self, rotated: bool) -> FbInkRect {
        fbink_get_last_rect(rotated)
    }

    /// Restore the contents of a dump back to the framebuffer
    pub fn restore(&self, dump: &FbInkDump) -> Result<(), FbInkError> {
        fbink_restore(self.fbfd, &self.config, dump)
    }

    // Doesn't seem to work. Might make sense to just use the image crate for decoding and then
    // call print_raw_data
    // pub fn print_image<P: AsRef<Path>>(
    //     &self,
    //     path: P,
    //     x_off: i16,
    //     y_off: i16,
    // ) -> Result<(), FbInkError> {
    //     fbink_print_image(self.fbfd, &self.config, path, x_off, y_off)
    // }

    /// Print raw scanlines on the screen (packed pixels).
    pub fn print_raw_data(
        &self,
        data: &[u8],
        w: i32,
        h: i32,
        x_off: i16,
        y_off: i16,
    ) -> Result<(), FbInkError> {
        fbink_print_raw_data(self.fbfd, &self.config, data, w, h, x_off, y_off)
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

    /// The current canonical rotation of the framebuffer.
    pub fn current_rotation(&self) -> Result<CanonicalRotation, FbInkError> {
        self.reinit()?;
        Ok(self.state().canonical_rotation())
    }

    /// Control how fbink_init & fbink_reinit handle rotation on Sunxi SoCs
    pub fn sunxi_ntx_enforce_rota(&self, mode: SunxiForceRotation) -> ReinitResult {
        fbink_sunxi_ntx_enforce_rota(self.fbfd, &self.config, mode)
    }

    pub fn wait_for_complete(&self, marker: u32) -> Result<(), FbInkError> {
        fbink_wait_for_complete(self.fbfd, marker)
    }
    pub fn wait_for_last_complete(&self) -> Result<(), FbInkError> {
        self.wait_for_complete(fbink_sys::LAST_MARKER)
    }
    pub fn wait_for_any_complete(&self) -> Result<(), FbInkError> {
        fbink_wait_for_any_complete(self.fbfd)
    }
}
