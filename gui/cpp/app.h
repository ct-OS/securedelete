// Application-level Qt helpers exposed to Rust through gui/src/app.rs.
#pragma once

#include <QtGui/QGuiApplication>
#include <QtGui/QIcon>

#include "rust/cxx.h"

// Window / taskbar icon: the installed theme icon if there is one, otherwise
// the copy embedded in the resources (resources.qrc).
inline void set_app_icon(rust::Str name)
{
    const QString icon = QString::fromUtf8(name.data(), static_cast<qsizetype>(name.size()));
    QGuiApplication::setWindowIcon(QIcon::fromTheme(
        icon, QIcon(QStringLiteral(":/icons/hicolor/scalable/apps/%1.svg").arg(icon))));
    // Lets the desktop match windows to secure-delete.desktop.
    QGuiApplication::setDesktopFileName(icon);
}
