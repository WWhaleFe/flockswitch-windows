# Changelog

## 1.2.0 — 2026-11-08

- Haptics on a mouse with a motor (MX Master 4, HID++ feature 0x19B0): a pulse as the mouse is switched away and one when it comes back here, and short pulses while the cursor waits at a screen edge to switch (Settings → General → **Haptics**; both can be turned off). The strength follows the haptics setting in Logi Options+. A mouse without a motor is simply quiet; `list` shows the waveforms a mouse offers, and `set haptics.switch_waveform`, `haptics.arrive_waveform`, `haptics.edge_waveform` choose them (0-14, 27)

## 1.1.0 — 2026-11-08

- Switching at the screen edge, as Logi Flow does but without a network: rest the cursor against an outer edge of the screen for a moment and the group and the master mouse go to the channel set for that side (Settings → Switching → **Edges**: each side's channel, the time at the edge from 0.3 to 2 s, and whether the cursor is put at that edge when the mouse comes back). Each computer watches only its own screen; the corners do not count, nor does an edge that continues onto another display.
- Settings: the channel rows of the buttons, the gesture directions and the screen edges now start with a gray **Off** segment, which replaces the on/off switch above each of them. Gray, not the accent colour, so a chosen Off reads as switched off
- Command line: `set edges true|false`, `set edge.left|right|top|bottom <1-3|none>`, `set edge.dwell <ms>`, `set edge.cursor true|false`

## 1.0.2 — 2026-11-03

- Microsoft Store version: after an update, flockswitch starts again by itself, quietly in the notification area. Installing a Store update (also the automatic one) closes the running flockswitch, and until now the button did nothing until the next login. The setup program here already restarted it

## 1.0.1 — 2026-11-03

- flockswitch starts quietly as the ⇄ icon in the notification area, also the first time and at every login: no settings window opens. Open it from the icon's menu (**Settings…**) or by starting flockswitch again. Hovering the icon shows what it found (or the mouse it is looking for)
- Start at login passes `tray --login`, so a second copy started at login (for example the Store version next to the download) never opens the settings window either. An existing start-at-login entry is updated on the next start

## 1.0.0 — 2026-11-02

- First version for the Microsoft Store (as Flockswitch, by WWhaleFe), the same program as the download here. In the Store version, start at login is managed by Windows: on from the first start, switched in Settings → Apps → Startup (the General tab opens it)
- The first start opens the settings window, and while no mouse is found its header names the one flockswitch needs (Logitech MX Master 3 or 4)
- `flockswitch list` also shows the version and whether it runs as the Store package

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
