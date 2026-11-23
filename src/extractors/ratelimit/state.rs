
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use super::bucket::{RateLimitBucket, RateLimitConfig};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RateLimitIdentity {
    User(i64),
    Ip(String),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct RateLimitKey {
    bucket: RateLimitBucket,
    identity: RateLimitIdentity,
}

struct BucketEntry {
    started_at: Instant,
    requests: u32,
}

#[derive(Clone)]
pub struct RateLimitState {
    entries: Arc<Mutex<HashMap<RateLimitKey, BucketEntry>>>,
    checks: Arc<AtomicUsize>,
    jwt_secret: Arc<[u8]>,
}

pub struct RateLimitResult {
    pub remaining: u32,
    pub retry_after: Duration,
}

impl RateLimitState {
    pub fn new(jwt_secret: impl AsRef<[u8]>) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            checks: Arc::new(AtomicUsize::new(0)),
            jwt_secret: Arc::from(jwt_secret.as_ref()),
        }
    }

    pub fn jwt_secret(&self) -> &[u8] {
        &self.jwt_secret
    }

    pub fn check(
        &self,
        bucket: RateLimitBucket,
        identity: RateLimitIdentity,
    ) -> Result<RateLimitResult, RateLimitResult> {
        self.check_with_cost(bucket, identity, 1)
    }

    pub fn check_with_cost(
        &self,
        bucket: RateLimitBucket,
        identity: RateLimitIdentity,
        cost: u32,
    ) -> Result<RateLimitResult, RateLimitResult> {
        let config = bucket.config();
        let now = Instant::now();

        let key = RateLimitKey {
            bucket,
            identity,
        };

        let mut entries = self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let entry = entries.entry(key).or_insert_with(|| BucketEntry {
            started_at: now,
            requests: 0,
        });

        // Reset an expired bucket.
        if now.duration_since(entry.started_at) >= config.window {
            entry.started_at = now;
            entry.requests = 0;
        }
        // Reject requests that would exceed the limit.
        if cost > config.max_requests.saturating_sub(entry.requests) {
            let retry_after = config.window
                .saturating_sub(now.duration_since(entry.started_at));

            return Err(RateLimitResult {
                remaining: config.max_requests.saturating_sub(entry.requests),
                retry_after,
            });
        }

        entry.requests += cost;

        let result = RateLimitResult {
            remaining: config.max_requests - entry.requests,
            retry_after: config.window
                .saturating_sub(now.duration_since(entry.started_at)),
        };


        // Periodically remove expired entries.
        if self.checks.fetch_add(1, Ordering::Relaxed) % 1024 == 0 {
            entries.retain(|key, entry| {
                let window = key.bucket.config().window;
                now.duration_since(entry.started_at) < window
            });
        }

        Ok(result)
    }

    pub fn clear(&self) {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }
}