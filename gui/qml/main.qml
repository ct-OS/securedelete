import QtQuick
import QtQuick.Layouts
import QtQuick.Dialogs
import org.deepin.dtk 1.0 as D
import org.securedelete.gui

D.ApplicationWindow {
    id: root

    width: 560
    height: 520
    minimumWidth: 480
    minimumHeight: 480
    visible: true
    title: qsTr("Secure Delete")
    D.DWindow.enabled: true
    flags: Qt.Window | Qt.WindowMinMaxButtonsHint | Qt.WindowCloseButtonHint | Qt.WindowTitleHint

    // Selected target (local path) and wipe mode.
    property string target: ""
    property bool deviceMode: modeBox.currentIndex === 1
    // Bumped whenever the device list changes, to re-check the target.
    property int deviceTick: 0
    property string targetError: (deviceTick, target === "" ? "" : wiper.checkTarget(target, deviceMode))
    property bool solidState: target !== "" && targetError === "" && wiper.isSolidState(target, deviceMode)
    property var devices: []
    property string devicesJson: ""
    property string resultText: ""
    property bool resultOk: false

    function localPath(url) {
        var s = url.toString();
        if (s.startsWith("file://"))
            s = decodeURIComponent(s.substring(7));
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
        // Drop a selection that was unplugged or became unusable.
        if (root.deviceMode && root.target !== ""
                && !root.devices.some(d => d.path === root.target && d.usable))
            root.target = "";
        root.deviceTick++;
    }

    function showConfirm() {
        confirmDialog.x = root.x + (root.width - confirmDialog.width) / 2;
        confirmDialog.y = root.y + Math.max(0, (root.height - confirmDialog.height) / 3);
        confirmDialog.show();
    }

    Component.onCompleted: setTarget(wiper.initialTarget())

    // Poll for plugged / unplugged drives while the device list is shown.
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

    header: D.TitleBar {
        icon.name: "secure-delete"
        title: root.title
        aboutDialog: aboutComponent
        // Same layout as DTK's default menu (theme, separator, about, exit),
        // without the "Help" entry: there is no Deepin manual for this app.
        menu: D.Menu {
            D.ThemeMenu { }
            D.MenuSeparator { }
            D.AboutAction { aboutDialog: aboutComponent }
            D.QuitAction { }
        }
    }

    // Standard Deepin About dialog.
    Component {
        id: aboutComponent
        D.AboutDialog {
            modality: Qt.NonModal
            productIcon: "secure-delete"
            productName: qsTr("Secure Delete")
            version: Qt.application.version
            description: qsTr("Securely wipes files and storage devices by overwriting their data with standard sanitization methods (DoD 5220.22-M, Gutmann, NIST 800-88…).")
            websiteName: "github.com/ct-OS/securedelete"
            websiteLink: "https://github.com/ct-OS/securedelete"
            // No `license`: DTK's label for it binds its width to its own
            // layout and collapses the right-hand column (recursive rearrange).
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

        // File / device switch.
        D.ButtonBox {
            id: modeBox
            Layout.alignment: Qt.AlignHCenter
            property int currentIndex: 0
            D.ToolButton {
                text: qsTr("File")
                checkable: true
                checked: modeBox.currentIndex === 0
                onClicked: { modeBox.currentIndex = 0; root.setTarget(""); }
            }
            D.ToolButton {
                text: qsTr("Device")
                checkable: true
                checked: modeBox.currentIndex === 1
                onClicked: { modeBox.currentIndex = 1; root.setTarget(""); }
            }
        }

        // Drop zone (file mode).
        Rectangle {
            visible: !root.deviceMode
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumHeight: 150
            radius: 12
            color: dropArea.containsDrag ? Qt.rgba(palette.highlight.r, palette.highlight.g, palette.highlight.b, 0.12)
                                         : "transparent"
            border.width: 2
            border.color: dropArea.containsDrag ? palette.highlight
                                                : Qt.rgba(palette.windowText.r, palette.windowText.g, palette.windowText.b, 0.15)

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

                D.QtIcon {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.preferredWidth: 64
                    Layout.preferredHeight: 64
                    name: root.target === "" ? "document-new" : "text-x-generic"
                }
                D.Label {
                    Layout.fillWidth: true
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.WrapAnywhere
                    text: root.target === "" ? qsTr("Drag a file here") : root.target
                    font: root.target === "" ? D.DTK.fontManager.t5 : D.DTK.fontManager.t7
                }
                D.Button {
                    Layout.alignment: Qt.AlignHCenter
                    text: root.target === "" ? qsTr("Choose file…") : qsTr("Choose another…")
                    onClicked: fileDialog.open()
                }
            }
        }

        // Detected storage devices (device mode).
        ColumnLayout {
            visible: root.deviceMode
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 8

            RowLayout {
                Layout.fillWidth: true
                D.Label {
                    text: qsTr("Storage devices")
                    font: D.DTK.fontManager.t6
                }
                Item { Layout.fillWidth: true }
                D.ToolButton {
                    icon.name: "view-refresh"
                    onClicked: root.refreshDevices()
                    D.ToolTip.visible: hovered
                    D.ToolTip.text: qsTr("Refresh")
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 130
                radius: 10
                color: "transparent"
                border.width: 1
                border.color: Qt.rgba(palette.windowText.r, palette.windowText.g, palette.windowText.b, 0.15)

                ListView {
                    id: deviceList
                    anchors.fill: parent
                    anchors.margins: 4
                    clip: true
                    spacing: 2
                    model: root.devices

                    delegate: D.ItemDelegate {
                        id: deviceItem
                        required property var modelData
                        width: ListView.view.width
                        highlighted: root.target === modelData.path
                        opacity: modelData.usable ? 1.0 : 0.6
                        onClicked: if (modelData.usable) root.setTarget(modelData.path)

                        contentItem: RowLayout {
                            spacing: 10
                            D.QtIcon {
                                Layout.preferredWidth: 32
                                Layout.preferredHeight: 32
                                name: deviceItem.modelData.icon
                            }
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 1
                                RowLayout {
                                    Layout.fillWidth: true
                                    D.Label {
                                        Layout.fillWidth: true
                                        elide: Text.ElideMiddle
                                        text: deviceItem.modelData.title
                                        font: D.DTK.fontManager.t6
                                        color: deviceItem.highlighted ? palette.highlightedText : palette.windowText
                                    }
                                    D.Label {
                                        text: deviceItem.modelData.size
                                        font: D.DTK.fontManager.t7
                                        color: deviceItem.highlighted ? palette.highlightedText : palette.windowText
                                    }
                                }
                                D.Label {
                                    Layout.fillWidth: true
                                    elide: Text.ElideRight
                                    opacity: 0.7
                                    font: D.DTK.fontManager.t8
                                    text: deviceItem.modelData.detail
                                    color: deviceItem.highlighted ? palette.highlightedText : palette.windowText
                                }
                                D.Label {
                                    Layout.fillWidth: true
                                    visible: !deviceItem.modelData.usable
                                    elide: Text.ElideRight
                                    font: D.DTK.fontManager.t8
                                    color: "#ff5736"
                                    text: deviceItem.modelData.reason
                                }
                            }
                        }
                    }
                }

                D.Label {
                    anchors.centerIn: parent
                    visible: deviceList.count === 0
                    opacity: 0.6
                    text: qsTr("No storage devices found")
                }
            }

            D.Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                opacity: 0.7
                font: D.DTK.fontManager.t8
                text: wiper.needsElevation()
                      ? qsTr("The whole device will be overwritten. It must not be mounted. You will be asked for an administrator password.")
                      : qsTr("The whole device will be overwritten. It must not be mounted.")
            }
        }

        // Options.
        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: 12
            rowSpacing: 10

            D.Label { text: qsTr("Method") }
            D.ComboBox {
                id: methodBox
                Layout.fillWidth: true
                model: wiper.methodNames()
                Component.onCompleted: currentIndex = wiper.defaultMethod()
            }

            Item { width: 1; height: 1 }
            D.CheckBox {
                id: verifyBox
                text: qsTr("Verify the last pass after wiping")
            }
        }

        // Validation / SSD notes.
        D.Label {
            Layout.fillWidth: true
            visible: root.targetError !== "" || root.solidState
            wrapMode: Text.WordWrap
            font: D.DTK.fontManager.t8
            color: root.targetError !== "" ? "#ff5736" : "#f5a623"
            text: root.targetError !== ""
                  ? root.targetError
                  : qsTr("The target is on a solid-state drive: overwriting may not reach every block (wear leveling). For whole SSDs, prefer the CLI's --secure-erase.")
        }
    }

    // Progress / result panel, kept enabled while busy so Cancel works.
    footer: Item {
        implicitHeight: footerLayout.implicitHeight + 40

        ColumnLayout {
            id: footerLayout
            anchors.fill: parent
            anchors.margins: 20
            spacing: 8

            D.Label {
                Layout.fillWidth: true
                visible: wiper.busy
                elide: Text.ElideRight
                font: D.DTK.fontManager.t8
                text: wiper.phase
            }
            D.ProgressBar {
                Layout.fillWidth: true
                visible: wiper.busy
                from: 0
                to: 1
                value: wiper.progress
            }
            D.Label {
                Layout.fillWidth: true
                visible: !wiper.busy && root.resultText !== ""
                wrapMode: Text.WordWrap
                color: root.resultOk ? "#2ca02c" : "#ff5736"
                text: root.resultText
            }
            RowLayout {
                Layout.fillWidth: true
                spacing: 10
                Item { Layout.fillWidth: true }
                D.Button {
                    visible: wiper.busy
                    text: qsTr("Cancel")
                    onClicked: wiper.cancel()
                }
                D.WarningButton {
                    visible: !wiper.busy
                    enabled: root.target !== "" && root.targetError === ""
                    text: qsTr("Wipe")
                    onClicked: root.showConfirm()
                }
            }
        }
    }

    // Irreversible action: always ask first.
    D.DialogWindow {
        id: confirmDialog
        width: 400
        modality: Qt.WindowModal
        icon: "dialog-warning"

        ColumnLayout {
            width: parent.width
            spacing: 12

            D.Label {
                Layout.fillWidth: true
                horizontalAlignment: Text.AlignHCenter
                font: D.DTK.fontManager.t5
                text: qsTr("Wipe permanently?")
            }
            D.Label {
                Layout.fillWidth: true
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.Wrap
                text: qsTr("%1 will be overwritten with “%2” and cannot be recovered.")
                      .arg(root.target).arg(methodBox.currentText)
            }
            RowLayout {
                Layout.fillWidth: true
                Layout.topMargin: 8
                spacing: 10
                D.Button {
                    Layout.fillWidth: true
                    text: qsTr("Cancel")
                    onClicked: confirmDialog.close()
                }
                D.WarningButton {
                    Layout.fillWidth: true
                    text: qsTr("Wipe")
                    onClicked: {
                        confirmDialog.close();
                        root.resultText = "";
                        wiper.start(root.target, methodBox.currentIndex, verifyBox.checked, root.deviceMode);
                    }
                }
            }
        }
    }
}
