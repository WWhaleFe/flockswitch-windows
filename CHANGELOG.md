# Changelog

## 0.4.1 — 2026-10-31

- Keyboard shortcuts: one per channel (Settings → Switching → Shortcuts, off until turned on; Ctrl+Alt+1 / 2 / 3 until changed). Pressed on any keyboard, they send the group and the master mouse to that channel. Only devices on this PC can be moved. Registered with Windows (no administrator rights); while you set a new one the shortcuts pause, so pressing it records it instead of switching
- Command line: `set hotkeys true|false`, `set hotkey.<1-3> <keys|none>`
- Settings → General and the README say which mice flockswitch was built and tested with (MX Master 3 and 4); other Logitech mice are not tested yet

## 0.4.0 — 2026-10-31

- flockswitch for Windows now has its own repository (WWhaleFe/flockswitch-windows), prepared for the Microsoft Store; the macOS version moved to WWhaleFe/flockswitch-mac. Switching, devices and settings are the same as 0.3.5, and existing settings are kept
- Removed for the Store: the in-app update check and download (new versions come from the Store or the releases page) and "Report to the developer". flockswitch makes no internet connections
