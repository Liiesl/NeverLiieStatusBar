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

## Installation

Download the latest installer or portable zip from the [Releases](https://github.com/Liiesl/NeverLiieStatusBar/releases) page.

The app auto-updates — when a new version is released, an update indicator appears in the bar.

## Building from Source

Requires Windows 10/11, Rust (edition 2024), and Git.

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

## Runtime

- The tray hook DLL must be accessible from Explorer's process for tray icons to work.
- In release mode the app runs as a Windows GUI application (no console window).
- On startup, the app checks GitHub Releases for updates. If an update is available, an "Update available" indicator appears in the bar. Clicking it opens the update popup where you can download and restart.

## Project Structure

```
NeverLiieStatusBar/
├── src/                    # Main application
│   ├── main.rs             # Entry point (iced daemon + VelopackApp init)
│   ├── app.rs              # State, messages, update loop
│   ├── config.rs           # Dimensions, colors, timing
│   ├── services/           # Background services
│   │   ├── updater.rs      # Velopack update check/download/apply
│   │   ├── audio.rs        # Audio volume/mute/device
│   │   ├── battery.rs      # Battery and power plans
│   │   ├── brightness.rs   # Screen brightness
│   │   ├── keyboard.rs     # Layout indicator/switcher
│   │   ├── network.rs      # Wi-Fi management
│   │   ├── profile.rs      # User profile + avatar
│   │   └── wireless.rs     # Wi-Fi/Bluetooth/Airplane toggles
│   ├── platform/           # OS-level integrations
│   │   ├── win32.rs        # Window flags, DWM, z-order
│   │   ├── ipc.rs          # Named pipe IPC server
│   │   ├── systray.rs      # Tray icon manager
│   │   └── icon_utils.rs   # HICON conversion
│   └── ui/                 # UI layer
│       ├── bar.rs          # Status bar layout
│       └── popup/          # Popup panels
│           ├── update.rs   # Update popup
│           ├── audio.rs
│           ├── battery.rs
│           ├── keyboard.rs
│           ├── network.rs
│           ├── profile.rs
│           ├── settings.rs
│           └── tray.rs
├── tray-hook/              # System tray hook DLL (cdylib)
├── iced/                   # Vendored iced GUI framework (git submodule)
├── .github/workflows/
│   └── release.yml         # Build + package + publish on tag push
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
