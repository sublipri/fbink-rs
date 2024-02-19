use fbink_sys::*;
use std::ffi::CString;
use std::mem::MaybeUninit;

pub fn main() {
    let config = MaybeUninit::<FBInkConfig>::zeroed();
    let mut config = unsafe { config.assume_init() };
    config.is_halfway = true;
    config.is_centered = true;
    let fbfd = unsafe { fbink_open() };
    if fbfd == -1 {
        eprintln!("Failed to open FBInk");
        return;
    }
    let rv = unsafe { fbink_init(fbfd, &config) };
    if rv < 0 {
        eprintln!("Failed to initialize FBInk");
        return;
    }
    let msg = CString::new("Hello, world!").unwrap();
    unsafe { fbink_print(fbfd, msg.as_ptr(), &config) };
    unsafe { fbink_close(fbfd) };
}
