# Microsoft Store 목록 개편안 (1.7.0)

작성: 2026-11-10 · 대상: Partner Center → Flockswitch(9NN9ZLHLGSKN) → Store listings (ko-KR, en-US)
기준 화면: Windows 1.7.0. 설명은 1.7.0 화면의 메뉴·탭·문구와 기본값을 그대로 따릅니다(코드로 확인).
적용: 1.7.0 패키지, 새 설명, 새 스크린샷을 한 제출로 올립니다.

---

## 1. 요약

"Store 설명이 빈약하다"는 피드백을 받아 현재 목록을 점검했습니다. 지금 공개된 설명은 1.0.0(2026-11-02) 때 쓴 네 문단뿐입니다. 이 앱이 왜 필요한지, 화면 어디에 어떤 기능이 있는지, 처음에 무엇을 설정해야 하는지가 빠져 있습니다. 1.1~1.5에서 추가된 기능도 하나도 소개되지 않습니다.

개편안은 Windows 앱 하나만을 설명합니다. 다른 운영체제 버전이나 다른 배포처는 언급하지 않습니다. 구성은 아래 순서를 따릅니다.

1. **목적**: 왜 쓰는지, 누구에게 필요한지
2. **원리**: 어떻게 동작하는지
3. **시작하기**: 처음에 무엇을 설정하는지
4. **화면 안내**: 어디에 어떤 기능이 있는지
5. **활용 예**
6. **요구 사항**
7. **개인정보**
8. **문제 해결**

| 항목 | 현재 | 개편안 |
|---|---|---|
| 설명 길이 | 약 500자, 4문단 | 한국어 약 3,800자, 영어 약 6,400자, 10개 구획. 한도는 10,000자 |
| 목적·사용 사례 | 한 문장 | "이런 분께 필요합니다"와 활용 예 3가지 |
| 화면 안내 | 없음 | 알림 영역 메뉴, 설정의 전환·장치·일반 탭, 진동 설정 창을 항목별로 |
| 소개된 전환 방법 | 마우스 버튼 1가지 | 마우스 버튼·제스처·단축키·화면 모서리 4가지 |
| 1.1 이후 기능 | 없음 | 모서리 전환, 대기 효과(색·크기), 진동, 마우스 찾기, 함께 이동할 장치 |
| 시작 방법 | "모든 컴퓨터에 설치하세요" 한 줄 | 6단계 설정 순서 |
| 문제 해결 | 없음 | 자주 겪는 4가지 |
| 제품 기능(Product features) | 없음 | 14개 항목 |
| 이번 버전의 새로운 기능 | 없음 | 1.8.0·1.7.1·1.7.0 변경 사항 |
| 스크린샷 | 작은 설정 창 3장, 1.0 화면 | 6장 구성안과 캡션 |

## 2. 현재 목록의 문제점

1. **목적이 드러나지 않음.** "Easy-Switch 버튼으로 바꿉니다"는 Easy-Switch를 모르는 사람에게 와닿지 않습니다. 어떤 불편을 덜어 주는지부터 말해야 합니다. 그 불편은 "기기마다 채널 버튼을 따로 눌러야 한다"는 것입니다.
2. **화면 안내가 없음.** 설치 후 알림 영역 아이콘만 생기고 창이 뜨지 않습니다. 그래서 "어디서 설정하는지" 모르는 경우가 생깁니다. 탭별로 무엇이 있는지 설명이 필요합니다.
3. **기능이 1.0에 멈춰 있음.** 단축키, 화면 모서리 전환, 대기 효과, 진동(MX Master 4), 마우스 찾기가 설명에 없습니다.
4. **시작 순서가 없음.** 실제로 막히는 지점은 세 가지입니다.
   - 채널 번호 맞추기
   - 오가는 컴퓨터마다 flockswitch 실행하기
   - Logi Options+에서 버튼을 "동작 없음"으로 바꾸기
   지금 설명에는 이 중 하나만 나옵니다.
5. **훑어보기 어려움.** Store 설명은 서식(굵게, 제목)을 지원하지 않습니다. 줄바꿈과 기호(■, •, [ ])로 구획을 나눕니다.
6. **스크린샷이 정보를 주지 못함.** 큰 캔버스 가운데에 작은 설정 창만 있어서 글씨를 읽을 수 없고, 1.0 화면입니다.

