# Compila la GUI de Windows (Qt Quick Controls, sin DTK), reune Qt con
# windeployqt, comprueba que arranca sin errores de QML y crea el .zip.
# Necesita Qt 6 (MSVC) en el PATH, p. ej. con jurplel/install-qt-action.
#
#   pwsh .github/workflows/windows-gui.ps1 -Version 0.2.0
param(
    [Parameter(Mandatory = $true)][string]$Version
)
$ErrorActionPreference = 'Stop'

cargo build --release --locked -p secure_delete -p secure_delete_gui
if ($LASTEXITCODE) { exit $LASTEXITCODE }

$name = "secure-delete-gui-$Version-windows-x86_64"
Remove-Item -Recurse -Force $name, "$name.zip" -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $name | Out-Null
Copy-Item target/release/secure-delete-gui.exe, target/release/secure_delete.exe, README.md $name

# windeployqt busca los modulos QML que importan los ficheros de --qmldir;
# main.qml importa DTK (solo Linux), asi que se le pasa solo la interfaz basica.
$qmlDir = Join-Path $env:RUNNER_TEMP 'secure-delete-qml'
New-Item -ItemType Directory -Force -Path $qmlDir | Out-Null
Copy-Item gui/qml/main_basic.qml $qmlDir
windeployqt --release --qmldir $qmlDir --no-system-d3d-compiler --no-opengl-sw "$name/secure-delete-gui.exe"
if ($LASTEXITCODE) { exit $LASTEXITCODE }

# Prueba de arranque sin pantalla: la ventana debe cargarse sin errores de QML.
$env:QT_QPA_PLATFORM = 'offscreen'
$env:QT_FORCE_STDERR_LOGGING = '1'
$proc = Start-Process -FilePath "$name/secure-delete-gui.exe" -PassThru `
    -RedirectStandardError gui-stderr.txt -RedirectStandardOutput gui-stdout.txt
Start-Sleep -Seconds 10
$exited = $proc.HasExited
if (-not $exited) { Stop-Process -Id $proc.Id -Force }
$log = (Get-Content gui-stderr.txt -Raw) + (Get-Content gui-stdout.txt -Raw)
Write-Host "--- salida de la GUI ---"
Write-Host $log
if ($exited) { throw "La GUI termino al arrancar (codigo $($proc.ExitCode))" }
if ($log -match 'qrc:/\S+\.qml:\d+|is not installed|failed to load component') {
    throw 'La GUI muestra errores de QML al arrancar'
}

Compress-Archive -Path $name -DestinationPath "$name.zip"
Get-ChildItem "$name.zip"
