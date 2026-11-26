# Microsoft Store 목록 (2.0.0)

Partner Center에 그대로 붙여 넣는 문구와 스크린샷 순서입니다. 1.7.0 개편안을 2.0 기준으로 전부 바꿨습니다.

- **핵심 변화:** 1.x는 "Logitech 장치를 버튼 하나로 함께 전환"이 전부였습니다. 2.0은 **로컬 네트워크로 마우스·키보드·클립보드 공유**가 첫째 기능이고, Logitech 전환은 둘째 기능입니다. 설명도 이 순서로 씁니다.
- **스크린샷:** `store-shots/ko/1~6.png`, `store-shots/en/1~6.png` (1920×1080, 2.0.0 실제 설정 창, 이 저장소의 GitHub Windows 러너에서 촬영).
- **글자 수 한도:** 설명 10,000자, 이번 버전의 새로운 기능 1,500자, 제품 기능 20개×200자, 짧은 설명 1,000자(게임 외에는 표시되지 않는 곳이 많아 짧게), 검색어 7개.

## 1. 한국어 (ko-KR)

### 1-1. 설명 (Description)

```
flockswitch는 책상 위 여러 컴퓨터를 마우스·키보드 한 벌로 쓰게 해 주는 무료 오픈소스 앱입니다. 2.0부터 두 가지 방식을 함께 씁니다.

■ 네트워크 공유 (2.0 새 기능)
같은 로컬 네트워크에 있는 내 Windows PC와 Mac 사이에서 마우스·키보드·클립보드를 공유합니다. 커서를 화면 가장자리로 밀면 옆 컴퓨터 화면으로 넘어가고, 그 컴퓨터를 이 PC의 마우스·키보드로 바로 조작합니다. 마우스·키보드 브랜드는 상관없습니다.

• 마스터 한 대에 서브를 최대 3대까지 연결합니다. 서브는 마스터의 화면 배치와 효과 설정을 따릅니다.
• 근처 컴퓨터를 자동으로 찾아 줍니다(mDNS). 찾지 못하면 IP 주소로 추가합니다.
• 짝 맺기: 두 컴퓨터에 같은 숫자가 보이는지 확인하고 양쪽에서 확인을 누릅니다. 짝을 맺은 컴퓨터끼리만 연결합니다.
• 모든 통신은 TLS 1.3으로 암호화합니다.
• 화면 배치: 서브가 이 화면의 왼쪽·오른쪽·위·아래 어디에 있는지 정합니다.
• 클립보드 공유: 글·이미지·파일을 복사한 채 넘어가면 커서를 따라 함께 갑니다. 받은 파일은 다운로드\flockswitch 폴더에 저장되고 바로 붙여넣을 수 있습니다(파일·이미지는 100MB까지).
• 넘김 대기 시간: 가장자리에서 잠시 머물러야 넘어가게 하고, 그동안 카운트다운 효과를 보여 줍니다. 없음으로 두면 바로 넘어갑니다.
• Ctrl을 누른 채 밀 때만 넘어가게 할 수 있어, 실수로 넘어가는 일을 막습니다.
• 도착하면 넘어간 화면에서 커서 위치를 링·레이더 효과로 보여 줍니다.
• Mac과 Windows 사이에서 Cmd와 Ctrl을 바꿔 줍니다. Mac의 Caps Lock·Shift+Space를 Windows 한/영 키로 보낼 수 있습니다(Mac에서 설정).
• 어디서든 Ctrl+Alt+Shift+Q로 이 PC로 돌아옵니다.

■ Logitech Easy-Switch 함께 전환
Logitech 마우스와 키보드는 Easy-Switch 채널(1·2·3)마다 컴퓨터를 하나씩 기억합니다. 원래는 키보드와 마우스의 채널 버튼을 기기마다 따로 눌러야 하지만, flockswitch는 아래 방법 중 하나로 마우스와 키보드를 한 번에 같은 채널로 옮깁니다. 이 방식은 컴퓨터끼리 네트워크 연결이 필요 없어서, 컴퓨터를 네트워크로 묶을 수 없는 곳에서도 동작합니다.

• 마우스 버튼: MX Master의 휠 모드 전환 버튼, 제스처 버튼, 휠 클릭 중 고른 버튼을 전환 버튼으로 바꿉니다. 버튼마다 이동할 채널을 정합니다.
• 제스처: 제스처 버튼을 누르기, 누른 채 위·아래·왼쪽·오른쪽으로 움직이기, 다섯 가지에 각각 채널을 정합니다.
• 키보드 단축키: 채널마다 단축키 하나(기본 Ctrl+Alt+1·2·3).
• 화면 가장자리: 커서를 가장자리에 잠시 대면 그 방향에 정한 채널로 이동합니다. 기다리는 동안 링·반원·막대·펄스·레이저·보텍스·스캔·리퀴드 효과 중 하나가 줄어들며 남은 시간을 보여 줍니다.
• 함께 이동할 장치: Bolt·Unifying 수신기와 블루투스 장치를 섞어 고릅니다.
• 진동: MX Master 4처럼 진동 모터가 있는 마우스는 떠날 때·돌아왔을 때·가장자리에서 진동으로 알려 줍니다.
• 돌아오면 휠 모드 되돌리기: 마우스가 돌아오면 이 PC에서 쓰던 휠 모드(걸림·무한 회전)로 되돌립니다.

■ 함께 쓰면 편한 기능
• 마우스 찾기: 단축키, 다른 컴퓨터에서 도착, 마우스 흔들기로 커서 주변에 효과를 띄워 커서를 찾습니다.
• 설정 백업: 설정을 문서\flockswitch 폴더에 저장하고 불러옵니다. Mac·Linux 버전에서 저장한 파일도 불러올 수 있습니다.
• 한국어 / English
• 알림 영역에서 조용히 동작하며, 로그인할 때 자동으로 시작할 수 있습니다(관리자 권한 불필요).

■ 이런 분께 맞습니다
• Windows PC와 Mac을 나란히 두고 쓰는 분
• 책상 위 키보드·마우스를 한 벌로 줄이고 싶은 분
• 한 컴퓨터에서 복사한 글·이미지·파일을 다른 컴퓨터에 바로 붙여넣고 싶은 분
• MX Master 마우스와 Logitech 키보드를 여러 컴퓨터에 페어링해 둔 분

■ 필요한 것
• Windows 10 또는 11
• 네트워크 공유: 같은 로컬 네트워크에 연결된 컴퓨터 2대 이상, 각 컴퓨터에 flockswitch(Mac은 flockswitch-mac, GitHub에서 무료). Windows 방화벽에서 개인 네트워크 연결을 허용해야 합니다.
• Logitech 전환: 휠 모드 전환 버튼·제스처 버튼·휠 클릭이 있는 Logitech MX Master 마우스(MX Master 3·4에서 시험). 각 컴퓨터에 키보드와 마우스를 같은 채널 번호로 페어링해 두세요. Logi Options+와 함께 쓸 수 있으며, 쓸 버튼은 Options+에서 "동작 없음"으로 바꿔 두세요.

■ 알아 두세요
• UAC(사용자 계정 컨트롤) 화면과 잠금 화면에서는 입력을 넘기지 못합니다. 관리자 권한으로 실행한 창에 입력하려면 flockswitch도 관리자 권한으로 실행해야 합니다.
• 네트워크 공유는 같은 로컬 네트워크 안에서만 동작합니다. 인터넷 너머의 컴퓨터와는 연결하지 않습니다.
• Logitech 가장자리 전환을 켜면 그동안 네트워크 넘기기가 일시 정지됩니다. 두 기능 중 하나만 화면 가장자리를 맡습니다.

■ 개인정보
• 계정이 없고, 광고와 사용 기록 수집이 없습니다.
• 이 PC의 Logitech 장치, 그리고 사용자가 짝을 맺은 로컬 네트워크의 내 컴퓨터와만 통신합니다. 외부 서버로 입력이나 클립보드를 보내지 않습니다.
• 인터넷 연결은 Microsoft Store에 새 버전을 묻는 것뿐이며, 설정에서 끌 수 있습니다.

flockswitch는 Logitech과 관계없는 개인 오픈소스 프로젝트입니다(MIT 라이선스). Logitech, MX Master, Easy-Switch, Logi Bolt, Unifying, Logi Options+는 Logitech의 상표입니다. Mac, macOS는 Apple Inc.의 상표입니다.
```

