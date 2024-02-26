use fbink_rs::FbInk;
use std::thread::sleep;
use std::time::Duration;

pub fn main() {
    let fbink = FbInk::new(Default::default()).unwrap();
    let dump = fbink.dump().unwrap();
    // do something to change what's on the screen to see if the dump works
    sleep(Duration::from_millis(3000));
    fbink.restore(&dump).unwrap();
}
