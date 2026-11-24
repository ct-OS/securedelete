//! Translations: a Spanish Qt catalogue (`i18n/secure-delete_es.ts`, compiled
//! by `build.rs`) installed at start-up, plus `tr` for strings built in Rust.

#[cxx_qt::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("i18n.h");
        fn install_translator(qm: &[u8]) -> bool;
        fn ui_languages() -> Vec<String>;
        fn translate(context: &str, text: &str) -> String;
    }
}

static QM_ES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/secure-delete_es.qm"));

/// Install the catalogue matching the system language, if there is one.
/// Must be called after the QGuiApplication exists and before QML loads.
pub fn install() {
    let preferred = ffi::ui_languages();
    let wants_spanish = preferred
        .iter()
        .take_while(|l| !l.starts_with("en"))
        .any(|l| l == "es" || l.starts_with("es-") || l.starts_with("es_"));
    if wants_spanish {
        ffi::install_translator(QM_ES);
    }
}

/// Translate a string from the Rust side (context "WipeController").
pub fn tr(text: &str) -> String {
    ffi::translate("WipeController", text)
}