### 1-2. 짧은 설명 (Short description)

```
로컬 네트워크로 Windows PC와 Mac 사이에서 마우스·키보드·클립보드를 공유하고, Logitech 마우스와 키보드를 버튼 하나로 함께 다른 컴퓨터로 전환합니다.
```

### 1-3. 이번 버전의 새로운 기능 (What's new in this version)

```
2.0.0
• 네트워크 공유: 같은 로컬 네트워크의 Windows PC와 Mac 사이에서 마우스·키보드·클립보드(글·이미지·파일)를 공유합니다. 마우스·키보드 브랜드는 상관없습니다.
• 마스터 한 대에 서브 최대 3대. 자동 찾기 또는 IP로 추가, 숫자 확인으로 짝 맺기, TLS 1.3 암호화.
• 화면 배치, 넘김 대기 시간과 카운트다운 효과, Ctrl을 누른 채 밀 때만 넘기기, 도착 시 커서 위치 표시.
• Mac↔Windows Cmd/Ctrl 바꾸기, Mac Caps Lock·Shift+Space로 Windows 한/영 전환(Mac에서 설정).
• 설정 창을 네트워크·전환·장치·효과·일반 탭으로 새로 구성했습니다.
• Logitech 버튼·단축키·가장자리 전환, 진동, 마우스 찾기, 설정 백업은 그대로입니다. 1.x 설정을 그대로 이어받습니다.
```

