# sqlx 0.9 `sqlx::query`에 `format!` 문자열을 넣으면 컴파일 에러
- task: T02 · PR: (이 PR) · 날짜: 2026-11-09
- 영역: backend/tests

## 증상
`sqlx::query(&format!("UPDATE ... {x} ..."))` →
`the trait SqlSafeStr is not implemented for &String` (help: `&'static str`만 구현).

## 원인
sqlx 0.9부터 `query()`·`query_scalar()`는 SQL 인자로 `&'static str`(또는 `AssertSqlSafe`로 감싼 값)만 받는다. 동적 SQL로 인한 인젝션을 컴파일 단계에서 막으려는 변경.

## 해결
값은 SQL에 이어 붙이지 않고 바인딩으로: `sqlx::query("UPDATE wallets SET balance = $2 WHERE user_id = $1").bind(u).bind(bad)`.

## 재발 방지
- 테스트 코드에서도 SQL은 문자열 리터럴 + `.bind()`. 정말 동적 SQL이 필요하면 `QueryBuilder`를 쓴다(`AssertSqlSafe`는 쓰지 않는다).
- conventions.md "SQL 문자열에 사용자 입력을 이어 붙이기" 금지 규칙과 같은 내용이라 문서 추가 반영은 없음.
