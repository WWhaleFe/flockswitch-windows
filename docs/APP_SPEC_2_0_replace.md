# flockswitch 2.0.0: flockswitch2 becomes flockswitch (replaces 1.x), and Korean/English switching from a Mac master

Repos: the 2.0 code (test.3) was imported into the existing public repos on branch `v2`:
`AppDev/flockswitch/mac` (WWhaleFe/flockswitch-mac, 1.x = 0.14.0) and
`AppDev/flockswitch/windows` (WWhaleFe/flockswitch-windows, 1.x = 1.10.0, also on the
Microsoft Store as WWhaleFe.Flockswitch). Each repo has its own copy of `netcore/`
(path dependency `netcore`). Both copies must stay identical: make the same netcore change
in both.

Use `git show v0.14.0:<path>` (mac) / `git show v1.10.0:<path>` (windows) to see what 1.x
used for every name below. 2.0 must install over 1.x, keep its settings, take its update
channel, login item and Store identity, and from then on be the only app.

## A. Rename flockswitch2 -> flockswitch (both repos)

- Binary / exe / crate name `flockswitch`; app `flockswitch.app`; mac bundle id
  `io.github.wwhalefe.flockswitch`; user-visible name "flockswitch" everywhere (window
  titles, tray, header, texts, how-it-works window, dialogs, report body, CLI help).
- Settings folder: exactly 1.x's (`<config_dir>/flockswitch/`), so a 1.x `config.toml`,
  `devices.toml`, `button-state.toml`, `wheel-mode.toml`, status files load as they are.
  Backups folder: 1.x's (`Documents/flockswitch`).
- One-time migration on first start of 2.0: if `<config_dir>/flockswitch2/flockswitch/`
  exists (test builds), copy its `net/` folder (identity.der, identity.key: keep the
  device id so existing pairings survive) and its `[network]` section (incl. peers) into
  the 2.0 config when the 2.0 config has no `[network]` yet; never delete the flockswitch2
  folder. Mark done with a file `migrated-from-flockswitch2` in the settings folder. Unit test.
- Single-instance ports: 1.x's (47851 app, 47852 settings). Remove 47861/47862 from the
  "port used by flockswitch" check text and list.
- Login item: 1.x's (mac LaunchAgent label/plist `flockswitch`; windows Run key / Store
  startup task as in 1.x). Windows installer: 1.x's Inno `AppId`, `AppName`,
  `DefaultDirName`, `OutputBaseFilename=flockswitch-setup-{#AppVersion}`, exe
  `flockswitch.exe`, so setup upgrades 1.x in place. Portable zip
  `flockswitch-windows-portable-<v>.zip` with `flockswitch/flockswitch.exe`.
- Store manifest (`packaging/windows/store/AppxManifest.xml`): 1.x's identity placeholders
  and app id, exe `flockswitch.exe`, keep `privateNetworkClientServer`, DisplayName
  "Flockswitch", startup task as 1.x. Check `.github/workflows/store.yml` still builds it.
- Updates: `RELEASES_API` / `RELEASES_PAGE` back to `WWhaleFe/flockswitch-mac` and
  `WWhaleFe/flockswitch-windows`; asset names as 1.x expects
  (`flockswitch-macos-arm64-<v>.zip` containing `flockswitch/flockswitch.app`,
  `flockswitch-setup-<v>.exe`, `flockswitch-windows-portable-<v>.zip`, `SHA256SUMS.txt`).
  The 1.x updater must find and install 2.0.0 (read 1.x `update.rs` `pick_asset` and the
  zip layout in 1.x `.github/workflows/release.yml`).
- mac permissions code: tccutil and identifiers use `io.github.wwhalefe.flockswitch`;
  `sign-app.sh` signs with `--identifier io.github.wwhalefe.flockswitch` (identity name
  "flockswitch2 Self Signed" stays as is, it is only the keychain label).
  `clear_stale_grants` matters here: 1.x users have ad-hoc grants that will not match.
