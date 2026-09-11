fn main() {
    // The device's libEGL.so is a shim whose egl* symbols all live in libmali.so, which it only
    // pulls in via DT_NEEDED. ld does not resolve through that, so link libmali explicitly,
    // after gpui_linux's objects (link args are appended at the end of the link line).
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("aarch64") {
        println!("cargo:rustc-link-arg=-lmali");
    }
}
