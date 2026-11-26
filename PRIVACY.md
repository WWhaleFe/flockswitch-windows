# Privacy Policy - flockswitch for Windows

**English** · [한국어](#개인정보처리방침--flockswitch-windows)

Effective date: November 26, 2026 · Applies to flockswitch for Windows 2.0.0 and later, from the Microsoft Store or from [GitHub Releases](https://github.com/WWhaleFe/flockswitch-windows/releases).

## Summary

flockswitch collects no personal data and sends nothing about you anywhere. Its only internet connection is the look for a new version, which you can turn off. Settings and pairing identities stay on your PC. With network sharing enabled, input and shared clipboard content travel only between your paired computers on the local network. Received files are saved in Downloads/flockswitch.

## What flockswitch does with data

flockswitch talks to the Logitech devices connected to your PC (through a Logi Bolt or Unifying receiver, or Bluetooth) using Logitech's HID++ protocol, to read and change their Easy-Switch channel and to use the mouse buttons you turn on.

It keeps these files in `%APPDATA%\flockswitch\` (for the Microsoft Store version, in the app's own private copy of that folder):

| File | Contents |
|---|---|
| `config.toml` | Your settings: buttons, channels, keyboard shortcuts, devices in the group, language |
| `devices.toml` | Logitech devices seen on this PC: product ID (e.g. "B042"), name (e.g. "MX Master 4") and kind (mouse, keyboard) - so the settings window can list them while they are on another computer |
| `button-state.toml` | How each mouse button was set before flockswitch used it, so it can be put back |
| `status.toml` | What the app sees right now (master mouse, devices on this PC, last switch), for the settings window |
| `flockswitch.log`, `settings.log` | A log of what the app did (devices found, channel switches, errors). Started fresh at each launch |

Network settings store computer names, paired device IDs and certificate fingerprints. The net/ folder stores this computer’s pairing certificate and private key. Received clipboard files are stored in Downloads/flockswitch; keystrokes are forwarded while sharing and are not logged.

## What flockswitch does not do

- **No internet beyond updates.** At start and once a day, while **Settings → General → Updates → Check for new versions automatically** is on (and whenever you press **Check now**), it looks for a new version: the Microsoft Store version asks the Microsoft Store through Windows; the version from GitHub asks GitHub's public releases API over https and, to install, downloads the new release file from GitHub and checks it against the release's published SHA-256 list. These requests carry only what any web request does (your IP address, and a "flockswitch/<version>" user agent); no account, settings, devices or other data are sent. With **Install new versions automatically** on, a new version is installed by itself. It sends no reports and contacts no other server.
- **No tracking, analytics, advertising or crash reporting.**
- **Input sharing only when enabled.** While controlling a paired computer, keyboard and mouse input is captured and sent to that computer. Clipboard text, images and files are transferred when clipboard sharing is enabled. Local discovery uses mDNS; paired traffic uses authenticated TLS on the local network (default TCP port 47160). There is no relay server or cloud storage.
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

# 개인정보처리방침 - flockswitch (Windows)

[English](#privacy-policy--flockswitch-for-windows) · **한국어**

시행일: 2026년 11월 26일 · Microsoft Store 또는 [GitHub Releases](https://github.com/WWhaleFe/flockswitch-windows/releases)에서 받은 flockswitch Windows 2.0.0 이상에 적용됩니다.

## 요약

flockswitch는 개인정보를 수집하지 않으며 사용자에 관한 어떤 정보도 외부로 보내지 않습니다. 인터넷 연결은 새 버전 확인뿐이며, 끌 수 있습니다. 설정과 페어링 식별자는 이 PC에 보관합니다. 네트워크 공유를 켜면 입력과 공유 클립보드 내용은 로컬 네트워크에서 페어링한 내 컴퓨터 사이에만 전송됩니다. 받은 파일은 다운로드/flockswitch에 저장합니다.

## flockswitch가 다루는 정보

flockswitch는 이 PC에 연결된 Logitech 장치(Logi Bolt·Unifying 수신기 또는 블루투스)와 Logitech HID++ 프로토콜로 통신해, Easy-Switch 채널을 읽고 바꾸며 사용자가 켠 마우스 버튼을 사용합니다.

다음 파일을 `%APPDATA%\flockswitch\`에 보관합니다(Microsoft Store 버전은 앱 전용으로 분리된 같은 폴더).

| 파일 | 내용 |
|---|---|
| `config.toml` | 설정: 버튼, 채널, 키보드 단축키, 그룹 장치, 언어 |
| `devices.toml` | 이 PC에서 본 Logitech 장치의 제품 ID(예: "B042"), 이름(예: "MX Master 4"), 종류(마우스·키보드) - 장치가 다른 컴퓨터에 가 있어도 설정창에 표시하기 위함 |
| `button-state.toml` | flockswitch가 쓰기 전 각 마우스 버튼의 원래 상태 - 되돌려 놓기 위함 |
| `status.toml` | 현재 상태(마스터 마우스, 이 PC에 있는 장치, 최근 전환) - 설정창 표시용 |
| `flockswitch.log`, `settings.log` | 앱 동작 기록(찾은 장치, 채널 전환, 오류). 실행할 때마다 새로 씀 |

네트워크 설정에 컴퓨터 이름, 페어링한 장치 ID와 인증서 지문을 보관합니다. net/ 폴더에 이 컴퓨터의 인증서와 개인 키를 보관합니다. 받은 클립보드 파일은 다운로드/flockswitch에 저장하며 키 입력은 공유 중 전송하고 로그에 기록하지 않습니다.

## flockswitch가 하지 않는 것

- **업데이트 외 인터넷 연결 없음.** **설정 → 일반 → 업데이트 → 자동으로 새 버전 확인**이 켜져 있으면 시작할 때와 하루에 한 번(그리고 **지금 확인**을 누를 때) 새 버전을 확인합니다. Microsoft Store 버전은 Windows를 통해 Microsoft Store에 묻고, GitHub에서 받은 버전은 GitHub의 공개 릴리스 API에 https로 묻습니다. 설치할 때는 GitHub에서 새 릴리스 파일을 내려받아, 릴리스에 함께 게시된 SHA-256 목록과 맞는지 확인합니다. 이 요청에는 일반적인 웹 요청에 포함되는 정보(IP 주소, "flockswitch/<버전>" 사용자 에이전트)만 담기며, 계정·설정·장치 등 다른 정보는 보내지 않습니다. **새 버전 자동 설치**가 켜져 있으면 새 버전을 스스로 설치합니다. 리포트를 보내거나 그 밖의 서버에 연결하지 않습니다.
- **추적·분석·광고·충돌 보고 없음.**
- **공유를 켰을 때만 입력 전달.** 페어링한 컴퓨터를 제어하는 동안 키보드·마우스 입력을 받아 그 컴퓨터로 전송합니다. 클립보드 공유를 켜면 글·이미지·파일도 전송합니다. 로컬 탐색에는 mDNS를, 페어링 통신에는 로컬 네트워크의 인증된 TLS 연결(기본 TCP 포트 47160)을 사용합니다. 중계 서버나 클라우드 저장소는 없습니다.
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
