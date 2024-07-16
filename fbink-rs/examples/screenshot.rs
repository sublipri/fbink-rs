use fbink_rs::FbInk;
use std::fs;

pub fn main() {
    let fbink = FbInk::new(Default::default()).unwrap();
    let bytes = fbink.screenshot(image::ImageFormat::Png).unwrap();
    fs::write("/tmp/screenshot.png", bytes).unwrap();
}
