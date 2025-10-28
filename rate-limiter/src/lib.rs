use std::sync::Arc;
use std::time::{Duration, SystemTime};

use dashmap::DashMap;
use parking_lot::Mutex;

#[derive(Debug, Clone)]
pub struct BucketConfig {
    pub rate: f64,
    pub capacity: f64,
}

#[derive(Debug, Clone)]
pub struct BucketState {
    pub tokens: f64,
    pub last_refill: SystemTime,
    pub config: BucketConfig,
}

#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    async fn try_acquire(&self, key: &str, tokens: u32) -> bool;
    async fn get_state(&self, key: &str) -> Option<BucketState>;
}

pub struct InMemoryLimiter {
    map: DashMap<String, Arc<Mutex<BucketState>>>,
    default: BucketConfig,
}

impl InMemoryLimiter {
    pub fn new(default: BucketConfig) -> Self {
        Self {
            map: DashMap::new(),
            default,
        }
    }

    fn refill(state: &mut BucketState) {
        let now = SystemTime::now();
        let elapsed = now
            .duration_since(state.last_refill)
            .unwrap_or_else(|_| Duration::from_secs(0));
        let gained = elapsed.as_secs_f64() * state.config.rate;
        state.tokens = (state.tokens + gained).min(state.config.capacity);
        state.last_refill = now;
    }
}

#[async_trait::async_trait]
impl RateLimiter for InMemoryLimiter {
    async fn try_acquire(&self, key: &str, tokens: u32) -> bool {
        let entry = self.map.entry(key.to_string()).or_insert_with(|| {
            Arc::new(Mutex::new(BucketState {
                tokens: self.default.capacity,
                last_refill: SystemTime::now(),
                config: self.default.clone(),
            }))
        });

        let bucket = entry.value().clone();
        let mut s = bucket.lock();
        Self::refill(&mut s);
        let req = tokens as f64;

        if s.tokens >= req {
            s.tokens -= req;
            true
        } else {
            false
        }
    }

    async fn get_state(&self, key: &str) -> Option<BucketState> {
        self.map.get(key).map(|entry| entry.value().lock().clone())
    }
}