## 3. 작성 원칙

- **Windows 앱만 설명:** 다른 운영체제 버전, 다른 앱 마켓, 다른 내려받기 경로는 언급하지 않습니다. 소스 저장소 전체 링크도 넣지 않고, 지원 연락처(Issues)만 둡니다. 오가는 다른 컴퓨터에도 flockswitch가 있어야 한다는 점은 운영체제 이름 없이 씁니다.
- **정확성:**
  - Windows 앱의 실제 화면 문구(`src/i18n.rs`)와 기능만 씁니다.
  - Windows 앱에 없는 항목은 넣지 않습니다. 예: 업데이트 확인, "개발자에게".
  - 시험 범위는 그대로 밝힙니다. MX Master 3·4로 만들고 시험했고, 다른 Logitech 마우스는 아직 시험하지 않았습니다.
- **정책 준수:**
  - 다른 회사 상표(Logitech, MX Master, Easy-Switch, Logi Bolt, Unifying, Logi Options+, Logitech Flow)는 설명에서만 쓰고 검색어에는 넣지 않습니다(정책 10.1.3).
  - 상표 고지와 "Logitech과 관계없음"을 유지합니다.
- **개인정보:** Store 버전의 인터넷 연결은 Microsoft Store에 새 버전을 묻는 것뿐이고(끌 수 있음), 사용자 정보는 보내지 않습니다. 코드에서 확인했고, 이 점을 분명히 씁니다.
- **두 언어 동일:** ko-KR과 en-US의 구성과 내용을 같게 맞춥니다. 패키지가 두 언어를 선언하므로 두 목록이 모두 필요합니다.
- **용어:** 앱 화면의 용어를 그대로 씁니다. 예: "진동"("파형" 아님), "이동할 채널", "함께 이동할 장치", "대기 효과", "머무는 시간".

---

## 4. 붙여 넣을 문구: 한국어 (ko-KR)

### 4-1. 설명 (Description)

