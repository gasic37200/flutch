# 코드 관례 · 안티패턴 · 주의점

> 모든 서브에이전트가 코드를 쓰기 전에 읽는다. 여기 없는 판단은 AGENTS §5(Ponytail)를 따른다.

## 1. 정해진 관례 (프론트·백엔드가 맞춰야 하는 것)
| 항목 | 규칙 |
|---|---|
| JSON 필드 이름 | **camelCase** (`#[serde(rename_all = "camelCase")]`) |
| id·금액 | JSON number(i64), Rust `i64`, DB `BIGINT`. 금액에 `f64` 금지 |
| 시간 | Rust `chrono::DateTime<Utc>`, DB `timestamptz`(Postgres UTC 고정), JSON RFC 3339 UTC. **화면 표시·날짜 경계는 KST**(`chrono_tz::Asia::Seoul` / `Intl … timeZone: 'Asia/Seoul'`). 서버 로컬 시간(`Local::now()`)에 의존하지 않는다 |
| 에러 | 백엔드 `thiserror` 기반 `AppError` → `{error:{code,message,requestId}}`. `anyhow`는 `main`·스크립트에서만 |
| 외부 연동 trait | `LiveKitApi`·`TossApi`는 `async_trait` + `Arc<dyn …>`로 주입(테스트에서 fake로 교체) |
| 공용 상태 | `Arc<AppState>`를 Axum `State`로. 전역 `static`·`lazy_static` 상태 금지 |
| 프론트 서버 상태 | TanStack Query. 쿼리 키는 `['broadcast', id]`처럼 도메인부터 |
| 프론트 API 타입 | `src/api/generated/`(ts-rs)만 사용, 손으로 타입 복제 금지 |

## 2. 안티패턴 (하지 않는다)
**공통**
- 테스트를 코드에 맞춰 고쳐서 통과시키기. 실패하면 코드를 고치거나 보고한다.
- 시나리오 테스트에서 DB를 mock하기. DB는 `#[sqlx::test]`의 진짜 Postgres, mock은 LiveKit·토스 fake만.
- 에러를 삼키기(`let _ = …`, 빈 `catch`, `.ok()`로 버리기). 버릴 때는 이유를 한 줄 주석 + 로그.
- 기능 PR에 관계없는 리팩터링·포맷 변경 섞기.
- `docs/api.md`에 없는 엔드포인트·필드를 몰래 추가하기. 추가하면 같은 PR에서 api.md 갱신.
- `TODO`만 남기기. 한계가 있으면 `shortcut: <한계>, <언제 개선>`.
- 디버그 출력(`println!`, `dbg!`, `console.log`) 남기기. 로그는 `tracing`만.
- 머지된 마이그레이션 수정.
- 몇 줄짜리 기능 때문에 의존성 추가.

**보안·돈**
- 클라이언트가 보낸 금액·대상·사용자 id를 믿기. 항상 서버에서 세션·주문·DB 값으로 다시 확인.
- 프론트에서만 권한 확인(버튼 숨기기)하고 서버 확인 생략.
- 세션 토큰을 `localStorage`에 두기. 쿠키(httpOnly)만.
- 돈 관련 화면에서 낙관적 업데이트(응답 전에 잔액 바꾸기).
- 익명 후원자 실명을 공개 응답·공개 이벤트에 섞기.
- SQL 문자열에 사용자 입력을 이어 붙이기. `query!` 매크로의 바인딩만.

