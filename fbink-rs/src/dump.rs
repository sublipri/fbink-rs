use crate::thin::fbink_free_dump_data;
use crate::thin::fbink_restore;
use crate::{error::FbInkError, FbInk, FbInkRect};

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use std::slice;

use fbink_sys as raw;
use image::{imageops, ColorType, DynamicImage, ImageFormat};
use proc_mounts::MountIter;

pub trait Dump {
    fn data(&self) -> &[u8];
    fn size(&self) -> usize;
    fn stride(&self) -> usize;
    fn area(&self) -> FbInkRect;
    fn clip(&self) -> FbInkRect;
    fn rota(&self) -> u8;
    fn bpp(&self) -> u8;
    fn is_full(&self) -> bool;
    /// Crop the regions of the dump. Doesn't touch the actual data but affects calls to restore
    fn crop(&mut self, left: u16, top: u16, width: u16, height: u16);
    fn crop_rect(&mut self, rect: FbInkRect);
    fn restore(&self, fbink: &FbInk) -> Result<(), FbInkError>;

    /// Clone the dump's data and convert it to a DynamicImage
    fn dynamic_image(&self) -> Result<DynamicImage, FbInkError>;
    /// Return a reference to a DynamicImage. If the dump is a SunxiDump this won't allocate.
    /// Otherwise it will clone the data the first time it is called.
    fn dynamic_image_ref(&mut self) -> Result<&DynamicImage, FbInkError>;
    /// Encode the dump's data in the given image format and return the bytes
    fn encode(&self, encoding: ImageFormat) -> Result<Vec<u8>, FbInkError> {
        let color_type = match self.bpp() {
            32 => ColorType::Rgba8,
            24 => ColorType::Rgb8,
            // not sure if these are correct
            16 => ColorType::L16,
            8 => ColorType::L8,
            _ => {
                let msg = format!("Can't encode a dump with {} bpp", self.bpp());
                return Err(FbInkError::NotSupported(msg));
            }
        };
        let mut writer = Cursor::new(Vec::new());
        image::write_buffer_with_format(
            &mut writer,
            self.data(),
            self.area().width.into(),
            self.area().height.into(),
            color_type,
            encoding,
        )?;
        Ok(writer.into_inner())
    }
    /// Overlay an image on the dump and print it to the framebuffer
    fn print_overlay(
        &mut self,
        fbink: &FbInk,
        overlay: &DynamicImage,
        x_offset: u32,
        y_offset: u32,
    ) -> Result<(), FbInkError> {
        let mut to_print = self.dynamic_image_ref()?.crop_imm(
            x_offset,
            y_offset,
            overlay.width(),
            overlay.height(),
        );
        image::imageops::overlay(&mut to_print, overlay, 0, 0);
        let (Ok(width), Ok(height)) = (overlay.width().try_into(), overlay.height().try_into())
        else {
            return Err(FbInkError::SunxiDumpError);
        };
        let (Ok(x), Ok(y)) = (x_offset.try_into(), y_offset.try_into()) else {
            return Err(FbInkError::SunxiDumpError);
        };
        fbink.print_raw_data(to_print.as_bytes(), width, height, x, y)
    }
}

#[derive(Debug, Clone)]
pub struct FbInkDump {
    raw: raw::FBInkDump,
    image: Option<DynamicImage>,
}

impl Drop for FbInkDump {
    fn drop(&mut self) {
        fbink_free_dump_data(&mut self.raw).unwrap();
    }
}

impl FbInkDump {
    pub fn new(raw: raw::FBInkDump) -> Self {
        Self { raw, image: None }
    }
    pub fn as_raw(&self) -> &raw::FBInkDump {
        &self.raw
    }
}

