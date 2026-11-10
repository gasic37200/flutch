# 디자인 시안 보드 원본

> 원본: [FLUTCH 디자인 시안 (아티팩트, 비공개)](https://claude.ai/artifact/X4mcgYsXY9tngprQf1qh6N) — 2026-11-10 메인 세션이 내보냄. 서브에이전트는 아티팩트를 열 수 없으므로 **이 폴더가 시안의 기준**이다.
> 시안을 고치면 메인 세션이 다시 내보낸다.

- 파일 하나 = 보드 하나. `<x-dc>` 안의 HTML·인라인 스타일이 시안 그대로다(색·간격·글꼴 크기·사선 모서리 `ch12`/`ch6`·배지·문구). `{{…}}`·`<sc-for>`는 예시 데이터 반복이다.
- 이미지는 `../brand/` 파일을 가리킨다(아티팩트의 `/_blob/…`를 해시로 대조해 바꿈). 프론트에서는 `/brand/<파일>`.
- 시안의 hex는 참고값이다. 코드에서는 `frontend/src/styles/tokens.css` 토큰만 쓴다(AGENTS §6). 시안 색이 토큰에 없으면 토큰을 추가한다.
- 보드는 자체 런타임(`support.js`)이 있어야 렌더된다. 브라우저로 열지 말고 **소스를 읽는다**.

| 보드 | 화면 | 프론트 경로 | 담당 task |
|---|---|---|---|
| Main.dc.html | 디자인 시스템(토큰·타이포·컬러) | — | T05d |
| Components.dc.html | 공통 컴포넌트·모달·토스트·드롭다운·입력·버튼·탭·스켈레톤·빈 상태·배지·툴팁 | — | T05d |
| Guest.dc.html | 비로그인 헤더·채팅 | 전역 | T05d (채팅 부분은 T17) |
| Errors.dc.html | 에러 3종 | `*` | T05d |
| Home.dc.html · HomeMobile.dc.html | 홈 데스크톱·모바일 | `/` | T06c (히어로·CLUTCH 패널은 T29, FEED는 T30) |
| Explore.dc.html | 탐색·검색 | `/explore` | T06c |
| BroadcastForm.dc.html | 방송 등록·수정 | `/broadcasts/new` · `/broadcasts/:id/edit` | T06c |
| Auth.dc.html · Signup.dc.html | 로그인·가입 | `/login` · `/signup` | T06c |
| SmallScreens.dc.html | 내 정보·탈퇴·카카오 닉네임·결제 결과·약관·스튜디오 모달·차단 입력창 | `/me` `/terms` `/privacy` 외 | T06c(내 정보·약관) · T09c(스튜디오 모달) · T20b·T23·T28·T18u |
| Watch.dc.html · WatchMobile.dc.html · WatchStates.dc.html | 시청 데스크톱·모바일·상태 4종 | `/live/:id` | T11b (채팅 T17, 게이지·클러치 T19, 후원 T23) |
| Studio.dc.html · StudioPrelive.dc.html · StudioMobile.dc.html | 송출 스튜디오(방송 중·송출 전·모바일) | `/studio?broadcast=` | T09c |
| Overlays.dc.html | 후원 창·알림·SUPER CLUTCH 연출·시청자 팝업 | — | T23·T24·T18u |
| Streamer.dc.html | 스트리머 페이지·관리 탭 | `/users/:id` | T26 |
| Wallet.dc.html | 내 지갑·충전 창 | `/wallet` | T23 |
| Replay.dc.html | 다시보기 목록·상세 | `/replay/:id` | T26 |
| HeroSuper.dc.html | 홈 히어로 SUPER CLUTCH | `/` | T29 |
| Following.dc.html | PULSE | `/pulse` | T27 |