```
마우스 버튼 한 번으로 Logitech 마우스와 키보드를 함께 다른 컴퓨터로 넘깁니다.

Logitech 마우스와 키보드는 Easy-Switch 채널(1·2·3)에 컴퓨터를 하나씩 기억해 두고 오갈 수 있습니다. 하지만 원래는 키보드의 채널 버튼, 마우스 바닥의 채널 버튼을 기기마다 따로 눌러야 합니다. flockswitch는 이 전환을 마우스 버튼 하나, 키보드 단축키, 또는 화면 가장자리로 한 번에 해 줍니다. 컴퓨터끼리 네트워크로 연결하지 않으므로, Logitech Flow를 쓸 수 없는 회사·공용·보안 네트워크에서도 동작합니다.

■ 이런 분께 필요합니다
• 한 책상에서 데스크톱과 노트북처럼 컴퓨터 두세 대를 키보드·마우스 한 벌로 쓰는 분
• 기기마다 채널 버튼을 누르는 것이 번거롭거나, 키보드만 넘어가고 마우스는 남는 일이 잦은 분
• 회사 보안 정책 때문에 컴퓨터끼리 네트워크로 묶을 수 없는 분

■ 이렇게 동작합니다
1. 전환에 쓸 버튼을 정합니다. MX Master 마우스의 휠 모드 전환 버튼, 제스처 버튼, 휠 클릭 중에서 고릅니다.
2. 버튼을 누르면 함께 묶어 둔 키보드와 다른 장치가 먼저 고른 채널로 넘어갑니다.
3. 마지막으로 마우스가 같은 채널로 넘어갑니다. 넘어간 컴퓨터에서 실행 중인 flockswitch가 다음 전환을 이어받습니다.

■ 처음 설정하기
1. 키보드와 마우스를 각 컴퓨터에 같은 채널 번호로 페어링합니다. 예: 채널 1 = 이 PC, 채널 2 = 노트북. 모든 기기에서 같은 번호가 같은 컴퓨터를 가리켜야 합니다.
2. 오가는 각 컴퓨터에서도 flockswitch가 실행 중이어야 돌아오는 전환이 됩니다.
3. Logi Options+에서 전환에 쓸 버튼을 "동작 없음"으로 바꿉니다. 그대로 두면 버튼을 누를 때 Options+ 기능도 함께 실행됩니다.
4. 설치하면 작업 표시줄 오른쪽 알림 영역에 ⇄ 아이콘이 생깁니다. 안 보이면 ^ 를 눌러 숨겨진 아이콘을 확인하세요. 아이콘 메뉴에서 "설정…"을 엽니다.
5. 전환 탭에서 쓸 버튼을 켜고 "이동할 채널"을 고릅니다.
6. 장치 탭에서 함께 넘어갈 키보드를 고릅니다. 바꾼 내용은 저장 버튼 없이 바로 적용됩니다.

■ 화면 안내: 어디에 무엇이 있나요

[알림 영역 ⇄ 아이콘 메뉴]
• 동작 상태, 최근 전환한 채널과 시각, 앱 버전 확인
• 마우스 버튼으로 전환 켜기/끄기
• 설정… / flockswitch 종료

[설정 → 전환 탭]
• 마우스 버튼으로 전환: 전체 켜기/끄기. 끄면 버튼이 원래 기능으로 돌아갑니다.
• 버튼별 설정: 휠 모드, 제스처, 휠 클릭, 단축키, 모서리의 다섯 칸이 있습니다.
• 휠 모드 · 휠 클릭: 버튼을 켜고 "이동할 채널"을 고릅니다. 이 컴퓨터의 채널은 잠겨 있어 고를 수 없습니다. 휠 클릭을 켜면 휠을 눌렀을 때 원래 동작 대신 전환만 합니다.
• 제스처: "누르기", 그리고 누른 채 위·아래·왼쪽·오른쪽으로 움직였다 떼기. 다섯 동작에 각각 채널을 정합니다. 방향을 하나라도 켜면 버튼을 누르는 동안 커서가 멈춥니다.
• 단축키: 채널마다 키보드 단축키 하나(기본 Ctrl+Alt+1/2/3). "바꾸기"로 원하는 조합을 누르고 "지우기"로 없앱니다. 어떤 키보드로 눌러도 됩니다. 제목 옆 ⓘ에 마우스를 올리면 쓸 수 있는 조합(Ctrl이나 Alt와 글자·숫자, 또는 F키)과 쓸 수 없는 조합(Windows 키 조합, 다른 앱이 이미 쓰는 조합 등)을 알려 줍니다.
• 모서리: 위·아래·왼쪽·오른쪽 가장자리마다 그쪽에 있는 컴퓨터의 채널을 정합니다.
  - 머무는 시간: 0.3초, 0.5초, 1초, 2초 중에서 고릅니다.
  - Ctrl 키를 누른 동안에만 전환: 실수로 넘어가는 것을 막습니다.
  - 돌아오면 커서를 보냈던 쪽에 놓기
  - 대기 효과: 링, 반원, 막대, 펄스 파동, 네온 레이저, 파티클 보텍스, 사이버 스캔, 리퀴드, 랜덤 중에서 고릅니다. 효과마다 색상, 크기(75~150%), 줄어드는 방향(링·반원·막대)을 따로 정하고 "미리 보기"로 확인합니다. 랜덤은 매번 효과와 함께 방향과 작아지며 사라지기도 무작위로 고릅니다.

[설정 → 장치 탭]
• 마스터 마우스: 전환 버튼을 쓸 마우스. 보통은 자동 선택으로 둡니다.
• 함께 이동할 장치: 마우스와 함께 넘어갈 키보드나 다른 장치. Logi Bolt 수신기, Unifying 수신기, 블루투스로 연결된 장치를 섞어서 고를 수 있습니다. 지금 다른 컴퓨터에 가 있는 장치도 "다른 PC"로 표시됩니다.

[설정 → 일반 탭]
• 시작하기 전에: Logi Options+ 설정 안내
• 마우스 찾기: 커서 위치를 펄스 링, 레이더, 스포트라이트, 랜덤 효과로 강조합니다. 단축키로 표시(기본 Ctrl+Alt+L), 다른 컴퓨터에서 마우스가 도착했을 때, 마우스를 빠르게 흔들었을 때 중에서 고릅니다. 색상과 크기도 정합니다. 흔들기는 얼마나 오래 흔들어야 하는지(0.5·1·1.5·2초)와 흔드는 동안 누르고 있을 키(없음·Ctrl·Alt·Shift)를 정할 수 있어, 평소 마우스를 움직일 때 잘못 뜨지 않습니다.
• 진동: 진동 모터가 있는 마우스(현재 MX Master 4)용. 요약을 보고 "진동 설정…"으로 자세히 정합니다.
• 언어(한국어/English), 로그인 시 자동 실행(시작 프로그램 설정 열기)
• 업데이트: 현재 버전, 지금 확인 / 지금 업데이트, Store 페이지. 지금 확인으로 새 버전을 찾으면 "지금 업데이트"로 Microsoft Store에서 바로 받아 설치합니다. "자동으로 새 버전 확인"(시작할 때와 하루에 한 번)과 "새 버전 자동 설치"를 각각 켜고 끌 수 있습니다. 설치하는 동안 flockswitch가 잠시 닫혔다가 다시 켜집니다.
• 설정 백업: "저장"은 지금 설정을 문서\flockswitch 폴더에 파일로 저장하고(최대 10개, 개수 표시), "불러오기…"는 그 폴더의 파일을 골라 확인을 거쳐 불러옵니다. "관리…"에서 파일 이름을 바꾸거나 휴지통으로 지웁니다. "폴더 열기"로 폴더를 열어 파일을 다른 PC로 옮길 수 있습니다. 채널은 PC마다 다르므로 다른 PC에서 불러온 뒤에는 채널을 확인하세요.
• 설정 초기화…, flockswitch 제거…

[진동 설정 창]
• 진동 사용, 진동 세기(Options+ 설정 따르기, 25~100%)
• 마우스가 떠날 때 / 마우스가 돌아왔을 때: 각각 켜고 진동 종류를 고릅니다. 랜덤도 고를 수 있습니다.
• 모서리에서 기다리는 동안: 점점 강하게 또는 일정하게
• 진동 확인: 고른 진동을 고른 세기로 마우스에서 바로 울립니다.
• 적용 / 확인 / 닫기: 닫기를 누르면 창을 열기 전 상태로 되돌립니다.

■ 이렇게 써 보세요
• 데스크톱(채널 1)과 노트북(채널 2): 양쪽 모두 휠 모드 전환 버튼에 "상대편 채널"을 지정합니다. 같은 버튼 하나로 오갈 수 있습니다.
• 모니터를 나란히 두었다면: 노트북이 오른쪽에 있으면 데스크톱의 "오른쪽" 모서리에 채널 2를, 노트북의 "왼쪽" 모서리에 채널 1을 정합니다. 커서를 밀면 넘어갑니다.
• 세 대를 쓴다면: 제스처 버튼을 누른 채 왼쪽 = 채널 1, 오른쪽 = 채널 3처럼 방향마다 컴퓨터를 정합니다.

■ 필요한 것
• Windows 10(1809 이상) 또는 Windows 11, x64
• Logitech MX Master 3 또는 MX Master 4 (Logi Bolt 수신기, Unifying 수신기 또는 블루투스 연결). 이 두 마우스로 만들고 시험했습니다. 같은 기능을 가진 다른 Logitech 마우스도 동작할 수 있지만 아직 시험하지 않았습니다.
• Logi Options+와 함께 쓰는 것을 전제로 합니다.

■ 개인정보와 안전
• 이 PC에 연결된 Logitech 장치와만 통신합니다. 계정이 없으며, 어떤 정보도 수집하거나 보내지 않습니다. 인터넷 연결은 Microsoft Store에 새 버전을 묻는 것뿐이고, 설정에서 끌 수 있습니다.
• 드라이버나 서비스를 설치하지 않습니다. flockswitch를 끄거나 종료하면 버튼이 원래 설정 그대로 돌아갑니다.
• 로그인할 때 창 없이 알림 영역에서 조용히 시작합니다. 자동 실행은 Windows 설정 → 앱 → 시작 프로그램에서 끌 수 있습니다.

■ 잘 안 될 때
• 아이콘이 안 보여요: 알림 영역의 ^ 안쪽을 확인하세요. 시작 메뉴에서 flockswitch를 다시 실행하면 설정 창이 열립니다.
• 버튼을 누르면 다른 기능도 같이 실행돼요: Logi Options+에서 그 버튼을 "동작 없음"으로 바꾸세요.
• 채널 하나를 고를 수 없어요: 그 채널이 지금 이 컴퓨터라서 잠겨 있는 것입니다.
• "지원 마우스를 찾는 중"이 계속돼요: 마우스가 이 PC의 채널에 연결되어 있는지, 수신기나 블루투스 연결을 확인하세요.

■ 문의
문제 신고와 제안: https://github.com/WWhaleFe/flockswitch-windows/issues

Logitech과 관계없는 개인 오픈소스(MIT)입니다. Logitech, MX Master, Easy-Switch, Logi Bolt, Unifying, Logi Options+, Logitech Flow는 Logitech의 상표입니다.
```

