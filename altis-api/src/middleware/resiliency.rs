use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failure detected, failing fast
    HalfOpen, // Testing if service is back
}

pub struct CircuitBreaker {
    pub name: String,
    pub state: RwLock<CircuitState>,
    pub failure_count: AtomicUsize,
    pub failure_threshold: usize,
    pub reset_timeout: Duration,
    pub last_failure: RwLock<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(name: &str, threshold: usize, timeout: Duration) -> Self {
        Self {
            name: name.to_string(),
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicUsize::new(0),
            failure_threshold: threshold,
            reset_timeout: timeout,
            last_failure: RwLock::new(None),
        }
    }

    pub async fn check(&self) -> bool {
        let state = *self.state.read().await;
        if state == CircuitState::Closed {
            return true;
        }

        if state == CircuitState::Open {
            let last_fail = *self.last_failure.read().await;
            if let Some(instant) = last_fail {
                if instant.elapsed() > self.reset_timeout {
                    let mut s = self.state.write().await;
                    *s = CircuitState::HalfOpen;
                    tracing::info!("Circuit Breaker [{}] moving to Half-Open", self.name);
                    return true;
                }
            }
            return false;
        }

        // Half-Open allows one request through
        true
    }

    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        if *state == CircuitState::HalfOpen {
            *state = CircuitState::Closed;
            self.failure_count.store(0, Ordering::SeqCst);
            tracing::info!("Circuit Breaker [{}] recovered to Closed", self.name);
        } else if *state == CircuitState::Closed {
            self.failure_count.store(0, Ordering::SeqCst);
        }
    }

    pub async fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        let mut state = self.state.write().await;
        
        if count >= self.failure_threshold || *state == CircuitState::HalfOpen {
            *state = CircuitState::Open;
            let mut last = self.last_failure.write().await;
            *last = Some(Instant::now());
            tracing::error!("Circuit Breaker [{}] TRIPPED to Open. Failures: {}", self.name, count);
        }
    }
}

pub async fn circuit_breaker_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    // Determine which circuit to use based on path
    let path = req.uri().path();
    let cb = if path.contains("/orders") && path.contains("/pay") {
        Some(&state.resiliency.payment_cb)
    } else if path.contains("/ndc") {
        Some(&state.resiliency.ndc_cb)
    } else {
        None
    };

    if let Some(cb) = cb {
        if !cb.check().await {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Circuit Breaker [{}] is OPEN", cb.name)
            ).into_response();
        }

        let response = next.run(req).await;

        if response.status().is_server_error() {
            cb.record_failure().await;
        } else {
            cb.record_success().await;
        }

        response.into_response()
    } else {
        next.run(req).await.into_response()
    }
}