### 1-4. 제품 기능 (Product features)

```
로컬 네트워크로 마우스·키보드 공유(브랜드 무관)
Windows PC와 Mac 사이 클립보드 공유(글·이미지·파일)
마스터 한 대에 서브 최대 3대
숫자 확인 짝 맺기와 TLS 1.3 암호화
화면 배치와 넘김 대기 시간·카운트다운 효과
Mac↔Windows Cmd/Ctrl 바꾸기와 한/영 전환
마우스 버튼 하나로 Logitech 마우스·키보드 함께 전환
제스처·단축키·화면 가장자리로 채널 전환
Bolt·Unifying·블루투스 장치 함께 이동
MX Master 4 진동 알림
마우스 찾기 효과
설정 백업(Mac·Linux 버전과 호환)
계정·광고·사용 기록 수집 없음
```

### 1-5. 검색어 (Search terms, 최대 7개)

```
마우스 키보드 공유
클립보드 공유
KVM
Easy-Switch
MX Master
멀티 컴퓨터
Mac Windows
```

## 2. English (en-US)

### 2-1. Description

```
flockswitch lets you run several computers on your desk from one mouse and keyboard. It is free and open source. From 2.0 it works two ways, and you can use both.

■ Network sharing (new in 2.0)
Share your mouse, keyboard and clipboard between your Windows PCs and Macs on the same local network. Push the cursor off the edge of the screen and it lands on the computer next to it, which you then control with this PC's mouse and keyboard. Any brand of mouse and keyboard works.

• One master with up to three subs. Subs follow the master's screen layout and effect settings.
• Finds nearby computers automatically (mDNS), or add one by IP address.
• Pairing: check that both computers show the same number and confirm on both. Only paired computers connect.
• All traffic is encrypted with TLS 1.3.
• Screen layout: set whether each sub sits left, right, above or below this screen.
• Clipboard sharing: copy text, an image or files, cross over, and it goes along with the cursor. Received files are saved in Downloads\flockswitch, ready to paste (files and images up to 100 MB).
• Crossing delay: make the cursor rest at the edge for a moment before it crosses, with a countdown effect while it waits, or cross straight away.
• Cross only while holding Ctrl, so you never cross by accident.
• On arrival, a ring or radar effect shows where the cursor is.
• Swaps Cmd and Ctrl between Mac and Windows. The Mac's Caps Lock or Shift+Space can act as the Windows Korean/English key (set on the Mac).
• Ctrl+Alt+Shift+Q brings you back to this PC from anywhere.

■ Logitech Easy-Switch, all devices at once
Logitech mice and keyboards remember one computer on each Easy-Switch channel (1, 2, 3). Normally you press the channel button on the keyboard and the one under the mouse separately. flockswitch moves the mouse and keyboard to the same channel in one go, in any of the ways below. This needs no network between the computers, so it also works where they cannot be linked over a network.

• Mouse button: turn the wheel mode button, gesture button or wheel click of an MX Master into a switch button, each with its own channel.
• Gestures: give a channel to a press of the gesture button and to holding it while moving up, down, left or right.
• Keyboard shortcuts: one per channel (Ctrl+Alt+1, 2, 3 by default).
• Screen edge: rest the cursor on an edge to move to the channel set for that side. A ring, semicircle, bar, pulse, laser, vortex, scan or liquid effect counts down while it waits.
• Devices that move along: mix Bolt, Unifying and Bluetooth devices.
• Haptics: mice with a haptic motor, such as the MX Master 4, pulse when they leave, arrive and wait at an edge.
• Wheel mode restored: when the mouse comes back, it returns to the wheel mode (ratchet or free spin) you used on this PC.

■ Also included
• Find the cursor: show an effect around the cursor from a shortcut, on arrival or when you shake the mouse.
• Settings backup: save settings to Documents\flockswitch and load them again, including files saved by the Mac and Linux versions.
• English / Korean
• Runs quietly in the notification area and can start when you sign in (no administrator rights needed).

■ Made for
• People who use a Windows PC and a Mac side by side
• Anyone who wants one keyboard and mouse on the desk instead of two
• Copying text, images or files on one computer and pasting them on another
• MX Master owners with their mouse and Logitech keyboard paired to several computers

■ What you need
• Windows 10 or 11
• Network sharing: two or more computers on the same local network, each running flockswitch (on a Mac, flockswitch-mac, free on GitHub). Allow private network connections in Windows Firewall.
• Logitech switching: a Logitech MX Master mouse with a wheel mode button, gesture button or wheel click (tested with MX Master 3 and 4). Pair the keyboard and mouse to each computer on the same channel number. Works alongside Logi Options+; set the buttons you use to "No action" there.

■ Good to know
• Input cannot cross on UAC prompts or the lock screen. To type into a window running as administrator, run flockswitch as administrator too.
• Network sharing works only within the same local network; it does not connect to computers over the internet.
• While Logitech edge switching is on, network crossing pauses. Only one of the two takes the screen edges.

■ Privacy
• No account, no ads, no usage tracking.
• It talks only to this PC's Logitech devices and to computers you paired on your local network. Input and clipboard never go to an outside server.
• Its only internet use is asking the Microsoft Store for a new version, which you can turn off in the settings.

flockswitch is a personal open-source project, not affiliated with Logitech (MIT license). Logitech, MX Master, Easy-Switch, Logi Bolt, Unifying and Logi Options+ are trademarks of Logitech. Mac and macOS are trademarks of Apple Inc.
```