### 4-2. 짧은 설명 (Short description)

```
마우스 버튼 한 번, 단축키, 또는 화면 가장자리로 Logitech 마우스와 키보드를 함께 다른 컴퓨터로 넘깁니다. 컴퓨터끼리 네트워크 연결 없이 동작합니다.
```

### 4-3. 이번 버전의 새로운 기능 (What's new in this version)

```
1.8.0
• 모서리 대기 효과마다 색상·크기·방향을 따로 저장합니다. 링을 바꿔도 막대는 그대로입니다.
• 랜덤은 매번 효과와 함께 줄어드는 방향과 작아지며 사라지기도 무작위로 골라 다양하게 보여 줍니다.
• 설정 백업: 버튼을 저장·불러오기…·관리…·폴더 열기로 줄이고 저장된 개수(최대 10개)를 보여 줍니다. 관리에서 파일 이름을 바꾸거나 휴지통으로 지웁니다.
• 설정 창이 더 높게 열리고, 내용이 길면 화면 안에서 한 번 늘어나 일반 탭이 스크롤 없이 보입니다.

1.7.1
• 진동 설정을 '마우스 찾기' 아래 별도 카드로 옮기고, 설정 창의 줄과 구분선 사이 간격을 넓혀 화면을 정돈했습니다.

1.7.0
• 설정 백업: 설정을 문서\flockswitch 폴더에 파일로 저장하고 불러옵니다. 파일을 옮겨 다른 PC에서도 같은 설정을 쓸 수 있습니다.
• 마우스 찾기의 흔들기: 흔드는 시간(0.5~2초)과 함께 누를 키(Ctrl·Alt·Shift)를 정할 수 있어, 잠깐 흔들린 것으로는 뜨지 않습니다.
• 단축키 옆 ⓘ: Windows에서 쓸 수 있는 조합과 쓸 수 없는 조합을 알려 줍니다.
• 배율이 다른 모니터로 설정 창을 옮겨도 창 크기가 그대로 유지됩니다.
• 지금 확인으로 새 버전을 찾으면 지금 업데이트로 바로 설치하고, 실패하면 이유를 보여 줍니다.
• 로그 보기 버튼을 뺐습니다.
```

