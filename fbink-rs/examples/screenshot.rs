use fbink_rs::{FbInk, FbInkConfig};
use std::fs;

pub fn main() {
    let mut fbink = FbInk::new(FbInkConfig::default()).unwrap();
    let bytes = fbink.screenshot(image::ImageOutputFormat::Png).unwrap();
    fs::write("/tmp/screenshot.png", bytes).unwrap();
}
