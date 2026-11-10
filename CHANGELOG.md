# Changelog

## 1.7.1 — 2026-11-10

- Haptics is back in its own card under Find the cursor in the left column, as on the Mac; in 1.7.0 it sat at the top of the language card and broke the layout
- More room between rows and the lines between them. Rows with a segment bar or buttons on the right (size, language, shortcut) sat almost on the line below, because the control pulled the row's height in; now the space above and below is even
- The settings window opens 960 tall to fit the extra room, so the General tab still fits without scrolling

## 1.7.0 — 2026-11-10

- **Settings backup** (Settings → General): **Save** writes the settings as `flockswitch-settings-<computer>-<date-time>.toml` into Documents\flockswitch, **Load…** lists the files there, newest first, and after you confirm loads the one chosen (a file that is not a flockswitch settings file, or has a value this version cannot use, is refused with the reason), and **Open folder** opens that folder in Explorer. A file saved by the macOS or Linux version loads too: settings Windows does not have are passed over, and a shortcut Windows cannot use (one with ⌘) is left empty. Channels are set per computer, so check them after loading on another computer
- Find the cursor by shaking: it now takes shaking back and forth without a break for a time you choose, **0.5 / 1 / 1.5 / 2 s** (`set locate.shake_time 1000`, the default), and optionally only while **Ctrl**, **Alt** or **Shift** is held (`set locate.shake_key ctrl`). A brief jiggle or ordinary movement no longer brings it up
- Shortcuts: a round ⓘ beside the shortcut settings (Switching → Shortcuts, and the find-the-cursor shortcut) tells which combinations Windows allows (Ctrl or Alt, with or without Shift, plus a letter or number; an F key alone or with modifiers) and which it does not (Windows key combinations, ones another app has registered; Ctrl+Alt may type characters on some layouts). The rule under the shortcuts moved into it
- Settings window: moved to a monitor with other scaling, it keeps its size instead of growing far too large (its size limits were kept in the first monitor's pixels); the haptics window keeps its size too
- Updates: **Check now** that finds a new version offers **Update now**, which installs it in the background (through the Microsoft Store for the Store version, from GitHub otherwise), whatever the automatic switches say. A new version the tray app already found is offered at once, and a failed check or install says why. The explanation moved into an ⓘ beside the title
- General tab: haptics is a row in the card with the language and start at login, so the tab still fits the window. **Show log** is gone (the log is still written)

## 1.6.0 — 2026-11-10

- **Automatic updates**: Settings → General → **Updates** shows the version, has **Check now** / **Update now**, and two switches, **Check for new versions automatically** (at start and once a day) and **Install new versions automatically** (`set check_updates`, `set auto_update`). The tray menu offers a new version too
  - Microsoft Store version: asks the Microsoft Store and installs the new version in the background; flockswitch closes for a moment and starts again
  - Setup and portable versions: look at the GitHub releases, check the downloaded file against the release's SHA256SUMS.txt, then run the setup program quietly or, portable, swap the program file and start again
- The haptics window's 'Feel' button is now called **Test**
- Releases include SHA256SUMS.txt
- Privacy policy: the update check and download are the only internet connection

## 1.5.0 — 2026-11-10

- Haptics window: each situation on one line (name, pulse, Feel, switch), and 'use haptics / strength' set apart from the situations with more room between rows
- Haptics settings tidied: the pulse as the mouse leaves and the one when it is back are switched on and off on their own, each with its pulse beside it. Waiting at an edge offers **Rising / Even**; the edge waveform, which went unused by default, is folded into that choice (`set haptics.arrive true|false`)
- **Feel**: the haptics window plays the chosen pulse on the mouse at the chosen strength, leaving the mouse's own strength as it was
- The window says plainly that only a Logitech mouse with a haptic motor (currently the MX Master 4) vibrates
- Random pulse: choose **Random** for the leaving or arriving pulse and each one is a different pulse the mouse offers
- Effect size: the edge countdown and the find-the-cursor highlight each have a size, 75 to 150% (`set edge.anim_size`, `set locate.size`; 50 to 200% on the command line)
- A more consistent settings window: buttons, drop-downs and segment bars share one height, buttons a minimum width, row labels one weight, and the other-colour button is round like the rest. On the General tab the haptics summary moved to the right column, so the tab fits without scrolling

## 1.4.0 — 2026-11-09

- Effect colours: the screen-edge countdown and the find-the-cursor highlight each have their own colour (seven ready-made, any other from the picker). `set edge.anim_color #0A84FF`, `set locate.color #34C759`
- Random effects: choose **Random** for the edge countdown or the highlight and each one shows a different effect
- A larger settings window (840×840 to start, 760×640 to 1200×1100) with two columns on the Switching and General tabs; the Switching tab fits without scrolling at the default size

## 1.3.0 — 2026-11-09

- Settings window: its size stays within fixed limits (540×680 to start, 480×580 to 680×820), it no longer maximizes, and it keeps its size when you change tabs (a long page scrolls inside). The General tab gains Show log and Reset settings
- Haptics window: **Haptics Settings…** on the General tab opens a modal window with on/off, strength and the waveforms for switching, arrival and waiting at an edge, closed with Apply (stay open), OK (save and close) or Close (revert). A waveform you apply is played once
- Screen-edge countdown: while the cursor waits, a yellow effect runs down near it and the switch happens as it vanishes. Choose ring, semicircle, bar, pulse wave, neon laser, particle vortex, cyber scan or liquid, with direction, shrink and preview. Moving away calls it off at once. Drawn in click-through windows that follow each monitor's scaling
- Option to switch at a screen edge **only while Ctrl is held** (`set edge.control true`)
- Rising haptics: while waiting at an edge the pulses get quicker and stronger, with one more as it switches
- Find the cursor: a pulse ring, radar or spotlight around the cursor, from a shortcut, when the mouse arrives from another computer, or when it is shaken (Settings → General)
- Receivers: devices that answer slowly while waking are read again, stale entries for a device that moved to another slot go away, a device that reports no wireless ID is no longer listed under an empty one, and offline paired devices are looked for more often while no mouse has been found
- A device without switching is no longer read again forever

## 1.2.0 — 2026-11-08

- Haptics on a mouse with a motor (MX Master 4, HID++ feature 0x19B0): a pulse as the mouse is switched away and one when it comes back here, and short pulses while the cursor waits at a screen edge to switch (Settings → General → **Haptics**; both can be turned off). The strength follows the haptics setting in Logi Options+. A mouse without a motor is simply quiet; `list` shows the waveforms a mouse offers, and `set haptics.switch_waveform`, `haptics.arrive_waveform`, `haptics.edge_waveform` choose them (0-14, 27)
- Haptics: the strength can be chosen (Settings → General → Haptics: Options+'s own setting, or 25/50/75/100 %). A chosen strength goes to the mouse at once, with a pulse to judge it by, and is applied again whenever the mouse is here, since the mouse forgets it when it loses power; `set haptics.intensity <0-100>`

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
