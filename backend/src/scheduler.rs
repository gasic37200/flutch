//! 주기 작업(architecture §5). 각 도메인이 자기 `register(&mut Scheduler)`에서
//! `s.every("이름", 주기, |state| async move { ... })`로 등록한다. 공용 파일은 건드리지 않는다.
//! 작업이 에러·패닉으로 끝나도 로그만 남기고 다음 주기에 다시 돈다. 같은 작업은 겹쳐 돌지 않는다.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::MissedTickBehavior;

use crate::AppState;
use crate::error::AppError;

type JobFuture = Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>;
type JobFn = Arc<dyn Fn(Arc<AppState>) -> JobFuture + Send + Sync>;

#[derive(Default)]
pub struct Scheduler {
    jobs: Vec<(&'static str, Duration, JobFn)>,
}

impl Scheduler {
    /// 모든 도메인의 주기 작업을 모은다.
    pub fn with_all_domains() -> Self {
        let mut s = Self::default();
        crate::auth::register(&mut s);
        crate::live::register(&mut s);
        crate::wallet::register(&mut s);
        crate::clutch::register(&mut s);
        crate::recording::register(&mut s);
        s
    }

    pub fn every<F, Fut>(&mut self, name: &'static str, period: Duration, job: F)
    where
        F: Fn(Arc<AppState>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), AppError>> + Send + 'static,
    {
        self.jobs
            .push((name, period, Arc::new(move |s| Box::pin(job(s)))));
    }

    pub fn start(self, state: Arc<AppState>) {
        for (name, period, job) in self.jobs {
            let state = state.clone();
            tokio::spawn(run_every(name, period, move || job(state.clone())));
        }
    }
}

async fn run_every<F>(name: &'static str, period: Duration, job: F)
where
    F: Fn() -> JobFuture,
{
    let mut tick = tokio::time::interval(period);
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        tick.tick().await;
        // 별도 task로 돌려 패닉도 JoinError로 받아 루프를 살린다.
        match tokio::spawn(job()).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::error!(job = name, error = %e, "scheduled job failed"),
            Err(e) => tracing::error!(job = name, error = %e, "scheduled job panicked"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn keeps_running_after_error_and_panic() {
        let runs = Arc::new(AtomicU32::new(0));
        let counter = runs.clone();
        let handle = tokio::spawn(run_every("test", Duration::from_millis(5), move || {
            let n = counter.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                match n {
                    0 => Err(AppError::Internal("first run fails".into())),
                    1 => panic!("second run panics"),
                    _ => Ok(()),
                }
            })
        }));
        tokio::time::sleep(Duration::from_millis(100)).await;
        handle.abort();
        assert!(runs.load(Ordering::SeqCst) >= 3);
    }
}
