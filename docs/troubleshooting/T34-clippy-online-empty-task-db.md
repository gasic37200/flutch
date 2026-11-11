# 새 task DB를 만든 직후 clippy가 query! 매크로 에러 192개
- task: T34 · PR: (이 PR) · 날짜: 2026-11-11
- 영역: backend (sqlx 매크로)

## 증상
`sqlx database create`로 `flutch_t34`를 만들고 `DATABASE_URL`을 그 DB로 둔 채 `cargo clippy --all-targets -- -D warnings` → `could not compile flutch (lib) due to 192 previous errors`. 같은 코드로 `cargo test`·`SQLX_OFFLINE=true` 빌드는 통과.

## 원인
`DATABASE_URL`이 있으면 `query!` 매크로가 `.sqlx` 캐시 대신 그 DB에 붙어 쿼리를 검사한다. 막 만든 DB는 마이그레이션 전이라 테이블이 없어 모든 매크로가 실패한다(`#[sqlx::test]`는 테스트마다 따로 DB를 만들어 마이그레이션하므로 영향 없음).

## 해결
검사 명령은 `SQLX_OFFLINE=true`로 돌렸다(CI와 같은 `.sqlx` 캐시 기준). 서버를 한 번 띄우거나 `sqlx migrate run`으로 task DB를 마이그레이션해도 된다.

## 재발 방지
새 task DB를 만든 뒤 clippy·build는 `SQLX_OFFLINE=true`로, 또는 `sqlx migrate run` 뒤에. 쿼리를 바꿨을 때만 마이그레이션된 DB로 `cargo sqlx prepare -- --all-targets`.
