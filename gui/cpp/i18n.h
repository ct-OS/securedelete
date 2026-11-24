// Small Qt i18n helpers exposed to Rust through gui/src/i18n.rs.
#pragma once

#include <QtCore/QCoreApplication>
#include <QtCore/QLocale>
#include <QtCore/QString>
#include <QtCore/QTranslator>

#include "rust/cxx.h"

// Install a compiled .qm catalogue. `qm` must outlive the application
// (it is a static embedded with include_bytes!).
inline bool install_translator(rust::Slice<const std::uint8_t> qm)
{
    static QTranslator translator;
    if (qm.empty() || !translator.load(qm.data(), static_cast<int>(qm.size())))
        return false;
    return QCoreApplication::installTranslator(&translator);
}

// UI languages in order of preference, e.g. ["es-ES", "es", "en"].
inline rust::Vec<rust::String> ui_languages()
{
    rust::Vec<rust::String> out;
    for (const QString &lang : QLocale::system().uiLanguages())
        out.push_back(rust::String(lang.toStdString()));
    return out;
}

inline rust::String translate(rust::Str context, rust::Str text)
{
    const std::string ctx(context);
    const std::string src(text);
    return rust::String(
        QCoreApplication::translate(ctx.c_str(), src.c_str()).toStdString());
}
