use std::path::{Path, PathBuf};
use std::process::Command;

use cxx_qt_build::{CxxQtBuilder, QmlModule};

/// Locate Qt's `lrelease`: `$LRELEASE`, Qt's bin dir per qmake, or PATH.
fn lrelease() -> PathBuf {
    if let Some(p) = std::env::var_os("LRELEASE") {
        return p.into();
    }
    for qmake in ["qmake6", "qmake"] {
        if let Ok(out) = Command::new(qmake)
            .args(["-query", "QT_INSTALL_BINS"])
            .output()
        {
            let dir = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let exe = format!("lrelease{}", std::env::consts::EXE_SUFFIX);
            let candidate = Path::new(&dir).join(exe);
            if out.status.success() && candidate.is_file() {
                return candidate;
            }
        }
    }
    "lrelease".into()
}

/// Compile the `.ts` sources into `$OUT_DIR/<name>.qm`. Without lrelease the
/// catalogue is left empty and the UI simply stays in English.
fn compile_translation(name: &str, sources: &[&str]) {
    let qm = Path::new(&std::env::var("OUT_DIR").unwrap()).join(format!("{name}.qm"));
    for ts in sources {
        println!("cargo:rerun-if-changed={ts}");
    }
    let ok = Command::new(lrelease())
        .arg("-silent")
        .args(sources)
        .arg("-qm")
        .arg(&qm)
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        println!("cargo:warning=lrelease not found or failed; building without translations");
        std::fs::write(&qm, []).unwrap();
    }
}

fn main() {
    // QML strings (refresh with `lupdate qml -ts i18n/secure-delete_es.ts`)
    // plus the hand-maintained Rust strings.
    compile_translation(
        "secure-delete_es",
        &["i18n/secure-delete_es.ts", "i18n/rust_es.ts"],
    );
    for f in ["cpp/i18n.h", "cpp/app.h", "data/secure-delete.svg"] {
        println!("cargo:rerun-if-changed={f}");
    }

    // main.qml needs DTK (Deepin Tool Kit), which only exists on Linux;
    // main_basic.qml uses plain Qt Quick Controls and works everywhere.
    let qml: &[&str] = if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        &["qml/main.qml", "qml/main_basic.qml"]
    } else {
        &["qml/main_basic.qml"]
    };
    CxxQtBuilder::new_qml_module(QmlModule::new("org.securedelete.gui").qml_files(qml))
        .qt_module("Quick")
        .include_dir("cpp")
        .qrc("resources.qrc")
        .files(["src/controller.rs", "src/i18n.rs", "src/app.rs"])
        .build();
}
