use fbink_rs::config::{FbInkConfig, Font};
use fbink_rs::FbInk;

pub fn main() {
    // Initialize FBInk
    let config = FbInkConfig {
        font: Font::Fatty,
        is_centered: true,
        is_halfway: true,
        ..Default::default()
    };
    let fbink = match FbInk::new(config) {
        Ok(fbink) => fbink,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    // Get some info about the device
    let state = fbink.state();
    let device = state.device_details();
    let rotation = state.canonical_rotation();
    // Print to screen
    match fbink.print("Hello, world!") {
        Ok(n) => println!("Printed {n} lines to {device} in {rotation} rotation"),
        Err(e) => eprintln!("Failed to print to {device}. {e}"),
    }
}
