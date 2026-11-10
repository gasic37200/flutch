# `cargo sqlx prepare`가 테스트의 `query!` 캐시를 지워 CI clippy 실패
- task: T20b · PR: #93 · 날짜: 2026-11-10
- 영역: backend/.sqlx

## 증상
로컬 검사는 모두 통과했는데 CI `cargo clippy --all-targets`(SQLX_OFFLINE=true)가 `there is no cached data for this query`로 실패. 위치는 `tests/scenarios/follow.rs`의 `sqlx::query!("INSERT INTO follows ...")`.

## 원인
AGENTS §3대로 `cargo sqlx prepare`만 돌리면 라이브러리 타깃의 쿼리만 모으고, 거기에 없는 `.sqlx` 파일(테스트에서만 쓰는 쿼리)은 지운다. 로컬은 DB에 붙어 컴파일하므로 캐시가 없어도 통과한다.

## 해결
지워진 파일을 main에서 되살리고, `cargo sqlx prepare --check -- --all-targets` + `SQLX_OFFLINE=true cargo clippy --all-targets`로 확인.

## 재발 방지
- 쿼리를 바꾸면 `cargo sqlx prepare -- --all-targets`로 테스트 타깃까지 모은다. `git status`에 `.sqlx` 삭제(D)가 보이면 의심한다.
- AGENTS §3 명령을 `cargo sqlx prepare -- --all-targets`로 바꾸는 것을 메인 세션에 요청(이 PR 범위 밖).
