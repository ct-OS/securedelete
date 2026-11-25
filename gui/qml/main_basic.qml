// Interface for systems without DTK (Windows, macOS, or Linux with
// SECURE_DELETE_UI=basic): plain Qt Quick Controls, same WipeController.
// Keep the user-visible strings identical to main.qml where possible so the
// translations can be shared.
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import org.securedelete.gui

ApplicationWindow {
    id: root

    width: 560
    height: 540
    minimumWidth: 460
    minimumHeight: 480
    visible: true
    title: qsTr("Secure Delete")

    readonly property bool devicesSupported: wiper.supportsDevices()
    property string target: ""
    property bool deviceMode: devicesSupported && modeBar.currentIndex === 1
    property int deviceTick: 0
    property string targetError: (deviceTick, target === "" ? "" : wiper.checkTarget(target, deviceMode))
    property bool solidState: target !== "" && targetError === "" && wiper.isSolidState(target, deviceMode)
    property var devices: []
    property string devicesJson: ""
    property string resultText: ""
    property bool resultOk: false

    readonly property color errorColor: "#d93025"
    readonly property color warningColor: "#b26a00"
    readonly property color okColor: "#1e8e3e"

    function localPath(url) {
        var s = url.toString();
        if (s.startsWith("file:///") && Qt.platform.os === "windows")
            return decodeURIComponent(s.substring(8)).replace(/\//g, "\\");
        if (s.startsWith("file://"))
            return decodeURIComponent(s.substring(7));
        return s;
    }

    function setTarget(path) {
        root.target = path;
        root.resultText = "";
    }

    function refreshDevices() {
        var json = wiper.devicesJson();
        if (json === root.devicesJson)
            return;
        root.devicesJson = json;
        root.devices = JSON.parse(json);
        if (root.deviceMode && root.target !== ""
                && !root.devices.some(d => d.path === root.target && d.usable))
            root.target = "";
        root.deviceTick++;
    }

    Component.onCompleted: setTarget(wiper.initialTarget())

    Timer {
        interval: 2000
        repeat: true
        triggeredOnStart: true
        running: root.deviceMode && !wiper.busy && root.visible
        onTriggered: root.refreshDevices()
    }

    WipeController {
        id: wiper
        onFinished: function (ok, message) {
            root.resultOk = ok;
            if (ok) {
                root.resultText = root.deviceMode ? qsTr("The device was wiped.")
                                                  : qsTr("The file was wiped and deleted.");
                if (!root.deviceMode)
                    root.target = "";
            } else {
                root.resultText = message;
            }
        }
    }

    header: ToolBar {
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 12
            Image {
                source: "qrc:/icons/hicolor/scalable/apps/secure-delete.svg"
                sourceSize: Qt.size(24, 24)
                Layout.preferredWidth: 24
                Layout.preferredHeight: 24
            }
            Label {
                text: root.title
                font.bold: true
                Layout.fillWidth: true
            }
            ToolButton {
                text: "☰"
                onClicked: appMenu.open()
                Menu {
                    id: appMenu
                    y: parent.height
                    MenuItem {
                        text: qsTr("About")
                        onTriggered: aboutDialog.open()
                    }
                    MenuSeparator { }
                    MenuItem {
                        text: qsTr("Exit")
                        onTriggered: Qt.quit()
                    }
                }
            }
        }
    }

    FileDialog {
        id: fileDialog
        title: qsTr("Choose a file to wipe")
        onAccepted: root.setTarget(root.localPath(selectedFile))
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 14
        enabled: !wiper.busy

        TabBar {
            id: modeBar
            visible: root.devicesSupported
            Layout.alignment: Qt.AlignHCenter
            TabButton {
                text: qsTr("File")
                width: implicitWidth + 24
                onClicked: root.setTarget("")
            }
            TabButton {
                text: qsTr("Device")
                width: implicitWidth + 24
                onClicked: root.setTarget("")
            }
        }

        // Drop zone (file mode).
        Rectangle {
            visible: !root.deviceMode
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumHeight: 150
            radius: 10
            color: dropArea.containsDrag ? Qt.rgba(root.palette.highlight.r, root.palette.highlight.g,
                                                   root.palette.highlight.b, 0.12)
                                         : "transparent"
            border.width: 2
            border.color: dropArea.containsDrag ? root.palette.highlight
                                                : Qt.rgba(root.palette.windowText.r, root.palette.windowText.g,
                                                          root.palette.windowText.b, 0.2)

            DropArea {
                id: dropArea
                anchors.fill: parent
                onDropped: function (drop) {
                    if (drop.hasUrls)
                        root.setTarget(root.localPath(drop.urls[0]));
                }
            }

            ColumnLayout {
                anchors.centerIn: parent
                width: parent.width - 40
                spacing: 10

                Image {
                    Layout.alignment: Qt.AlignHCenter
                    source: "qrc:/icons/hicolor/scalable/apps/secure-delete.svg"
                    sourceSize: Qt.size(64, 64)
                    opacity: root.target === "" ? 0.5 : 1.0
                }
                Label {
                    Layout.fillWidth: true
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.WrapAnywhere
                    text: root.target === "" ? qsTr("Drag a file here") : root.target
                    font.pixelSize: root.target === "" ? 18 : 13
                }
                Button {
                    Layout.alignment: Qt.AlignHCenter
                    text: root.target === "" ? qsTr("Choose file…") : qsTr("Choose another…")
                    onClicked: fileDialog.open()
                }
            }
        }

        // Detected storage devices (device mode, Linux only).
        ColumnLayout {
            visible: root.deviceMode
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 8

            RowLayout {
                Layout.fillWidth: true
                Label {
                    text: qsTr("Storage devices")
                    font.bold: true
                }
                Item { Layout.fillWidth: true }
                Button {
                    text: qsTr("Refresh")
                    onClicked: root.refreshDevices()
                }
            }

            Frame {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 130
                padding: 2

                ListView {
                    id: deviceList
                    anchors.fill: parent
                    clip: true
                    model: root.devices

                    delegate: ItemDelegate {
                        id: deviceItem
                        required property var modelData
                        width: ListView.view.width
                        highlighted: root.target === modelData.path
                        enabled: modelData.usable
                        onClicked: root.setTarget(modelData.path)

                        contentItem: ColumnLayout {
                            spacing: 1
                            RowLayout {
                                Layout.fillWidth: true
                                Label {
                                    Layout.fillWidth: true
                                    elide: Text.ElideMiddle
                                    text: deviceItem.modelData.title
                                    font.bold: true
                                }
                                Label { text: deviceItem.modelData.size }
                            }
                            Label {
                                Layout.fillWidth: true
                                elide: Text.ElideRight
                                opacity: 0.7
                                text: deviceItem.modelData.detail
                            }
                            Label {
                                Layout.fillWidth: true
                                visible: !deviceItem.modelData.usable
                                elide: Text.ElideRight
                                color: root.errorColor
                                text: deviceItem.modelData.reason
                            }
                        }
                    }
                }

                Label {
                    anchors.centerIn: parent
                    visible: deviceList.count === 0
                    opacity: 0.6
                    text: qsTr("No storage devices found")
                }
            }

            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                opacity: 0.7
                text: wiper.needsElevation()
                      ? qsTr("The whole device will be overwritten. It must not be mounted. You will be asked for an administrator password.")
                      : qsTr("The whole device will be overwritten. It must not be mounted.")
            }
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: 12
            rowSpacing: 10

            Label { text: qsTr("Method") }
            ComboBox {
                id: methodBox
                Layout.fillWidth: true
                model: wiper.methodNames()
                Component.onCompleted: currentIndex = wiper.defaultMethod()
            }

            Item { width: 1; height: 1 }
            CheckBox {
                id: verifyBox
                text: qsTr("Verify the last pass after wiping")
            }
        }

        Label {
            Layout.fillWidth: true
            visible: root.targetError !== "" || root.solidState
            wrapMode: Text.WordWrap
            color: root.targetError !== "" ? root.errorColor : root.warningColor
            text: root.targetError !== ""
                  ? root.targetError
                  : qsTr("The target is on a solid-state drive: overwriting may not reach every block (wear leveling). For whole SSDs, prefer the CLI's --secure-erase.")
        }
    }

    footer: Item {
        implicitHeight: footerLayout.implicitHeight + 32

        ColumnLayout {
            id: footerLayout
            anchors.fill: parent
            anchors.margins: 16
            spacing: 8

            Label {
                Layout.fillWidth: true
                visible: wiper.busy
                elide: Text.ElideRight
                text: wiper.phase
            }
            ProgressBar {
                Layout.fillWidth: true
                visible: wiper.busy
                from: 0
                to: 1
                value: wiper.progress
            }
            Label {
                Layout.fillWidth: true
                visible: !wiper.busy && root.resultText !== ""
                wrapMode: Text.WordWrap
                color: root.resultOk ? root.okColor : root.errorColor
                text: root.resultText
            }
            RowLayout {
                Layout.fillWidth: true
                Item { Layout.fillWidth: true }
                Button {
                    visible: wiper.busy
                    text: qsTr("Cancel")
                    onClicked: wiper.cancel()
                }
                Button {
                    visible: !wiper.busy
                    enabled: root.target !== "" && root.targetError === ""
                    text: qsTr("Wipe")
                    highlighted: true
                    onClicked: confirmDialog.open()
                }
            }
        }
    }

    Dialog {
        id: confirmDialog
        anchors.centerIn: parent
        width: Math.min(420, root.width - 40)
        modal: true
        title: qsTr("Wipe permanently?")

        Label {
            width: confirmDialog.availableWidth
            wrapMode: Text.Wrap
            text: qsTr("%1 will be overwritten with “%2” and cannot be recovered.")
                  .arg(root.target).arg(methodBox.currentText)
        }

        footer: DialogButtonBox {
            Button {
                text: qsTr("Cancel")
                DialogButtonBox.buttonRole: DialogButtonBox.RejectRole
            }
            Button {
                text: qsTr("Wipe")
                DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
            }
        }

        onAccepted: {
            root.resultText = "";
            wiper.start(root.target, methodBox.currentIndex, verifyBox.checked, root.deviceMode);
        }
    }

    Dialog {
        id: aboutDialog
        anchors.centerIn: parent
        width: Math.min(440, root.width - 40)
        modal: true
        title: qsTr("About")
        standardButtons: Dialog.Close

        ColumnLayout {
            width: aboutDialog.availableWidth
            spacing: 8

            Image {
                Layout.alignment: Qt.AlignHCenter
                source: "qrc:/icons/hicolor/scalable/apps/secure-delete.svg"
                sourceSize: Qt.size(80, 80)
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                text: qsTr("Secure Delete")
                font.pixelSize: 18
                font.bold: true
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                text: qsTr("Version %1").arg(Qt.application.version)
                opacity: 0.7
            }
            Label {
                Layout.fillWidth: true
                Layout.topMargin: 6
                wrapMode: Text.WordWrap
                horizontalAlignment: Text.AlignHCenter
                text: qsTr("Securely wipes files and storage devices by overwriting their data with standard sanitization methods (DoD 5220.22-M, Gutmann, NIST 800-88…).")
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                textFormat: Text.RichText
                text: "<a href='https://github.com/ct-OS/securedelete'>github.com/ct-OS/securedelete</a>"
                onLinkActivated: link => Qt.openUrlExternally(link)
                HoverHandler { cursorShape: Qt.PointingHandCursor }
            }
        }
    }
}
