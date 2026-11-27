# Microsoft Store 목록 (2.1.0)

Partner Center에 그대로 붙여 넣는 문구와 스크린샷 순서입니다. 2.0.0 목록을 2.1.0 기준으로 업데이트했습니다.

- **핵심 변화:** 1.x는 "Logitech 장치를 버튼 하나로 함께 전환"이 전부였습니다. 2.0부터는 **로컬 네트워크로 마우스·키보드·클립보드 공유**가 첫째 기능이고, Logitech 전환은 둘째 기능입니다. 2.1은 사이드바 설정 창(크기 고정), 연결과 배치 페이지의 역할 선택과 5×3 배치 지도(서브끼리 직접 넘기기), 서브끼리 클립보드 공유, 키보드 페이지의 운영체제 쌍별 'OS 전환 시' 규칙, 연결 다시 찾기와 절전·잠금 해제 뒤 자동 재연결, 컴퓨터별 입력 보내기 스위치, Store를 통한 지금 업데이트, 대기 중 CPU·메모리 절감을 더했습니다.
- **스크린샷:** `store-shots/ko/1~6.png`, `store-shots/en/1~6.png` (1920×1080, 2.1.0 새 사이드바 설정 창, GitHub Windows 러너에서 촬영).
- **글자 수 한도:** 설명 10,000자, 이번 버전의 새로운 기능 1,500자, 제품 기능 20개×200자, 짧은 설명 1,000자(게임 외에는 표시되지 않는 곳이 많아 짧게), 검색어 7개.

## 1. 한국어 (ko-KR)

### 1-1. 설명 (Description)

```
flockswitch는 책상 위 여러 컴퓨터를 마우스·키보드 한 벌로 쓰게 해 주는 무료 오픈소스 앱입니다. 2.0부터 두 가지 방식을 함께 씁니다.

■ 네트워크 공유
같은 로컬 네트워크에 있는 내 Windows PC와 Mac 사이에서 마우스·키보드·클립보드를 공유합니다. 커서를 화면 가장자리로 밀면 옆 컴퓨터 화면으로 넘어가고, 그 컴퓨터를 이 PC의 마우스·키보드로 바로 조작합니다. 마우스·키보드 브랜드는 상관없습니다.

• 연결과 배치 페이지에서 이 컴퓨터의 역할(마스터·서브)을 고릅니다. 마스터 한 대에 서브를 최대 3대까지 연결하며, 서브는 마스터의 화면 배치와 효과 설정을 따릅니다.
• 근처 컴퓨터를 자동으로 찾아 줍니다(mDNS). 찾지 못하면 IP 주소로 추가합니다. 연결 다시 찾기 버튼으로 바로 다시 연결할 수 있고, 절전 모드에서 깨어나거나 잠금을 풀면 자동으로 다시 연결합니다.
• 짝 맺기: 두 컴퓨터에 같은 숫자가 보이는지 확인하고 양쪽에서 확인을 누릅니다. 짝을 맺은 컴퓨터끼리만 연결합니다.
• 모든 통신은 TLS 1.3으로 암호화합니다.
• 5×3 배치 지도: 마스터와 서브를 5×3 칸에 놓아 실제 책상 위 위치대로 배치합니다. 나란히 놓은 서브끼리도 서로 바로 넘어갑니다.
• 짝 맺은 컴퓨터마다 '이 컴퓨터의 마우스·키보드로 다른 컴퓨터 쓰기' 스위치를 켜거나 끌 수 있습니다(마스터에서 설정).
• 클립보드 공유: 글·이미지·파일을 복사한 채 넘어가면 커서를 따라 함께 갑니다. 서브끼리도 마스터를 거쳐 클립보드를 주고받습니다. 받은 파일은 다운로드\flockswitch 폴더에 저장되고 바로 붙여넣을 수 있습니다(파일·이미지는 100MB까지).
• 넘김 대기 시간: 가장자리에서 잠시 머물러야 넘어가게 하고, 그동안 카운트다운 효과를 보여 줍니다. 없음으로 두면 바로 넘어갑니다.
• Ctrl을 누른 채 밀 때만 넘어가게 할 수 있어, 실수로 넘어가는 일을 막습니다.
• 도착하면 넘어간 화면에서 커서 위치를 링·레이더 효과로 보여 줍니다.
• 키보드(OS 전환 시): 운영체제 쌍과 방향마다 보조키(Cmd/Ctrl 등, 기능·자리 기준과 대응표), 한/영 키, Caps Lock 길게 누르기, 한자 키, 이동·편집 키, 화면 캡처 단축키 규칙을 정합니다. 서브에서 바꿔도 마스터에 반영됩니다.
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
• 사이드바 설정 창: 네트워크(연결과 배치·넘기기·키보드·클립보드·이 컴퓨터), 로지텍(버튼·단축키·모서리 전환·장치), 앱(효과·일반) 메뉴를 크기 고정 창에서 오갑니다.
• 마우스 찾기: 단축키, 다른 컴퓨터에서 도착, 마우스 흔들기로 커서 주변에 효과를 띄워 커서를 찾습니다.
• 설정 백업: 설정을 문서\flockswitch 폴더에 저장하고 불러옵니다. Mac·Linux 버전에서 저장한 파일도 불러올 수 있습니다.
• 지금 업데이트: 새 버전이 나오면 설정 창에서 Microsoft Store를 통해 바로 설치합니다.
• 한국어 / English
• 알림 영역에서 적은 CPU와 메모리로 조용히 동작하며, 로그인할 때 자동으로 시작할 수 있습니다(관리자 권한 불필요).

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
2.1.0
• 새 설정 창: 탭 대신 왼쪽 사이드바(네트워크: 연결과 배치·넘기기·키보드·클립보드·이 컴퓨터, 로지텍: 버튼·단축키·모서리 전환·장치, 앱: 효과·일반)가 있는 크기 고정 창으로 바뀌었습니다.
• 연결과 배치: 이 페이지에서 역할(마스터·서브)을 고르고, 5×3 배치 지도에 컴퓨터를 놓습니다. 나란히 놓은 서브끼리 서로 바로 넘어갑니다.
• 컴퓨터마다 '이 컴퓨터의 마우스·키보드로 다른 컴퓨터 쓰기' 스위치를 두었습니다(마스터에서 설정).
• 연결 다시 찾기 버튼과, 절전 모드에서 깨어나거나 잠금을 풀면 자동으로 다시 연결하는 기능을 더했습니다.
• 서브끼리 클립보드 공유: 마스터를 거쳐 서브 사이에서도 글·이미지·파일 클립보드를 주고받습니다. 클립보드 페이지에서 보낼 항목, 받은 폴더와 최근 파일, 전송 상태를 확인합니다.
• 키보드 페이지와 'OS 전환 시' 규칙: 운영체제 쌍과 방향마다 보조키(Cmd/Ctrl 등, 기능·자리 기준과 대응표), 한/영 키, Caps Lock 길게 누르기, 한자 키, 이동·편집 키를 따로 정합니다. 서브에서도 바꿀 수 있으며 기존 2.0 키 설정을 그대로 이어받습니다.
• 지금 업데이트: 새 버전이 있으면 설정 창에서 Microsoft Store를 통해 바로 설치합니다.
• 마우스 흔들어 찾기 감지를 다듬고, 대기 중 CPU와 메모리 사용량을 줄였습니다.
```

