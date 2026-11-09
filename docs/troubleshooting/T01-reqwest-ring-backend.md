# reqwest 최신(0.13)은 rustls 기본 백엔드가 aws-lc라 ring을 쓰려면 0.12로 고정
- task: T01 · PR: (생성 후 기입) · 날짜: 2026-11-09
- 영역: backend/Cargo.toml

## 증상
`cargo add reqwest --no-default-features --features rustls-tls,json` → `error: unrecognized feature for crate reqwest: rustls-tls` (0.13.5가 선택됨).

## 원인
reqwest 0.13에서 기능 이름이 바뀌었다: `rustls`(암호 백엔드 aws-lc-rs) / `rustls-no-provider`. 0.12의 `rustls-tls`는 ring을 쓴다. architecture §3은 aws-lc 빌드 도구(cmake 등)가 필요 없게 ring을 쓰기로 했다.

## 해결
`reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }`. `cargo tree -i aws-lc-rs`가 "did not match any packages"인지로 확인(sqlx는 `tls-rustls-ring-webpki`).

## 재발 방지
reqwest를 0.13으로 올릴 때는 `rustls-no-provider` + `rustls`(ring 기능) 직접 의존 + 시작 시 `install_default()`가 필요하다. 올린 뒤 `cargo tree -i aws-lc-rs`로 확인.
