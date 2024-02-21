Rust bindings for [FBInk](https://github.com/NiLuJe/FBInk) intended for use with Kobo devices.

There are unsafe bindings generated with [bindgen](https://github.com/rust-lang/rust-bindgen), and an incomplete safe Rust wrapper.

I have very limited experience with C and unsafe Rust, so use at your own risk!

It won't work if built with a standard Rust toolchain. The `Dockerfile` bundles an appropriate toolchain. Use it with [cargo-cross](https://github.com/cross-rs/cross/) by running e.g:

`cross build --release --example hello --target armv7-unknown-linux-musleabihf`

To use in another crate you'll need to copy `Cross.toml`, `Dockerfile` and `.cargo/config.toml`

If you have issues building it, you can try using an alternative [fbink-sys crate](https://github.com/Szybet/fbink-sys) that uses a different method for compiling FBInk. Changing the fbink-rs crate from this repo to depend on that in its `Cargo.toml` should work if you can successfully build their `fbink-sys`