### 1-4. 제품 기능 (Product features)

```
로컬 네트워크로 마우스·키보드 공유(브랜드 무관)
5×3 배치 지도와 서브끼리 직접 화면 넘기기
Windows PC와 Mac 사이 클립보드 공유(글·이미지·파일, 서브끼리 공유)
마스터 한 대에 서브 최대 3대, 컴퓨터별 입력 보내기 스위치
숫자 확인 짝 맺기와 TLS 1.3 암호화
연결 다시 찾기와 절전·잠금 해제 뒤 자동 재연결
운영체제 쌍별 'OS 전환 시' 키 규칙(Cmd/Ctrl, 한/영, 한자, 이동·편집 키)
넘김 대기 시간·카운트다운 효과와 Ctrl 누른 채 넘기기
마우스 버튼 하나로 Logitech 마우스·키보드 함께 전환
제스처·단축키·화면 가장자리로 채널 전환
Bolt·Unifying·블루투스 장치 함께 이동
MX Master 4 진동 알림
마우스 찾기 효과
크기 고정 사이드바 설정 창과 Store 지금 업데이트
설정 백업(Mac·Linux 버전과 호환)
적은 대기 CPU·메모리 사용, 계정·광고·사용 기록 수집 없음
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

■ Network sharing
Share your mouse, keyboard and clipboard between your Windows PCs and Macs on the same local network. Push the cursor off the edge of the screen and it lands on the computer next to it, which you then control with this PC's mouse and keyboard. Any brand of mouse and keyboard works.

• Choose this computer's role (master or sub) on the Connection and layout page. One master connects with up to three subs, and subs follow the master's screen layout and effect settings.
• Finds nearby computers automatically (mDNS), or add one by IP address. Use Find connections again to reconnect at once, with automatic reconnecting after sleep or unlock.
• Pairing: check that both computers show the same number and confirm on both. Only paired computers connect.
• All traffic is encrypted with TLS 1.3.
• 5x3 layout map: place the master and subs on a 5x3 grid to match your desk. Subs placed next to each other cross directly from sub to sub.
• Per-computer switch to allow or block using other computers with that computer's mouse and keyboard (set on the master).
• Clipboard sharing: copy text, an image or files, cross over, and it goes along with the cursor. Subs also share the clipboard with each other through the master. Received files are saved in Downloads\flockswitch, ready to paste (files and images up to 100 MB).
• Crossing delay: make the cursor rest at the edge for a moment before it crosses, with a countdown effect while it waits, or cross straight away.
• Cross only while holding Ctrl, so you never cross by accident.
• On arrival, a ring or radar effect shows where the cursor is.
• Keyboard (When switching OS): set rules per OS pair and direction for modifier keys (Cmd/Ctrl and others, by function or position with an editable table), Korean/English keys, Caps Lock long press, Hanja key, navigation and edit keys, and screen capture shortcuts. Rules can be edited on a sub as well.
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
• Sidebar settings window: fixed-size window with a sidebar for Network (Connection and layout, Crossing, Keyboard, Clipboard, This computer), Logitech (Button, Shortcut, Edge switching, Devices), and App (Effects, General).
• Find the cursor: show an effect around the cursor from a shortcut, on arrival or when you shake the mouse.
• Settings backup: save settings to Documents\flockswitch and load them again, including files saved by the Mac and Linux versions.
• Update now: install new versions straight through the Microsoft Store from the settings window.
• English / Korean
• Runs quietly in the notification area with low idle CPU and memory use, and can start when you sign in (no administrator rights needed).

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
2.1.0
• New settings window: fixed-size window with a sidebar (Network: Connection and layout, Crossing, Keyboard, Clipboard, This computer; Logitech: Button, Shortcut, Edge switching, Devices; App: Effects, General) instead of tabs.
• Connection and layout: choose this computer's role (master or sub) and place computers on a 5x3 layout map. Subs placed side by side cross directly to each other.
• Per-computer switch to use other computers with this computer's mouse and keyboard (set on the master).
• Find connections again button, plus automatic reconnecting after waking from sleep or unlocking the session.
• Sub-to-sub clipboard sharing: share text, images and files between subs through the master, with controls for what is sent, the received folder, recent file and transfer status on the Clipboard page.
• Keyboard page with 'When switching OS' rules per OS pair and direction: modifier matching (Cmd/Ctrl and others by function or position with an editable table), Korean/English keys, Caps Lock long press, Hanja key, and navigation and edit keys. Editable from subs too, and existing 2.0 key settings carry over.
• Update now: install a newer version straight through the Microsoft Store when one is available.
• Improved mouse-shake detection for finding the cursor, and lower idle CPU and memory use.
```

