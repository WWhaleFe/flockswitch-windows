# Microsoft Store 등록 안내

flockswitch Windows 버전을 Microsoft Store에 MSIX 패키지로 올리는 순서와, Partner Center에 그대로 붙여 넣을 문구입니다.

- **패키지 형식: MSIX.** Store가 인증 후 직접 서명하므로 코드 서명 인증서를 살 필요가 없습니다. (MSI/EXE로 올리면 신뢰된 CA 인증서가 필요합니다.)
- **같은 프로그램:** GitHub 릴리스의 exe와 같은 빌드를 패키지에 넣습니다. 앱이 패키지 안에서 실행되는지 스스로 알아채고, 그때만 다르게 동작합니다.
  - **로그인 시 자동 실행:** Run 레지스트리 대신 패키지의 시작 작업을 씁니다. 처음 실행한 뒤부터 켜져 있고, Windows 설정 → 앱 → 시작 프로그램에서 켜고 끕니다. 설정 창의 해당 줄에는 그 화면을 여는 버튼이 있습니다.
  - **제거 안내 문구:** 시작 메뉴에서 제거하라고 안내합니다.
  - **설정 파일:** Windows가 앱 전용으로 분리해 두고, 앱을 제거하면 함께 지웁니다.
- **명령줄:** 패키지에는 `flockswitch` 명령 별칭이 있어 터미널에서 `flockswitch list`를 쓸 수 있습니다.

## 1. Partner Center에서 이름 예약

