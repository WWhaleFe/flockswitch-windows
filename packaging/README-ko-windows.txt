flockswitch (Windows)

[처음 한 번]
1. 이 폴더를 오래 둘 위치(예: 문서\flockswitch)에 압축 해제합니다.
   * "로그인 시 자동 실행"은 이 위치를 기억하므로, 나중에 폴더를 옮기면 다시 켜야 합니다.
2. Logi Options+ → MX Master 4 → 전환에 쓸 버튼(휠 모드 전환 / 제스처 / 휠 클릭) → "동작 없음".
3. flockswitch.exe 를 더블클릭합니다. (파란 보안 창: 추가 정보 → 실행)
   작업 표시줄 오른쪽 아래 알림 영역에 파란 ⇄ 아이콘이 생깁니다.
   안 보이면 ^ (숨겨진 아이콘 표시)를 눌러 확인하고, 끌어서 바깥에 고정할 수 있습니다.

[설정] 아이콘을 클릭(또는 우클릭)하면 메뉴가 열립니다.
- 상태 확인, 마우스 버튼으로 전환 켜기/끄기
- 설정… : 설정 창이 열림. 바꾸는 즉시 적용되고 창은 열린 채로 유지
  · 전환 탭: 버튼별(휠 모드 전환 / 제스처 / 휠 클릭) 켜기, 이동할 채널(이 PC의 채널은 고를 수 없음)
             제스처 버튼은 누르기와 누른 채 상하좌우 움직이기를 각각 설정
             단축키: 채널마다 키보드 단축키(기본 Ctrl+Alt+1/2/3), 어떤 키보드로 눌러도 됨
  · 장치 탭: 마스터 마우스, 함께 이동할 장치(그룹)
  · 일반 탭: 언어(한국어/English), 로그인 시 자동 실행, 제거
- 종료

[명령줄도 그대로 사용 가능] PowerShell에서
  .\flockswitch.exe list          장치·채널 확인
  .\flockswitch.exe help          전체 명령

설정 위치: %APPDATA%\flockswitch\ (config.toml, devices.toml, flockswitch.log)
새 버전: https://github.com/WWhaleFe/flockswitch-windows/releases