### 2-2. Short description

```
Share your mouse, keyboard and clipboard between Windows PCs and Macs over your local network, and move a Logitech mouse and keyboard to another computer together with one button.
```

### 2-3. What's new in this version

```
2.0.0
• Network sharing: share mouse, keyboard and clipboard (text, images, files) between Windows PCs and Macs on the same local network. Any brand of mouse and keyboard.
• One master with up to three subs. Automatic discovery or add by IP, number-check pairing, TLS 1.3 encryption.
• Screen layout, crossing delay with a countdown effect, cross only while holding Ctrl, cursor shown on arrival.
• Cmd/Ctrl swap between Mac and Windows; Mac Caps Lock or Shift+Space as the Windows Korean/English key (set on the Mac).
• A new settings window with Network, Switching, Devices, Effects and General tabs.
• Logitech button, shortcut and edge switching, haptics, find the cursor and settings backup are all still here. Your 1.x settings carry over.
```

### 2-4. Product features

```
Share mouse and keyboard over your local network (any brand)
Clipboard sharing between Windows and Mac (text, images, files)
One master with up to three subs
Number-check pairing and TLS 1.3 encryption
Screen layout and crossing delay with a countdown effect
Cmd/Ctrl swap and Korean/English key between Mac and Windows
Move a Logitech mouse and keyboard together with one button
Switch channels by gesture, shortcut or screen edge
Bolt, Unifying and Bluetooth devices move along
Haptic feedback on the MX Master 4
Find-the-cursor effects
Settings backup, compatible with the Mac and Linux versions
No account, no ads, no tracking
```