### 4-4. 제품 기능 (Product features, 항목당 200자 이내)

```
마우스 버튼 한 번으로 키보드와 마우스를 함께 다른 컴퓨터로 전환
휠 모드 전환 버튼·제스처 버튼·휠 클릭 중 원하는 버튼 사용
제스처 버튼을 누른 채 위·아래·왼쪽·오른쪽으로 채널 선택
채널마다 키보드 단축키(기본 Ctrl+Alt+1/2/3)
화면 가장자리에 커서를 대면 그쪽 컴퓨터로 이동
모서리 대기 효과 8종과 랜덤, 효과마다 색상·크기·방향 선택
Ctrl을 누른 동안에만 모서리 전환하는 옵션
Logi Bolt·Unifying·블루투스 장치를 섞어서 함께 이동
마우스 찾기: 단축키·도착·흔들기로 커서 강조(흔드는 시간·함께 누를 키 선택)
설정 백업: 파일로 저장(최대 10개)·이름 바꾸기·삭제, 다른 PC에서 불러오기
MX Master 4 진동: 떠날 때·돌아왔을 때·모서리 대기, 진동 확인
컴퓨터끼리 네트워크 연결 없이 동작
Microsoft Store 자동 업데이트(켜고 끌 수 있음)
계정·데이터 수집 없음
드라이버·서비스 설치 없음, 끄면 버튼이 원래대로
한국어·영어 지원
```

### 4-5. 검색어 (최대 7개, 다른 회사 이름 제외)

```
mouse switch, keyboard switch, multi computer, KVM, multi PC, channel switch, edge switch
```

검색어는 영어 Partner Center 화면(/en-us/)에서만 편집합니다. 한국어 화면에서는 검색어가 저장되지 않고 사라집니다.

