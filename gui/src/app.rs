//! Application icon and desktop integration.

#[cxx_qt::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("app.h");
        fn set_app_icon(name: &str);
    }
}

/// Icon name, also used as the `.desktop` file name.
pub const ICON: &str = "secure-delete";

/// Must be called after the QGuiApplication exists.
pub fn set_icon() {
    ffi::set_app_icon(ICON);
}
