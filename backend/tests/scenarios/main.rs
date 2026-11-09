//! 시나리오 테스트 한 바이너리. 도메인마다 파일 하나를 만들고 아래에 `mod` 한 줄을 추가한다.
//! 실행: `cargo test --test scenarios <파일·테스트 이름 일부>`

mod common;

mod auth;
mod csrf;
mod health;
mod schema;