impl Dump for FbInkDump {
    fn data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.raw.data, self.raw.size) }
    }
    fn size(&self) -> usize {
        self.raw.size
    }
    fn stride(&self) -> usize {
        self.raw.stride
    }
    fn area(&self) -> FbInkRect {
        self.raw.area
    }
    fn clip(&self) -> FbInkRect {
        self.raw.clip
    }
    fn rota(&self) -> u8 {
        self.raw.rota
    }
    fn bpp(&self) -> u8 {
        self.raw.bpp
    }
    fn is_full(&self) -> bool {
        self.raw.is_full
    }
    fn crop(&mut self, left: u16, top: u16, width: u16, height: u16) {
        self.raw.clip.left = left;
        self.raw.clip.top = top;
        self.raw.clip.width = width;
        self.raw.clip.height = height;
        self.raw.is_full = false;
    }
    fn crop_rect(&mut self, rect: FbInkRect) {
        self.raw.clip = rect;
        self.raw.is_full = false;
    }
    fn restore(&self, fbink: &FbInk) -> Result<(), FbInkError> {
        fbink_restore(fbink.fbfd, &fbink.config, self)
    }
    fn dynamic_image_ref(&mut self) -> Result<&DynamicImage, FbInkError> {
        if self.image.is_none() {
            self.image = Some(self.dynamic_image()?);
        }
        Ok(self.image.as_ref().unwrap())
    }
    fn dynamic_image(&self) -> Result<DynamicImage, FbInkError> {
        type Gray16Image = image::ImageBuffer<image::Luma<u16>, Vec<u16>>;
        let width = self.area().width as u32;
        let height = self.area().height as u32;
        let err_msg = "Unable to convert dump to DynamicImage";
        let image = match self.bpp() {
            32 => {
                let buf = self.data().to_vec();
                DynamicImage::ImageRgba8(image::RgbaImage::from_raw(width, height, buf).unwrap())
            }
            24 => {
                let buf = self.data().to_vec();
                DynamicImage::ImageRgb8(image::RgbImage::from_raw(width, height, buf).unwrap())
            }
            8 => {
                let buf = self.data().to_vec();
                DynamicImage::ImageLuma8(image::GrayImage::from_raw(width, height, buf).unwrap())
            }
            // Not sure if this is the correct thing to do.
            16 => {
                let (prefix, shorts, suffix) = unsafe { self.data().align_to::<u16>() };
                if !prefix.is_empty() || !suffix.is_empty() {
                    return Err(FbInkError::NotSupported(err_msg.into()));
                }
                let buf = shorts.to_vec();
                DynamicImage::ImageLuma16(Gray16Image::from_raw(width, height, buf).unwrap())
            }
            _ => return Err(FbInkError::NotSupported(err_msg.into())),
        };
        Ok(image)
    }
}

pub struct SunxiDump {
    image: DynamicImage,
    clip: FbInkRect,
    area: FbInkRect,
    rota: u8,
    bpp: u8,
    is_full: bool,
}

impl Dump for SunxiDump {
    fn data(&self) -> &[u8] {
        self.image.as_bytes()
    }

    fn size(&self) -> usize {
        self.data().len()
    }

    fn stride(&self) -> usize {
        self.image.width() as usize * (self.bpp() as usize / 8)
    }

    fn area(&self) -> FbInkRect {
        self.area
    }

    fn clip(&self) -> FbInkRect {
        self.clip
    }

    fn rota(&self) -> u8 {
        self.rota
    }

    fn bpp(&self) -> u8 {
        self.bpp
    }

    fn is_full(&self) -> bool {
        self.is_full
    }

    fn dynamic_image_ref(&mut self) -> Result<&DynamicImage, FbInkError> {
        Ok(&self.image)
    }

    fn dynamic_image(&self) -> Result<DynamicImage, FbInkError> {
        Ok(self.image.clone())
    }

    fn crop(&mut self, left: u16, top: u16, width: u16, height: u16) {
        self.clip.left = left;
        self.clip.top = top;
        self.clip.width = width;
        self.clip.height = height;
        self.is_full = false;
    }
    fn crop_rect(&mut self, rect: FbInkRect) {
        self.clip = rect;
        self.is_full = false;
    }
    fn restore(&self, fbink: &FbInk) -> Result<(), FbInkError> {
        if self.is_full {
            let (width, height) = (self.area.width.into(), self.area.height.into());
            fbink.print_raw_data(self.data(), width, height, 0, 0)
        } else {
            let c = self.clip;
            let cropped =
                self.image
                    .crop_imm(c.left.into(), c.top.into(), c.width.into(), c.height.into());
            let (width, height) = (c.width.into(), c.height.into());
            let (Ok(x), Ok(y)) = (c.left.try_into(), c.top.try_into()) else {
                return Err(FbInkError::SunxiDumpError);
            };
            fbink.print_raw_data(cropped.as_bytes(), width, height, x, y)
        }
    }
}

impl SunxiDump {
    pub fn new(current_rota: u8) -> Result<Self, FbInkError> {
        let mount_path = PathBuf::from("/mnt/flash");
        let bmp_path = mount_path.join("workingbuffer.bmp");
        fs::create_dir_all(&mount_path)?;

        // Ensure a tmpfs is mounted so we write the image to memory
        if !MountIter::new()?.any(|r| r.is_ok_and(|m| m.dest == mount_path)) {
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
        let rv = fs::read_to_string(sysfs_path)?.trim().parse::<i32>();
        if rv != Ok(0) {
            return Err(FbInkError::SunxiDumpError);
        }

        let mut decoded = image::io::Reader::open(bmp_path)?.decode()?;
        imageops::flip_vertical_in_place(&mut decoded);
        let decoded = match current_rota {
            0 => decoded.rotate270(),
            2 => decoded.rotate90(),
            3 => decoded.rotate180(),
            _ => decoded,
        };
        let area = FbInkRect {
            left: 0,
            top: 0,
            width: decoded.width() as u16,
            height: decoded.height() as u16,
        };
        Ok(Self {
            image: decoded,
            clip: FbInkRect::default(),
            area,
            rota: current_rota,
            bpp: 24,
            is_full: true,
        })
    }
}
