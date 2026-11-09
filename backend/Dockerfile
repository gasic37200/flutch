# 빌드 컨텍스트: backend/ (ubuntu-24.04-arm 러너에서 네이티브 arm64로 빌드한다)
FROM rust:1-bookworm AS build
WORKDIR /app
ENV SQLX_OFFLINE=true
# 의존성 레이어 캐시: 소스가 바뀌어도 Cargo.toml/lock이 같으면 의존성 컴파일을 재사용한다.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && touch src/lib.rs \
    && cargo build --release && rm -rf src target/release/deps/flutch* target/release/flutch*
COPY .sqlx .sqlx
COPY migrations migrations
COPY src src
# COPY가 원본 mtime을 유지하므로 touch로 더미 빌드 산출물을 확실히 무효화한다.
RUN touch src/main.rs src/lib.rs && cargo build --release --bin flutch

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 flutch
ENV TZ=Asia/Seoul
COPY --from=build /app/target/release/flutch /usr/local/bin/flutch
USER flutch
CMD ["flutch"]
