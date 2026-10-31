# Privacy Policy — flockswitch for Windows

**English** · [한국어](#개인정보처리방침--flockswitch-windows)

Effective date: October 31, 2026 · Applies to flockswitch for Windows 0.4.0 and later, from the Microsoft Store or from [GitHub Releases](https://github.com/WWhaleFe/flockswitch-windows/releases).

## Summary

flockswitch collects no personal data and sends nothing anywhere. It makes no internet connections. Everything it keeps stays on your PC, in files you can read and delete.

## What flockswitch does with data

flockswitch talks to the Logitech devices connected to your PC (through a Logi Bolt or Unifying receiver, or Bluetooth) using Logitech's HID++ protocol, to read and change their Easy-Switch channel and to use the mouse buttons you turn on.

It keeps these files in `%APPDATA%\flockswitch\` (for the Microsoft Store version, in the app's own private copy of that folder):

| File | Contents |
|---|---|
| `config.toml` | Your settings: buttons, channels, keyboard shortcuts, devices in the group, language |
| `devices.toml` | Logitech devices seen on this PC: product ID (e.g. "B042"), name (e.g. "MX Master 4") and kind (mouse, keyboard) — so the settings window can list them while they are on another computer |
| `button-state.toml` | How each mouse button was set before flockswitch used it, so it can be put back |
| `status.toml` | What the app sees right now (master mouse, devices on this PC, last switch), for the settings window |
| `flockswitch.log`, `settings.log` | A log of what the app did (devices found, channel switches, errors). Started fresh at each launch |

No serial numbers, account details, names, locations, files, or anything you type are stored.

## What flockswitch does not do

- **No internet.** It does not check for updates, send reports, or contact any server. Updates come from the Microsoft Store (or from the GitHub releases page, when you choose to download one).
- **No tracking, analytics, advertising or crash reporting.**
- **No keyboard monitoring.** Keyboard shortcuts are registered with Windows (RegisterHotKey); Windows tells flockswitch only when one of its own shortcuts is pressed. No other key presses reach the app.
- **No sharing.** Nothing is sold or shared with anyone, including the developer and Logitech.

One local network port, `127.0.0.1:47851` (and `47852` while the settings window is open), is used only to stop a second copy from running. It accepts connections from this PC only, and no data is sent or received on it.

## Deleting your data

- In flockswitch: **Settings… → General → Uninstall flockswitch… → Uninstall** puts the mouse buttons back, removes start-at-login, deletes the folder above and quits.
- Uninstalling the app in **Windows Settings → Apps** also removes the Microsoft Store version's data.
- You can also delete `%APPDATA%\flockswitch\` yourself at any time.

## Children

flockswitch collects no data from anyone, including children.

## Changes

If this policy changes, the new version is published at this address with a new effective date. The history of this file is public in the repository.

## Contact

Questions: feedback.wwhale@gmail.com, or [GitHub issues](https://github.com/WWhaleFe/flockswitch-windows/issues).

flockswitch is an independent open-source project, not affiliated with Logitech. "Logitech", "MX Master", "Easy-Switch", "Bolt" and "Unifying" are trademarks of Logitech.

---

# 개인정보처리방침 — flockswitch (Windows)

[English](#privacy-policy--flockswitch-for-windows) · **한국어**

시행일: 2026년 10월 31일 · Microsoft Store 또는 [GitHub Releases](https://github.com/WWhaleFe/flockswitch-windows/releases)에서 받은 flockswitch Windows 0.4.0 이상에 적용됩니다.

## 요약

flockswitch는 개인정보를 수집하지 않으며 어떤 정보도 외부로 보내지 않습니다. 인터넷에 연결하지 않습니다. 앱이 보관하는 내용은 모두 이 PC 안의 파일에 있으며, 직접 열어 보거나 지울 수 있습니다.

## flockswitch가 다루는 정보

flockswitch는 이 PC에 연결된 Logitech 장치(Logi Bolt·Unifying 수신기 또는 블루투스)와 Logitech HID++ 프로토콜로 통신해, Easy-Switch 채널을 읽고 바꾸며 사용자가 켠 마우스 버튼을 사용합니다.

다음 파일을 `%APPDATA%\flockswitch\`에 보관합니다(Microsoft Store 버전은 앱 전용으로 분리된 같은 폴더).

| 파일 | 내용 |
|---|---|
| `config.toml` | 설정: 버튼, 채널, 키보드 단축키, 그룹 장치, 언어 |
| `devices.toml` | 이 PC에서 본 Logitech 장치의 제품 ID(예: "B042"), 이름(예: "MX Master 4"), 종류(마우스·키보드) — 장치가 다른 컴퓨터에 가 있어도 설정창에 표시하기 위함 |
| `button-state.toml` | flockswitch가 쓰기 전 각 마우스 버튼의 원래 상태 — 되돌려 놓기 위함 |
| `status.toml` | 현재 상태(마스터 마우스, 이 PC에 있는 장치, 최근 전환) — 설정창 표시용 |
| `flockswitch.log`, `settings.log` | 앱 동작 기록(찾은 장치, 채널 전환, 오류). 실행할 때마다 새로 씀 |

일련번호, 계정 정보, 이름, 위치, 파일, 사용자가 입력한 내용은 저장하지 않습니다.

## flockswitch가 하지 않는 것

- **인터넷 연결 없음.** 업데이트 확인, 리포트 전송 등 어떤 서버에도 연결하지 않습니다. 업데이트는 Microsoft Store(또는 사용자가 직접 받는 GitHub 릴리스)로만 합니다.
- **추적·분석·광고·충돌 보고 없음.**
- **키보드 입력 감시 없음.** 키보드 단축키는 Windows에 등록(RegisterHotKey)하며, Windows는 flockswitch에 등록된 단축키가 눌렸을 때만 알려 줍니다. 그 밖의 키 입력은 앱에 전달되지 않습니다.
- **제3자 제공 없음.** 개발자와 Logitech을 포함해 누구에게도 정보를 판매하거나 제공하지 않습니다.

중복 실행을 막기 위해 이 PC 안에서만 쓰는 포트 `127.0.0.1:47851`(설정창이 열려 있을 때는 `47852`도)을 사용합니다. 이 PC 안의 연결만 받으며, 이 포트로 주고받는 데이터는 없습니다.

## 정보 삭제

- 앱에서: **설정… → 일반 → flockswitch 제거… → 제거**를 누르면 마우스 버튼을 원래대로 돌려놓고, 자동 실행을 해제하고, 위 폴더를 지운 뒤 종료합니다.
- **Windows 설정 → 앱**에서 제거하면 Microsoft Store 버전의 데이터도 함께 지워집니다.
- 언제든 `%APPDATA%\flockswitch\` 폴더를 직접 지워도 됩니다.

## 아동

flockswitch는 아동을 포함한 누구의 정보도 수집하지 않습니다.

## 변경

이 방침이 바뀌면 새 시행일과 함께 이 주소에 게시합니다. 이 파일의 변경 기록은 저장소에서 누구나 볼 수 있습니다.

## 문의

feedback.wwhale@gmail.com 또는 [GitHub 문제 신고](https://github.com/WWhaleFe/flockswitch-windows/issues)

flockswitch는 Logitech과 관계없는 개인 오픈소스 프로젝트입니다. "Logitech", "MX Master", "Easy-Switch", "Bolt", "Unifying"은 Logitech의 상표입니다.