1. [Partner Center](https://partner.microsoft.com/dashboard)에 개발자 계정으로 로그인합니다. 개인 개발자 계정은 무료입니다.
2. **앱 및 게임 → 새 제품 → MSIX 또는 PWA 앱**을 고르고 이름을 예약합니다. **Flockswitch**로 예약해 두었고, 패키지의 표시 이름(DisplayName)도 정확히 이 이름을 씁니다.
3. **제품 관리 → 제품 ID**에서 아래 세 값을 복사합니다. 대소문자까지 정확히 복사해야 합니다.
   - `Package/Identity/Name` (예: `12345WWhaleFe.flockswitch`)
   - `Package/Identity/Publisher` (예: `CN=XXXXXXXX-XXXX-...`)
   - `Package/Properties/PublisherDisplayName`

## 2. 패키지 ID를 GitHub에 넣기

flockswitch-windows 저장소 → **Settings → Secrets and variables → Actions → Variables**에 아래 세 변수를 만듭니다. 값을 알려 주시면 대신 넣어 드릴 수도 있습니다.

| 변수 | 값 |
|---|---|
| `STORE_IDENTITY_NAME` | Package/Identity/Name |
| `STORE_PUBLISHER` | Package/Identity/Publisher |
| `STORE_PUBLISHER_DISPLAY_NAME` | PublisherDisplayName |

변수가 없으면 시험용 ID로 만들어집니다. 그 패키지는 PC에서 시험해 볼 수는 있지만 Store에는 올릴 수 없습니다.

## 3. 버전

Store 패키지 버전은 네 자리이고, **첫 자리가 0이면 안 되며** 마지막 자리는 0이어야 합니다. 그래서 Cargo.toml의 `0.4.x`는 Store에 올릴 수 없습니다. 첫 Store 버전은 **1.0.0**(패키지 `1.0.0.0`)으로 올리는 것을 권합니다. `make-msix.ps1`은 첫 자리가 0이면 경고합니다.

## 4. 패키지 만들기

- **GitHub에서:** 버전 태그(`v*`)를 푸시하거나 **Actions → Store package → Run workflow**를 실행합니다. 결과물(artifact)은 두 가지입니다.
  - `flockswitch-store-msix`: Partner Center에 올릴 서명 안 된 `.msix`
  - `flockswitch-store-test-signed`: 시험 인증서로 서명한 `.msix`와 `flockswitch-test.cer`. 내 PC에서 설치해 보는 용도입니다.
- **Windows PC에서 직접:** Windows SDK와 Rust가 필요합니다.
  ```powershell
  cargo build --release
  packaging\windows\store\make-msix.ps1 -Exe target\release\flockswitch.exe -OutDir out\store `
      -IdentityName "<Name>" -Publisher "<CN=...>" -PublisherDisplayName "<표시 이름>"
  ```

워크플로는 시험 서명한 패키지를 실제로 설치하고, 패키지 안에서 `flockswitch list`가 실행되는지까지 확인합니다.

## 5. 올리기 전에 내 PC에서 시험

1. `flockswitch-store-test-signed`를 받아 압축을 풉니다.
2. 관리자 PowerShell에서 시험 인증서를 신뢰 목록에 넣습니다.
   `Import-Certificate -FilePath flockswitch-test.cer -CertStoreLocation Cert:\LocalMachine\TrustedPeople`
3. `.msix`를 더블클릭해 설치합니다. GitHub 릴리스판 flockswitch가 실행 중이면 먼저 종료하세요.
4. 확인할 것:
   - 처음 실행하면 설정 창 없이 알림 영역 아이콘만 나타나는지 (아이콘 메뉴의 "설정…"으로 열림)
   - 마우스 버튼으로 전환되는지
   - 트레이 메뉴가 나오는지
   - 설정 → 일반의 "시작 프로그램 설정" 버튼이 동작하는지
   - 다시 로그인했을 때 자동 실행되는지
   - 제거 후 설정이 지워지는지
5. 선택: Windows SDK에 들어 있는 **Windows App Certification Kit**으로 미리 검사합니다. 관리자 권한으로 실행하세요.
   `appcert.exe test -appxpackagepath <파일>.msix -reportoutputpath report.xml`
6. 시험이 끝나면 시험 인증서를 지웁니다(인증서 관리자 → 신뢰할 수 있는 사용자).

## 6. 제출 항목

새 제출에서 아래를 채웁니다.

| 항목 | 내용 |
|---|---|
| 가격 및 사용 가능 여부 | 무료, 모든 시장 |
| 속성 → 범주 | 유틸리티 및 도구 (Utilities & tools) |
| 속성 → 시스템 요구 사항 | 최소 하드웨어: 마우스. 비고: "Logitech MX Master 3 or 4 (Logi Bolt, Unifying or Bluetooth)" |
| 연령 등급 | IARC 설문. 폭력·성적 내용·도박 없음, 사용자 간 소통·공유 없음, 위치 정보·개인정보 수집 없음, 앱 내 구매 없음, 인터넷 연결 없음 → 보통 전체 이용가 |
| 패키지 | `flockswitch-<버전>-x64.msix` 업로드 |
| 스토어 목록 | 한국어(ko-KR)와 영어(en-US) 둘 다. 패키지가 두 언어를 선언하므로 두 목록이 모두 필요합니다 (아래 문구) |
| 개인정보 처리방침 URL | `https://github.com/WWhaleFe/flockswitch-windows/blob/main/PRIVACY.md` |
| 지원 연락처 | `https://github.com/WWhaleFe/flockswitch-windows/issues` |
| 제출 옵션 → 제한된 기능 | runFullTrust 사유 (아래 문구) |
| 제출 옵션 → 인증 참고 사항 | 아래 문구 |

### 스크린샷

데스크톱용으로 최소 1장, 권장 4장 이상이 필요합니다. PNG, **1366×768 이상**이어야 합니다. 설정 창(520×720)만 잘라 내면 너무 작으니, 바탕화면 위에 창을 띄운 화면 전체를 찍습니다(Win+Shift+S → 전체 화면).

1. 전환 탭: 버튼별 설정과 이동할 채널(이 PC 채널이 회색으로 잠긴 모습)
2. 전환 탭: 제스처 버튼(↑↓←→)
3. 장치 탭: 마스터 마우스와 함께 이동할 장치
4. 트레이 메뉴
5. (선택) 단축키 탭

## 7. 붙여 넣을 문구

### 짧은 설명 (ko-KR)

마우스 버튼 한 번으로 Logitech 마우스와 키보드를 함께 다른 컴퓨터로 넘깁니다.

### 설명 (ko-KR)

flockswitch는 Logitech MX Master 마우스의 휠 모드 전환 버튼, 제스처 버튼, 휠 클릭 중 고른 버튼을 Easy-Switch 버튼으로 바꿉니다. 버튼을 누르면 함께 묶은 키보드와 다른 마우스가 먼저, 마우스가 마지막으로 같은 Easy-Switch 채널로 이동합니다.

컴퓨터끼리 네트워크로 연결할 필요가 없어서 Logitech Flow를 쓸 수 없는 공용·보안 네트워크에서도 동작합니다. 오가는 모든 컴퓨터에 flockswitch를 설치하세요(macOS 버전도 있습니다).

• 버튼마다 이동할 채널 지정. 이 PC의 채널은 자동으로 잠김
• 제스처 버튼: 누른 채 위·아래·왼쪽·오른쪽으로 움직여 채널 선택
• 키보드 단축키로도 전환
• Logi Bolt, Unifying, 블루투스 장치를 섞어서 함께 이동
• Logi Options+와 함께 사용. 끄면 버튼이 원래대로 돌아감
• 인터넷 연결 없음, 계정 없음, 데이터 수집 없음

필요: Logitech MX Master 3 또는 MX Master 4 (Logi Bolt 수신기, Unifying 수신기 또는 블루투스). 다른 Logitech 마우스는 아직 시험하지 않았습니다.

Logitech과 관계없는 개인 오픈소스입니다. Logitech, MX Master, Easy-Switch, Bolt, Unifying은 Logitech의 상표입니다.

### Short description (en-US)

Press one mouse button to move your Logitech mouse and keyboard to another computer together.

### Description (en-US)

flockswitch turns a button of your Logitech MX Master mouse — the wheel mode-shift button, the gesture button or the wheel click — into an Easy-Switch key. Press it and your keyboard and other devices switch to the same Easy-Switch channel, then the mouse follows.

It needs no network between the computers, so it works where Logitech Flow cannot (guest or locked-down networks). Install flockswitch on every computer you switch between (a macOS version is available too).

• Choose the channel each button goes to; this PC's own channel is locked automatically
• Gesture button: hold it and move up, down, left or right to pick a channel
• Keyboard shortcuts switch too
• Mix Logi Bolt, Unifying and Bluetooth devices in one group
• Runs alongside Logi Options+; the button goes back to normal when flockswitch is off
• No internet connection, no account, no data collected

Requires a Logitech MX Master 3 or MX Master 4 (Logi Bolt receiver, Unifying receiver or Bluetooth). Other Logitech mice have not been tested yet.

Not affiliated with Logitech. Logitech, MX Master, Easy-Switch, Bolt and Unifying are trademarks of Logitech.

### 검색어 (최대 7개)

Logitech, Easy-Switch, MX Master, mouse, keyboard, KVM, multi computer

### runFullTrust 사유 (영어로 입력)

flockswitch is a desktop utility for Logitech mice. It needs full trust to: open the Logitech receiver and Bluetooth HID devices and exchange HID++ reports with them (to catch the chosen mouse button and send the Easy-Switch "change host" command); show a notification-area (tray) icon and menu; register global keyboard shortcuts; and start its own settings window as a second process. It makes no network connections and writes only to its own settings folder.

### 인증 참고 사항 (영어로 입력)

flockswitch works with a Logitech MX Master 3 or MX Master 4 mouse connected through a Logi Bolt or Unifying receiver or Bluetooth; other hardware is not used.

The app starts quietly as an icon in the notification area (it may be in the hidden icons, under the ^ arrow); no window opens on launch or at login. Without such a mouse: hovering the icon and the first line of its menu say "Looking for a supported mouse (Logitech MX Master 3 or 4)…". The menu's "Settings…" opens the settings window (starting the app again opens it too), whose header says the same and whose Devices tab says that no mouse with the button was found yet. The menu also quits the app. All settings can be changed and are saved, but nothing switches without the mouse.

With the mouse: turn on a button in Settings → Switching and choose a channel in "Go to channel"; pressing that mouse button sends the mouse (and any devices chosen in the Devices tab) to that Easy-Switch channel, i.e. to another computer.

The app starts at login through its startup task (switchable in Settings → Apps → Startup). It makes no internet connections and needs no account. Source code: https://github.com/WWhaleFe/flockswitch-windows
