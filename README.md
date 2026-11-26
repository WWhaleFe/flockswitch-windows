# flockswitch for Windows

**English** · [한국어](README.ko.md)

Share a mouse, keyboard and clipboard over the local network, or switch Logitech devices together.

flockswitch turns buttons of an MX Master mouse - the wheel mode-shift button, the gesture button and/or the wheel click (pressing the wheel) - into an Easy-Switch key. When you press one, every device in your group (keyboard, other mice) switches to the same Easy-Switch channel, then the mouse follows. Logitech channel switching needs no network between the computers, so it works where Logitech Flow cannot (guest/public or locked-down networks).

This repository is the Windows version, also in the [Microsoft Store](https://apps.microsoft.com/detail/9NN9ZLHLGSKN). The others live in [flockswitch-mac](https://github.com/WWhaleFe/flockswitch-mac), [flockswitch-linux](https://github.com/WWhaleFe/flockswitch-linux) and [flockswitch-android](https://github.com/WWhaleFe/flockswitch-android); all of them switch the same way and can be mixed.

> Not affiliated with Logitech. "Logitech", "MX Master", "Easy-Switch", "Bolt" and "Unifying" are trademarks of Logitech.

## 2.0.0: network sharing

Share a mouse, keyboard and clipboard between your own Mac and Windows computers on the local network. Logitech hardware is optional for network sharing; existing Easy-Switch buttons, channels, gestures, haptics and effects remain available. The refreshed settings window separates Network and Logitech controls.

In **Network**, accept the notice, choose one **Master** and up to three **Subs**, then discover and pair computers and arrange their positions. Clipboard sharing includes text, images and files (received in `Downloads/flockswitch`). Network sharing takes the screen edges when enabled; enabling Logitech edges pauses network crossing until Logitech edges are off again.

**2.1** adds a layout map (place your computers like they sit on your desk; subs placed side by side cross to each other directly), a per-computer switch for using one computer's mouse and keyboard on the others, clipboard sharing between subs, key rules per pair of systems (Network → Keyboard: modifier mapping, Korean/English keys, Hanja, navigation keys, screen capture; for Windows the pairs are Mac, Linux and Android), **Find connections again**, and reconnecting by itself after sleep or unlock. The settings window is one fixed-size window with a sidebar. The mouse-shake detector for finding the cursor is quicker and watches both axes.

2.0 upgrades 1.x in place, preserving `%APPDATA%\flockswitch`, `Documents/flockswitch` backups, login startup and the Store identity. Test-build network settings and pairing identity are imported once when no network section exists, leaving the source intact.

## Requirements

- Windows 10 or 11
- For Logitech channel switching: a Logitech MX Master mouse with a wheel mode-shift button, gesture button or wheel click, connected with a Logi Bolt receiver, a Unifying receiver or Bluetooth
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
- New versions: the app looks for them and installs them itself (**Settings → General → Updates**, below).

## Use

An ⇄ icon appears in the notification area (click ^ if it is hidden). Its menu shows the status, turns switching on or off and opens **Settings…** - one window of a fixed size with a sidebar of pages (Network: Connection and layout, Crossing, Keyboard, Clipboard, This computer; Logitech: Button switching, Shortcut switching, Edge switching, Devices; App: Effects, General). A page that is longer than the window scrolls inside it and the window never changes size; every change applies at once (choosing Settings… again brings it back to the front). The card at the foot of the sidebar says where the mouse is (the lime dot means it is on this PC), what the network is doing and which device is the master mouse.

- **Connection and layout**: the sharing switch (after accepting the cautions; Windows may ask once to allow the network in its firewall), this computer's role (master or sub, changeable while no computer is paired), pairing with the matching six digits, nearby computers and adding one by IP, the paired computers (link and delay, a switch for using that computer's mouse and keyboard on the others, **Unpair**), **Find connections again** (flockswitch also reconnects by itself after sleep or unlock), and the layout map: on the master click a computer, then an empty cell of the 5 x 3 grid to move it; subs placed side by side cross to each other directly; a sub sees the master's layout, locked. A note shows when the background app is not running
- **Crossing**: the wait before crossing (with the countdown chosen in Effects), crossing only while Control is held, showing the cursor on arrival, and the return shortcut Ctrl+Alt+Shift+Q
- **Keyboard** (**When switching OS**): for each pair of systems and direction (this PC with a Mac, Linux or Android), how modifier keys are matched (by function, by position, or left as they are) with a mapping table you can change key by key, which keys switch Korean/English and what the other system understands, Caps Lock held long as a real Caps Lock, the Hanja key, navigation keys (line, word, document, delete) and screen capture shortcuts. A sub may edit too; its change goes to the master. **Compare keys by system** shows what each system calls what. The 2.0 Cmd/Ctrl, Caps Lock, Shift+Space and Hangul-key switches are these rules now (existing settings carry over)
- **Clipboard**: share text, images and files with the cursor, share between subs through the master, the received folder (`Downloads\flockswitch`) and the last file received, and the progress of a transfer
- **This computer**: the role (to be changed in Connection and layout), name, port, IP addresses and device ID
- **Button switching**: on/off, and per button (wheel mode-shift, gesture, wheel click - several can be on): the channel it goes to - one channel, chosen on each computer since settings are kept per computer; this PC's own channel is shown but cannot be chosen. The gesture button can also switch by holding it and moving the mouse up, down, left or right, each direction with its own channel (the cursor stays still while the button is held). With the wheel mode-shift button switching, **Restore the wheel mode on return** (on by default, `set keep_wheel_mode`) puts the wheel back to the ratchet or free-spinning mode it had on this computer when the mouse comes back: on a device where flockswitch has not taken the button yet (just after the mouse arrives there, e.g. on Android), a press flips the wheel instead.
- **Shortcut switching**: keyboard shortcuts, one per channel (Ctrl+Alt+1 / 2 / 3 until you change them), pressed on any keyboard. They send the group and the master mouse to that channel. The ⓘ beside the title lists the combinations Windows allows: Ctrl or Alt (with or without Shift) plus a letter or number, or an F key alone or with modifiers; Windows key combinations and ones another app has registered cannot be used. Only devices on this PC can be moved, so a keyboard that does not move along (a wired one, say) can send them away but not bring them back - press the shortcut on the other computer for that. No administrator rights needed.
- **Edge switching**: rest the cursor against an outer edge of the screen for a moment (0.3 to 2 s, your choice) and the group and the master mouse go to the channel set for that side. No network: each computer watches its own screen, so turn it on on every computer and give each side the channel of the computer that stands there. While it waits an effect runs down near the cursor (set in Effects: ring, semicircle, bar, pulse, laser, vortex, scan, liquid or random), each effect with its own colour and size, and the ring, semicircle and bar with their own direction (the ring and semicircle can also shrink away); random picks an effect, and a direction and shrinking, at random each time. You can require Ctrl to be held. When the mouse comes back, the cursor appears at the edge it left by. The corners do not count, and with several displays an edge that continues onto another display does not switch.
- **Devices**: the master mouse, and the devices that move along (Bolt, Unifying and Bluetooth devices can be mixed; devices on another computer are listed too)
- **Effects**: the countdown effect (each effect keeps its own colour and size), find the cursor (a pulse ring, radar, spotlight or random effect from a shortcut, on arrival from another computer, or by shaking the mouse for 0.5 to 2 s, optionally only while Ctrl, Alt or Shift is held), and haptics on a mouse with a motor, currently the MX Master 4 (a summary and the **Haptics Settings…** window: on/off, strength, a pulse as the mouse leaves and one when it is back, each with its own waveform or random, rising or even pulses at an edge, and **Test**)
- **General**: language (Korean or English; the first run follows the system), start at login (current user only, no admin rights; the Store version is managed by Windows and **Startup settings** opens its page), updates (the version; **Check now**; when a newer version is known, **Update now**, which downloads, checks the file against the release's SHA256SUMS.txt, runs the setup program quietly or swaps the portable program, and restarts - the Store version has the Microsoft Store install it, and Windows may show one Store dialog; the release page, or the Store page for the Store version; and two switches: **Check for new versions automatically** and **Install new versions automatically**, which the Store version leaves to the Microsoft Store), settings backup (**Save** writes the settings as a .toml file into Documents\flockswitch, up to 10 files, with the count shown; **Load…** loads one from there after you confirm; **Manage…** renames or deletes them, deleted files going to the Recycle Bin; **Open folder** opens it so files can be copied to and from other computers; files saved by the macOS and Linux versions load too, and channels, which are set per computer, should be checked after loading on another computer), reset settings, uninstall (the Store version is removed in Windows Settings → Apps)

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

flockswitch talks to this PC’s Logitech devices and your paired computers on the local network. Its only internet connection is the look for (and download of) a new version, which you can turn off; nothing about you is sent. (A port on `127.0.0.1` is held only to stop a second copy from running; nothing is sent or received on it.) Full policy: [PRIVACY.md](PRIVACY.md).

## Status

| Connection | Status |
|---|---|
| Logi Bolt receiver | tested |
| Unifying receiver | implemented, untested on Windows |
| Bluetooth | tested |

## Build

```sh
cargo build --release                                      # on Windows
cargo build --release --target x86_64-pc-windows-gnu       # from macOS/Linux (needs mingw-w64)
```

The Microsoft Store package (MSIX): `packaging/windows/store/make-msix.ps1`, built by the *Store package* workflow; see [packaging/windows/store/README.md](packaging/windows/store/README.md) (Korean) for submitting it.

## License

MIT
