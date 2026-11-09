# flockswitch for Windows

**English** · [한국어](README.ko.md)

Press one mouse button and move your Logitech mouse **and** keyboard to another computer together.

flockswitch turns buttons of an MX Master mouse — the wheel mode-shift button, the gesture button and/or the wheel click (pressing the wheel) — into an Easy-Switch key. When you press one, every device in your group (keyboard, other mice) switches to the same Easy-Switch channel, then the mouse follows. It needs no network between the computers, so it works where Logitech Flow cannot (guest/public or locked-down networks).

This repository is the Windows version, prepared for the Microsoft Store. The others live in [flockswitch-mac](https://github.com/WWhaleFe/flockswitch-mac), [flockswitch-linux](https://github.com/WWhaleFe/flockswitch-linux) and [flockswitch-android](https://github.com/WWhaleFe/flockswitch-android); all of them switch the same way and can be mixed.

> Not affiliated with Logitech. "Logitech", "MX Master", "Easy-Switch", "Bolt" and "Unifying" are trademarks of Logitech.

## Requirements

- Windows 10 or 11
- A Logitech MX Master mouse with a wheel mode-shift button, gesture button or wheel click, connected with a Logi Bolt receiver, a Unifying receiver or Bluetooth
- Optional: a Logitech keyboard or other devices on the same channels
- Logi Options+ (recommended; flockswitch is meant to run alongside it)

## Supported mice

flockswitch was built and tested with the **MX Master 3** and **MX Master 4**. Other Logitech mice with the same buttons (HID++ 2.0 *Change Host* and reprogrammable controls) may work but have **not been tested yet**; testing them is planned. Reports on other mice are welcome.

## How it works

flockswitch talks HID++ directly to the Logi Bolt / Unifying receiver (or a Bluetooth-connected device):

1. It diverts the buttons you turn on (controls `0x00C4` wheel mode-shift, `0x00C3` gesture, `0x0052` wheel click) so presses come to flockswitch.
2. On a press it sends *Change Host* (feature `0x1814`) to the group devices first, then to the mouse.
3. When the devices come back, it notices the link notification and diverts the button again.

Install flockswitch on **every** computer you switch between (flockswitch-mac on Macs), and pair each device to the same channel number on each computer (e.g. channel 1 = Windows PC, channel 2 = Mac).

## Download

Get the latest build from [Releases](../../releases).

- **Installer**: run `flockswitch-setup-<version>.exe`. It installs for the current user only (no administrator rights; `%LOCALAPPDATA%\Programs\flockswitch`), adds Start-menu entries and an uninstaller (Settings → Apps), and upgrades in place.
- **Portable**: unzip `flockswitch-windows-portable-<version>.zip` to a folder you will keep (start-at-login remembers this location) and run `flockswitch.exe`.
- If Windows shows "Windows protected your PC", choose **More info → Run anyway**.

New versions are not downloaded by the app; get them from Releases (or, once listed, the Microsoft Store).

## Use

An ⇄ icon appears in the notification area (click ^ if it is hidden). Its menu shows the status, turns switching on or off and opens **Settings…** — a window with three tabs where every change applies at once (choosing Settings… again brings it back to the front):

- **Switching**: on/off, and per button (wheel mode-shift, gesture, wheel click — several can be on): the channel it goes to — one channel, chosen on each computer since settings are kept per computer; this PC's own channel is shown but cannot be chosen. The gesture button can also switch by holding it and moving the mouse up, down, left or right, each direction with its own channel (the cursor stays still while the button is held).
- **Shortcuts** (on the same tab, next to the buttons): keyboard shortcuts, one per channel (Ctrl+Alt+1 / 2 / 3 until you change them), pressed on any keyboard. They send the group and the master mouse to that channel. Only devices on this PC can be moved, so a keyboard that does not move along (a wired one, say) can send them away but not bring them back — press the shortcut on the other computer for that. No administrator rights needed.
- **Edges** (same tab): rest the cursor against an outer edge of the screen for a moment (0.3 to 2 s, your choice) and the group and the master mouse go to the channel set for that side. No network: each computer watches its own screen, so turn it on on every computer and give each side the channel of the computer that stands there. While it waits an effect in the colour you choose runs down near the cursor (ring, semicircle, bar, pulse, laser, vortex, scan, liquid or random), and you can require Ctrl to be held. When the mouse comes back, the cursor appears at the edge it left by. The corners do not count, and with several displays an edge that continues onto another display does not switch.
- **Devices**: the master mouse, and the devices that move along (Bolt, Unifying and Bluetooth devices can be mixed; devices on another computer are listed too)
- **General**: haptics on an MX Master 4 (a summary and the **Haptics Settings…** window: on/off, strength, waveforms, pulses that rise towards the switch), find the cursor (a pulse ring, radar, spotlight or random effect in the colour you choose, from a shortcut, on arrival from another computer, or by shaking the mouse), show the log, reset settings, the note on Logi Options+, language (Korean or English; the first run follows the system), start at login (current user only, no admin rights), uninstall

Before setting it up, set each button you turn on to **No action** in Logi Options+, so pressing it does not also run an Options+ action. When flockswitch quits or a button is turned off, the button is put back exactly as it was (including Options+'s own setting).

Command line: `flockswitch.exe help` (`list`, `switch <N>`, `set <key> <value>`, `run`; e.g. `set hotkeys true`, `set hotkey.2 ctrl+alt+2`).

Settings live in `%APPDATA%\flockswitch\`.

If something does not work, open a [problem report](../../issues/new/choose) with the output of `flockswitch.exe list`.

## Uninstall

1. Open **Settings…** → **Uninstall flockswitch…** → **Uninstall**. This puts the mouse button back to normal, removes start-at-login and deletes the settings folder, then quits.
   (Command line alternative: quit flockswitch, then run `flockswitch.exe uninstall`.)
2. Remove the installed app in Settings → Apps, or delete the portable flockswitch folder.

flockswitch installs no drivers or services and changes no system settings; it only adds a start-at-login entry for the current user when you turn that option on.

## Privacy

flockswitch talks only to the Logitech devices connected to this PC. It makes no internet connections and sends nothing anywhere. (A port on `127.0.0.1` is held only to stop a second copy from running; nothing is sent or received on it.) Full policy: [PRIVACY.md](PRIVACY.md).

## Status

| Connection | Status |
|---|---|
| Logi Bolt receiver | tested |
| Unifying receiver | implemented, untested on Windows |
| Bluetooth | implemented, untested |

## Build

```sh
cargo build --release                                      # on Windows
cargo build --release --target x86_64-pc-windows-gnu       # from macOS/Linux (needs mingw-w64)
```

The Microsoft Store package (MSIX): `packaging/windows/store/make-msix.ps1`, built by the *Store package* workflow; see [packaging/windows/store/README.md](packaging/windows/store/README.md) (Korean) for submitting it.

## License

MIT
