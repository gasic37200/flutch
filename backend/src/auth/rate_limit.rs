//! IP별 요청 제한(메모리 고정 창). IP는 저장하지 않고 이 맵에만 둔다(features/10 개인정보).

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use axum::http::HeaderMap;

use crate::error::AppError;

/// 맵이 이 크기를 넘으면 창이 지난 항목을 지운다.
const CLEANUP_AT: usize = 10_000;

pub struct RateLimiter {
    max: u32,
    window: Duration,
    hits: Mutex<HashMap<String, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn new(max: u32, window: Duration) -> Self {
        Self {
            max,
            window,
            hits: Mutex::default(),
        }
    }

    /// 한 번 센다. 창 안에서 `max`회를 넘으면 남은 초와 함께 429.
    pub fn hit(&self, key: &str) -> Result<(), AppError> {
        let now = Instant::now();
        let mut hits = self.hits.lock().unwrap_or_else(PoisonError::into_inner);
        if hits.len() > CLEANUP_AT {
            hits.retain(|_, (start, _)| now.duration_since(*start) < self.window);
        }
        let (start, count) = hits.entry(key.to_owned()).or_insert((now, 0));
        if now.duration_since(*start) >= self.window {
            (*start, *count) = (now, 0);
        }
        if *count >= self.max {
            let left = self.window.saturating_sub(now.duration_since(*start));
            return Err(AppError::RateLimited {
                retry_after_sec: (left.as_millis() as i64 + 999) / 1000,
            });
        }
        *count += 1;
        Ok(())
    }
}

/// 요청 제한용 클라이언트 IP: `X-Forwarded-For` 첫 값. 없으면 빈 문자열(한 묶음으로 센다).
// shortcut: 접속 주소로 신뢰 프록시를 확인하지 않는다. 운영은 backend 포트가 외부에 열려 있지 않고 nginx가
// XFF를 $remote_addr로 덮어써 위조할 수 없다. backend를 직접 노출하게 되면 ConnectInfo로 프록시 대역을 확인한다.
pub fn client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|ip| ip.trim().to_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_max_per_key_and_resets_after_window() {
        let limiter = RateLimiter::new(2, Duration::from_millis(50));
        assert!(limiter.hit("a").is_ok());
        assert!(limiter.hit("a").is_ok());
        assert!(matches!(
            limiter.hit("a"),
            Err(AppError::RateLimited { retry_after_sec: 1 })
        ));
        assert!(limiter.hit("b").is_ok());
        std::thread::sleep(Duration::from_millis(60));
        assert!(limiter.hit("a").is_ok());
    }
}
