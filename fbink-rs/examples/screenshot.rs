use fbink_rs::FbInk;
use std::fs;

pub fn main() {
    let mut fbink = FbInk::with_defaults().unwrap();
    let bytes = fbink.screenshot(image::ImageOutputFormat::Png).unwrap();
    fs::write("/tmp/screenshot.png", bytes).unwrap();
}
