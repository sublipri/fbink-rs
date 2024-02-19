pub use crate::config::FbInkConfig;
use crate::error::FbInkError;
pub use crate::state::{CanonicalRotation, FbInkState};

use std::ffi::CString;
use std::fs;
use std::io::Cursor;
use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::process::Command;
use std::slice;

use fbink_sys as raw;
use image::{imageops, ColorType, ImageOutputFormat};
use proc_mounts::MountIter;

pub mod config;
pub mod error;
pub mod state;

#[derive(Debug)]
pub struct FbInk {
    pub config: FbInkConfig,
    fbfd: std::os::raw::c_int,
    raw_dump: Option<raw::FBInkDump>,
}

impl Drop for FbInk {
    fn drop(&mut self) {
        unsafe { raw::fbink_close(self.fbfd) };
        if let Some(ref mut raw_dump) = self.raw_dump {
            unsafe { raw::fbink_free_dump_data(raw_dump) };
        }
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
            libc::EXIT_SUCCESS => Ok(Self {
                config,
                fbfd,
                raw_dump: None,
            }),
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

    /// # Safety
    ///
    /// See fbink_dump in fbink.h
    pub unsafe fn raw_dump(&mut self) -> Result<raw::FBInkDump, FbInkError> {
        let mut dump = MaybeUninit::<raw::FBInkDump>::zeroed();
        // returns negative error codes
        let rv = raw::fbink_dump(self.fbfd, dump.as_mut_ptr());
        match -rv {
            libc::EXIT_SUCCESS => Ok(dump.assume_init()),
            libc::EXIT_FAILURE => Err(FbInkError::ExitFailure("dump".into())),
            libc::ENOSYS => {
                let msg = "FBInk was built without image support".into();
                Err(FbInkError::NotSupported(msg))
            }
            x => Err(FbInkError::Other(x)),
        }
    }

    /// Dump the contents of the framebuffer. Clones the data from FBInk to avoid safety issues
    pub fn dump(&mut self, encoding: Option<ImageOutputFormat>) -> Result<FbInkDump, FbInkError> {
        if self.raw_dump.is_none() {
            let raw_dump = MaybeUninit::<raw::FBInkDump>::zeroed();
            self.raw_dump = Some(unsafe { raw_dump.assume_init() });
        }
        let dump = &mut self.raw_dump.unwrap();
        // returns negative error codes
        let rv = unsafe { raw::fbink_dump(self.fbfd, dump) };
        dbg!(&dump);
        match -rv {
            libc::EXIT_SUCCESS => (),
            libc::EXIT_FAILURE => return Err(FbInkError::ExitFailure("dump".into())),
            libc::ENOSYS => {
                let msg = "FBInk was built without image support".into();
                return Err(FbInkError::NotSupported(msg));
            }
            x => return Err(FbInkError::Other(x)),
        }
        let raw_buf: &[u8] = unsafe { slice::from_raw_parts_mut(dump.data, dump.size) };

        let buf = if let Some(ref encoding) = encoding {
            let color_type = match dump.bpp {
                32 => ColorType::Rgba8,
                24 => ColorType::Rgb8,
                // not sure if these are correct
                16 => ColorType::L16,
                8 => ColorType::L8,
                _ => {
                    let msg = format!("Can't encode a dump with {} bpp", dump.bpp);
                    return Err(FbInkError::NotSupported(msg));
                }
            };
            let mut writer = Cursor::new(Vec::new());
            image::write_buffer_with_format(
                &mut writer,
                raw_buf,
                dump.area.width.into(),
                dump.area.height.into(),
                color_type,
                encoding.clone(),
            )
            .unwrap();
            writer.into_inner()
        } else {
            raw_buf.to_vec()
        };

        Ok(FbInkDump {
            data: buf,
            stride: dump.stride,
            area: dump.area,
            clip: dump.clip,
            rota: dump.rota,
            bpp: dump.bpp,
            is_full: dump.is_full,
            encoding,
        })
    }

    /// Restore the contents of a raw dump back to the framebuffer.
    pub fn restore(&mut self, raw_dump: &raw::FBInkDump) -> Result<(), FbInkError> {
        let rv = unsafe { raw::fbink_restore(self.fbfd, &self.config.into(), raw_dump) };
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
    pub fn screenshot(&mut self, encoding: ImageOutputFormat) -> Result<Vec<u8>, FbInkError> {
        let state = self.state();
        if state.is_sunxi {
            Ok(dump_sunxi(Some(encoding), state.current_rota)?)
        } else {
            Ok(self.dump(Some(encoding))?.data)
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

#[derive(Debug, Clone)]
pub struct FbInkDump {
    pub data: Vec<u8>,
    pub stride: usize,
    pub area: raw::FBInkRect,
    pub clip: raw::FBInkRect,
    pub rota: u8,
    pub bpp: u8,
    pub is_full: bool,
    pub encoding: Option<ImageOutputFormat>,
}

// Attempted to convert a FbInkDump back to a raw::FBInkDump, but it causes a segfault when
// passed to fbink_restore

// impl TryFrom<FbInkDump> for raw::FBInkDump {
//     type Error = FbInkError;
//
//     fn try_from(mut dump: FbInkDump) -> Result<Self, Self::Error> {
//         if dump.encoding.is_some() {
//             return Err(FbInkError::NotSupported(
//                 "Can't convert an encoded dump".into(),
//             ));
//         }
//         Ok(raw::FBInkDump {
//             data: dump.data.as_mut_ptr(),
//             size: dump.data.len(),
//             stride: dump.stride,
//             area: dump.area,
//             clip: dump.clip,
//             rota: dump.rota,
//             bpp: dump.bpp,
//             is_full: dump.is_full,
//         })
//     }
// }

// Adapted from FBInk/utils/sunxigrab.sh
fn dump_sunxi(
    encoding: Option<ImageOutputFormat>,
    current_rota: u8,
) -> Result<Vec<u8>, FbInkError> {
    // Path is hard-coded by the kernel
    let mount_path = PathBuf::from("/mnt/flash");
    let bmp_path = mount_path.join("workingbuffer.bmp");
    fs::create_dir_all(&mount_path).unwrap();

    if !MountIter::new()?.any(|m| m.unwrap().dest == mount_path) {
        Command::new("mount")
            .arg("-t")
            .arg("tmpfs")
            .arg("tmpfs")
            .arg(mount_path)
            .arg("-o")
            .arg("noatime,size=3M")
            .output()?;
    }

    // Trigger a dump of the framebuffer by reading from the sysfs file
    let sysfs_path = "/sys/devices/virtual/disp/disp/waveform/get_working_buffer";
    let rv: i32 = fs::read_to_string(sysfs_path)?.trim().parse().unwrap();

    if rv == 0 {
        println!("Working buffer dumped to {}", bmp_path.display());
    } else {
        println!("Failed to dump the working buffer!");
    }
    let mut decoded = image::io::Reader::open(bmp_path)?.decode()?;
    imageops::flip_vertical_in_place(&mut decoded);
    let decoded = match current_rota {
        0 => decoded.rotate270(),
        2 => decoded.rotate90(),
        3 => decoded.rotate180(),
        _ => decoded,
    };

    if let Some(encoding) = encoding {
        let mut encoded = Cursor::new(Vec::new());
        decoded.write_to(&mut encoded, encoding)?;
        Ok(encoded.into_inner())
    } else {
        Ok(decoded.into_bytes())
    }
}
