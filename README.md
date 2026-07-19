# NeverLiieStatusBar

A lightweight, auto-hiding status bar for Windows built in Rust. Companion module for the [NeverLiie](https://github.com/Liiesl/NeverLiie) project.

## Features

- **Auto-hide** — slides off-screen when an app is in the foreground; reappears on hover at the top edge (500ms dwell)
- **Auto-update** — checks GitHub Releases for updates via [Velopack](https://velopack.io); download and restart in-app
- **Profile** — shows Windows user name and avatar
- **Clock** — real-time clock with date
- **Audio** — speaker/mic volume, mute, device switching
- **Media** — play/pause/skip, track info via SMTC
- **Network** — Wi-Fi scan, connect/disconnect, saved profiles
- **Battery** — charge status, remaining time, power plan switching
- **Brightness** — screen brightness control via WMI
- **Keyboard** — layout indicator and switcher (EN, JA, KO, ZH, etc.)
- **Wireless toggles** — Wi-Fi, Bluetooth, Airplane Mode, Battery Saver
- **System tray** — intercepts Explorer tray icons via a native hook DLL
- **Quick Settings** — combined panel for toggles, sliders, and power actions

## Requirements

- Windows 10/11
- Rust (edition 2024)
- Git
- [.NET SDK 8+](https://dotnet.microsoft.com/download) (for `vpk` packaging tool)

## Build

Clone with submodules:

```bash
git clone --recurse-submodules https://github.com/Liiesl/NeverLiieStatusBar.git
cd NeverLiieStatusBar
```

Build the main application:

```bash
cargo build --release
```

Build the tray hook DLL (required for system tray icon interception):

```bash
cargo build -p nl-tray-hook --release
```

Place `nl_tray_hook.dll` next to the main executable.

## Packaging & Release

Install the Velopack CLI tool:

```bash
dotnet tool install -g vpk
```

Package a release (stages only the exe + DLL, runs vpk):

```bash
package.bat
```

Or manually:

```bash
mkdir staging
copy target\release\neverliie-statusbar.exe staging\
copy target\release\nl_tray_hook.dll staging\
vpk pack -u NeverLiieStatusBar -v 0.1.0 -p ./staging -e neverliie-statusbar.exe
```

Output in `Releases/`:
- `NeverLiieStatusBar-win-Setup.exe` — standalone installer
- `NeverLiieStatusBar-0.1.0-full.nupkg` — update package
- `RELEASES` — updater manifest

To publish, upload the files from `Releases/` to a GitHub Release tagged `v<version>`.

### Automated Releases

Pushing a `v*` tag triggers the GitHub Actions workflow which builds, packages, and creates a release automatically:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## Runtime

- The tray hook DLL must be accessible from Explorer's process for tray icons to work.
- In release mode the app runs as a Windows GUI application (no console window).
- On startup, the app checks GitHub Releases for updates. If an update is available, an arrow icon appears next to the profile in the bar. Clicking it opens the update popup where you can download and restart.

## Project Structure

```
NeverLiieStatusBar/
├── src/                    # Main application
│   ├── main.rs             # Entry point (iced daemon + VelopackApp init)
│   ├── app.rs              # State, messages, update loop
│   ├── updater.rs          # Velopack update check/download/apply
│   ├── bar_ui.rs           # Bar layout
│   ├── popup.rs            # Popup panel UIs (including update popup)
│   ├── config.rs           # Dimensions, colors, timing
│   ├── audio_control.rs    # Audio volume/mute/device
│   ├── battery_control.rs  # Battery and power plans
│   ├── brightness_control.rs
│   ├── wireless_control.rs
│   ├── keyboard_control.rs
│   ├── network.rs          # Wi-Fi management
│   ├── profile_control.rs  # User profile + avatar
│   ├── ipc.rs              # Named pipe IPC server
│   ├── systray.rs          # Tray icon manager
│   └── icon_utils.rs       # HICON conversion (SSE2)
├── tray-hook/              # System tray hook DLL (cdylib)
│   └── src/lib.rs
├── iced/                   # Vendored iced GUI framework (git submodule)
├── .github/workflows/
│   ├── release.yml         # Build + vpk package on tag push
│   └── notify-parent.yml   # Submodule update notification
├── package.bat             # Local packaging script
└── Cargo.toml              # Workspace manifest
```

## Tech Stack

| Dependency | Purpose |
|-----------|---------|
| [iced](https://github.com/iced-rs/iced) 0.14 | GUI framework (daemon mode) |
| [tokio](https://github.com/tokio-rs/tokio) | Async runtime |
| [windows](https://github.com/microsoft/windows-rs) 0.62 | Win32/WinRT bindings |
| [chrono](https://github.com/chronotope/chrono) | Date/time formatting |
| [serde](https://github.com/serde-rs/serde) | JSON serialization |
| [interprocess](https://github.com/kotauskas/interprocess) | Named pipe IPC |
| [image](https://github.com/image-rs/image) | Avatar and icon processing |
| [lucide-icons](https://github.com/nicholasgasior/iced-lucide-icons) | Icon font |
| [velopack](https://github.com/velopack/velopack) | Auto-update and installer framework |

## Configuration

Runtime constants are defined in `src/config.rs`:

- Bar height: 40px
- Animation: 300ms ease-out cubic
- Auto-hide delay: 400ms
- Hover dwell time: 500ms
- Popup width: 320px
- Dark theme: `rgb(25,25,25)` background, light text

## License

See the repository for license details.