---

## 5. 붙여 넣을 문구: 영어 (en-US)

### 5-1. Description

```
Press one mouse button and your Logitech mouse and keyboard move to another computer together.

Logitech mice and keyboards remember one computer on each Easy-Switch channel (1, 2, 3) and can hop between them. Normally, though, you press the channel button on the keyboard and the one under the mouse separately, device by device. flockswitch does it in one go: from a mouse button, a keyboard shortcut or the edge of the screen. The computers are not networked together, so it works where Logitech Flow cannot, such as office, guest or locked-down networks.

■ Who it is for
• You use two or three computers at one desk, say a desktop and a laptop, with one keyboard and mouse
• Pressing each device's channel button is a chore, or the keyboard moves and the mouse stays behind
• Your company's security rules don't let you link the computers over a network

■ How it works
1. Pick the button that switches: the wheel mode-shift button, the gesture button or the wheel click of your MX Master mouse.
2. Press it and the keyboard and other devices you grouped move to the chosen channel first.
3. The mouse follows last. flockswitch running on the computer it lands on takes over the next switch.

■ Getting started
1. Pair your keyboard and mouse with each computer on the same channel number, e.g. channel 1 = this PC, channel 2 = laptop. A number must mean the same computer on every device.
2. Switching back works when flockswitch runs on each computer you switch between.
3. In Logi Options+, set the button you will use to "No action", or pressing it also runs its Options+ action.
4. After installing, an ⇄ icon appears in the notification area at the right of the taskbar (click ^ if it is hidden). Choose "Settings…" from its menu.
5. On the Switching tab, turn on a button and choose "Go to channel".
6. On the Devices tab, choose the keyboard that moves along. Changes apply at once, with no Save button.

■ Where everything is

[⇄ icon menu in the notification area]
• See the status, the last switch and the app version
• Turn switching with mouse buttons on or off
• Settings… / Quit flockswitch

[Settings → Switching tab]
• Switch with mouse buttons: on or off for everything. Off gives the buttons their own functions back.
• Per button: five pages — Wheel mode, Gesture, Wheel click, Shortcuts and Edges.
• Wheel mode and Wheel click: turn the button on and choose "Go to channel". This computer's own channel is locked so you can't pick it. With Wheel click on, pressing the wheel switches instead of doing its usual job.
• Gesture: "Press", and hold-and-move up, down, left or right, then release. Each of the five has its own channel. With any direction on, the cursor stays still while the button is held.
• Shortcuts: one keyboard shortcut per channel (Ctrl+Alt+1/2/3 by default). "Change" records a new combination, "Clear" removes it. Any keyboard works. Hover the ⓘ beside the title to see which combinations work (Ctrl or Alt with a letter or number, or an F key) and which do not (Windows key combinations, ones another app already uses and so on).
• Edges: give the top, bottom, left and right edges the channel of the computer on that side.
  - Time at edge: 0.3, 0.5, 1 or 2 s.
  - Only while Ctrl is held: stops accidental switches.
  - On return, put the cursor at that edge.
  - Countdown: ring, semicircle, bar, pulse wave, neon laser, particle vortex, cyber scan, liquid or random. Each effect keeps its own colour, size (75 to 150%) and, for the ring, semicircle and bar, the way it drains. Check it with "Preview". Random also picks the direction and shrinking away at random each time.

[Settings → Devices tab]
• Master mouse: the mouse whose button switches. Usually left on Automatic.
• Devices that move along: the keyboard or other devices that go with the mouse. Mix devices on a Logi Bolt receiver, a Unifying receiver and Bluetooth. Devices on another computer right now are listed as such.

[Settings → General tab]
• Before you start: a note on Logi Options+.
• Find the cursor: highlight the cursor with a pulse ring, radar, spotlight or random effect. It can show with a shortcut (Ctrl+Alt+L by default), on arrival from another computer, or when you shake the mouse, in the colour and size you choose. For shaking, choose how long it must go on (0.5, 1, 1.5 or 2 s) and a key to hold while shaking (none, Ctrl, Alt or Shift), so ordinary mouse movement does not bring it up.
• Haptics: for a mouse with a haptic motor (currently the MX Master 4). A summary, and "Haptics Settings…" for the details.
• Language (Korean / English), Start at login (opens the startup settings).
• Updates: the version, Check now / Update now, and the Store page. When Check now finds a new version, "Update now" gets it from the Microsoft Store and installs it right away. "Check for new versions automatically" (at start and once a day) and "Install new versions automatically" each switch on and off. While it installs, flockswitch closes for a moment and starts again.
• Settings backup: "Save" writes the current settings as a file into Documents\flockswitch (up to 10, with the count shown), and "Load…" picks a file there and, once you confirm, loads it. "Manage…" renames files or deletes them to the Recycle Bin. "Open folder" opens the folder, so files can be taken to another PC. Channels differ from PC to PC, so check them after loading on another PC.
• Reset settings…, Uninstall flockswitch…

[Haptics Settings window]
• Use haptics, and Strength (as set in Options+, or 25 to 100%).
• As the mouse leaves / When the mouse is back: each with its own switch and pulse, or Random.
• While waiting at an edge: Rising or Even.
• Test: play the chosen pulse on the mouse at the chosen strength.
• Apply / OK / Close: Close puts back what was set when the window opened.

■ Ideas to try
• Desktop (channel 1) and laptop (channel 2): on each, give the wheel mode-shift button the other's channel. One button takes you back and forth.
• Screens side by side: with the laptop on the right, set the desktop's right edge to channel 2 and the laptop's left edge to channel 1, then push the cursor across.
• Three computers: hold the gesture button and move left for channel 1 or right for channel 3.

■ Requirements
• Windows 10 (1809 or later) or Windows 11, x64
• Logitech MX Master 3 or MX Master 4 (Logi Bolt receiver, Unifying receiver or Bluetooth). flockswitch was built and tested with these two. Other Logitech mice with the same features may work but have not been tested yet.
• Meant to run alongside Logi Options+.

■ Privacy and safety
• It talks only to the Logitech devices connected to this PC. No account, nothing collected or sent. Its only internet use is asking the Microsoft Store for a new version, which you can turn off in the settings.
• No drivers or services are installed. Turn flockswitch off or quit it and the buttons go back to exactly how they were.
• It starts quietly in the notification area at sign-in, with no window. Turn that off in Windows Settings → Apps → Startup.

■ If something doesn't work
• No icon: look under ^ in the notification area. Starting flockswitch again from the Start menu opens the settings.
• The button does something else too: set it to "No action" in Logi Options+.
• One channel can't be chosen: it is this computer's channel, so it is locked.
• "Looking for a supported mouse" stays: check that the mouse is on this PC's channel and its receiver or Bluetooth connection.

■ Contact
Problems and ideas: https://github.com/WWhaleFe/flockswitch-windows/issues

An independent open-source project (MIT), not affiliated with Logitech. Logitech, MX Master, Easy-Switch, Logi Bolt, Unifying, Logi Options+ and Logitech Flow are trademarks of Logitech.
```

