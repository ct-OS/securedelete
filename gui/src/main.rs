//! Graphical front-end for `secure_delete`, built with Qt 6 and the Deepin
//! Tool Kit (DtkDeclarative). The UI lives in `qml/main.qml`; the wipe logic
//! is the shared `secure_delete` library, driven by `controller.rs`.

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
        engine.load(&QUrl::from("qrc:/qt/qml/org/securedelete/gui/qml/main.qml"));
    }

    if let Some(app) = app.as_mut() {
        std::process::exit(app.exec());
    }
}
