use crate::error::FbInkError;

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use std::slice;

use fbink_sys as raw;
use image::{imageops, ColorType, DynamicImage, ImageOutputFormat};
use proc_mounts::MountIter;

#[derive(Debug, Clone)]
pub struct FbInkDump {
    raw: raw::FBInkDump,
}

impl Drop for FbInkDump {
    fn drop(&mut self) {
        unsafe { raw::fbink_free_dump_data(&mut self.raw) };
    }
}

impl FbInkDump {
    pub fn new(raw: raw::FBInkDump) -> Self {
        Self { raw }
    }
    pub fn data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts_mut(self.raw.data, self.raw.size) }
    }
    pub fn size(&self) -> usize {
        self.raw.size
    }
    pub fn stride(&self) -> usize {
        self.raw.stride
    }
    pub fn area(&self) -> raw::FBInkRect {
        self.raw.area
    }
    pub fn clip(&self) -> raw::FBInkRect {
        self.raw.clip
    }
    pub fn rota(&self) -> u8 {
        self.raw.rota
    }
    pub fn bpp(&self) -> u8 {
        self.raw.bpp
    }
    pub fn is_full(&self) -> bool {
        self.raw.is_full
    }
    pub fn raw_dump(&self) -> &raw::FBInkDump {
        &self.raw
    }
    /// Crop the regions of the dump. Doesn't touch the actual data but affects calls to restore
    pub fn crop(&mut self, left: u16, top: u16, width: u16, height: u16) {
        self.raw.clip.left = left;
        self.raw.clip.top = top;
        self.raw.clip.width = width;
        self.raw.clip.height = height;
        self.raw.is_full = false;
    }
    /// Encode the dump's data in the given image format and return the bytes
    pub fn encode(&self, encoding: ImageOutputFormat) -> Result<Vec<u8>, FbInkError> {
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
        )
        .unwrap();
        Ok(writer.into_inner())
    }
    /// Clone the dump's data and convert it to a DynamicImage
    pub fn dynamic_image(&self) -> Result<DynamicImage, FbInkError> {
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

// Adapted from FBInk/utils/sunxigrab.sh
pub fn dump_sunxi(
    encoding: Option<ImageOutputFormat>,
    current_rota: u8,
) -> Result<Vec<u8>, FbInkError> {
    // Path is hard-coded by the kernel
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

    if let Some(encoding) = encoding {
        let mut encoded = Cursor::new(Vec::new());
        decoded.write_to(&mut encoded, encoding)?;
        Ok(encoded.into_inner())
    } else {
        Ok(decoded.into_bytes())
    }
}