### 5-2. Short description

```
Move your Logitech mouse and keyboard to another computer together with one mouse button, a shortcut or the screen edge. The computers need no network between them.
```

### 5-3. What's new in this version

```
1.8.0
• Each edge countdown effect keeps its own colour, size and direction: changing the ring no longer changes the bar.
• Random now varies more: each time it also picks the direction and shrinking away at random.
• Settings backup: shorter buttons (Save, Load…, Manage…, Open folder) and the number saved, up to 10. Manage renames files or deletes them to the Recycle Bin.
• The settings window opens taller and, when a page is longer, grows once within the screen, so the General tab fits without scrolling.

1.7.1
• Haptics has its own card again under Find the cursor, and rows and their dividers have more room, for a tidier settings window.

1.7.0
• Settings backup: save the settings as a file in Documents\flockswitch and load them again, on this PC or, after copying the file, on another.
• Shaking to find the cursor: choose how long to shake (0.5 to 2 s) and a key to hold (Ctrl, Alt or Shift), so a brief jiggle no longer brings it up.
• An ⓘ beside the shortcuts tells which combinations Windows allows and which it does not.
• The settings window keeps its size when moved to a monitor with other scaling.
• When Check now finds a new version, Update now installs it at once, and says why if it cannot.
• The Show log button is gone.
```

### 5-4. Product features

