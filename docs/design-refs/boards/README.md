# 디자인 시안 보드 원본

> 원본: [FLUTCH 디자인 시안 (아티팩트, 비공개)](https://claude.ai/artifact/X4mcgYsXY9tngprQf1qh6N) — 2026-11-10 메인 세션이 내보냄, 2026-11-18 디자인 세션이 최신본(파동 이미지·Beat 단위 반영)과 `assets/`·렌더 도구로 갱신. 서브에이전트는 아티팩트를 열 수 없으므로 **이 폴더가 시안의 기준**이다.
> 시안을 고치면 메인 세션이 다시 내보낸다.

> **2026-11-18 재내보내기(사용자 시안 갱신)**: 보드를 최신 아티팩트로 교체하고 폭별 미리보기 보드(`*W768/W1024/W1920/W2560`, `*Mobile`), `BrandAssets`(E 보드: 이모티콘·배너), `Comp*`(컴포넌트 시트)를 추가했다. 새 보드의 이미지는 `/_blob/<id>` 그대로다 — 아래 표로 프론트 파일에 대응한다.
>
> | `/_blob/` id | 프론트 파일 |
> |---|---|
> | 93d57471… | `/brand/logo-flutch.png` (로고) |
> | 3d8747c0… | `/brand/bg-speed.jpg` (배경 텍스처) |
> | e1bda6ef… | `/brand/banner-play-on-flutch.webp` (E2 사이드바 배너, 프레임 포함 — 테두리·모서리 덧씌우지 않음) |
> | 9c0e7194… 84a8a7f1… ec28e700… efce14b0… 7c90d19c… d68c5d66… 3174a44d… 3b2d6fe3… 0b9432dd… 3f272b25… 2e3aa2a2… b382b800… 328e7a62… 585a570b… 8a7fc297… c1b8fdae… | `/brand/emotes/01-basic` 02-excited 03-tilt 04-surprised 05-sleepy 06-angry 07-smitten 08-peek 09-crying 10-heart 11-tongue 12-meh 13-bye 14-sweat 15-hiding 16-humming `.png` (E1 순서) |
> | fb2b18c2… bc352f43… 10d98373… 등 원형 얼굴 | 기존 `/brand/face-*.png`(프로필 사진 전용) |

- 파일 하나 = 보드 하나. `<x-dc>` 안의 HTML·인라인 스타일이 시안 그대로다(색·간격·글꼴 크기·사선 모서리 `ch12`/`ch6`·배지·문구). `{{…}}`·`<sc-for>`는 예시 데이터 반복이다.
- 이미지는 `/_blob/<id>` 그대로이고 원본 파일은 `assets/<id>.*`에 있다. 프론트 파일 대응은 위 표(파동 아이콘 `f6543586…`는 아직 프론트에 없다 → `frontend/public/brand/wave.png`로 복사해 보드의 `.wave` 처럼 CSS mask + `currentColor`로 쓴다. 모든 파동 아이콘이 이 이미지다).
- 시안의 hex는 참고값이다. 코드에서는 `frontend/src/styles/tokens.css` 토큰만 쓴다(AGENTS §6). 시안 색이 토큰에 없으면 토큰을 추가한다.
- 보드는 `../tools/design-ref.mjs`로 **정적 페이지·기준 PNG로 렌더**된다. 소스만 읽고 구현하지 말고, 화면 task는 아래 diff로 맞춘다.

## 시안 diff (화면 task 필수, 2026-11-18)
WSL에서 (Playwright는 `~/qa-tools`에 설치돼 있음):
```bash
export PLAYWRIGHT=/home/dbsrh/qa-tools/node_modules/playwright/index.mjs
node docs/design-refs/tools/design-ref.mjs render                 # 모든 보드 → docs/design-refs/out/<보드>.png (기준 이미지)
node docs/design-refs/tools/design-ref.mjs diff Home http://<task>.localhost:<VITE_PORT>/
node docs/design-refs/tools/design-ref.mjs diff Wallet http://…/wallet --storage /tmp/<task>/state.json   # 로그인 화면은 Playwright storageState
```
- `diff`는 보드의 미리보기 폭·높이(예: `Home` 1440, `HomeW768` 768, `HomeMobile` 390)로 앱을 찍어 `out/<보드>.diff.png`(시안 | 앱 | 빨강=다른 픽셀)와 다른 픽셀 비율을 낸다. `out/`은 git에 안 올라간다.
- **완료 기준**: diff 이미지에서 빨강이 **데이터 차이(이름·숫자·썸네일·아바타)뿐**이어야 한다. 위치·크기·간격·색·글꼴·모서리·아이콘·문구가 다르면 고친다. 비율 숫자는 참고용(데이터가 달라 0%는 안 나옴).
- 폭별 보드가 있는 화면은 **그 폭 전부** diff한다(`Home`·`HomeW768`·`HomeW1024`·`HomeW1920`·`HomeW2560`·`HomeMobile`).
- PR 본문에 보드별 diff 결과(비율 + 남은 차이가 데이터뿐인지)를 적는다. 시안이 기능과 맞지 않아 일부러 다르게 한 곳은 이유를 적는다.

| 보드 | 화면 | 프론트 경로 | 담당 task |
|---|---|---|---|
| Main.dc.html | 디자인 시스템(토큰·타이포·컬러) | — | T05d |
| CompActions · CompFeedback · CompOverlay · CompData · BrandAssets | 공통 컴포넌트 A~D 시트, E 브랜드 에셋(이모티콘·배너) | — | T05d |
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