## 3. Rust · Axum · SQLx 주의점
- **요청 처리 경로에서 `unwrap()`·`expect()` 금지.** `?`로 `AppError`에 올린다. 허용: 테스트, 시작 시 설정 로드(실패하면 바로 종료가 맞는 곳).
- **`std::sync::Mutex` 가드를 쥔 채 `.await` 금지**(데드락·컴파일 에러). 짧게 쥐고 놓거나 `tokio::sync::Mutex`. 공유 상태는 가능하면 DB에.
- **CPU를 오래 쓰는 일은 `spawn_blocking`**: argon2 해시·검증이 대표적. 그냥 async 안에서 돌리면 다른 요청이 멈춘다.
- **돈은 트랜잭션 하나로**: `let mut tx = pool.begin().await?;` → 조건부 차감 → 원장 추가 → `tx.commit().await?`. 중간에 `?`로 빠지면 자동 롤백되니 commit을 잊지 말 것.
- **조건부 UPDATE 결과 확인**: `rows_affected() == 0`이면 잔액 부족 → 409. 확인 안 하면 돈이 안 빠졌는데 후원이 기록된다.
- **`query!` 매크로는 컴파일 시 DB가 필요**: 쿼리를 바꾸면 `cargo sqlx prepare`로 `.sqlx/`를 갱신해 커밋. CI는 `SQLX_OFFLINE=true`.
- **NULL 가능 컬럼은 `Option<T>`**: SQLx가 nullable을 추론 못 하면 `col as "col?"` / `"col!"`로 명시.
- **`tokio::spawn`한 작업의 패닉·에러가 조용히 사라진다**: 스케줄러·웹훅 후속 처리는 결과를 받아 `tracing::error!`로 남기고, 루프는 한 번 실패해도 다음 주기를 계속 돈다.
- **broadcast 채널은 크기 제한 + `Lagged` 처리**: 느린 WebSocket 클라이언트가 `RecvError::Lagged`를 받으면 끊지 말고 "다시 조회" 이벤트를 보내거나 재접속시킨다.
- **빌림 검사기를 `clone()` 남발로 피하지 말 것**: 큰 구조체는 `Arc`로 공유. 작은 `String`·`i64` clone은 괜찮다.
- **수명·제네릭을 미리 일반화하지 말 것**: 핸들러는 owned 타입(`String`, `Vec`)으로 단순하게.
- **산술 오버플로**: 금액 합산은 `checked_add` 또는 DB에서 계산. 디버그 빌드에서만 패닉하고 릴리스에서는 조용히 넘친다.
- **비밀값이 `Debug`로 새는 것**: 설정 구조체에 `#[derive(Debug)]`를 붙이면 로그에 키가 찍힌다. 비밀 필드는 수동 `Debug`로 가린다.
- **`clippy` 경고를 `#[allow]`로 덮지 말 것**: 꼭 필요하면 이유 주석.
- **빌드 위치**: WSL Ubuntu의 `~/flutch`에서 빌드한다. `/mnt/c/...`(Windows 폴더)에서 빌드하면 매우 느리다.

## 4. React · LiveKit 주의점
- **화면 공유·카메라는 사용자 클릭 안에서 시작**: `getDisplayMedia`·`getUserMedia`를 `useEffect`에서 바로 부르면 브라우저가 막는다.
- **언마운트 시 정리**: LiveKit `room.disconnect()`, 로컬 트랙 `stop()`. 안 하면 페이지를 떠나도 카메라 불이 켜져 있고 송출이 이어진다.
- **React StrictMode는 effect를 두 번 실행**: 연결·구독 코드는 두 번 불려도 한 번만 연결되게(정리 함수 필수).
- **자동재생은 음소거로 시작**: 소리 있는 자동재생은 막힌다. "소리 켜기"는 사용자 클릭으로.
- **데이터 조회를 `useEffect + fetch`로 직접 하지 말 것**: TanStack Query 사용. WebSocket 이벤트는 `queryClient.setQueryData`/`invalidateQueries`로 반영.
- **TypeScript `any` 금지**: 생성 타입 사용. 모르면 `unknown` 후 좁히기.
- **색·간격 하드코딩 금지**: 디자인 토큰(Tailwind 설정)만.
- **컴포넌트에 로직 넣지 말 것**: 로직은 `features/<도메인>/hooks/`, 컴포넌트는 표현만.
