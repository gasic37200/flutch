// 새 마이그레이션 파일만 추가해도 sqlx::migrate!가 다시 컴파일되게 한다.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
