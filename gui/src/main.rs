//! Graphical front-end for `secure_delete`, built with Qt 6. On Linux the UI
//! is `qml/main.qml`, made with the Deepin Tool Kit (DtkDeclarative); on other
//! systems (and with `SECURE_DELETE_UI=basic`) it is `qml/main_basic.qml`, made
//! with plain Qt Quick Controls. The wipe logic is the shared `secure_delete`
//! library, driven by `controller.rs`.

// No console window behind the GUI on Windows.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod controller;
mod i18n;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    let mut app = QGuiApplication::new();
    if let Some(mut app) = app.as_mut() {
        app.as_mut()
            .set_application_name(&QString::from("secure-delete"));
        app.as_mut()
            .set_organization_name(&QString::from("secure-delete"));
        // Shown in the About dialog (Qt.application.version).
        app.as_mut()
            .set_application_version(&QString::from(env!("CARGO_PKG_VERSION")));
    }

    app::set_icon();
    i18n::install();

    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(main_qml()));
    }

    if let Some(app) = app.as_mut() {
        std::process::exit(app.exec());
    }
}

/// The DTK interface on Linux unless `SECURE_DELETE_UI=basic`; the plain Qt
/// Quick Controls one everywhere else.
fn main_qml() -> &'static str {
    const DTK: &str = "qrc:/qt/qml/org/securedelete/gui/qml/main.qml";
    const BASIC: &str = "qrc:/qt/qml/org/securedelete/gui/qml/main_basic.qml";
    if cfg!(target_os = "linux") && std::env::var("SECURE_DELETE_UI").as_deref() != Ok("basic") {
        DTK
    } else {
        BASIC
    }
}