```
Switch keyboard and mouse to another computer with one mouse button
Use the wheel mode-shift button, gesture button or wheel click
Hold the gesture button and move up, down, left or right to pick a channel
A keyboard shortcut per channel (Ctrl+Alt+1/2/3 by default)
Rest the cursor on a screen edge to move to the computer on that side
Eight edge countdown effects plus random, each with its own colour, size and direction
Option to switch at an edge only while Ctrl is held
Mix Logi Bolt, Unifying and Bluetooth devices in one group
Find the cursor from a shortcut, on arrival or by shaking the mouse (how long, and a key to hold)
Settings backup: save up to 10 files, rename or delete them, load on another PC
MX Master 4 haptics as the mouse leaves, comes back and waits, with Test
No network needed between the computers
Automatic updates from the Microsoft Store (can be turned off)
No account, no data collected
No drivers or services; buttons go back to normal when off
Korean and English
```

### 5-5. Search terms

```
mouse switch, keyboard switch, multi computer, KVM, multi PC, channel switch, edge switch
```

---

## 6. 스크린샷 구성안 (사용자 촬영)

- **형식:** PNG, 1366×768 이상의 가로형.
- **찍는 법:** 설정 창을 바탕화면 위에 크게 띄우고 전체 화면을 찍습니다(Win+Shift+S → 전체 화면). 창만 잘라 내면 Store에서 글씨가 너무 작게 보입니다.
- **언어:** 한국어 목록에는 한국어 화면을, 영어 목록에는 영어 화면을 씁니다. 언어는 설정 → 일반 → 언어에서 바꿉니다.
- **캡션:** 각 스크린샷에 넣습니다.

| # | 화면 | 캡션(ko-KR) | Caption (en-US) |
|---|---|---|---|
| 1 | 전환 탭: 버튼별 설정과 이동할 채널 | 버튼마다 이동할 채널을 정합니다 | Give each mouse button its channel |
| 2 | 전환 탭 → 모서리: 방향별 채널과 대기 효과 | 화면 가장자리에 대면 그쪽 컴퓨터로 | Rest on a screen edge to move to that computer |
| 3 | 모서리에서 대기 효과가 줄어드는 순간(커서 근처) | 남은 시간을 보여 주는 대기 효과 | A countdown effect while it waits |
| 4 | 장치 탭: 함께 이동할 장치 | 키보드와 다른 장치를 함께 이동 | Keyboard and other devices move along |
| 5 | 일반 탭(마우스 찾기·진동·업데이트) | 마우스 찾기, 진동, 자동 업데이트 | Find the cursor, haptics and automatic updates |
| 5-1 | 진동 설정 창 | 상황마다 진동을 고르고 바로 확인 | Pick a pulse for each moment and test it |
| 6 | 알림 영역 ⇄ 메뉴 | 알림 영역에서 조용히 동작 | Runs quietly in the notification area |

## 7. 적용 순서 (이미지 교체 요청을 받으면)

1. Partner Center에서 이전 제출이 게시됐는지 확인합니다.
2. **Start update**로 새 제출을 만들고, 패키지에 1.7.0 MSIX를 올립니다.
3. Store listings → **한국어**:
   - 4-1~4-4를 붙여 넣습니다.
   - 기존 스크린샷을 지우고 새 스크린샷과 6번의 캡션을 넣습니다.
4. Store listings → **English**: 5-1~5-4와 스크린샷을 같은 방식으로 넣습니다.
5. 검색어(4-5 / 5-5)는 영어 Partner Center 화면(/en-us/)에서 넣습니다.
   - 칩 입력란 바깥을 클릭한 뒤 저장합니다.
   - 저장 후 "Additional information"을 펼쳐 칩이 남아 있는지 확인합니다.
6. 저장한 뒤 Submit for certification으로 제출합니다.
7. Store 공개 페이지(https://apps.microsoft.com/detail/9NN9ZLHLGSKN)에서 설명과 스크린샷이 바뀌었는지 확인합니다.

## 8. 참고

- **글자 수 한도:** 설명 10,000자, 이번 버전의 새로운 기능 1,500자, 제품 기능 20개×200자, 검색어 7개.
- **Logitech Flow 비교 문구:** 지금 공개된 설명에도 있고, 인증을 통과했습니다.
- **README와의 관계:** 이 문서는 `packaging/windows/store/README.md` 7장의 1.0 시절 문구를 대신합니다. 적용한 뒤 README 7장이 이 문서를 가리키도록 바꿉니다.
