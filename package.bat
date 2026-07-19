@echo off
setlocal

set /p VERSION="Enter version (e.g. 0.1.0): "

echo Building release...
cargo build --release
if %errorlevel% neq 0 (
    echo Build failed!
    exit /b 1
)

echo Building tray hook...
cargo build -p nl-tray-hook --release
if %errorlevel% neq 0 (
    echo Tray hook build failed!
    exit /b 1
)

echo Copying files to staging...
if exist staging rmdir /s /q staging
mkdir staging
copy target\release\neverliie-statusbar.exe staging\
copy target\release\nl_tray_hook.dll staging\

echo Packaging with vpk...
vpk pack -u NeverLiieStatusBar -v %VERSION% -p ./staging -e neverliie-statusbar.exe
if %errorlevel% neq 0 (
    echo Packaging failed!
    exit /b 1
)

rmdir /s /q staging

echo.
echo Done! Check Releases\ for:
echo   - NeverLiieStatusBar-win-Setup.exe  (installer)
echo   - NeverLiieStatusBar-%VERSION%-full.nupkg  (update package)
echo   - RELEASES  (update manifest)