- Info.plist: Bluetooth, local network and Bonjour keys as now, names flockswitch.
- Workflows (`.github/workflows/*.yml` from 1.x): make them build the 2.0 tree (netcore
  inside the repo, `cargo test` also for `netcore/`). mac release on GitHub: sign with
  the fixed identity: import a p12 from secrets `MAC_SIGN_P12` (base64) and
  `MAC_SIGN_P12_PASSWORD` into a temporary keychain, then `packaging/macos/sign-app.sh`
  with `FLOCK_SIGN_ID` set to its SHA1; fail the job if the secrets are missing on a tag.
- Versions: mac `2.0.0` (was 0.14.0), windows `2.0.0` (was 1.10.0; Store package
  2.0.0.0). Store and update version checks compare numerically, so 2.0.0 is newer.
- Docs: README(.ko).md, CHANGELOG(.ko).md entry 2.0.0 (network sharing of mouse,
  keyboard and clipboard between computers, master/sub, Logitech features kept, edge
  owner rule, new look), PRIVACY.md (windows: local network traffic between the user's own
  computers, nothing to the internet except update checks). No em dash characters.
- Remove leftovers that only made sense for a side-by-side install (e.g. "copy
  flockswitch's config.toml on first run", "installed beside flockswitch.app" texts).

## B. Korean/English switching when a Mac controls a Windows PC

On a Mac, Korean users switch the input source with Caps Lock (macOS "use Caps Lock to
switch") or Shift+Space (a common custom shortcut). On Windows the Korean IME toggles
with the Hangul key (VK_HANGUL, HID LANG1 0x90, scan 0x72 / "Right Alt" on Korean layouts).

When the sender is a Mac and the receiver is Windows (decided in netcore on the sending
side, like `swap_cmd_ctrl`):
- Caps Lock (HID 0x39) press -> send a LANG1 (Hangul) tap to Windows instead of Caps
  Lock, when `network.mac_capslock_to_hangul` is true (default true). Mac Caps Lock arrives
  as flagsChanged with only a state flip: treat each flip as one tap (down+up of LANG1).
  Holding Caps Lock long on the Mac (real caps lock) is not needed on Windows; that is
  what the setting can turn off.
- Shift+Space -> send a LANG1 tap instead of the Space, when
  `network.mac_shift_space_to_hangul` is true (default true): on Space down while a Shift
  is held and no other modifier, send Shift up for the held Shift(s), LANG1 down+up, then
  Shift down again; swallow the matching Space up. Other Space presses pass unchanged.
- While capturing, the Mac's own input source must not flip because of these keys if it
  can be avoided (the tap swallows them); document in a code comment if Caps Lock state
  still toggles locally (it is handled below the event tap).
- The reverse direction (Windows -> Mac: Hangul -> Caps Lock) already exists as
  `lang1_on_mac`; keep it.
- These two settings belong to the sending computer, but a master hands its network
  options to subs; include them in `MasterSettings` like `swap_cmd_ctrl`.
- UI: Network -> [키보드·클립보드] sub-tab, two switches under the Cmd/Ctrl swap row:
  "맥 Caps Lock으로 Windows 한/영 전환" / "Mac Caps Lock switches Korean/English on Windows",
  "맥 Shift+Space로 Windows 한/영 전환" / "Mac Shift+Space switches Korean/English on Windows",
  with an (i) note: "맥이 Windows를 제어할 때 맥에서 쓰던 한/영 전환 키를 Windows 한/영 키로 바꿔 보냅니다." /
  "When a Mac controls a Windows PC, the Mac's Korean/English key is sent as the Windows Hangul key."
  Show the rows on both mac and windows builds (a Windows master's subs may be Macs that
  control nothing; the rows only matter on a Mac, so on Windows show them disabled with
  hint "맥에서 설정합니다" / "Set on the Mac").
- netcore unit tests for both mappings (Mac->Windows mapped, Mac->Mac and Windows->any
  unchanged, settings off = unchanged, Shift restored after the Shift+Space tap).

## Checks
- mac repo: `cargo build --release --offline`, `cargo test --offline`, and
  `cargo test --offline --manifest-path netcore/Cargo.toml`, no warnings.
- windows repo: `cargo check --offline --target x86_64-pc-windows-gnu --all-targets`, no
  warnings; netcore tests.
- `grep -rn flockswitch2` in src/, packaging/, netcore/, docs: only the migration code and
  its test may mention it.
- No em dash characters.