### 2-5. Search terms

```
mouse keyboard sharing
clipboard sharing
KVM
Easy-Switch
MX Master
multiple computers
Mac Windows
```

## 3. 스크린샷과 캡션

`store-shots/ko/`는 한국어 목록에, `store-shots/en/`은 영어 목록에 올립니다. 순서대로 올리고 캡션을 넣습니다.

| # | 화면 | 캡션(ko-KR) | Caption (en-US) |
|---|---|---|---|
| 1 | 네트워크 탭: 짝 맺은 Mac·노트북과 화면 배치 | 로컬 네트워크로 마우스·키보드·클립보드 공유 | Share mouse, keyboard and clipboard over your network |
| 2 | 전환 탭: 버튼별 이동할 채널 | 마우스 버튼 하나로 Logitech 장치를 함께 전환 | Move Logitech devices together with one button |
| 3 | 장치 탭: 마스터 마우스와 함께 이동할 장치 | 키보드와 다른 장치도 함께 이동 | Your keyboard and other devices move along |
| 4 | 효과 탭: 대기·도착 효과 | 넘어가기 전 카운트다운과 도착 표시 | Countdown before crossing, a marker on arrival |
| 5 | 일반 탭: 진동·마우스 찾기·업데이트·백업 | 진동, 마우스 찾기, 설정 백업 | Haptics, find the cursor, settings backup |

## 4. 적용 순서

1. Partner Center → 앱 **Flockswitch** → 2.0.0 패키지가 든 제출이 게시됐는지 확인합니다. 아직 인증 중이면 끝난 뒤에 합니다.
2. **Start update**(업데이트 시작)로 새 제출을 만듭니다. 패키지는 그대로 둡니다(목록만 바꾸는 제출도 인증을 거칩니다).
3. Store listings → **한국어**: 1-1~1-4를 붙여 넣고, 기존 스크린샷을 모두 지운 뒤 `store-shots/ko/1~5.png`를 순서대로 올리고 3장의 캡션을 넣습니다.
4. Store listings → **English**: 2-1~2-4와 `store-shots/en/1~5.png`를 같은 방식으로 넣습니다.
5. 검색어(1-5 / 2-5)는 Partner Center를 영어(/en-us/)로 열어 넣습니다. 칩 입력란 바깥을 클릭한 뒤 저장하고, "Additional information"을 펼쳐 칩이 남아 있는지 확인합니다.
6. 저장 → **Submit for certification**.
7. 게시되면 https://apps.microsoft.com/detail/9NN9ZLHLGSKN 에서 설명과 스크린샷을 확인합니다.