### 2-4. Product features

```
Share mouse and keyboard over your local network (any brand)
5x3 layout map with direct sub-to-sub screen crossing
Clipboard sharing between Windows and Mac (text, images, files, sub-to-sub)
One master with up to three subs and a per-computer input switch
Number-check pairing and TLS 1.3 encryption
Find connections again and automatic reconnect after sleep or unlock
Per-OS-pair key rules (Cmd/Ctrl, Korean/English, Hanja, navigation and edit keys)
Crossing delay with a countdown effect and cross only while holding Ctrl
Move a Logitech mouse and keyboard together with one button
Switch channels by gesture, shortcut or screen edge
Bolt, Unifying and Bluetooth devices move along
Haptic feedback on the MX Master 4
Find-the-cursor effects
Fixed-size sidebar settings window and Store Update now
Settings backup, compatible with the Mac and Linux versions
Low idle CPU and memory use; no account, no ads, no tracking
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
| 1 | 연결과 배치: 5×3 배치도에 놓은 컴퓨터 4대 | 내 컴퓨터를 책상처럼 놓고 마우스·키보드 공유 | Place your computers like your desk and share one mouse and keyboard |
| 2 | 넘기기: 모서리와 넘어가는 조건 | 화면 모서리로 다른 컴퓨터에 넘어가기 | Cross to another computer at the screen edge |
| 3 | 키보드: OS 전환 시 규칙 (macOS → Windows) | 운영체제가 달라도 단축키와 한/영 키가 그대로 | Shortcuts and the Korean/English key keep working across systems |
| 4 | 클립보드: 서브끼리 공유와 받은 파일 | 서브끼리도 글·이미지·파일 클립보드 공유 | Clipboard text, images and files, even between subs |
| 5 | 로지텍 버튼 전환 | 마우스 버튼 하나로 Logitech 장치를 함께 전환 | Move Logitech devices together with one button |
| 6 | 일반: 업데이트·백업 | Microsoft Store로 바로 업데이트, 설정 백업 | Update through the Microsoft Store, back up settings |

## 4. 적용 순서

1. Partner Center → 앱 **Flockswitch** → 2.1.0 패키지가 든 제출이 게시됐는지 확인합니다. 아직 인증 중이면 끝난 뒤에 합니다.
2. **Start update**(업데이트 시작)로 새 제출을 만듭니다. 패키지는 그대로 둡니다(목록만 바꾸는 제출도 인증을 거칩니다).
3. Store listings → **한국어**: 1-1~1-4를 붙여 넣고, 기존 스크린샷을 모두 지운 뒤 `store-shots/ko/1~6.png`를 순서대로 올리고 3장의 캡션을 넣습니다.
4. Store listings → **English**: 2-1~2-4와 `store-shots/en/1~6.png`를 같은 방식으로 넣습니다.
5. 검색어(1-5 / 2-5)는 Partner Center를 영어(/en-us/)로 열어 넣습니다. 칩 입력란 바깥을 클릭한 뒤 저장하고, "Additional information"을 펼쳐 칩이 남아 있는지 확인합니다.
6. 저장 → **Submit for certification**.
7. 게시되면 https://apps.microsoft.com/detail/9NN9ZLHLGSKN 에서 설명과 스크린샷을 확인합니다.
