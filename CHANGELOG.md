# Changelog

## 0.4.4 — 2026-11-02

- “Go to channel”: this PC's own channel is locked as soon as flockswitch starts. In 0.4.3 it was only found after the first press or after the mouse reconnected, so until then every channel could be chosen
- “Go to channel”: this PC's own channel is drawn as a grayed-out, sunken box, so it is clearly not available

## 0.4.3 — 2026-11-02

- Switching: each button (and each gesture direction) now goes to one channel, “Go to channel” (이동할 채널), instead of stepping through several. Settings are kept per computer, so choose on each PC where it goes; this PC's own channel is shown but cannot be chosen. Settings from earlier versions keep working and show where a press goes from this PC

## 0.4.2 — 2026-11-02

- Settings window: opens again on PCs where it did nothing (seen with Intel UHD 620). It now draws with Direct3D 12 only instead of also trying Vulkan, whose Intel driver could hang without an error
- Settings window: if nothing is on screen after 8 seconds, it starts again with OpenGL, and says so if that does not work either; a stuck window no longer keeps "Settings…" from doing anything
- Settings window: when it cannot start because its local port is reserved or blocked (Hyper-V, WSL, Docker), it says so and logs why, instead of quitting silently

## 0.4.1 — 2026-10-31

- Keyboard shortcuts: one per channel (Settings → Switching → Shortcuts, off until turned on; Ctrl+Alt+1 / 2 / 3 until changed). Pressed on any keyboard, they send the group and the master mouse to that channel. Only devices on this PC can be moved. Registered with Windows (no administrator rights); while you set a new one the shortcuts pause, so pressing it records it instead of switching
- Command line: `set hotkeys true|false`, `set hotkey.<1-3> <keys|none>`
- Settings → General and the README say which mice flockswitch was built and tested with (MX Master 3 and 4); other Logitech mice are not tested yet

## 0.4.0 — 2026-10-31

- flockswitch for Windows now has its own repository (WWhaleFe/flockswitch-windows), prepared for the Microsoft Store; the macOS version moved to WWhaleFe/flockswitch-mac. Switching, devices and settings are the same as 0.3.5, and existing settings are kept
- Removed for the Store: the in-app update check and download (new versions come from the Store or the releases page) and "Report to the developer". flockswitch makes no internet connections
